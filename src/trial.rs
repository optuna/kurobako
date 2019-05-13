use crate::time::{Stopwatch, Timestamp};
use kurobako_core::parameter::ParamValue;
use kurobako_core::solver::UnobservedObs;
use kurobako_core::Result;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use yamakan::budget::Budget;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrialRecord {
    pub ask: AskRecord,
    pub evals: Vec<EvalRecord>,
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

    pub fn consumption(&self) -> u64 {
        self.evals.iter().map(|e| e.cost()).sum()
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
    pub start_budget: u64,
    pub end_budget: u64,
    pub start_time: Timestamp,
    pub end_time: Timestamp,
}
impl EvalRecord {
    pub fn with<F>(
        watch: &Stopwatch,
        start_budget: u64,
        budget: &mut Budget,
        f: F,
    ) -> Result<(Self, Vec<FiniteF64>)>
    where
        F: FnOnce(&mut Budget) -> Result<Vec<FiniteF64>>,
    {
        let before_consumption = budget.consumption;
        let start_time = watch.elapsed();
        let values = f(budget)?;
        let end_time = watch.elapsed();
        let cost = budget.consumption - before_consumption;
        let this = Self {
            value: values[0].get(), // TODO
            start_budget,
            end_budget: start_budget + cost,
            start_time,
            end_time,
        };
        Ok((this, values))
    }

    pub fn cost(&self) -> u64 {
        self.end_budget - self.start_budget
    }
}
