use super::{JsonValue, TrialRecord};
use crate::runner::StudyRunnerOptions;
use crate::time::DateTime;
use chrono::Local;
use kurobako_core::num::FiniteF64;
use kurobako_core::problem::{ProblemRecipe, ProblemSpec};
use kurobako_core::solver::{SolverRecipe, SolverSpec};
use kurobako_core::{Error, Result};
use rustats::fundamental::average;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAndSpec<T> {
    pub spec: T,
    pub recipe: JsonValue, // TODO: KurobakoProblemRecipe or KurobakoSolverRecipe
}
impl RecipeAndSpec<ProblemSpec> {
    pub fn id(&self) -> Id {
        Id {
            name: &self.spec.name,
            version: self.spec.version.as_ref().map(|s| s.as_str()),
            recipe: &self.recipe,
        }
    }
}
impl RecipeAndSpec<SolverSpec> {
    pub fn id(&self) -> Id {
        Id {
            name: &self.spec.name,
            version: self.spec.version.as_ref().map(|s| s.as_str()),
            recipe: &self.recipe,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Id<'a> {
    pub name: &'a str,
    pub version: Option<&'a str>,
    pub recipe: &'a JsonValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub solver: RecipeAndSpec<SolverSpec>,
    pub problem: RecipeAndSpec<ProblemSpec>,
    pub runner: StudyRunnerOptions,
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub unevaluable_trials: usize,
    pub trials: Vec<TrialRecord>,
}
impl StudyRecord {
    pub fn new<O, P>(
        solver_recipe: &O,
        solver_spec: SolverSpec,
        problem_recipe: &P,
        problem_spec: ProblemSpec,
        runner: StudyRunnerOptions,
    ) -> Result<Self>
    where
        O: SolverRecipe,
        P: ProblemRecipe,
    {
        let solver = RecipeAndSpec {
            recipe: JsonValue::new(track!(
                serde_json::to_value(solver_recipe).map_err(Error::from)
            )?),
            spec: solver_spec,
        };
        let problem = RecipeAndSpec {
            recipe: JsonValue::new(track!(
                serde_json::to_value(problem_recipe).map_err(Error::from)
            )?),
            spec: problem_spec,
        };
        Ok(StudyRecord {
            solver,
            problem,
            runner,
            start_time: Local::now(),
            end_time: Local::now(), // dummy value
            unevaluable_trials: 0,
            trials: Vec::new(),
        })
    }

    pub fn finish(&mut self) {
        self.end_time = Local::now();
    }

    pub fn study_budget(&self) -> u64 {
        self.runner.budget * self.trial_budget()
    }

    pub fn trial_budget(&self) -> u64 {
        self.problem.spec.evaluation_expense.get()
    }

    pub fn intermediate_trials(&self, budget: u64) -> impl Iterator<Item = TrialRecord> {
        let mut trials = HashMap::<ObsId, TrialRecord>::new();
        let mut target_trials = Vec::new();
        for trial in &self.trials {
            if let Some(t) = trials.get_mut(&trial.obs_id) {
                t.evaluate.expense += trial.evaluate.expense;
            } else {
                trials.insert(trial.obs_id, trial.clone());
            }

            if trials[&trial.obs_id].evaluate.expense == budget {
                target_trials.push(trials[&trial.obs_id].clone());
            }
        }

        target_trials.into_iter()
    }

    pub fn scorer(&self) -> Scorer {
        Scorer::new(self)
    }

    pub fn complete_trials(&self) -> impl Iterator<Item = (u64, TrialRecord)> {
        let mut trials = HashMap::<ObsId, TrialRecord>::new();
        let mut completed = Vec::new();
        let mut consumption = 0;
        for trial in &self.trials {
            consumption += trial.evaluate.expense;
            if let Some(t) = trials.get_mut(&trial.obs_id) {
                t.evaluate.expense += trial.evaluate.expense;
            // TODO: adjust other fields
            } else {
                trials.insert(trial.obs_id, trial.clone());
            }

            if trials[&trial.obs_id].evaluate.expense >= self.trial_budget() {
                completed.push((consumption, trials[&trial.obs_id].clone()));
            }
        }

        completed.into_iter()
    }

    pub fn best_value(&self) -> Option<FiniteF64> {
        self.scorer()
            .best_value(self.study_budget())
            .map(|v| FiniteF64::new(v).unwrap_or_else(|e| panic!("{}", e)))
    }

    pub fn auc(&self) -> Option<FiniteF64> {
        self.scorer().auc(self.study_budget())
    }
}

#[derive(Debug)]
pub struct Scorer {
    lower_bound: f64,
    bests: Vec<(u64, f64)>,
    // TODO:
    ask_wallclocks: Vec<(u64, f64)>,
}
impl Scorer {
    fn new(study: &StudyRecord) -> Self {
        let mut trials = HashMap::<ObsId, u64>::new();
        let mut consumption = 0;
        let mut bests: Vec<(u64, f64)> = Vec::new();
        let mut ask_wallclocks: Vec<(u64, f64)> = Vec::new();
        for trial in &study.trials {
            *trials.entry(trial.obs_id).or_default() += trial.evaluate.expense;
            consumption += trial.evaluate.expense;

            ask_wallclocks.push((consumption, trial.ask.elapsed.get()));
            if trials[&trial.obs_id] >= study.trial_budget() {
                let value = trial.evaluate.values[0].get();
                if bests.is_empty() || Some(value) <= bests.last().map(|t| t.1) {
                    let consumption = if bests.is_empty() { 0 } else { consumption }; // TODO: remove
                    bests.push((consumption, value));
                }
            }
        }

        Self {
            bests,
            lower_bound: study.problem.spec.values_domain[0].min().get(),
            ask_wallclocks,
        }
    }

    pub fn best_value(&self, budget: u64) -> Option<f64> {
        self.bests
            .iter()
            .take_while(|t| t.0 <= budget)
            .map(|t| t.1)
            .last()
    }

    pub fn ask_wallclock(&self, budget: u64) -> f64 {
        self.ask_wallclocks
            .iter()
            .take_while(|t| t.0 <= budget)
            .map(|t| t.1)
            .sum::<f64>()
    }

    pub fn auc(&self, budget: u64) -> Option<FiniteF64> {
        // TODO: change starting point (for trials that support pruning)
        let auc =
            average((0..budget).map(|i| {
                self.best_value(i).unwrap_or_else(|| unimplemented!()) - self.lower_bound
            }));
        Some(FiniteF64::new(auc).unwrap_or_else(|e| panic!("{}", e)))
    }
}
