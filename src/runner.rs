use crate::problem::KurobakoProblemRecipe;
use crate::record::StudyRecord;
use crate::solver::KurobakoSolverRecipe;
use crate::study::StudyRecipe;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use kurobako_core::problem::{BoxProblem, ProblemFactory as _, ProblemSpec};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{BoxSolver, SolverFactory as _, SolverSpec};
use kurobako_core::{Error, ErrorKind, Result};
use lazy_static::lazy_static;
use rand;
use serde_json;
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
        Self {
            mpb: MultiProgress::new(),
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
                    let result = track!(result.and_then(|record| serde_json::to_writer(
                        std::io::stdout().lock(),
                        &record
                    )
                    .map_err(Error::from)));
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
    study_record: StudyRecord,
    rng: ArcRng,
    pb: ProgressBar,
    cancel: Cancel,
}
impl StudyRunner {
    fn new(study: &StudyRecipe, mpb: &MultiProgress, cancel: Cancel) -> Result<Self> {
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

        let pb = mpb.add(ProgressBar::new(problem_spec.steps.last() * study.budget));
        let pb_style = ProgressStyle::default_bar().template(&format!(
            "(STUDY) [{{elapsed_precise}}] [STEPS {{pos:>6}}/{{len}} \
             {{percent:>3}}%] [ETA {{eta:>3}}] {:?} {:?}",
            solver_spec.name, problem_spec.name
        ));
        pb.set_style(pb_style.clone());

        let study_record = StudyRecord::new();
        Ok(Self {
            solver,
            solver_spec,
            problem,
            problem_spec,
            study_record,
            rng,
            pb,
            cancel,
        })
    }

    fn run(mut self) -> Result<()> {
        self.pb.reset_elapsed();

        // self.pb.set_message(&format!("item #{}", i + 1));
        for _ in 0..10 {
            ::std::thread::sleep_ms(::rand::random::<u32>() % 1000);
            self.pb.inc(1);
            // if rand::random::<u32>() % 10 == 0 {
            //     track_panic!(ErrorKind::Bug);
            // }
        }
        self.pb.inc(self.problem_spec.steps.last());
        ::std::thread::sleep_ms(1000);

        //self.pb.finish_with_message("done");
        self.pb.finish_and_clear();
        Ok(())
    }
}

// #[derive(Debug)]
// pub struct StudyRunner<S, P>
// where
//     P: Problem,
// {
//     rng: ThreadRng,
//     solver: S,
//     problem: P,
//     study_record: StudyRecord,
//     pub study_budget: Budget, // TODO
//     scheduler: TrialThreadScheduler<P::Evaluator>,
// }
// impl<S, P> StudyRunner<S, P>
// where
//     S: Solver,
//     P: Problem,
// {
//     pub fn new<SR, PR>(
//         solver_recipe: &SR,
//         problem_recipe: &PR,
//         options: &StudyRunnerOptions,
//     ) -> Result<Self>
//     where
//         SR: SolverRecipe<Solver = S>,
//         PR: ProblemRecipe<Problem = P>,
//     {
//         let problem = track!(problem_recipe.create_problem())?;
//         let problem_spec = problem.specification();

//         let solver = track!(solver_recipe.create_solver(problem_spec.clone()))?;
//         let solver_spec = solver.specification();

//         let study_budget =
//             Budget::new(options.budget as u64 * problem_spec.evaluation_expense.get());

//         let study_record = track!(StudyRecord::new(
//             solver_recipe,
//             solver_spec,
//             problem_recipe,
//             problem_spec,
//             options.clone()
//         ))?;

//         Ok(Self {
//             rng: rand::thread_rng(),
//             solver,
//             problem,
//             study_record,
//             study_budget,
//             scheduler: TrialThreadScheduler::new(options),
//         })
//     }

//     pub fn run(mut self) -> Result<StudyRecord> {
//         while !self.study_budget.is_consumed() {
//             while let Some(mut thread) = self.scheduler.pop_idle_thread() {
//                 let trial = track!(self.ask_trial())?;
//                 thread.trial = Some(trial);
//                 self.scheduler.threads.push(thread);
//             }

//             track!(self.evaluate_trial())?;
//         }

//         self.study_record.finish();
//         Ok(self.study_record)
//     }

//     // TODO
//     pub fn run_once(&mut self, budget: &mut Budget) -> Result<()> {
//         track_assert!(!self.study_budget.is_consumed(), ErrorKind::Bug);

//         while !self.study_budget.is_consumed() && !budget.is_consumed() {
//             while let Some(mut thread) = self.scheduler.pop_idle_thread() {
//                 let trial = track!(self.ask_trial())?;
//                 thread.trial = Some(trial);
//                 self.scheduler.threads.push(thread);
//             }

//             let before = self.study_budget.consumption;
//             track!(self.evaluate_trial())?;
//             let after = self.study_budget.consumption;
//             budget.consumption += after - before;
//         }

//         if self.study_budget.is_consumed() {
//             self.study_record.finish();
//         }
//         Ok(())
//     }

//     pub fn study(&self) -> &StudyRecord {
//         &self.study_record
//     }

//     fn ask_trial(&mut self) -> Result<TrialState<P::Evaluator>> {
//         loop {
//             let (result, elapsed) = ElapsedSeconds::time(|| {
//                 track!(self.solver.ask(&mut self.rng, unsafe { &mut ID_GEN }))
//             });
//             let mut obs = result?;
//             obs.param.budget_mut().amount = self.next_evaluation_point(obs.param.budget().amount);

//             let evaluator = if let Some(pending) = self.scheduler.pendings.remove(&obs.id) {
//                 pending.evaluator
//             } else if self.scheduler.cancelled.contains(&obs.id) {
//                 trace!(
//                     "{:?} has been cancelled: budget={:?}",
//                     obs.id,
//                     obs.param.budget()
//                 );
//                 continue;
//             } else {
//                 track!(self.problem.create_evaluator(obs.id))?
//             };

//             let params = obs.param.get().clone();
//             return Ok(TrialState {
//                 obs,
//                 evaluator,
//                 ask: AskRecord { params, elapsed },
//             });
//         }
//     }

//     fn evaluate_trial(&mut self) -> Result<()> {
//         self.scheduler.threads.sort_by_key(|t| t.priority());

//         let thread = &mut self.scheduler.threads[0];
//         let mut trial = track_assert_some!(thread.trial.take(), ErrorKind::Bug);
//         let mut trial_budget = trial.obs.param.budget();

//         trace!("Thread[{}]: budget={:?}", thread.id, trial_budget);

//         let (result, elapsed) = ElapsedSeconds::time(|| {
//             track!(trial
//                 .evaluator
//                 .evaluate(trial.obs.param.get(), &mut trial_budget))
//         });
//         let expense = trial_budget.consumption - trial.obs.param.budget().consumption;

//         match result {
//             Ok(values) => {
//                 self.study_budget.consumption += expense;
//                 thread.budget_consumption += expense;
//                 *trial.obs.param.budget_mut() = trial_budget;
//                 let evaluate = EvaluateRecord {
//                     values: values.clone(),
//                     elapsed,
//                     expense,
//                 };

//                 let obs = trial.obs.clone().map_value(|()| values);
//                 let solver = &mut self.solver;
//                 let (result, elapsed) = ElapsedSeconds::time(|| track!(solver.tell(obs)));
//                 result?;
//                 let tell = TellRecord { elapsed };

//                 self.study_record.trials.push(TrialRecord {
//                     thread_id: thread.id,
//                     obs_id: trial.obs.id,
//                     ask: trial.ask,
//                     evaluate,
//                     tell,
//                 });

//                 if trial_budget.consumption < self.trial_max_budget() {
//                     track_assert!(trial_budget.is_consumed(), ErrorKind::Other; trial_budget);
//                     let pending = Pending {
//                         evaluator: trial.evaluator,
//                         seqno: self.study_budget.consumption,
//                     };
//                     self.scheduler.pendings.insert(trial.obs.id, pending);
//                     if let Some(max_pendings) = self.study_record.runner.max_pendings {
//                         if max_pendings < self.scheduler.pendings.len() {
//                             let id = self
//                                 .scheduler
//                                 .pendings
//                                 .iter()
//                                 .map(|t| (t.1.seqno, *t.0))
//                                 .min()
//                                 .unwrap_or_else(|| unreachable!())
//                                 .1;
//                             self.scheduler.pendings.remove(&id);
//                             self.scheduler.cancelled.insert(id);
//                         }
//                     }
//                 }
//             }
//             Err(e) => {
//                 if *e.kind() == ErrorKind::UnevaluableParams {
//                     self.study_record.unevaluable_trials += 1;
//                     warn!(
//                         "Unevaluable parameters ({}): {}",
//                         self.study_record.unevaluable_trials, e
//                     );
//                     track_assert!(
//                         self.study_record.unevaluable_trials < 10000,
//                         ErrorKind::Other
//                     );
//                 } else {
//                     return Err(e);
//                 }
//             }
//         }
//         Ok(())
//     }

//     fn trial_max_budget(&self) -> u64 {
//         self.study_record.problem.spec.evaluation_expense.get()
//     }

//     fn next_evaluation_point(&self, candidate: u64) -> u64 {
//         use std::cmp;

//         if self.study_record.runner.steps.is_empty() {
//             candidate
//         } else {
//             self.study_record
//                 .runner
//                 .steps
//                 .iter()
//                 .cloned()
//                 .rev()
//                 .take_while(|p| candidate <= *p)
//                 .last()
//                 .map(|n| cmp::min(n, self.trial_max_budget()))
//                 .unwrap_or_else(|| self.trial_max_budget())
//         }
//     }
// }

// #[derive(Debug)]
// struct TrialThreadScheduler<E> {
//     threads: Vec<TrialThread<E>>,
//     pendings: HashMap<ObsId, Pending<E>>,
//     cancelled: HashSet<ObsId>,
// }
// impl<E> TrialThreadScheduler<E> {
//     fn new(options: &StudyRunnerOptions) -> Self {
//         Self {
//             threads: (0..options.concurrency).map(TrialThread::new).collect(),
//             pendings: HashMap::new(),
//             cancelled: HashSet::new(),
//         }
//     }

//     fn pop_idle_thread(&mut self) -> Option<TrialThread<E>> {
//         for i in 0..self.threads.len() {
//             if self.threads[i].trial.is_none() {
//                 return Some(self.threads.swap_remove(i));
//             }
//         }
//         None
//     }
// }

// #[derive(Debug)]
// struct TrialState<E> {
//     obs: UnobservedObs,
//     evaluator: E,
//     ask: AskRecord,
// }

// #[derive(Debug)]
// struct TrialThread<E> {
//     id: usize,
//     budget_consumption: u64,
//     trial: Option<TrialState<E>>,
// }
// impl<E> TrialThread<E> {
//     fn new(id: usize) -> Self {
//         Self {
//             id,
//             budget_consumption: 0,
//             trial: None,
//         }
//     }

//     fn priority(&self) -> u64 {
//         self.trial.as_ref().map_or(std::u64::MAX, |t| {
//             t.obs.param.budget().amount + self.budget_consumption
//         })
//     }
// }

// #[derive(Debug)]
// struct Pending<E> {
//     evaluator: E,
//     seqno: u64,
// }
