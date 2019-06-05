//! Synthetic test functions.
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec, Values};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

pub mod mfb;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum SyntheticProblemRecipe {
    Mfb(self::mfb::MfbProblemRecipe),
}
impl ProblemRecipe for SyntheticProblemRecipe {
    type Problem = SyntheticProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            SyntheticProblemRecipe::Mfb(r) => track!(r.create_problem()).map(SyntheticProblem::Mfb),
        }
    }
}

#[derive(Debug)]
pub enum SyntheticProblem {
    Mfb(self::mfb::MfbProblem),
}
impl Problem for SyntheticProblem {
    type Evaluator = SyntheticEvaluator;

    fn specification(&self) -> ProblemSpec {
        match self {
            SyntheticProblem::Mfb(p) => p.specification(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        match self {
            SyntheticProblem::Mfb(p) => track!(p.create_evaluator(id)).map(SyntheticEvaluator::Mfb),
        }
    }
}

#[derive(Debug)]
pub enum SyntheticEvaluator {
    Mfb(self::mfb::MfbEvaluator),
}
impl Evaluate for SyntheticEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        match self {
            SyntheticEvaluator::Mfb(e) => track!(e.evaluate(params, budget)),
        }
    }
}
