use crate::problem::ProblemSpec;
use crate::solver::{ObservedObs, UnobservedObs};
use crate::Result;
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::fmt;
use structopt::StructOpt;

pub trait FilterRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Filter: Filter;

    fn create_filter(&self) -> Result<Self::Filter>;
}

pub trait Filter {
    fn specification(&self) -> FilterSpec;

    fn filter_problem_spec(&mut self, spec: &mut ProblemSpec) -> Result<()>;
    fn filter_ask<R: Rng>(&mut self, rng: &mut R, obs: &mut UnobservedObs) -> Result<()>;
    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: &mut ObservedObs) -> Result<()>;
}

enum Arg<'a> {
    Spec(&'a mut ProblemSpec),
    Ask(&'a mut dyn RngCore, &'a mut UnobservedObs),
    Tell(&'a mut dyn RngCore, &'a mut ObservedObs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterSpec {
    pub name: String,
}

pub struct BoxFilter {
    spec: FilterSpec,
    filter: Box<dyn FnMut(Arg) -> Result<()>>,
}
impl BoxFilter {
    pub fn new<T>(mut inner: T) -> Self
    where
        T: 'static + Filter,
    {
        let spec = inner.specification();
        let filter = Box::new(move |arg: Arg| match arg {
            Arg::Spec(a) => inner.filter_problem_spec(a),
            Arg::Ask(mut a, b) => inner.filter_ask(&mut a, b),
            Arg::Tell(mut a, b) => inner.filter_tell(&mut a, b),
        });
        Self { spec, filter }
    }
}
impl Filter for BoxFilter {
    fn specification(&self) -> FilterSpec {
        self.spec.clone()
    }

    fn filter_problem_spec(&mut self, spec: &mut ProblemSpec) -> Result<()> {
        track!((self.filter)(Arg::Spec(spec)))
    }

    fn filter_ask<R: Rng>(&mut self, mut rng: &mut R, obs: &mut UnobservedObs) -> Result<()> {
        track!((self.filter)(Arg::Ask(&mut rng, obs)))
    }

    fn filter_tell<R: Rng>(&mut self, mut rng: &mut R, obs: &mut ObservedObs) -> Result<()> {
        track!((self.filter)(Arg::Tell(&mut rng, obs)))
    }
}
impl fmt::Debug for BoxFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxFilter{{ name: {:?}, .. }}", self.spec.name)
    }
}
