use crate::distribution::Distribution;
use failure::Fallible;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::ParamSpace;

pub trait ProblemSpec: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn make_problem(&self) -> Fallible<Self::Problem>;
}

pub trait Problem {
    type Evaluator: Evaluate;

    fn problem_space(&self) -> ProblemSpace;
    fn evaluation_cost_hint(&self) -> usize;
    fn make_evaluator(&mut self, params: &[f64]) -> Fallible<Self::Evaluator>;
}

pub trait Evaluate {
    fn evaluate(&mut self, budget: &mut Budget) -> Fallible<f64>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSpace(Vec<Distribution>);
impl ProblemSpace {
    pub fn new(distributions: Vec<Distribution>) -> Self {
        Self(distributions)
    }

    pub fn distributions(&self) -> &[Distribution] {
        &self.0
    }
}
impl ParamSpace for ProblemSpace {
    type External = Vec<f64>;
    type Internal = Vec<f64>;

    fn internal_range(&self) -> Range<Self::Internal> {
        Range {
            start: self.0.iter().map(|d| d.low()).collect(),
            end: self.0.iter().map(|d| d.high()).collect(),
        }
    }

    fn internalize(&self, param: &Self::External) -> Self::Internal {
        param.clone()
    }

    fn externalize(&self, param: &Self::Internal) -> Self::External {
        param.clone()
    }
}
