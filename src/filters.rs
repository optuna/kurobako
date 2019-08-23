// TODO: Move to `kurobako_filters` crate
use kurobako_core::filter::{Filter, FilterRecipe, FilterSpec};
use kurobako_core::num::FiniteF64;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{ObservedObs, UnobservedObs};
use kurobako_core::{Error, Result};
use rand::distributions::Distribution as _;
use rand::Rng;
use rand_distr::Normal;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct GaussianNoiseFilterRecipe {
    #[structopt(long, default_value = "0.1")]
    level: f64,
}
impl FilterRecipe for GaussianNoiseFilterRecipe {
    type Filter = GaussianNoiseFilter;

    fn create_filter(&self) -> Result<Self::Filter> {
        Ok(GaussianNoiseFilter {
            level: self.level,
            values_domain: Vec::new(),
        })
    }
}

#[derive(Debug)]
pub struct GaussianNoiseFilter {
    level: f64,

    // TODO: use (for example) 90%-tile instead of min-max
    values_domain: Vec<MinMax<FiniteF64>>, // observed
}
impl Filter for GaussianNoiseFilter {
    fn specification(&self) -> FilterSpec {
        FilterSpec {
            name: "gaussian-noise".to_owned(),
        }
    }

    fn filter_problem_spec(&mut self, _spec: &mut ProblemSpec) -> Result<()> {
        Ok(())
    }

    fn filter_ask<R: Rng>(&mut self, _rng: &mut R, _obs: &mut UnobservedObs) -> Result<()> {
        Ok(())
    }

    fn filter_tell<R: Rng>(&mut self, rng: &mut R, obs: &mut ObservedObs) -> Result<()> {
        if self.values_domain.is_empty() {
            self.values_domain = obs
                .value
                .iter()
                .map(|&v| track!(MinMax::new(v, v)).map_err(Error::from))
                .collect::<Result<Vec<_>>>()?;
            trace!("Initial values domain: {:?}", self.values_domain);
            return Ok(());
        }

        let mut values = Vec::with_capacity(obs.value.len());
        for (value, domain) in obs.value.iter().zip(self.values_domain.iter_mut()) {
            if value < domain.min() {
                *domain = track!(MinMax::new(*value, *domain.max()))?;
                trace!("Value domain updated: {:?}", domain);
            } else if value > domain.max() {
                *domain = track!(MinMax::new(*domain.min(), *value))?;
                trace!("Value domain updated: {:?}", domain);
            }

            let sd = domain.width().get() * self.level;
            let normal = Normal::new(value.get(), sd).unwrap_or_else(|e| panic!("TODO: {:?}", e));
            let noised_value = track!(FiniteF64::new(normal.sample(rng)))?;
            trace!(
                "Noised value: {} (original={})",
                noised_value.get(),
                value.get()
            );
            values.push(noised_value);
        }

        obs.value = values;
        Ok(())
    }
}
