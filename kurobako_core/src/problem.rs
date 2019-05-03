use crate::distribution::Distribution;
use crate::Result;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::Budget;

pub trait ProblemSpec: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn make_problem(&self) -> Result<Self::Problem>;
}

pub trait Problem {
    type Evaluator: Evaluate;

    fn problem_space(&self) -> ProblemSpace;
    fn evaluation_cost(&self) -> u64;
    fn value_range(&self) -> MinMax<f64>;
    fn make_evaluator(&mut self, params: &[f64]) -> Result<Option<Self::Evaluator>>;
}

pub trait Evaluate {
    fn evaluate(&mut self, budget: &mut Budget) -> Result<f64>;
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
