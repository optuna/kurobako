use crate::problem::KurobakoProblemRecipe;
use kurobako_core::json;
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{
    BoxProblem, Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct ConcatProblemRecipe {
    pub recipes: Vec<json::JsonValue>,
}
impl ProblemRecipe for ConcatProblemRecipe {
    type Problem = ConcatProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let problems: Vec<BoxProblem> = self
            .recipes
            .iter()
            .map(|r| {
                let recipe: KurobakoProblemRecipe =
                    track!(serde_json::from_value(r.get().clone()).map_err(Error::from))?;
                track!(recipe.create_problem())
            })
            .collect::<Result<_>>()?;
        Ok(ConcatProblem { problems })
    }
}

#[derive(Debug)]
pub struct ConcatProblem {
    problems: Vec<BoxProblem>,
}
impl Problem for ConcatProblem {
    type Evaluator = ConcatEvaluator;

    fn specification(&self) -> ProblemSpec {
        let specs = self
            .problems
            .iter()
            .map(|p| p.specification())
            .collect::<Vec<_>>();
        ProblemSpec {
            name: specs
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join("|"),
            version: None, // TOD
            params_domain: specs
                .iter()
                .flat_map(|s| s.params_domain.iter().cloned())
                .collect(),
            values_domain: specs
                .iter()
                .flat_map(|s| s.values_domain.iter().cloned())
                .collect(),
            evaluation_expense: NonZeroU64::new(1).unwrap(), // TODO
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(), // TODO
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        panic!()
    }
}

#[derive(Debug)]
pub struct ConcatEvaluator {}
impl Evaluate for ConcatEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        panic!()
    }
}
