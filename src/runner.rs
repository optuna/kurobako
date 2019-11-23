use crate::problem::KurobakoProblemRecipe;
use crate::record::{StudyRecord, StudyRecordBuilder, TrialRecordBuilder};
use crate::solver::KurobakoSolverRecipe;
use crate::study::{Scheduling, StudyRecipe};
use crate::time::ElapsedSeconds;
use indicatif::{MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, Evaluator as _, Problem as _, ProblemFactory as _, ProblemSpec,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{BoxSolver, Solver as _, SolverFactory as _, SolverSpec};
use kurobako_core::trial::{AskedTrial, EvaluatedTrial, IdGen, TrialId};
use kurobako_core::{Error, ErrorKind, Result};
use lazy_static::lazy_static;
use rand;
use rand::seq::SliceRandom;
use serde_json;
use std::collections::HashMap;
use std::io::Write as _;
use std::num::NonZeroUsize;
use std::sync::atomic::{self, AtomicUsize};
use std::sync::{Arc, Mutex};
use std::thread;
use structopt::StructOpt;
use trackable::error::ErrorKindExt;

lazy_static! {
    static ref REGISTRY: Mutex<FactoryRegistry> = Mutex::new(FactoryRegistry::new::<
        KurobakoProblemRecipe,
        KurobakoSolverRecipe,
    >());
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct RunnerOpt {
    #[structopt(long, default_value = "1")]
    pub parallelism: NonZeroUsize,
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

#[derive(Debug)]
pub struct Runner {
    mpb: MultiProgress,
    opt: RunnerOpt,
    cancel: Cancel,
}
impl Runner {
    pub fn new(opt: RunnerOpt) -> Self {
        let mpb = MultiProgress::with_draw_target(ProgressDrawTarget::stderr_with_hz(1));
        Self {
            mpb,
            opt,
            cancel: Cancel::new(),
        }
    }

    pub fn run(mut self) -> Result<()> {
        let recipes = track!(self.read_study_recipes())?;
        let pb = self.create_pb(&recipes);
        let runners = track!(self.create_study_runners(&recipes))?;

        self.spawn_runners(runners, pb);
        track!(self.mpb.join().map_err(|e| ErrorKind::Other.cause(e)))?;
        eprintln!();

        if let Some(e) = self.cancel.take() {
            Err(e)
        } else {
            Ok(())
        }
    }

    fn spawn_runners(&self, runners: Vec<StudyRunner>, pb: ProgressBar) {
        pb.tick();

        let pb_len = runners.len() as u64;
        let runners = Arc::new(Mutex::new(
            runners.into_iter().map(Some).collect::<Vec<_>>(),
        ));

        let next_index = Arc::new(AtomicUsize::new(0));
        for _ in 0..self.opt.parallelism.get() {
            let pb = pb.clone();
            let runners = Arc::clone(&runners);
            let next_index = Arc::clone(&next_index);
            let cancel = self.cancel.clone();
            thread::spawn(move || {
                while !cancel.is_canceled() {
                    let i = next_index.fetch_add(1, atomic::Ordering::SeqCst);
                    let runner = {
                        let mut runners = runners.lock().unwrap_or_else(|e| panic!("{}", e));
                        if i >= runners.len() {
                            break;
                        }
                        runners[i].take().unwrap_or_else(|| unreachable!())
                    };

                    let result = track!(runner.run());

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

    fn create_study_runners(&self, recipes: &[StudyRecipe]) -> Result<Vec<StudyRunner>> {
        recipes
            .iter()
            .map(|recipe| track!(StudyRunner::new(recipe, &self.mpb, self.cancel.clone())))
            .collect()
    }

    fn create_pb(&self, recipes: &[StudyRecipe]) -> ProgressBar {
        let pb = self.mpb.add(ProgressBar::new(recipes.len() as u64));
        let template = format!(
            "(ALL) [{{elapsed_precise}}] [STUDIES \
             {{pos:>6}}/{{len}} {{percent:>3}}%] [ETA {{eta:>3}}] {{msg}}",
        );
        let style = ProgressStyle::default_bar().template(&template);
        pb.set_style(style);
        pb
    }
}

#[derive(Debug)]
struct StudyRunner {
    solver: BoxSolver,
    solver_spec: SolverSpec,
    problem: BoxProblem,
    problem_spec: ProblemSpec,
    study_record: StudyRecordBuilder,
    rng: ArcRng,
    pb: ProgressBar,
    cancel: Cancel,
    idg: IdGen,
    threads: EvaluationThreads,
    study_steps: u64,
}
impl StudyRunner {
    fn new(study: &StudyRecipe, mpb: &MultiProgress, cancel: Cancel) -> Result<Self> {
        // TODO: check capabilities
        let registry = track!(REGISTRY.lock().map_err(Error::from))?;

        let random_seed = study.seed.unwrap_or_else(rand::random);
        let rng = ArcRng::new(random_seed);

        let problem_factory = track!(registry.get_or_create_problem_factory(&study.problem))?;
        let problem_factory = track!(problem_factory.lock().map_err(Error::from))?;
        let problem_spec = track!(problem_factory.specification())?;
        let problem = track!(problem_factory.create_problem(rng.clone()))?;

        let solver_factory = track!(registry.get_or_create_solver_factory(&study.solver))?;
        let solver_factory = track!(solver_factory.lock().map_err(Error::from))?;
        let solver_spec = track!(solver_factory.specification())?;
        let solver = track!(solver_factory.create_solver(rng.clone(), &problem_spec))?;

        let study_steps = problem_spec.steps.last() * study.budget;
        let pb = mpb.add(ProgressBar::new(study_steps));
        let pb_style = ProgressStyle::default_bar().template(&format!(
            "(STUDY) [{{elapsed_precise}}] [STEPS {{pos:>6}}/{{len}} \
             {{percent:>3}}%] [ETA {{eta:>3}}] {:?} {:?}",
            solver_spec.name, problem_spec.name
        ));
        pb.set_style(pb_style.clone());

        let mut recipe = study.clone();
        recipe.seed = Some(random_seed);
        let study_record =
            StudyRecordBuilder::new(recipe, solver_spec.clone(), problem_spec.clone());
        let threads = EvaluationThreads::new(study, rng.clone());
        Ok(Self {
            solver,
            solver_spec,
            problem,
            problem_spec,
            study_record,
            rng,
            pb,
            cancel,
            idg: IdGen::new(),
            threads,
            study_steps,
        })
    }

    fn run(mut self) -> Result<StudyRecord> {
        self.pb.reset_elapsed();

        while self.pb.position() < self.study_steps {
            let start_step = self.pb.position();

            let (asked_trial, ask_elapsed) =
                ElapsedSeconds::try_time(|| track!(self.solver.ask(&mut self.idg)))?;

            // TODO: Fix scheduling implementation correctly.
            let thread = track!(self.threads.assign(&asked_trial, &self.problem))?;
            let thread_id = thread.thread_id;

            if let Some(next_step) = asked_trial.next_step {
                let problem_spec = &self.problem_spec;
                let ((elapsed_steps, evaluated_trial), evaluate_elapsed) =
                    ElapsedSeconds::try_time(|| {
                        track!(thread.evaluate(asked_trial.id, next_step, problem_spec))
                    })?;
                self.pb.inc(elapsed_steps);
                let end_step = self.pb.position();

                if end_step < self.study_steps {
                    let ((), tell_elapsed) = ElapsedSeconds::try_time(|| {
                        track!(self.solver.tell(evaluated_trial.clone()))
                    })?;

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
            } else {
                thread.prune(asked_trial.id);
            }
        }

        self.pb.finish_and_clear();
        Ok(self.study_record.finish())
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

    fn assign(
        &mut self,
        trial: &AskedTrial,
        problem: &BoxProblem,
    ) -> Result<&mut EvaluationThread> {
        if self
            .threads
            .iter()
            .find(|t| t.runnings.contains_key(&trial.id))
            .is_some()
        {
            let thread = self
                .threads
                .iter_mut()
                .find(|t| t.runnings.contains_key(&trial.id))
                .unwrap_or_else(|| unreachable!());
            Ok(thread)
        } else {
            match self.scheduling {
                Scheduling::Random => track!(self.assign_random(trial, problem)),
                Scheduling::Fair => track!(self.assign_fair(trial, problem)),
            }
        }
    }

    fn assign_random(
        &mut self,
        trial: &AskedTrial,
        problem: &BoxProblem,
    ) -> Result<&mut EvaluationThread> {
        let thread = track_assert_some!(self.threads.choose_mut(&mut self.rng), ErrorKind::Bug);
        let state = track!(EvaluatorState::new(problem, trial))?;
        thread.runnings.insert(trial.id, state);
        Ok(thread)
    }

    fn assign_fair(
        &mut self,
        trial: &AskedTrial,
        problem: &BoxProblem,
    ) -> Result<&mut EvaluationThread> {
        self.threads.sort_by_key(|t| t.elapsed_steps);
        let thread = track_assert_some!(self.threads.first_mut(), ErrorKind::Bug);
        let state = track!(EvaluatorState::new(problem, trial))?;
        thread.runnings.insert(trial.id, state);
        Ok(thread)
    }
}

#[derive(Debug)]
struct EvaluationThread {
    thread_id: usize,
    runnings: HashMap<TrialId, EvaluatorState>,
    elapsed_steps: u64,
}
impl EvaluationThread {
    fn new(thread_id: usize) -> Self {
        Self {
            thread_id,
            runnings: HashMap::new(),
            elapsed_steps: 0,
        }
    }

    fn prune(&mut self, trial_id: TrialId) {
        self.runnings.remove(&trial_id);
    }

    fn evaluate(
        &mut self,
        trial_id: TrialId,
        next_step: u64,
        problem_spec: &ProblemSpec,
    ) -> Result<(u64, EvaluatedTrial)> {
        let mut state = track_assert_some!(self.runnings.remove(&trial_id), ErrorKind::Bug);

        let next_step = track_assert_some!(
            problem_spec
                .steps
                .iter()
                .skip_while(|&s| s < next_step)
                .nth(0),
            ErrorKind::Bug
        );
        let (current_step, values) = track!(state.evaluator.evaluate(next_step))?;
        track_assert!(state.current_step <= current_step, ErrorKind::Bug);
        let elapsed_steps = current_step - state.current_step;
        self.elapsed_steps += elapsed_steps;

        state.current_step = current_step;
        if state.current_step < problem_spec.steps.last() && !values.is_empty() {
            self.runnings.insert(trial_id, state);
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
struct EvaluatorState {
    evaluator: BoxEvaluator,
    current_step: u64,
}
impl EvaluatorState {
    fn new(problem: &BoxProblem, trial: &AskedTrial) -> Result<Self> {
        let evaluator = track!(problem.create_evaluator(trial.params.clone()))?;
        Ok(Self {
            evaluator,
            current_step: 0,
        })
    }
}
