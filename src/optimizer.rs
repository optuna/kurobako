use crate::problem::ProblemSpace;
use failure::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan;
use yamakan::spaces::UniformF64;
use yamakan::Optimizer;

pub trait OptimizerBuilder: StructOpt + Serialize + for<'a> Deserialize<'a> {
    const OPTIMIZER_NAME: &'static str;

    type Optimizer: Optimizer<Param = Vec<f64>, Value = f64>;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error>;
}

#[derive(Debug)]
pub struct RandomOptimizer {
    inner: yamakan::optimizers::random::RandomOptimizer<UniformF64, f64>,
    dim: usize,
}
impl Optimizer for RandomOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        (0..self.dim).map(|_| self.inner.ask(rng)).collect()
    }

    fn tell(&mut self, _param: Self::Param, _value: Self::Value) {}
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct RandomOptimizerBuilder {}
impl OptimizerBuilder for RandomOptimizerBuilder {
    const OPTIMIZER_NAME: &'static str = "random";

    type Optimizer = RandomOptimizer;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        panic!()
    }
}
