use crate::distribution::Distribution;
use crate::problem::ProblemSpace;
use failure::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan;
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::spaces::F64;
use yamakan::Optimizer;

pub trait OptimizerBuilder: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Optimizer: Optimizer<Param = Vec<f64>, Value = f64>;

    fn optimizer_name(&self) -> &str;
    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error>;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum OptimizerSpec {
    Random(RandomOptimizerBuilder),
}
impl OptimizerBuilder for OptimizerSpec {
    type Optimizer = UnionOptimizer;

    fn optimizer_name(&self) -> &str {
        match self {
            OptimizerSpec::Random(x) => x.optimizer_name(),
        }
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        match self {
            OptimizerSpec::Random(x) => x.build(problem_space).map(UnionOptimizer::Random),
        }
    }
}

// TODO: rename
#[derive(Debug)]
pub enum UnionOptimizer {
    Random(RandomOptimizer),
}
impl Optimizer for UnionOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        match self {
            UnionOptimizer::Random(x) => x.ask(rng),
        }
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        match self {
            UnionOptimizer::Random(x) => x.tell(param, value),
        }
    }
}

#[derive(Debug)]
pub struct RandomOptimizer {
    inner: Vec<InnerRandomOptimizer<F64, f64>>,
}
impl Optimizer for RandomOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        self.inner.iter_mut().map(|o| o.ask(rng)).collect()
    }

    fn tell(&mut self, _param: Self::Param, _value: Self::Value) {}
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct RandomOptimizerBuilder {}
impl OptimizerBuilder for RandomOptimizerBuilder {
    type Optimizer = RandomOptimizer;

    fn optimizer_name(&self) -> &str {
        "random"
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        let inner = problem_space
            .distributions()
            .iter()
            .map(|d| {
                let Distribution::Uniform { low, high } = *d;
                InnerRandomOptimizer::new(F64 { low, high })
            })
            .collect();
        Ok(RandomOptimizer { inner })
    }
}

// TODO: ExternalProcessOptimizer
