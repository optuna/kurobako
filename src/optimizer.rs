use crate::distribution::Distribution;
use crate::float::NonNanF64;
use crate::problems::ProblemSpace;
use failure::Error;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan;
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::optimizers::tpe;
use yamakan::spaces::F64;
use yamakan::Optimizer;

pub use self::external_command::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder};
pub use self::optuna::{OptunaOptimizer, OptunaOptimizerBuilder};

mod external_command;
mod optuna;

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
    Tpe(TpeOptimizerBuilder),
    Optuna(OptunaOptimizerBuilder),
    Command(ExternalCommandOptimizerBuilder),
}
impl OptimizerBuilder for OptimizerSpec {
    type Optimizer = UnionOptimizer;

    fn optimizer_name(&self) -> &str {
        match self {
            OptimizerSpec::Random(x) => x.optimizer_name(),
            OptimizerSpec::Tpe(x) => x.optimizer_name(),
            OptimizerSpec::Optuna(x) => x.optimizer_name(),
            OptimizerSpec::Command(x) => x.optimizer_name(),
        }
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        match self {
            OptimizerSpec::Random(x) => x.build(problem_space).map(UnionOptimizer::Random),
            OptimizerSpec::Tpe(x) => x.build(problem_space).map(UnionOptimizer::Tpe),
            OptimizerSpec::Optuna(x) => x.build(problem_space).map(UnionOptimizer::Optuna),
            OptimizerSpec::Command(x) => x.build(problem_space).map(UnionOptimizer::Command),
        }
    }
}

// TODO: rename
#[derive(Debug)]
pub enum UnionOptimizer {
    Random(RandomOptimizer),
    Tpe(TpeOptimizer),
    Optuna(OptunaOptimizer),
    Command(ExternalCommandOptimizer),
}
impl Optimizer for UnionOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        match self {
            UnionOptimizer::Random(x) => x.ask(rng),
            UnionOptimizer::Tpe(x) => x.ask(rng),
            UnionOptimizer::Optuna(x) => x.ask(rng),
            UnionOptimizer::Command(x) => x.ask(rng),
        }
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        match self {
            UnionOptimizer::Random(x) => x.tell(param, value),
            UnionOptimizer::Tpe(x) => x.tell(param, value),
            UnionOptimizer::Optuna(x) => x.tell(param, value),
            UnionOptimizer::Command(x) => x.tell(param, value),
        }
    }
}

#[derive(Debug)]
pub struct TpeOptimizer {
    inner: Vec<tpe::TpeNumericalOptimizer<F64, NonNanF64>>,
}
impl Optimizer for TpeOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        self.inner.iter_mut().map(|o| o.ask(rng)).collect()
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        if value.is_nan() {
            return;
        }

        let value = NonNanF64::new(value);
        for (p, o) in param.into_iter().zip(self.inner.iter_mut()) {
            o.tell(p, value);
        }
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct TpeOptimizerBuilder {
    // TODO: options
}
impl OptimizerBuilder for TpeOptimizerBuilder {
    type Optimizer = TpeOptimizer;

    fn optimizer_name(&self) -> &str {
        "TPE"
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        let inner = problem_space
            .distributions()
            .iter()
            .map(|d| {
                let Distribution::Uniform { low, high } = *d;
                tpe::TpeNumericalOptimizer::new(F64 { low, high })
            })
            .collect();
        Ok(TpeOptimizer { inner })
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
