use crate::distribution::Distribution;
use crate::float::NonNanF64;
use crate::{Error, ProblemSpace};
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs, ObsId};
use yamakan::optimizers::random::RandomOptimizer as InnerRandomOptimizer;
use yamakan::optimizers::Optimizer;
use yamakan::optimizers::{knn, tpe};
use yamakan::spaces::F64;

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
    Knn(KnnOptimizerBuilder),
    Tpe(TpeOptimizerBuilder),
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
            OptimizerSpec::Knn(x) => x.build(problem_space, eval_cost).map(UnionOptimizer::Knn),
            OptimizerSpec::Tpe(x) => x.build(problem_space, eval_cost).map(UnionOptimizer::Tpe),
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
    Knn(KnnOptimizer),
    Tpe(TpeOptimizer),
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
            UnionOptimizer::Knn(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Tpe(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Optuna(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Gpyopt(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Asha(x) => track!(x.ask(rng, idgen)),
            UnionOptimizer::Command(x) => track!(x.ask(rng, idgen)),
        }
    }

    fn tell(&mut self, o: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        match self {
            UnionOptimizer::Random(x) => track!(x.tell(o)),
            UnionOptimizer::Knn(x) => track!(x.tell(o)),
            UnionOptimizer::Tpe(x) => track!(x.tell(o)),
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

// TODO: move
#[derive(Debug)]
struct ConstIdGenerator(ObsId);
impl IdGen for ConstIdGenerator {
    fn generate(&mut self) -> yamakan::Result<ObsId> {
        Ok(self.0)
    }
}

#[derive(Debug)]
pub struct TpeOptimizer {
    inner: Vec<tpe::TpeNumericalOptimizer<F64, NonNanF64>>,
}
impl Optimizer for TpeOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
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

    fn tell(&mut self, mut obs: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        if obs.value.is_nan() {
            return Ok(());
        }

        let value = NonNanF64::new(obs.value);
        obs.param.budget_mut().consume(1);
        for (p, o) in obs.param.get().iter().cloned().zip(self.inner.iter_mut()) {
            let obs = Obs {
                id: obs.id,
                param: p,
                value,
            };
            track!(o.tell(obs))?;
        }
        Ok(())
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

// TODO
#[derive(Debug)]
pub struct TpeOptimizerNoBudget<V> {
    inner: Vec<tpe::TpeNumericalOptimizer<F64, V>>,
}
impl<V> Optimizer for TpeOptimizerNoBudget<V>
where
    V: Ord + Clone,
{
    type Param = Vec<f64>;
    type Value = V;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
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

    fn tell(&mut self, obs: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        for (p, o) in obs.param.into_iter().zip(self.inner.iter_mut()) {
            let obs = Obs {
                id: obs.id,
                param: p,
                value: obs.value.clone(),
            };
            track!(o.tell(obs))?;
        }
        Ok(())
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

// fn is_24(n: &usize) -> bool {
//     *n == 24
// }

// fn is_false(b: &bool) -> bool {
//     !*b
// }

// fn default_ei_candidates() -> usize {
//     24
// }

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct TpeOptimizerBuilder {
    #[structopt(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    // #[serde(skip_serializing_if = "is_24", default = "default_ei_candidates")]
    // #[structopt(long, default_value = "24")]
    // pub ei_candidates: usize,

    // #[serde(skip_serializing_if = "is_false", default)]
    // #[structopt(long)]
    // pub prior_uniform: bool,

    // #[serde(skip_serializing_if = "is_false", default)]
    // #[structopt(long)]
    // pub uniform_sigma: bool,

    // #[serde(skip_serializing_if = "is_false", default)]
    // #[structopt(long)]
    // pub uniform_weight: bool,

    // #[serde(skip_serializing_if = "Option::is_none")]
    // #[structopt(long)]
    // pub divide_factor: Option<f64>,
}
impl TpeOptimizerBuilder {
    // TODO
    fn build2<V>(&self, problem_space: &ProblemSpace) -> Result<TpeOptimizerNoBudget<V>, Error>
    where
        V: Ord,
    {
        let inner = problem_space
            .distributions()
            .iter()
            .map(|d| {
                let Distribution::Uniform { low, high } = *d;
                let strategy = ::yamakan::optimizers::tpe::DefaultStrategy::default();
                let param_space = track!(F64::new(low, high)).expect("TODO");
                tpe::TpeNumericalOptimizer::with_strategy(param_space, strategy)
            })
            .collect();
        Ok(TpeOptimizerNoBudget { inner })
    }
}
impl OptimizerBuilder for TpeOptimizerBuilder {
    type Optimizer = TpeOptimizer;

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
                let strategy = ::yamakan::optimizers::tpe::DefaultStrategy::default();
                let param_space = track!(F64::new(low, high)).expect("TODO");
                tpe::TpeNumericalOptimizer::with_strategy(param_space, strategy)
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
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
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
        let mut idgen = ConstIdGenerator(id);
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

#[derive(Debug)]
pub struct KnnOptimizer {
    inner: Vec<knn::KnnOptimizer<F64, NonNanF64>>,
}
impl Optimizer for KnnOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param>> {
        let id = track!(idgen.generate())?;
        let mut idgen = ConstIdGenerator(id);
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

    fn tell(&mut self, mut obs: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        if obs.value.is_nan() {
            return Ok(());
        }

        let value = NonNanF64::new(obs.value);
        obs.param.budget_mut().consume(1);
        for (p, o) in obs.param.get().iter().cloned().zip(self.inner.iter_mut()) {
            let obs = Obs {
                id: obs.id,
                param: p,
                value,
            };
            track!(o.tell(obs))?;
        }
        Ok(())
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct KnnOptimizerBuilder {
    #[structopt(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}
impl OptimizerBuilder for KnnOptimizerBuilder {
    type Optimizer = KnnOptimizer;

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
                knn::KnnOptimizer::new(F64::new(low, high).expect("TODO"))
            })
            .collect();
        Ok(KnnOptimizer { inner })
    }
}
