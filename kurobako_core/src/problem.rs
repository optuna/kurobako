use crate::parameter::{ParamDomain, ParamValue};
use crate::Result;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;

pub trait Evaluate {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Vec<f64>>;
}

pub trait Problem {
    type Evaluator: Evaluate;

    fn params_domain(&self) -> Vec<ParamDomain>;
    fn values_domain(&self) -> Vec<MinMax<f64>>;
    fn evaluation_expense(&self) -> NonZeroU64;
    fn create_evaluator(&mut self) -> Result<Self::Evaluator>;
}

pub trait ProblemSpec: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn create_problem(&self) -> Result<Self::Problem>;
}
