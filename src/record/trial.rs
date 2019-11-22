use crate::time::ElapsedSeconds;
use kurobako_core::trial::{Params, TrialId, Values};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct TrialRecordBuilder {
    pub id: TrialId,
    pub thread_id: usize,
    pub params: Params,
    pub values: Values,
    pub start_step: u64,
    pub end_step: u64,
    pub ask_elapsed: ElapsedSeconds,
    pub tell_elapsed: ElapsedSeconds,
    pub evaluate_elapsed: ElapsedSeconds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub thread_id: usize,
    pub params: Params,
    pub evaluations: Vec<EvaluationRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub values: Values,
    pub start_step: u64,
    pub end_step: u64,
    pub ask_elapsed: ElapsedSeconds,
    pub tell_elapsed: ElapsedSeconds,
    pub evaluate_elapsed: ElapsedSeconds,
}
