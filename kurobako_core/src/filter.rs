use crate::solver::{ObservedObs, UnobservedObs};
use crate::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub trait FilterRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Filter: Filter;

    fn create_filter(&self) -> Result<Self::Filter>;
}

pub trait Filter {
    fn specification(&self) -> FilterSpec;

    fn filter_ask<R: Rng>(&mut self, rng: &mut R, obs: UnobservedObs) -> Result<UnobservedObs>;
    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: ObservedObs) -> Result<ObservedObs>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct FilterSpec {
    pub name: String,
}
