use crate::problem::KurobakoProblemRecipe;
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{
    BoxProblem, Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{json, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct HomonymProblemRecipe {
    #[structopt(long)]
    pub problems: Vec<json::JsonValue>,
}
impl ProblemRecipe for HomonymProblemRecipe {
    type Problem = HomonymProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let problems: Vec<KurobakoProblemRecipe> = self
            .problems
            .iter()
            .map(|p| track!(serde_json::from_value(p.get().clone()).map_err(Error::from)))
            .collect::<Result<_>>()?;

        panic!()
    }
}

#[derive(Debug)]
pub struct HomonymProblem {}
impl Problem for HomonymProblem {
    type Evaluator = HomonymEvaluator;

    fn specification(&self) -> ProblemSpec {
        panic!()
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        panic!()
    }
}

#[derive(Debug)]
pub struct HomonymEvaluator {}
impl Evaluate for HomonymEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        panic!()
    }
}
