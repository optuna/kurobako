use crate::problem::KurobakoProblemRecipe;
use crate::record::StudyRecord;
use crate::solver::KurobakoSolverRecipe;
use crate::study::StudyRecipe;
use kurobako_core::problem::{BoxProblem, ProblemFactory as _, ProblemSpec};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{BoxSolver, SolverFactory as _, SolverSpec};
use kurobako_core::{Error, Result};
use lazy_static::lazy_static;
use rand;
use std::sync::Mutex;

lazy_static! {
    static ref REGISTRY: Mutex<FactoryRegistry> = Mutex::new(FactoryRegistry::new::<
        KurobakoProblemRecipe,
        KurobakoSolverRecipe,
    >());
}

#[derive(Debug)]
pub struct StudyRunner {
    solver: BoxSolver,
    solver_spec: SolverSpec,
    problem: BoxProblem,
    problem_spec: ProblemSpec,
    study_record: StudyRecord,
    rng: ArcRng,
}
impl StudyRunner {
    pub fn new(study: &StudyRecipe) -> Result<Self> {
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

        let study_record = StudyRecord::new();
        Ok(Self {
            solver,
            solver_spec,
            problem,
            problem_spec,
            study_record,
            rng,
        })
    }

    pub fn run(mut self) -> Result<()> {
        panic!()
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

//         if self.study_record.runner.evaluation_points.is_empty() {
//             candidate
//         } else {
//             self.study_record
//                 .runner
//                 .evaluation_points
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
