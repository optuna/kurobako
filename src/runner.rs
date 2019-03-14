use crate::optimizer::{OptimizerBuilder, OptimizerSpec};
use crate::problem::{Problem, ProblemSpec};
use crate::study::StudyRecord;
use crate::time::Stopwatch;
use crate::trial::{AskRecord, EvalRecord, TrialRecord};
use failure::Error;
use rand::rngs::ThreadRng;
use rand::{self, Rng};
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
        problem: &P,
        budget: usize,
    ) -> Result<StudyRecord, Error>
    where
        O: OptimizerBuilder,
        P: Problem,
    {
        let mut optimizer = optimizer_builder.build(&problem.problem_space())?;
        let mut study_record = StudyRecord::new(optimizer_builder, problem, budget)?;
        let watch = Stopwatch::new();
        for _ in 0..budget {
            let ask = AskRecord::with(&watch, || optimizer.ask(&mut self.rng));
            let eval = EvalRecord::with(&watch, || problem.evaluate(&ask.params));
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

#[derive(Debug, Serialize, Deserialize)]
pub struct RunSpec {
    pub optimizer: OptimizerSpec,
    pub problem: ProblemSpec,
    pub budget: usize,
}
