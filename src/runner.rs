use crate::optimizer::{OptimizerBuilder, OptimizerSpec};
use crate::problems::BuiltinProblemSpec;
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use crate::{Evaluate, Problem, ProblemSpec};
use failure::Error;
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use yamakan::budget::Budget;
use yamakan::Optimizer;

#[derive(Debug)]
pub struct Runner<R = ThreadRng> {
    rng: R,
}
impl Runner<ThreadRng> {
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }
}
impl<R: Rng> Runner<R> {
    pub fn run<O, P>(
        &mut self,
        optimizer_builder: &O,
        problem_spec: &P,
        budget: usize,
    ) -> Result<StudyRecord, Error>
    where
        O: OptimizerBuilder,
        P: ProblemSpec,
    {
        let mut problem = problem_spec.make_problem()?;
        let mut optimizer = optimizer_builder.build(&problem.problem_space())?;
        let mut study_record = StudyRecord::new(
            optimizer_builder,
            problem_spec,
            budget,
            problem.value_range(),
        )?;
        let watch = Stopwatch::new();
        for _ in 0..budget {
            let ask = AskRecord::with(&watch, || optimizer.ask(&mut self.rng));
            let mut evaluator = problem.make_evaluator(&ask.params)?;
            let mut budget = Budget::new(1);
            let eval = EvalRecord::with(&watch, || evaluator.evaluate(&mut budget).expect("TODO"));
            optimizer.tell(ask.params.clone(), eval.value);

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
