use crate::time::ElapsedSeconds;
use kurobako_core::trial::{Params, TrialId, Values};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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
impl TrialRecord {
    pub fn value(&self, step: u64) -> Option<f64> {
        let mut current_step = 0;
        for eval in &self.evaluations {
            current_step += eval.elapsed_steps();
            if current_step == step {
                if eval.values.len() == 1 {
                    return Some(eval.values[0]);
                } else {
                    break;
                }
            } else if current_step > step {
                break;
            }
        }
        None
    }

    pub fn solver_elapsed(&self) -> Duration {
        let mut d = Duration::default();
        for eval in &self.evaluations {
            d += eval.ask_elapsed.to_duration() + eval.tell_elapsed.to_duration();
        }
        d
    }

    pub fn steps(&self) -> u64 {
        if let (Some(start), Some(end)) = (self.start_step(), self.end_step()) {
            end - start
        } else {
            0
        }
    }

    pub fn start_step(&self) -> Option<u64> {
        self.evaluations.get(0).map(|e| e.start_step)
    }

    pub fn end_step(&self) -> Option<u64> {
        self.evaluations.last().map(|e| e.end_step)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationRecord {
    pub values: Values,
    pub start_step: u64,
    pub end_step: u64,
    pub ask_elapsed: ElapsedSeconds, // TODO: latency
    pub tell_elapsed: ElapsedSeconds,
    pub evaluate_elapsed: ElapsedSeconds,
}
impl EvaluationRecord {
    pub fn elapsed_steps(&self) -> u64 {
        self.end_step - self.start_step
    }
}
