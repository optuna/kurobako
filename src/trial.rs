use crate::time::{Stopwatch, Timestamp};
use kurobako_core::parameter::ParamValue;
use kurobako_core::solver::UnobservedObs;
use kurobako_core::Result;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};

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
    pub params: Vec<ParamValue>,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl AskRecord {
    pub fn with<F>(watch: &Stopwatch, f: F) -> Result<(Self, UnobservedObs)>
    where
        F: FnOnce() -> Result<UnobservedObs>,
    {
        let start_time = watch.elapsed();
        let obs = f()?;
        let end_time = watch.elapsed();
        let this = Self {
            params: obs.param.get().clone(),
            start_time,
            end_time,
        };
        Ok((this, obs))
    }

    pub fn latency(&self) -> f64 {
        self.end_time.as_seconds() - self.start_time.as_seconds()
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
    pub fn with<F>(watch: &Stopwatch, f: F) -> (Self, Vec<FiniteF64>)
    where
        F: FnOnce() -> Vec<FiniteF64>,
    {
        let start_time = watch.elapsed();
        let values = f();
        let end_time = watch.elapsed();
        let this = Self {
            value: values[0].get(), // TODO
            cost: 1,                // TODO
            start_time,
            end_time,
        };
        (this, values)
    }
}
