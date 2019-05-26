use crate::parameter::ParamValue;
use crate::problem::ProblemSpec;
use crate::Result;
use rand::Rng;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use structopt::StructOpt;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGen, Obs};

pub trait SolverRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Solver: Solver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver>;
}

pub type UnobservedObs = Obs<Budgeted<Vec<ParamValue>>>;
pub type ObservedObs = Obs<Budgeted<Vec<ParamValue>>, Vec<FiniteF64>>;

pub trait Solver {
    fn specification(&self) -> SolverSpec;

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs>;

    fn tell(&mut self, obs: ObservedObs) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct SolverSpec {
    pub name: String,

    #[serde(default)]
    pub version: Option<String>,

    #[serde(default)]
    pub capabilities: SolverCapabilities,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SolverCapabilities(BTreeSet<SolverCapability>);
impl SolverCapabilities {
    pub fn all() -> Self {
        let all = [
            SolverCapability::Categorical,
            SolverCapability::Conditional,
            SolverCapability::Discrete,
            SolverCapability::LogUniform,
            SolverCapability::MultiObjective,
        ]
        .iter()
        .cloned()
        .collect();
        Self(all)
    }

    pub fn empty() -> Self {
        Self(BTreeSet::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn is_superset(&self, other: &Self) -> bool {
        self.0.is_superset(&other.0)
    }

    pub fn incapables(&self, required: &Self) -> Self {
        Self(required.0.difference(&self.0).cloned().collect())
    }

    pub fn contains(&self, c: SolverCapability) -> bool {
        self.0.contains(&c)
    }

    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = SolverCapability> {
        self.0.iter().cloned()
    }

    pub fn union(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn categorical(mut self) -> Self {
        self.0.insert(SolverCapability::Categorical);
        self
    }

    pub fn conditional(mut self) -> Self {
        self.0.insert(SolverCapability::Conditional);
        self
    }

    pub fn discrete(mut self) -> Self {
        self.0.insert(SolverCapability::Discrete);
        self
    }

    pub fn log_uniform(mut self) -> Self {
        self.0.insert(SolverCapability::LogUniform);
        self
    }

    pub fn multi_objective(mut self) -> Self {
        self.0.insert(SolverCapability::MultiObjective);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolverCapability {
    Categorical,
    Conditional,
    Discrete,
    LogUniform,
    MultiObjective,
}
