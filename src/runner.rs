use crate::optimizer::{OptimizerBuilder, OptimizerSpec};
use crate::problems::BuiltinProblemSpec;
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use crate::{Error, Evaluate, Problem, ProblemSpec};
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{Observation, SerialIdGenerator};
use yamakan::Optimizer;

#[derive(Debug)]
pub struct Runner<R = ThreadRng> {
    rng: R,
    idgen: SerialIdGenerator,
}
impl Runner<ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            idgen: SerialIdGenerator::new(),
        }
    }
}
impl<R: Rng> Runner<R> {
    pub fn run<O, P>(
        &mut self,
        optimizer_builder: &O,
        problem_spec: &P,
        budget_factor: usize,
    ) -> Result<StudyRecord, Error>
    where
        O: OptimizerBuilder,
        P: ProblemSpec,
    {
        let mut problem = problem_spec.make_problem()?;
        let mut optimizer = optimizer_builder.build(&problem.problem_space())?;
        let mut budget = Budget::new(budget_factor as u64 * problem.evaluation_cost());

        let mut study_record = StudyRecord::new(
            optimizer_builder,
            problem_spec,
            budget.amount(),
            problem.value_range(),
        )?;

        let watch = Stopwatch::new();
        while budget.remaining() > 0 {
            eprintln!("  # {:?}", budget);
            let (ask, obs_id, mut opt_budget) =
                track!(AskRecord::with(&watch, || optimizer.ask(&mut self.rng, &mut self.idgen)))?;
            let mut evaluator =
                if let Some(evaluator) = track!(problem.make_evaluator(&ask.params))? {
                    evaluator
                } else {
                    eprintln!("[WARN] Invalid parameters");
                    continue;
                };

            let old_consumption = opt_budget.consumption();
            let eval = EvalRecord::with(&watch, || {
                evaluator.evaluate(&mut opt_budget).expect("TODO")
            });
            budget.consume(opt_budget.consumption() - old_consumption);

            let obs = Observation {
                id: obs_id,
                param: Budgeted::new(opt_budget, ask.params.clone()),
                value: eval.value,
            };
            track!(optimizer.tell(obs))?;

            study_record.trials.push(TrialRecord {
                ask,
                evals: vec![eval],
                complete: true,
            });
        }
        Ok(study_record)
    }
}

#[derive(Debug)]
pub struct RunSpec<'a> {
    pub optimizer: &'a OptimizerSpec,
    pub problem: &'a BuiltinProblemSpec,
    pub budget: usize,
}
