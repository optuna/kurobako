use crate::distribution::Distribution;
use crate::{Error, ProblemSpace};
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{ConstIdGenerator, IdGen, Obs, ObsId};
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::parameters::F64;
use yamakan::{self, Optimizer};

pub use self::asha::{AshaOptimizer, AshaOptimizerSpec};
pub use self::external_command::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder};
pub use self::gpyopt::{GpyoptOptimizer, GpyoptOptimizerBuilder};
pub use self::optuna::{OptunaOptimizer, OptunaOptimizerBuilder};

mod asha;
mod external_command;
mod gpyopt;
mod optuna;

pub trait OptimizerBuilder: StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Optimizer: Optimizer<Param = Budgeted<Vec<f64>>, Value = f64>;

    fn build(&self, problem_space: &ProblemSpace, eval_cost: u64)
        -> Result<Self::Optimizer, Error>;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum OptimizerSpec {
    Random(RandomOptimizerBuilder),
    Optuna(OptunaOptimizerBuilder),
    Gpyopt(GpyoptOptimizerBuilder),
    Asha(AshaOptimizerSpec),
    Command(ExternalCommandOptimizerBuilder),
}
impl OptimizerBuilder for OptimizerSpec {
    type Optimizer = UnionOptimizer;

    fn build(
        &self,
        problem_space: &ProblemSpace,
        eval_cost: u64,
    ) -> Result<Self::Optimizer, Error> {
        match self {
            OptimizerSpec::Random(x) => x
                .build(problem_space, eval_cost)
                .map(UnionOptimizer::Random),
            OptimizerSpec::Optuna(x) => x
                .build(problem_space, eval_cost)
                .map(UnionOptimizer::Optuna),
            OptimizerSpec::Gpyopt(x) => x
                .build(problem_space, eval_cost)
                .map(UnionOptimizer::Gpyopt),
            OptimizerSpec::Asha(x) => x.build(problem_space, eval_cost).map(UnionOptimizer::Asha),
            OptimizerSpec::Command(x) => x
                .build(problem_space, eval_cost)
                .map(UnionOptimizer::Command),
        }
    }
}

// TODO: rename
#[derive(Debug)]
pub enum UnionOptimizer {
    Random(RandomOptimizer),
    Optuna(OptunaOptimizer),
    Gpyopt(GpyoptOptimizer),
    Asha(AshaOptimizer),
    Command(ExternalCommandOptimizer),
}
impl Optimizer for UnionOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        match self {
            UnionOptimizer::Random(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Optuna(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Gpyopt(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Asha(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Command(x) => track!(x.ask(rng, idgen)),
        }
    }

    fn tell(&mut self, o: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        match self {
            UnionOptimizer::Random(x) => track!(x.tell(o)),
            UnionOptimizer::Optuna(x) => track!(x.tell(o)),
            UnionOptimizer::Gpyopt(x) => track!(x.tell(o)),
            UnionOptimizer::Asha(x) => track!(x.tell(o)),
            UnionOptimizer::Command(x) => track!(x.tell(o)),
        }
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct RandomOptimizer {
    inner: Vec<InnerRandomOptimizer<F64, f64>>,
}
impl Optimizer for RandomOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator::new(id);
        let params = self
            .inner
            .iter_mut()
            .map(|o| track!(o.ask(rng, &mut idgen)).map(|obs| obs.param))
            .collect::<yamakan::Result<_>>()?;
        Ok(Obs {
            id,
            param: Budgeted::new(Budget::new(::std::u64::MAX), params),
            value: (),
        })
    }

    fn tell(&mut self, _: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        Ok(())
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

// TODO
#[derive(Debug)]
pub struct RandomOptimizerNoBudget<V> {
    inner: Vec<InnerRandomOptimizer<F64, V>>,
}
impl<V> Optimizer for RandomOptimizerNoBudget<V> {
    type Param = Vec<f64>;
    type Value = V;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator::new(id);
        let params = self
            .inner
            .iter_mut()
            .map(|o| track!(o.ask(rng, &mut idgen)).map(|obs| obs.param))
            .collect::<yamakan::Result<_>>()?;
        Ok(Obs {
            id,
            param: params,
            value: (),
        })
    }

    fn tell(&mut self, _: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        Ok(())
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct RandomOptimizerBuilder {}
impl RandomOptimizerBuilder {
    // TODO
    fn build2<V>(&self, problem_space: &ProblemSpace) -> Result<RandomOptimizerNoBudget<V>, Error> {
        let inner = problem_space
            .distributions()
            .iter()
            .map(|d| {
                let Distribution::Uniform { low, high } = *d;
                InnerRandomOptimizer::new(F64::new(low, high).expect("TODO"))
            })
            .collect();
        Ok(RandomOptimizerNoBudget { inner })
    }
}
impl OptimizerBuilder for RandomOptimizerBuilder {
    type Optimizer = RandomOptimizer;

    fn build(
        &self,
        problem_space: &ProblemSpace,
        _eval_cost: u64,
    ) -> Result<Self::Optimizer, Error> {
        let inner = problem_space
            .distributions()
            .iter()
            .map(|d| {
                let Distribution::Uniform { low, high } = *d;
                InnerRandomOptimizer::new(F64::new(low, high).expect("TODO"))
            })
            .collect();
        Ok(RandomOptimizer { inner })
    }
}
