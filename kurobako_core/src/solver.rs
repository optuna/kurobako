use crate::json::JsonValue;
use crate::parameter::ParamValue;
use crate::problem::ProblemSpec;
use crate::{ErrorKind, Result};
use rand::{Rng, RngCore};
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use structopt::StructOpt;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGen, Obs};

pub trait SolverRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Solver: Solver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver>;
}

// #[derive(Debug)]
// pub struct BoxSolverRecipe {
//     create_solver: Box<Fn(ProblemSpec) -> Result<BoxSolver>>,
// }
// impl BoxSolverRecipe {
//     pub fn new<R>(recipe: R) -> Self
//     where
//         R: 'static + SolverRecipe,
//     {

//     }
// }

// TODO: move to `crate::observations::*`
pub type UnobservedObs = Obs<Budgeted<Vec<ParamValue>>>;
pub type ObservedObs = Obs<Budgeted<Vec<ParamValue>>, Vec<FiniteF64>>;

pub trait Solver {
    fn specification(&self) -> SolverSpec;

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs>;

    fn tell(&mut self, obs: ObservedObs) -> Result<()>;
}

pub struct BoxSolver {
    spec: SolverSpec,
    solver: Box<FnMut(SolverArg) -> Result<Option<UnobservedObs>>>,
}
impl BoxSolver {
    pub fn new<S>(mut inner: S) -> Self
    where
        S: 'static + Solver,
    {
        let spec = inner.specification();
        let solver = Box::new(move |arg: SolverArg| match arg {
            SolverArg::Ask(mut rng, mut idg) => track!(inner.ask(&mut rng, &mut idg)).map(Some),
            SolverArg::Tell(obs) => track!(inner.tell(obs)).map(|_| None),
        });
        Self { spec, solver }
    }
}
impl Solver for BoxSolver {
    fn specification(&self) -> SolverSpec {
        self.spec.clone()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, mut rng: &mut R, mut idg: &mut G) -> Result<UnobservedObs> {
        if let Some(obs) = track!((self.solver)(SolverArg::Ask(&mut rng, &mut idg)))? {
            Ok(obs)
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        if let None = track!((self.solver)(SolverArg::Tell(obs)))? {
            Ok(())
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }
}
impl fmt::Debug for BoxSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolver {{ name: {:?}, .. }}", self.spec.name)
    }
}

enum SolverArg<'a> {
    Ask(&'a mut dyn RngCore, &'a mut dyn IdGen),
    Tell(ObservedObs),
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

// #[derive(Debug, Serialize, Deserialize)]
// pub struct SolverRecipePlaceHolder {
//     #[serde(flatten)]
//     json: JsonValue,

//     #[serde(skip)]
//     recipe: Option<BoxSolverRecipe>,
// }
// impl SolverRecipePlaceHolder {
//     // pub fn set_recipe<F>(&mut self, create_solver: F) -> Result<()>
//     // where
//     //     F: FnOnce(&JsonValue) -> Result<BoxSolverRecipe>,
//     // {
//     //     panic!()
//     // }
// }
// impl Clone for SolverRecipePlaceHolder {
//     fn clone(&self) -> Self {
//         Self {
//             json: self.json.clone(),
//             recipe: None,
//         }
//     }
// }
