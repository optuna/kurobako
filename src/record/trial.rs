use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::ParamValue;
use kurobako_core::time::Elapsed;
use serde::{Deserialize, Serialize};
use yamakan::observation::ObsId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub thread_id: usize,
    pub obs_id: ObsId,
    pub ask: AskRecord,
    pub evaluate: EvaluateRecord,
    pub tell: TellRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskRecord {
    pub params: Vec<ParamValue>,
    pub elapsed: Elapsed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluateRecord {
    pub values: Vec<FiniteF64>,
    pub elapsed: Elapsed,
    pub expense: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TellRecord {
    pub elapsed: Elapsed,
}
