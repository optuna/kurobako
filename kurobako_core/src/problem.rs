use crate::parameter::{ParamDomain, ParamValue};
use crate::Result;
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

pub trait Evaluate {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Vec<FiniteF64>>;
}

pub trait Problem {
    type Evaluator: Evaluate;

    fn specification(&self) -> ProblemSpec;
    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator>;
}

pub trait ProblemRecipe: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn create_problem(&self) -> Result<Self::Problem>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProblemSpec {
    pub name: String,
    pub version: Option<String>,
    pub params_domain: Vec<ParamDomain>,
    pub values_domain: Vec<MinMax<FiniteF64>>,
    pub evaluation_expense: NonZeroU64,
    pub capabilities: BTreeSet<EvaluatorCapability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvaluatorCapability {
    Concurrent,
    DynamicParamChange,
}
