use crate::record::{
    EvaluationRecord, ProblemRecord, SolverRecord, TrialRecord, TrialRecordBuilder,
};
use crate::study::{Scheduling, StudyRecipe};
use crate::time::DateTime;
use chrono::Local;
use kurobako_core::num::OrderedFloat;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::SolverSpec;
use kurobako_core::trial::TrialId;
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::num::NonZeroUsize;
use std::time::Duration;

#[derive(Debug)]
pub struct StudyRecordBuilder {
    recipe: StudyRecipe,
    solver: SolverSpec,
    problem: ProblemSpec,
    start_time: DateTime,
    trials: BTreeMap<TrialId, TrialRecord>,
}
impl StudyRecordBuilder {
    pub fn new(recipe: StudyRecipe, solver: SolverSpec, problem: ProblemSpec) -> Self {
        Self {
            recipe,
            solver,
            problem,
            start_time: Local::now(),
            trials: BTreeMap::new(),
        }
    }

    pub fn add_trial(&mut self, trial: TrialRecordBuilder) {
        let t = self.trials.entry(trial.id).or_insert_with(|| TrialRecord {
            thread_id: trial.thread_id,
            params: trial.params.clone(),
            evaluations: Vec::new(),
        });
        t.evaluations.push(EvaluationRecord {
            values: trial.values,
            start_step: trial.start_step,
            end_step: trial.end_step,
            ask_elapsed: trial.ask_elapsed,
            tell_elapsed: trial.tell_elapsed,
            evaluate_elapsed: trial.evaluate_elapsed,
        });
    }

    pub fn finish(self) -> StudyRecord {
        StudyRecord {
            start_time: self.start_time,
            end_time: Local::now(),
            budget: self.recipe.budget,
            seed: self.recipe.seed.unwrap_or_else(|| unreachable!()),
            concurrency: self.recipe.concurrency,
            scheduling: self.recipe.scheduling,
            solver: SolverRecord {
                recipe: self.recipe.solver,
                spec: self.solver,
            },
            problem: ProblemRecord {
                recipe: self.recipe.problem,
                spec: self.problem,
            },
            trials: self.trials.into_iter().map(|(_, v)| v).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub seed: u64,
    pub budget: u64,
    pub concurrency: NonZeroUsize,
    pub scheduling: Scheduling,
    pub solver: SolverRecord,
    pub problem: ProblemRecord,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn id(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&track!(
            serde_json::to_vec(&self.budget).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.concurrency).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.scheduling).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.solver).map_err(Error::from)
        )?);
        hasher.input(&track!(
            serde_json::to_vec(&self.problem).map_err(Error::from)
        )?);

        let mut id = String::with_capacity(64);
        for b in hasher.result().as_slice() {
            track_write!(&mut id, "{:02x}", b)?;
        }
        Ok(id)
    }

    pub fn best_value(&self) -> Option<f64> {
        let problem_steps = self.problem.spec.steps.last();
        self.trials
            .iter()
            .filter_map(|t| t.value(problem_steps))
            .map(OrderedFloat)
            .min()
            .map(|x| x.0)
    }

    pub fn auc(&self, start_step: u64) -> Option<f64> {
        let vars = self.problem.spec.values_domain.variables();
        if vars.len() != 1 {
            return None;
        }

        let mut global_min = vars[0].range().low();
        if !global_min.is_finite() {
            global_min = 0.0;
        }

        let problem_steps = self.problem.spec.steps.last();
        let mut trials = self
            .trials
            .iter()
            .filter_map(|t| {
                if let (Some(step), Some(value)) = (t.end_step(), t.value(problem_steps)) {
                    Some((step, value))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        trials.sort_by_key(|t| t.0);

        let mut prev_step = 0;
        let mut current_min = std::f64::INFINITY;
        let mut auc = 0.0;
        for (mut step, value) in trials {
            if step <= start_step {
                step = start_step;
            } else {
                if prev_step == 0 {
                    return None;
                }
                auc += (current_min - global_min) * (step - prev_step) as f64;
            }

            if value < current_min {
                current_min = value;
            }
            prev_step = step;
        }

        let study_steps = self.budget * problem_steps;
        auc += (current_min - global_min) * (study_steps - prev_step) as f64;

        Some(auc)
    }

    pub fn solver_elapsed(&self) -> Duration {
        self.trials.iter().map(|t| t.solver_elapsed()).sum()
    }

    pub fn first_complete_trial(&self) -> Option<&TrialRecord> {
        let problem_steps = self.problem.spec.steps.last();
        self.trials
            .iter()
            .filter(|t| t.steps() != problem_steps)
            .min_by_key(|t| t.start_step())
    }
}
