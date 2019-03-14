use crate::optimizer::OptimizerBuilder;
use crate::problem::Problem;
use crate::time::DateTime;
use crate::trial::TrialRecord;
use chrono::Local;
use failure::Error;
use serde_json::Value as JsonValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub optimizer: JsonValue, // TODO: OptimizerSpec
    pub problem: JsonValue,   // TODO: ProblemSpec
    pub budget: usize,
    pub start_time: DateTime,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn new<O, P>(optimizer_builder: &O, problem: &P, budget: usize) -> Result<Self, Error>
    where
        O: OptimizerBuilder,
        P: Problem,
    {
        Ok(StudyRecord {
            optimizer: serde_json::to_value(optimizer_builder)?,
            problem: serde_json::to_value(problem)?,
            budget,
            start_time: Local::now(),
            trials: Vec::new(),
        })
    }
}
