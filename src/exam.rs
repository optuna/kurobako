use crate::problem::KurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec, Values};
use kurobako_core::{json, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct ExamRecipe {
    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub solver: KurobakoSolverRecipe,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub problem: KurobakoProblemRecipe,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct ExamProblemRecipe {
    pub recipe: json::JsonValue,
}
impl ProblemRecipe for ExamProblemRecipe {
    type Problem = ExamProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let exam: ExamRecipe =
            track!(serde_json::from_value(self.recipe.get().clone()).map_err(Error::from))?;
        Ok(ExamProblem { exam })
    }
}

#[derive(Debug)]
pub struct ExamProblem {
    exam: ExamRecipe,
}
impl Problem for ExamProblem {
    type Evaluator = ExamEvaluator;

    fn specification(&self) -> ProblemSpec {
        panic!()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        panic!()
    }
}

#[derive(Debug)]
pub struct ExamEvaluator {}
impl Evaluate for ExamEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        panic!()
    }
}
