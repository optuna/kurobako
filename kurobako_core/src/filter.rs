use crate::solver::{ObservedObs, UnobservedObs};
use crate::{ErrorKind, Result};
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

    // TODO: filter_problem_spec
    fn filter_ask<R: Rng>(&mut self, rng: &mut R, obs: UnobservedObs) -> Result<UnobservedObs>;
    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: ObservedObs) -> Result<ObservedObs>;
}

enum Observation {
    Observed(ObservedObs),
    Unobserved(UnobservedObs),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterSpec {
    pub name: String,
}

pub struct BoxFilter {
    spec: FilterSpec,
    filter: Box<FnMut(&mut RngCore, Observation) -> Result<Observation>>,
}
impl BoxFilter {
    pub fn new<T>(mut inner: T) -> Self
    where
        T: 'static + Filter,
    {
        let spec = inner.specification();
        let filter = Box::new(move |mut rng: &mut RngCore, obs| match obs {
            Observation::Unobserved(obs) => {
                inner.filter_ask(&mut rng, obs).map(Observation::Unobserved)
            }
            Observation::Observed(obs) => {
                inner.filter_tell(&mut rng, obs).map(Observation::Observed)
            }
        });
        Self { spec, filter }
    }
}
impl Filter for BoxFilter {
    fn specification(&self) -> FilterSpec {
        self.spec.clone()
    }

    fn filter_ask<R: Rng>(&mut self, mut rng: &mut R, obs: UnobservedObs) -> Result<UnobservedObs> {
        if let Observation::Unobserved(obs) =
            track!((self.filter)(&mut rng, Observation::Unobserved(obs)))?
        {
            Ok(obs)
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }

    fn filter_tell<R: Rng>(&mut self, mut rng: &mut R, obs: ObservedObs) -> Result<ObservedObs> {
        if let Observation::Observed(obs) =
            track!((self.filter)(&mut rng, Observation::Observed(obs)))?
        {
            Ok(obs)
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }
}
impl fmt::Debug for BoxFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxFilter{{ name: {:?}, .. }}", self.spec.name)
    }
}
