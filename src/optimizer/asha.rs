use crate::optimizer::{OptimizerBuilder, RandomOptimizerBuilder, RandomOptimizerNoBudget};
use crate::{ProblemSpace, Result};
use rand::Rng;
use rustats::num::NonNanF64;
use yamakan::budget::{Budgeted, Leveled};
use yamakan::observation::{IdGen, Obs, ObsId};
use yamakan::optimizers::asha as inner;
use yamakan::{self, Optimizer};

fn is_default_reduction_factor(&n: &usize) -> bool {
    n == 2
}

fn is_default_max_rungs(&n: &usize) -> bool {
    n == 8
}

fn default_reduction_factor() -> usize {
    2
}

fn default_max_rungs() -> usize {
    8
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct AshaOptions {
    #[structopt(long, default_value = "2")]
    #[serde(default = "default_reduction_factor")]
    #[serde(skip_serializing_if = "is_default_reduction_factor")]
    pub reduction_factor: usize,

    #[structopt(long, default_value = "8")]
    #[serde(default = "default_max_rungs")]
    #[serde(skip_serializing_if = "is_default_max_rungs")]
    pub max_rungs: usize,
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
}
impl AshaOptimizerSpec {
    fn asha_options(&self) -> &AshaOptions {
        match self {
            AshaOptimizerSpec::Random { asha, .. } => asha,
        }
    }
}
impl OptimizerBuilder for AshaOptimizerSpec {
    type Optimizer = AshaOptimizer;

    fn build(&self, problem_space: &ProblemSpace, eval_cost: u64) -> Result<Self::Optimizer> {
        let options = self.asha_options();
        match self {
            AshaOptimizerSpec::Random { random, .. } => {
                let opt = track!(random.build2(problem_space))?;
                let opt = RandomOptimizerNoBudget { inner: opt.inner };

                let mut min_budget = eval_cost as f64;
                for _ in 0..options.max_rungs {
                    min_budget = (min_budget / options.reduction_factor as f64).ceil();
                }

                let min_budget = min_budget as u64;
                let max_budget = eval_cost;
                let opt = track!(inner::AshaBuilder::new()
                    .reduction_factor(options.reduction_factor)
                    .and_then(|b| b.finish(opt, max_budget, min_budget)))?;
                Ok(AshaOptimizer::Random(opt))
            }
        }
    }
}

#[derive(Debug)]
pub enum AshaOptimizer {
    Random(inner::AshaOptimizer<RandomOptimizerNoBudget<Leveled<NonNanF64>>, NonNanF64>),
}
impl Optimizer for AshaOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        match self {
            AshaOptimizer::Random(o) => track!(o.ask(rng, idgen)),
        }
    }

    fn tell(&mut self, obs: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        let obs = track!(obs.try_map_value(NonNanF64::new))?;
        match self {
            AshaOptimizer::Random(o) => track!(o.tell(obs)),
        }
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}
