use crate::float::NonNanF64;
use crate::optimizer::{
    OptimizerBuilder, RandomOptimizerBuilder, RandomOptimizerNoBudget, TpeOptimizerBuilder,
    TpeOptimizerNoBudget,
};
use crate::{ErrorKind, ProblemSpace, Result};
use rand::Rng;
use std::num::NonZeroUsize;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGenerator, Observation};
use yamakan::optimizers::asha as inner;
use yamakan::optimizers::asha::RungValue;
use yamakan::{self, Optimizer};

fn is_1(&n: &usize) -> bool {
    n == 1
}

fn is_0(&n: &usize) -> bool {
    n == 0
}

fn is_4(&n: &usize) -> bool {
    n == 4
}

fn is_16(&n: &usize) -> bool {
    n == 16
}

fn default_r() -> usize {
    1
}

fn default_s() -> usize {
    4
}

fn default_max_suspended() -> usize {
    16
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct AshaOptions {
    #[structopt(long, default_value = "1")]
    #[serde(default = "default_r")]
    #[serde(skip_serializing_if = "is_1")]
    pub r: usize,

    #[structopt(long, default_value = "4")]
    #[serde(default = "default_s")]
    #[serde(skip_serializing_if = "is_4")]
    pub s: usize,

    #[structopt(long, default_value = "0")]
    #[serde(default)]
    #[serde(skip_serializing_if = "is_0")]
    pub eta: usize,

    #[structopt(long, default_value = "16")]
    #[serde(default = "default_max_suspended")]
    #[serde(skip_serializing_if = "is_16")]
    pub max_suspended: usize,
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum AshaOptimizerSpec {
    Random {
        #[structopt(flatten)]
        #[serde(flatten)]
        asha: AshaOptions,

        #[structopt(flatten)]
        #[serde(flatten)]
        random: RandomOptimizerBuilder,
    },
    Tpe {
        #[structopt(flatten)]
        #[serde(flatten)]
        asha: AshaOptions,

        #[structopt(flatten)]
        #[serde(flatten)]
        tpe: TpeOptimizerBuilder,
    },
}
impl AshaOptimizerSpec {
    fn asha_options(&self) -> Result<inner::AshaOptions> {
        match self {
            AshaOptimizerSpec::Random { asha, .. } | AshaOptimizerSpec::Tpe { asha, .. } => {
                Ok(inner::AshaOptions {
                    r: track_assert_some!(NonZeroUsize::new(asha.r), ErrorKind::InvalidInput),
                    s: track_assert_some!(NonZeroUsize::new(asha.s), ErrorKind::InvalidInput),
                    eta: asha.eta,
                    max_suspended: track_assert_some!(
                        NonZeroUsize::new(asha.max_suspended),
                        ErrorKind::InvalidInput
                    ),
                })
            }
        }
    }
}
impl OptimizerBuilder for AshaOptimizerSpec {
    type Optimizer = AshaOptimizer;

    fn build(&self, problem_space: &ProblemSpace, eval_cost: u64) -> Result<Self::Optimizer> {
        let options = track!(self.asha_options())?;
        match self {
            AshaOptimizerSpec::Random { random, .. } => {
                let opt = track!(random.build2(problem_space))?;
                let opt = RandomOptimizerNoBudget { inner: opt.inner };
                Ok(AshaOptimizer::Random(inner::AshaOptimizer::with_options(
                    opt, eval_cost, options,
                )))
            }
            AshaOptimizerSpec::Tpe { tpe, .. } => {
                let opt = track!(tpe.build2(problem_space))?;
                let opt = TpeOptimizerNoBudget { inner: opt.inner };
                Ok(AshaOptimizer::Tpe(inner::AshaOptimizer::with_options(
                    opt, eval_cost, options,
                )))
            }
        }
    }
}

#[derive(Debug)]
pub enum AshaOptimizer {
    Random(inner::AshaOptimizer<RandomOptimizerNoBudget<RungValue<NonNanF64>>, NonNanF64>),
    Tpe(inner::AshaOptimizer<TpeOptimizerNoBudget<RungValue<NonNanF64>>, NonNanF64>),
}
impl Optimizer for AshaOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        match self {
            AshaOptimizer::Random(o) => track!(o.ask(rng, idgen)),
            AshaOptimizer::Tpe(o) => track!(o.ask(rng, idgen)),
        }
    }

    fn tell(&mut self, obs: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        let obs = obs.map_value(NonNanF64::new); // TODO
        match self {
            AshaOptimizer::Random(o) => track!(o.tell(obs)),
            AshaOptimizer::Tpe(o) => track!(o.tell(obs)),
        }
    }
}
