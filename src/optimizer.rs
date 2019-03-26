use crate::distribution::Distribution;
use crate::float::NonNanF64;
use crate::{Error, ProblemSpace};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use structopt::StructOpt;
use yamakan;
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::optimizers::tpe;
use yamakan::spaces::F64;
use yamakan::Optimizer;

pub use self::external_command::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder};
pub use self::gpyopt::{GpyoptOptimizer, GpyoptOptimizerBuilder};
pub use self::optuna::{OptunaOptimizer, OptunaOptimizerBuilder};

mod external_command;
mod gpyopt;
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
    RandomNormal(RandomOptimizerBuilder),
    Tpe(TpeOptimizerBuilder),
    Optuna(OptunaOptimizerBuilder),
    Gpyopt(GpyoptOptimizerBuilder),
    Command(ExternalCommandOptimizerBuilder),
}
impl OptimizerBuilder for OptimizerSpec {
    type Optimizer = UnionOptimizer;

    fn optimizer_name(&self) -> &str {
        match self {
            OptimizerSpec::RandomNormal(x) => x.optimizer_name(),
            OptimizerSpec::Tpe(x) => x.optimizer_name(),
            OptimizerSpec::Optuna(x) => x.optimizer_name(),
            OptimizerSpec::Gpyopt(x) => x.optimizer_name(),
            OptimizerSpec::Command(x) => x.optimizer_name(),
        }
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        match self {
            OptimizerSpec::RandomNormal(x) => x.build(problem_space).map(UnionOptimizer::Random),
            OptimizerSpec::Tpe(x) => x.build(problem_space).map(UnionOptimizer::Tpe),
            OptimizerSpec::Optuna(x) => x.build(problem_space).map(UnionOptimizer::Optuna),
            OptimizerSpec::Gpyopt(x) => x.build(problem_space).map(UnionOptimizer::Gpyopt),
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
    Gpyopt(GpyoptOptimizer),
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
            UnionOptimizer::Gpyopt(x) => x.ask(rng),
            UnionOptimizer::Command(x) => x.ask(rng),
        }
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        match self {
            UnionOptimizer::Random(x) => x.tell(param, value),
            UnionOptimizer::Tpe(x) => x.tell(param, value),
            UnionOptimizer::Optuna(x) => x.tell(param, value),
            UnionOptimizer::Gpyopt(x) => x.tell(param, value),
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

fn is_24(n: &usize) -> bool {
    *n == 24
}

fn is_false(b: &bool) -> bool {
    !*b
}

fn default_ei_candidates() -> usize {
    24
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct TpeOptimizerBuilder {
    #[serde(skip_serializing_if = "is_24", default = "default_ei_candidates")]
    #[structopt(long, default_value = "24")]
    pub ei_candidates: usize,

    #[serde(skip_serializing_if = "is_false", default)]
    #[structopt(long)]
    pub prior_uniform: bool,

    #[serde(skip_serializing_if = "is_false", default)]
    #[structopt(long)]
    pub uniform_sigma: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub divide_factor: Option<f64>,
}
impl Default for TpeOptimizerBuilder {
    fn default() -> Self {
        Self {
            ei_candidates: 24,
            prior_uniform: false,
            uniform_sigma: false,
            divide_factor: None,
        }
    }
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
                let mut pp = ::yamakan::optimizers::tpe::DefaultPreprocessor::default();
                if let Some(x) = self.divide_factor {
                    pp.divide_factor = x;
                }
                let options = tpe::TpeOptions::new(pp)
                    .prior_uniform(self.prior_uniform)
                    .uniform_sigma(self.uniform_sigma)
                    .ei_candidates(NonZeroUsize::new(self.ei_candidates).expect("TODO"));
                tpe::TpeNumericalOptimizer::with_options(F64 { low, high }, options)
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

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
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
