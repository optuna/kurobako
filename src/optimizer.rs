use crate::distribution::Distribution;
use crate::float::NonNanF64;
use crate::{Error, ProblemSpace};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use structopt::StructOpt;
use yamakan::observation::{IdGenerator, Observation, ObservationId};
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::optimizers::tpe;
use yamakan::spaces::F64;
use yamakan::{self, Optimizer};

pub use self::external_command::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder};
pub use self::gpyopt::{GpyoptOptimizer, GpyoptOptimizerBuilder};
pub use self::optuna::{OptunaOptimizer, OptunaOptimizerBuilder};

mod external_command;
mod gpyopt;
mod optuna;

pub trait OptimizerBuilder: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Optimizer: Optimizer<Param = Vec<f64>, Value = f64>;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error>;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum OptimizerSpec {
    Random(RandomOptimizerBuilder),
    Tpe(TpeOptimizerBuilder),
    Optuna(OptunaOptimizerBuilder),
    Gpyopt(GpyoptOptimizerBuilder),
    Command(ExternalCommandOptimizerBuilder),
}
impl OptimizerBuilder for OptimizerSpec {
    type Optimizer = UnionOptimizer;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        match self {
            OptimizerSpec::Random(x) => x.build(problem_space).map(UnionOptimizer::Random),
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

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        match self {
            UnionOptimizer::Random(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Tpe(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Optuna(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Gpyopt(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Command(x) => track!(x.ask(rng, idgen)),
        }
    }

    fn tell(&mut self, o: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        match self {
            UnionOptimizer::Random(x) => track!(x.tell(o)),
            UnionOptimizer::Tpe(x) => track!(x.tell(o)),
            UnionOptimizer::Optuna(x) => track!(x.tell(o)),
            UnionOptimizer::Gpyopt(x) => track!(x.tell(o)),
            UnionOptimizer::Command(x) => track!(x.tell(o)),
        }
    }
}

// TODO: move
#[derive(Debug)]
struct ConstIdGenerator(ObservationId);
impl IdGenerator for ConstIdGenerator {
    fn generate(&mut self) -> yamakan::Result<ObservationId> {
        Ok(self.0)
    }
}

#[derive(Debug)]
pub struct TpeOptimizer {
    inner: Vec<tpe::TpeNumericalOptimizer<F64, NonNanF64>>,
}
impl Optimizer for TpeOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
        let params = self
            .inner
            .iter_mut()
            .map(|o| track!(o.ask(rng, &mut idgen)).map(|obs| obs.param))
            .collect::<yamakan::Result<_>>()?;
        Ok(Observation {
            id,
            param: params,
            value: (),
        })
    }

    fn tell(&mut self, obs: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        if obs.value.is_nan() {
            return Ok(());
        }

        let value = NonNanF64::new(obs.value);
        for (p, o) in obs.param.into_iter().zip(self.inner.iter_mut()) {
            let obs = Observation {
                id: obs.id,
                param: p,
                value,
            };
            track!(o.tell(obs))?;
        }
        Ok(())
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

    #[serde(skip_serializing_if = "is_false", default)]
    #[structopt(long)]
    pub uniform_weight: bool,

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
            uniform_weight: false,
            divide_factor: None,
        }
    }
}
impl OptimizerBuilder for TpeOptimizerBuilder {
    type Optimizer = TpeOptimizer;

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
                    .uniform_weight(self.uniform_weight)
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

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
        let params = self
            .inner
            .iter_mut()
            .map(|o| track!(o.ask(rng, &mut idgen)).map(|obs| obs.param))
            .collect::<yamakan::Result<_>>()?;
        Ok(Observation {
            id,
            param: params,
            value: (),
        })
    }

    fn tell(&mut self, _: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct RandomOptimizerBuilder {}
impl OptimizerBuilder for RandomOptimizerBuilder {
    type Optimizer = RandomOptimizer;

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
