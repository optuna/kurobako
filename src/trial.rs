use crate::time::Timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub ask: AskRecord,
    pub evals: Vec<EvalRecord>,
    pub complete: bool,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskRecord {
    pub params: Vec<f64>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRecord {
    pub value: f64,
    pub cost: usize,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
