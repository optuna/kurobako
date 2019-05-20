use super::{JsonValue, TrialRecord};
use crate::runner::StudyRunnerOptions;
use crate::time::DateTime;
use chrono::Local;
use kurobako_core::problem::{ProblemRecipe, ProblemSpec};
use kurobako_core::solver::{SolverRecipe, SolverSpec};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeAndSpec<T> {
    pub recipe: JsonValue, // TODO: FullKurobakoProblemRecipe or KurobakoSolverRecipe
    pub spec: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub solver: RecipeAndSpec<SolverSpec>,
    pub problem: RecipeAndSpec<ProblemSpec>,
    pub runner: StudyRunnerOptions,
    pub start_time: DateTime,
    pub end_time: Option<DateTime>,
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
            end_time: None,
            unevaluable_trials: 0,
            trials: Vec::new(),
        })
    }

    pub fn finish(&mut self) {
        assert!(self.end_time.is_none());
        self.end_time = Some(Local::now());
    }
}
