use crate::time::{Stopwatch, Timestamp};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub ask: AskRecord,
    pub evals: Vec<EvalRecord>,
    pub complete: bool,
}
impl TrialRecord {
    pub fn value(&self) -> Option<f64> {
        self.evals.last().map(|x| x.value)
    }

    pub fn end_time(&self) -> Timestamp {
        self.evals
            .last()
            .map_or(Timestamp::new(0.0), |x| x.end_time)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskRecord {
    pub params: Vec<f64>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl AskRecord {
    pub fn with<F>(watch: &Stopwatch, f: F) -> Self
    where
        F: FnOnce() -> Vec<f64>,
    {
        let start_time = watch.elapsed();
        let params = f();
        let end_time = watch.elapsed();
        Self {
            params,
            start_time,
            end_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalRecord {
    pub value: f64,
    pub cost: usize,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl EvalRecord {
    pub fn with<F>(watch: &Stopwatch, f: F) -> Self
    where
        F: FnOnce() -> f64,
    {
        let start_time = watch.elapsed();
        let value = f();
        let end_time = watch.elapsed();
        Self {
            value,
            cost: 1, // TODO
            start_time,
            end_time,
        }
    }
}
