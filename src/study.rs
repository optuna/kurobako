use crate::float::NonNanF64;
use crate::optimizer::OptimizerBuilder;
use crate::time::DateTime;
use crate::trial::TrialRecord;
use crate::Name;
use crate::ProblemSpec;
use chrono::Local;
use failure::Error;
use kurobako_core::ValueRange;
use std::f64;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub optimizer: Name,
    pub problem: Name,
    pub budget: usize,
    pub value_range: ValueRange,
    pub start_time: DateTime,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn new<O, P>(
        optimizer_builder: &O,
        problem: &P,
        budget: usize,
        value_range: ValueRange,
    ) -> Result<Self, Error>
    where
        O: OptimizerBuilder,
        P: ProblemSpec,
    {
        Ok(StudyRecord {
            optimizer: Name::new(serde_json::to_value(optimizer_builder)?),
            problem: Name::new(serde_json::to_value(problem)?),
            budget,
            value_range,
            start_time: Local::now(),
            trials: Vec::new(),
        })
    }

    pub fn best_score(&self) -> f64 {
        let normalized_value = self
            .trials
            .iter()
            .filter_map(|t| t.value())
            .min_by_key(|v| NonNanF64::new(*v))
            .map(|v| self.value_range.normalize(v))
            .expect("TODO");
        1.0 - normalized_value
    }

    pub fn auc(&self) -> f64 {
        let mut vs = Vec::new();
        for v in self
            .trials
            .iter()
            .filter_map(|t| t.value())
            .map(|v| self.value_range.normalize(v))
        {
            if vs.is_empty() || Some(&v) < vs.last() {
                vs.push(v);
            }
        }
        (vs.len() as f64 - vs.iter().sum::<f64>()) / (vs.len() as f64)
    }

    pub fn ack_latencies<'a>(&'a self) -> impl Iterator<Item = f64> + 'a {
        self.trials.iter().map(|t| t.ask.latency())
    }

    pub fn best_trial(&self) -> Option<&TrialRecord> {
        self.trials
            .iter()
            .filter(|t| t.value().is_some())
            .min_by_key(|t| NonNanF64::new(t.value().expect("never fails")))
    }

    pub fn elapsed_time(&self) -> f64 {
        self.trials
            .last()
            .map_or(0.0, |t| t.end_time().as_seconds())
    }
}
