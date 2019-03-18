use crate::distribution::Distribution;
use failure::Fallible;
use serde::{Deserialize, Serialize};
use std::ops::Range;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::ParamSpace;

pub trait ProblemSpec: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn build(&self) -> Fallible<Self::Problem>;
}

pub trait Problem: Sized {
    fn name(&self) -> &str;
    fn problem_space(&self) -> ProblemSpace;
    fn evaluate(&self, params: &[f64], budget: &mut Budget) -> Fallible<Option<f64>>;
    fn try_close(&self) -> Fallible<Self>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemSpace(Vec<Distribution>);
impl ProblemSpace {
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
