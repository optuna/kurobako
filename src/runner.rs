use crate::record::{AskRecord, EvaluateRecord, StudyRecord, TellRecord, TrialRecord};
use crate::time::ElapsedSeconds;
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe};
use kurobako_core::solver::{Solver, SolverRecipe, UnobservedObs};
use kurobako_core::{ErrorKind, Result};
use rand;
use rand::rngs::ThreadRng;
use serde::{Deserialize, Serialize};
use std;
use std::collections::HashMap;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::{ObsId, SerialIdGenerator};

#[derive(Debug, Clone, PartialEq, Eq, Hash, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudyRunnerOptions {
    #[structopt(long, default_value = "20")]
    pub budget: u64,

    #[structopt(long, default_value = "1")]
    pub concurrency: usize,

    #[structopt(long)]
    pub evaluation_points: Vec<u64>,
}

#[derive(Debug)]
pub struct StudyRunner<S, P>
where
    P: Problem,
{
    rng: ThreadRng,
    idgen: SerialIdGenerator,
    solver: S,
    problem: P,
    study_record: StudyRecord,
    study_budget: Budget,
    scheduler: TrialThreadScheduler<P::Evaluator>,
}
impl<S, P> StudyRunner<S, P>
where
    S: Solver,
    P: Problem,
{
    pub fn new<SR, PR>(
        solver_recipe: &SR,
        problem_recipe: &PR,
        options: &StudyRunnerOptions,
    ) -> Result<Self>
    where
        SR: SolverRecipe<Solver = S>,
        PR: ProblemRecipe<Problem = P>,
    {
        let problem = track!(problem_recipe.create_problem())?;
        let problem_spec = problem.specification();

        let solver = track!(solver_recipe.create_solver(problem_spec.clone()))?;
        let solver_spec = solver.specification();

        let study_budget =
            Budget::new(options.budget as u64 * problem_spec.evaluation_expense.get());

        let study_record = track!(StudyRecord::new(
            solver_recipe,
            solver_spec,
            problem_recipe,
            problem_spec,
            options.clone()
        ))?;

        Ok(Self {
            rng: rand::thread_rng(),
            idgen: SerialIdGenerator::new(),
            solver,
            problem,
            study_record,
            study_budget,
            scheduler: TrialThreadScheduler::new(options),
        })
    }

    pub fn run(mut self) -> Result<StudyRecord> {
        while !self.study_budget.is_consumed() {
            while let Some(mut thread) = self.scheduler.pop_idle_thread() {
                let trial = track!(self.ask_trial())?;
                thread.trial = Some(trial);
                self.scheduler.threads.push(thread);
            }

            track!(self.evaluate_trial())?;
        }

        self.study_record.finish();
        Ok(self.study_record)
    }

    fn ask_trial(&mut self) -> Result<TrialState<P::Evaluator>> {
        let (result, elapsed) =
            ElapsedSeconds::time(|| track!(self.solver.ask(&mut self.rng, &mut self.idgen)));
        let mut obs = result?;
        obs.param.budget_mut().amount = self.next_evaluation_point(obs.param.budget().amount);

        let evaluator = if let Some(evaluator) = self.scheduler.pendings.remove(&obs.id) {
            evaluator
        } else {
            track!(self.problem.create_evaluator(obs.id))?
        };

        let params = obs.param.get().clone();
        Ok(TrialState {
            obs,
            evaluator,
            ask: AskRecord { params, elapsed },
        })
    }

    fn evaluate_trial(&mut self) -> Result<()> {
        self.scheduler.threads.sort_by_key(|t| t.priority());

        let thread = &mut self.scheduler.threads[0];
        let mut trial = track_assert_some!(thread.trial.take(), ErrorKind::Bug);
        let mut trial_budget = trial.obs.param.budget();

        debug!("Thread[{}]: budget={:?}", thread.id, trial_budget);

        let (result, elapsed) = ElapsedSeconds::time(|| {
            track!(trial
                .evaluator
                .evaluate(trial.obs.param.get(), &mut trial_budget))
        });
        let expense = trial_budget.consumption - trial.obs.param.budget().consumption;

        match result {
            Ok(values) => {
                self.study_budget.consumption += expense;
                thread.budget_consumption += expense;
                *trial.obs.param.budget_mut() = trial_budget;
                let evaluate = EvaluateRecord {
                    values: values.clone(),
                    elapsed,
                    expense,
                };

                let obs = trial.obs.clone().map_value(|()| values);
                let solver = &mut self.solver;
                let (result, elapsed) = ElapsedSeconds::time(|| track!(solver.tell(obs)));
                result?;
                let tell = TellRecord { elapsed };

                self.study_record.trials.push(TrialRecord {
                    thread_id: thread.id,
                    obs_id: trial.obs.id,
                    ask: trial.ask,
                    evaluate,
                    tell,
                });

                if trial_budget.consumption < self.trial_max_budget() {
                    track_assert!(trial_budget.is_consumed(), ErrorKind::Other; trial_budget);
                    self.scheduler
                        .pendings
                        .insert(trial.obs.id, trial.evaluator);
                }
            }
            Err(e) => {
                if *e.kind() == ErrorKind::UnevaluableParams {
                    self.study_record.unevaluable_trials += 1;
                    warn!(
                        "Unevaluable parameters ({}): {}",
                        self.study_record.unevaluable_trials, e
                    );
                    track_assert!(
                        self.study_record.unevaluable_trials < 10000,
                        ErrorKind::Other
                    );
                } else {
                    return Err(e);
                }
            }
        }
        Ok(())
    }

    fn trial_max_budget(&self) -> u64 {
        self.study_record.problem.spec.evaluation_expense.get()
    }

    fn next_evaluation_point(&self, candidate: u64) -> u64 {
        if self.study_record.runner.evaluation_points.is_empty() {
            candidate
        } else {
            self.study_record
                .runner
                .evaluation_points
                .iter()
                .cloned()
                .rev()
                .take_while(|p| candidate <= *p)
                .last()
                .unwrap_or_else(|| self.trial_max_budget())
        }
    }
}

#[derive(Debug)]
struct TrialThreadScheduler<E> {
    threads: Vec<TrialThread<E>>,
    pendings: HashMap<ObsId, E>,
}
impl<E> TrialThreadScheduler<E> {
    fn new(options: &StudyRunnerOptions) -> Self {
        Self {
            threads: (0..options.concurrency).map(TrialThread::new).collect(),
            pendings: HashMap::new(),
        }
    }

    fn pop_idle_thread(&mut self) -> Option<TrialThread<E>> {
        for i in 0..self.threads.len() {
            if self.threads[i].trial.is_none() {
                return Some(self.threads.swap_remove(i));
            }
        }
        None
    }
}

#[derive(Debug)]
struct TrialState<E> {
    obs: UnobservedObs,
    evaluator: E,
    ask: AskRecord,
}

#[derive(Debug)]
struct TrialThread<E> {
    id: usize,
    budget_consumption: u64,
    trial: Option<TrialState<E>>,
}
impl<E> TrialThread<E> {
    fn new(id: usize) -> Self {
        Self {
            id,
            budget_consumption: 0,
            trial: None,
        }
    }

    fn priority(&self) -> u64 {
        self.trial.as_ref().map_or(std::u64::MAX, |t| {
            t.obs.param.budget().amount + self.budget_consumption
        })
    }
}
