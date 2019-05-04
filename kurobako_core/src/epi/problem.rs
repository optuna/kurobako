use crate::parameter::ParamValue;
use crate::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec};
use crate::Result;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct ExternalProgramProblemRecipe {
    pub path: PathBuf,
    pub args: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub skip_lines: Option<usize>,
}
impl ProblemRecipe for ExternalProgramProblemRecipe {
    type Problem = ExternalProgramProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        panic!()
    }
}

#[derive(Debug)]
pub struct ExternalProgramProblem {}
impl Problem for ExternalProgramProblem {
    type Evaluator = ExternalProgramEvaluator;

    fn specification(&self) -> ProblemSpec {
        panic!()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        panic!()
    }
}

#[derive(Debug)]
pub struct ExternalProgramEvaluator {}
impl Evaluate for ExternalProgramEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Vec<FiniteF64>> {
        panic!()
    }
}
