use crate::optimizer::{OptimizerBuilder, OptimizerSpec};
use crate::problems::BuiltinProblemRecipe;
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe};
use kurobako_core::Error;
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use rustats::num::FiniteF64;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{Obs, SerialIdGenerator};
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
        problem_recipe: &P,
        budget_factor: usize,
    ) -> Result<StudyRecord, Error>
    where
        O: OptimizerBuilder,
        P: ProblemRecipe,
    {
        let mut problem = problem_recipe.create_problem()?;
        let problem_spec = problem.specification();
        let mut optimizer = optimizer_builder.build(
            &problem_spec.params_domain,
            problem_spec.evaluation_expense.get(),
        )?;
        let mut budget = Budget::new(budget_factor as u64 * problem_spec.evaluation_expense.get());

        let mut study_record = StudyRecord::new(
            optimizer_builder,
            problem_recipe,
            budget.amount,
            problem_spec.values_domain[0], // TODO
        )?;

        let watch = Stopwatch::new();
        while !budget.is_consumed() {
            // eprintln!("  # {:?}", budget);
            let (ask, obs_id, mut opt_budget) =
                track!(AskRecord::with(&watch, || optimizer.ask(&mut self.rng, &mut self.idgen)))?;
            let mut evaluator = track!(problem.create_evaluator(obs_id))?;

            let old_consumption = opt_budget.consumption;
            let eval = EvalRecord::with(&watch, || {
                use kurobako_core::parameter::ParamValue;

                let params = ask
                    .params
                    .iter()
                    .map(|p| ParamValue::Continuous(FiniteF64::new(*p).expect("TODO")))
                    .collect::<Vec<_>>();
                evaluator
                    .evaluate(&params, &mut opt_budget)
                    .expect("TODO")
                    .values[0]
                    .get()
            });
            budget.consumption += opt_budget.consumption - old_consumption;

            let obs = Obs {
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
    pub problem: &'a BuiltinProblemRecipe,
    pub budget: usize,
}
