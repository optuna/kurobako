//! `kurobako run` command.
use crate::problem::KurobakoProblemRecipe;
use crate::record::{StudyRecord, StudyRecordBuilder, TrialRecordBuilder};
use crate::solver::KurobakoSolverRecipe;
use crate::study::{Scheduling, StudyRecipe};
use crate::time::ElapsedSeconds;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use kurobako_core::problem::ProblemRecipe as _;
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, Evaluator as _, Problem as _, ProblemFactory as _, ProblemSpec,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{BoxSolver, Solver as _, SolverFactory as _, SolverRecipe as _};
use kurobako_core::trial::Values;
use kurobako_core::trial::{EvaluatedTrial, IdGen, NextTrial, TrialId};
use kurobako_core::{Error, ErrorKind, Result};
use rand::seq::SliceRandom;
use std::collections::{HashMap, VecDeque};
use std::io::Write as _;
use std::num::NonZeroUsize;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::thread;
use structopt::StructOpt;
use trackable::error::ErrorKindExt;

/// Options of the `kurobako run` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunnerOpt {
    /// Number of worker threads used to execute the benchmark.
    #[structopt(long, short = "p", default_value = "1")]
    pub parallelism: NonZeroUsize,

    /// Disables progress bar.
    #[structopt(long, short = "q")]
    pub quiet: bool,
}

#[derive(Debug, Clone)]
struct Cancel(Arc<Mutex<Option<Error>>>);
impl Cancel {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(None)))
    }

    fn is_canceled(&self) -> bool {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).is_some()
    }

    fn cancel(&self, e: Error) -> bool {
        let mut x = self.0.lock().unwrap_or_else(|e| panic!("{}", e));
        if x.is_none() {
            *x = Some(e);
            true
        } else {
            false
        }
    }

    fn take(&self) -> Option<Error> {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).take()
    }
}

/// Runner of a benchmark.
#[derive(Debug)]
pub struct Runner {
    mpb: Arc<MultiProgress>,
    opt: RunnerOpt,
    cancel: Cancel,
}
impl Runner {
    /// Makes a `Runner` instance.
    pub fn new(opt: RunnerOpt) -> Self {
        let target = if opt.quiet {
            ProgressDrawTarget::hidden()
        } else {
            ProgressDrawTarget::stderr_with_hz(1)
        };
        let mpb = MultiProgress::with_draw_target(target);
        Self {
            mpb: Arc::new(mpb),
            opt,
            cancel: Cancel::new(),
        }
    }

    /// Runs the benchmark.
    pub fn run(mut self) -> Result<()> {
        let recipes = track!(self.read_study_recipes())?;
        let pb = self.create_pb(&recipes);

        self.spawn_runners(recipes, pb);
        track!(self.mpb.join().map_err(|e| ErrorKind::Other.cause(e)))?;
        eprintln!();

        if let Some(e) = self.cancel.take() {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn spawn_runners(&self, recipes: Vec<StudyRecipe>, pb: ProgressBar) {
        pb.tick();

        let pb_len = recipes.len() as u64;
        let recipes = Arc::new(Mutex::new(
            recipes.into_iter().map(Some).collect::<Vec<_>>(),
        ));

        let next_index = Arc::new(AtomicUsize::new(0));
        for _ in 0..self.opt.parallelism.get() {
            let pb = pb.clone();
            let recipes = Arc::clone(&recipes);
            let next_index = Arc::clone(&next_index);
            let cancel = self.cancel.clone();
            let opt = self.opt.clone();
            let mpb = Arc::clone(&self.mpb);
            thread::spawn(move || {
                while !cancel.is_canceled() {
                    let i = next_index.fetch_add(1, atomic::Ordering::SeqCst);
                    let recipe = {
                        let mut recipes = recipes.lock().unwrap_or_else(|e| panic!("{}", e));
                        if i >= recipes.len() {
                            break;
                        }
                        recipes[i].take().unwrap_or_else(|| unreachable!())
                    };

                    let result = track!(StudyRunner::with_mpb(&recipe, &opt, &mpb))
                        .and_then(|runner| track!(runner.run()));

                    fn output(record: StudyRecord) -> Result<()> {
                        let stdout = std::io::stdout();
                        let mut stdout = stdout.lock();
                        track!(serde_json::to_writer(&mut stdout, &record).map_err(Error::from))?;
                        track!(writeln!(stdout).map_err(Error::from))?;
                        Ok(())
                    }
                    let result = track!(result.and_then(output));
                    pb.inc(1);

                    if let Err(e) = result {
                        if cancel.cancel(e) {
                            pb.finish_with_message("canceled");
                        }
                    } else if pb.position() == pb_len {
                        pb.finish_with_message("done");
                    }
                }
            });
        }
    }

    fn read_study_recipes(&mut self) -> Result<Vec<StudyRecipe>> {
        let stdin = std::io::stdin();
        serde_json::Deserializer::from_reader(stdin.lock())
            .into_iter()
            .map(|recipe| track!(recipe.map_err(Error::from)))
            .collect()
    }

    fn create_pb(&self, recipes: &[StudyRecipe]) -> ProgressBar {
        let pb = self.mpb.add(ProgressBar::new(recipes.len() as u64));
        let template =
            "(ALL) [{elapsed_precise}] [STUDIES {pos:>6}/{len} {percent:>3}%] [ETA {eta:>3}] {msg}";
        let style = ProgressStyle::default_bar().template(template);
        pb.set_style(style);
        pb
    }
}

#[derive(Debug)]
pub(crate) struct StudyRunner {
    solver: BoxSolver,
    problem: BoxProblem,
    problem_spec: ProblemSpec,
    study_record: StudyRecordBuilder,
    pb: ProgressBar,
    idg: IdGen,
    threads: EvaluationThreads,
    evaluators: HashMap<TrialId, EvaluatorState>,
    study_steps: u64,
    opt: RunnerOpt,
    _mpb: Option<MultiProgress>,
}
impl StudyRunner {
    pub fn new(study: &StudyRecipe) -> Result<Self> {
        let opt = RunnerOpt {
            parallelism: unsafe { NonZeroUsize::new_unchecked(1) },
            quiet: true,
        };
        let mpb = MultiProgress::with_draw_target(ProgressDrawTarget::hidden());
        let mut this = track!(Self::with_mpb(study, &opt, &mpb))?;
        this._mpb = Some(mpb);
        Ok(this)
    }

    fn with_mpb(study: &StudyRecipe, opt: &RunnerOpt, mpb: &MultiProgress) -> Result<Self> {
        let registry = FactoryRegistry::new::<KurobakoProblemRecipe, KurobakoSolverRecipe>();

        let random_seed = study.seed.unwrap_or_else(rand::random);
        let rng = ArcRng::new(random_seed);

        let problem_factory = track!(study.problem.create_factory(&registry))?;
        let problem_spec = track!(problem_factory.specification())?;
        let problem = track!(problem_factory.create_problem(rng.clone()))?;

        let solver_factory = track!(study.solver.create_factory(&registry))?;
        let solver_spec = track!(solver_factory.specification())?;

        let incapables = solver_spec
            .capabilities
            .incapables(&problem_spec.requirements())
            .collect::<Vec<_>>();
        track_assert!(incapables.is_empty(), ErrorKind::Incapable; incapables);

        let solver = track!(solver_factory.create_solver(rng.clone(), &problem_spec))?;

        let study_steps = problem_spec.steps.last() * study.budget;
        let pb = mpb.add(ProgressBar::new(study_steps));
        let pb_style = ProgressStyle::default_bar().template(&format!(
            "(STUDY) [{{elapsed_precise}}] [STEPS {{pos:>6}}/{{len}} \
             {{percent:>3}}%] [ETA {{eta:>3}}] {:?} {:?}",
            solver_spec.name, problem_spec.name
        ));
        pb.set_style(pb_style);

        let mut recipe = study.clone();
        recipe.seed = Some(random_seed);
        let study_record = StudyRecordBuilder::new(recipe, solver_spec, problem_spec.clone());
        let threads = EvaluationThreads::new(study, rng);
        Ok(Self {
            solver,
            problem,
            problem_spec,
            study_record,
            pb,
            idg: IdGen::new(),
            threads,
            evaluators: HashMap::new(),
            study_steps,
            opt: opt.clone(),
            _mpb: None,
        })
    }

    pub fn run_init(&mut self) -> Result<()> {
        self.pb.reset_elapsed();
        Ok(())
    }

    pub fn run_once(&mut self) -> Result<()> {
        track!(self.fill_waiting_queue())?;

        let start_step = self.pb.position();
        let thread = track!(self.threads.next())?;
        let thread_id = thread.thread_id;
        let WaitingTrial {
            asked_trial,
            ask_elapsed,
        } = track!(thread.next_trial())?;
        let next_step = track_assert_some!(asked_trial.next_step, ErrorKind::Bug);

        let problem_spec = &self.problem_spec;
        let evaluators = &mut self.evaluators;
        let ((elapsed_steps, evaluated_trial), evaluate_elapsed) =
            ElapsedSeconds::try_time(|| {
                track!(thread.evaluate(asked_trial.id, next_step, problem_spec, evaluators))
            })?;
        self.pb.inc(elapsed_steps);
        let end_step = self.pb.position();

        if end_step <= self.study_steps {
            let ((), tell_elapsed) =
                ElapsedSeconds::try_time(|| track!(self.solver.tell(evaluated_trial.clone())))?;

            self.study_record.add_trial(TrialRecordBuilder {
                id: asked_trial.id,
                thread_id,
                params: asked_trial.params,
                values: evaluated_trial.values,
                start_step,
                end_step,
                ask_elapsed,
                tell_elapsed,
                evaluate_elapsed,
            });
        }

        Ok(())
    }

    fn fill_waiting_queue(&mut self) -> Result<()> {
        while self.threads.has_idle_thread() {
            let (asked_trial, ask_elapsed) =
                ElapsedSeconds::try_time(|| track!(self.solver.ask(&mut self.idg)))?;

            if let Err(e) = track!(self.init_evaluator(&asked_trial)) {
                if *e.kind() != ErrorKind::UnevaluableParams {
                    return Err(e);
                } else {
                    let unevaluable = EvaluatedTrial {
                        id: asked_trial.id,
                        values: Values::new(vec![]),
                        current_step: 0,
                    };
                    track!(self.solver.tell(unevaluable))?
                }
            } else if asked_trial.next_step.is_some() {
                track!(self.threads.assign(&asked_trial, ask_elapsed))?;
            } else {
                track!(self.prune_evaluator(asked_trial.id))?;
            }
        }
        Ok(())
    }

    pub fn current_step(&self) -> u64 {
        self.pb.position()
    }

    pub fn max_step(&self) -> u64 {
        self.study_steps
    }

    pub fn best_values(&self) -> Option<&Values> {
        // Note that even if there are more than one trials on the pareto front,
        // the only last one will be returned.
        self.study_record.pareto_frontier().map(|x| x.2).last()
    }

    fn run(mut self) -> Result<StudyRecord> {
        track!(self.run_init())?;

        while self.pb.position() < self.study_steps {
            if self.pb.is_hidden() && !self.opt.quiet {
                eprintln!("DONE: {}/{}", self.pb.position(), self.study_steps);
            }
            track!(self.run_once())?;
        }

        self.pb.finish_and_clear();
        Ok(self.study_record.finish())
    }

    #[allow(clippy::map_entry)]
    fn init_evaluator(&mut self, trial: &NextTrial) -> Result<()> {
        if !self.evaluators.contains_key(&trial.id) {
            let evaluator = track!(EvaluatorState::new(&self.problem, trial))?;
            self.evaluators.insert(trial.id, evaluator);
        }
        Ok(())
    }

    fn prune_evaluator(&mut self, trial_id: TrialId) -> Result<()> {
        track_assert_some!(self.evaluators.remove(&trial_id), ErrorKind::InvalidInput; trial_id);
        Ok(())
    }
}

#[derive(Debug)]
struct EvaluationThreads {
    threads: Vec<EvaluationThread>,
    scheduling: Scheduling,
    rng: ArcRng,
}
impl EvaluationThreads {
    fn new(recipe: &StudyRecipe, rng: ArcRng) -> Self {
        Self {
            threads: (0..recipe.concurrency.get())
                .map(EvaluationThread::new)
                .collect(),
            scheduling: recipe.scheduling,
            rng,
        }
    }

    fn has_idle_thread(&self) -> bool {
        self.threads.iter().any(|t| t.is_idle())
    }

    fn next(&mut self) -> Result<&mut EvaluationThread> {
        match self.scheduling {
            Scheduling::Fair => {
                let thread = track_assert_some!(
                    self.threads.iter_mut().min_by_key(|t| t.elapsed_steps),
                    ErrorKind::Bug
                );
                Ok(thread)
            }
            Scheduling::Random => {
                let thread =
                    track_assert_some!(self.threads.choose_mut(&mut self.rng), ErrorKind::Bug);
                Ok(thread)
            }
        }
    }

    fn assign(&mut self, trial: &NextTrial, ask_elapsed: ElapsedSeconds) -> Result<()> {
        let thread = track_assert_some!(
            self.threads.iter_mut().find(|t| t.waitings.is_empty()),
            ErrorKind::Bug
        );
        thread.waitings.push_back(WaitingTrial {
            asked_trial: trial.clone(),
            ask_elapsed,
        });
        Ok(())
    }
}

#[derive(Debug)]
struct EvaluationThread {
    thread_id: usize,
    waitings: VecDeque<WaitingTrial>,
    elapsed_steps: u64,
}
impl EvaluationThread {
    fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            waitings: VecDeque::new(),
            elapsed_steps: 0,
        }
    }

    fn is_idle(&self) -> bool {
        self.waitings.is_empty()
    }

    fn next_trial(&mut self) -> Result<WaitingTrial> {
        let trial = track_assert_some!(self.waitings.pop_front(), ErrorKind::Bug);
        Ok(trial)
    }

    fn evaluate(
        &mut self,
        trial_id: TrialId,
        next_step: u64,
        problem_spec: &ProblemSpec,
        evaluators: &mut HashMap<TrialId, EvaluatorState>,
    ) -> Result<(u64, EvaluatedTrial)> {
        let mut state = track_assert_some!(evaluators.remove(&trial_id), ErrorKind::Bug);

        let next_step = track_assert_some!(
            problem_spec.steps.iter().find(|&s| s >= next_step),
            ErrorKind::Bug
        );
        let (current_step, values) = track!(state.evaluator.evaluate(next_step))?;
        track_assert!(state.current_step <= current_step, ErrorKind::Bug);
        let elapsed_steps = current_step - state.current_step;
        self.elapsed_steps += elapsed_steps;

        state.current_step = current_step;
        if state.current_step < problem_spec.steps.last() && !values.is_empty() {
            evaluators.insert(trial_id, state);
        }

        let evaluated = EvaluatedTrial {
            id: trial_id,
            values,
            current_step,
        };
        Ok((elapsed_steps, evaluated))
    }
}

#[derive(Debug)]
struct WaitingTrial {
    asked_trial: NextTrial,
    ask_elapsed: ElapsedSeconds,
}

#[derive(Debug)]
struct EvaluatorState {
    evaluator: BoxEvaluator,
    current_step: u64,
}
impl EvaluatorState {
    fn new(problem: &BoxProblem, trial: &NextTrial) -> Result<Self> {
        let evaluator = track!(problem.create_evaluator(trial.params.clone()))?;
        Ok(Self {
            evaluator,
            current_step: 0,
        })
    }
}
