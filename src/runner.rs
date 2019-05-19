use crate::problem::FullKurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe};
use kurobako_core::solver::UnobservedObs;
use kurobako_core::solver::{Solver, SolverRecipe};
use kurobako_core::{ErrorKind, Result};
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use std;
use std::collections::{HashMap, VecDeque};
use yamakan::budget::Budget;
use yamakan::observation::{ObsId, SerialIdGenerator};

#[derive(Debug)]
struct Trial<E> {
    evaluator: E,
    obs: UnobservedObs,
    record: TrialRecord,
}

#[derive(Debug)]
struct EvaluationThread<E> {
    id: usize,
    elapsed: u64,
    trial: Option<Trial<E>>,
}
impl<E> EvaluationThread<E> {
    fn new(id: usize) -> Self {
        Self {
            id,
            elapsed: 0,
            trial: None,
        }
    }

    fn priority(&self) -> u64 {
        self.trial.as_ref().map_or(std::u64::MAX, |t| {
            t.obs.param.budget().amount + self.elapsed
        })
    }
}

#[derive(Debug)]
struct Scheduler<E> {
    threads: Vec<EvaluationThread<E>>,
    runnables: VecDeque<Trial<E>>,
    pendings: HashMap<ObsId, Trial<E>>,
}
impl<E> Scheduler<E> {
    fn new(concurrency: usize) -> Self {
        Self {
            threads: (0..concurrency).map(EvaluationThread::new).collect(),
            runnables: VecDeque::new(),
            pendings: HashMap::new(),
        }
    }

    fn has_idle_thread(&self) -> bool {
        self.threads.iter().any(|t| t.trial.is_none())
    }
}

#[derive(Debug)]
pub struct Runner<S, P, R = ThreadRng>
where
    P: Problem,
{
    rng: R,
    idgen: SerialIdGenerator,
    solver: S,
    problem: P,
    study_budget: Budget,
    study_record: StudyRecord,
    watch: Stopwatch,
    errors: usize,
    evaluation_expense: u64,
    scheduler: Scheduler<P::Evaluator>,
}
impl<S, P> Runner<S, P, ThreadRng>
where
    S: Solver,
    P: Problem,
{
    pub fn new<SR, PR>(
        solver_recipe: &SR,
        problem_recipe: &PR,
        budget_factor: f64,
        concurrency: usize,
    ) -> Result<Self>
    where
        SR: SolverRecipe<Solver = S>,
        PR: ProblemRecipe<Problem = P>,
    {
        let problem = track!(problem_recipe.create_problem())?;
        let problem_spec = problem.specification();

        let solver = track!(solver_recipe.create_solver(problem_spec.clone()))?;

        let study_budget =
            Budget::new(budget_factor as u64 * problem_spec.evaluation_expense.get());

        let study_record = track!(StudyRecord::new(
            solver_recipe,
            problem_recipe,
            study_budget.amount,
            problem_spec.clone(),
            solver.specification(),
        ))?;

        Ok(Self {
            rng: rand::thread_rng(),
            idgen: SerialIdGenerator::new(),
            solver,
            problem,
            study_budget,
            study_record,
            watch: Stopwatch::new(),
            errors: 0,
            evaluation_expense: problem_spec.evaluation_expense.get(),
            scheduler: Scheduler::new(concurrency),
        })
    }
}
impl<S, P, R: Rng> Runner<S, P, R>
where
    S: Solver,
    P: Problem,
{
    pub fn run(mut self) -> Result<StudyRecord> {
        while !self.study_budget.is_consumed() {
            while self.scheduler.has_idle_thread() {
                track!(self.ask())?;
            }
            track!(self.evaluate())?;
        }

        // TODO
        // for trial in self.scheduler.runnables.drain(..) {
        // }
        // peindings
        // threads.trial

        Ok(self.study_record)
    }

    // TODO: rename
    fn ask(&mut self) -> Result<()> {
        let (ask, obs) = track!(AskRecord::with(&self.watch.clone(), || self
            .solver
            .ask(&mut self.rng, &mut self.idgen)))?;
        if let Some(mut trial) = self.scheduler.pendings.remove(&obs.id) {
            trial.obs = obs;
            self.scheduler.runnables.push_back(trial);
        } else {
            let evaluator = track!(self.problem.create_evaluator(obs.id))?;
            let record = TrialRecord { ask, evals: vec![] };
            let trial = Trial {
                evaluator,
                obs,
                record,
            };
            self.scheduler.runnables.push_back(trial);
        }

        for thread in &mut self.scheduler.threads {
            if thread.trial.is_none() {
                if let Some(trial) = self.scheduler.runnables.pop_front() {
                    thread.trial = Some(trial);
                }
            }
        }
        Ok(())
    }

    fn evaluate(&mut self) -> Result<()> {
        self.scheduler.threads.sort_by_key(|t| t.priority());
        let mut trial = track_assert_some!(self.scheduler.threads[0].trial.take(), ErrorKind::Bug);
        let mut trial_budget = trial.obs.param.budget();

        // TODO: debug!(..)
        eprintln!(
            "# Thread[{}]: budget={:?}",
            self.scheduler.threads[0].id, trial_budget
        );
        let eval_result = EvalRecord::with(
            &self.watch,
            self.study_budget.consumption,
            &mut trial_budget,
            |budget| track!(trial.evaluator.evaluate(trial.obs.param.get(), budget)),
        );
        match eval_result {
            Ok((eval, values)) => {
                self.errors = 0;

                self.study_budget.consumption += eval.cost();
                self.scheduler.threads[0].elapsed += eval.cost();

                *trial.obs.param.budget_mut() = trial_budget;
                let obs = trial.obs.clone().map_value(|()| values);
                track!(self.solver.tell(obs))?;

                trial.record.evals.push(eval);
                if trial_budget.consumption >= self.evaluation_expense {
                    self.study_record.trials.push(trial.record);
                } else {
                    track_assert!(trial_budget.is_consumed(), ErrorKind::Other; trial_budget);
                    self.scheduler.pendings.insert(trial.obs.id, trial);
                }
            }
            Err(e) => {
                // TODO
                eprintln!("# Error: {}", e);
                self.errors += 1;
                if self.errors > 1000 {
                    return Err(track!(e));
                }

                self.study_record.trials.push(trial.record);
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
pub struct RunSpec<'a> {
    pub solver: &'a KurobakoSolverRecipe,
    pub problem: &'a FullKurobakoProblemRecipe,
    pub budget: usize,
}
