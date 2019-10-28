//! Solver interface for black-box optimization.
use crate::problem::ProblemSpec;
use crate::trial::{IdGen, Trial};
use crate::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use structopt::StructOpt;

#[derive(Debug)]
pub struct SolverSpecBuilder {
    name: String,
    attrs: BTreeMap<String, String>,
    capabilities: BTreeSet<Capability>,
}
impl SolverSpecBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            attrs: BTreeMap::new(),
            capabilities: BTreeSet::new(),
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn capable(mut self, capability: Capability) -> Self {
        self.capabilities.insert(capability);
        self
    }

    pub fn finish(self) -> SolverSpec {
        SolverSpec {
            name: self.name,
            attrs: self.attrs,
            capabilities: Capabilities(self.capabilities),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverSpec {
    pub name: String,

    #[serde(default)]
    pub attrs: BTreeMap<String, String>,

    #[serde(default)]
    pub capabilities: Capabilities,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capabilities(BTreeSet<Capability>);
impl Capabilities {
    pub fn all() -> Self {
        let all = [
            Capability::UniformContinuous,
            Capability::UniformDiscrete,
            Capability::LogUniformContinuous,
            Capability::LogUniformDiscrete,
            Capability::Categorical,
            Capability::Conditional,
            Capability::MultiObjective,
        ]
        .iter()
        .copied()
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

    pub fn contains(&self, c: Capability) -> bool {
        self.0.contains(&c)
    }

    pub fn remove(&mut self, c: Capability) -> &mut Self {
        self.0.remove(&c);
        self
    }

    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = Capability> {
        self.0.iter().cloned()
    }

    pub fn union(mut self, mut other: Self) -> Self {
        self.0.append(&mut other.0);
        self
    }

    pub fn uniform_continuous(mut self) -> Self {
        self.0.insert(Capability::UniformContinuous);
        self
    }

    pub fn uniform_discrete(mut self) -> Self {
        self.0.insert(Capability::UniformDiscrete);
        self
    }

    pub fn log_uniform_continuous(mut self) -> Self {
        self.0.insert(Capability::LogUniformContinuous);
        self
    }

    pub fn log_uniform_discrete(mut self) -> Self {
        self.0.insert(Capability::LogUniformDiscrete);
        self
    }

    pub fn categorical(mut self) -> Self {
        self.0.insert(Capability::Categorical);
        self
    }

    pub fn conditional(mut self) -> Self {
        self.0.insert(Capability::Conditional);
        self
    }

    pub fn multi_objective(mut self) -> Self {
        self.0.insert(Capability::MultiObjective);
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
    UniformContinuous,
    UniformDiscrete,
    LogUniformContinuous,
    LogUniformDiscrete,
    Categorical,
    Conditional,
    MultiObjective,
}

pub trait SolverRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Solver: Solver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver>;
}

// pub struct BoxSolverRecipe {
//     create_solver: Box<dyn Fn(ProblemSpec) -> Result<BoxSolver>>,
// }
// impl BoxSolverRecipe {
//     pub fn new<R>(recipe: R) -> Self
//     where
//         R: 'static + SolverRecipe,
//     {
//         let create_solver =
//             Box::new(move |problem| track!(recipe.create_solver(problem)).map(BoxSolver::new));
//         Self { create_solver }
//     }

//     pub fn create_solver(&self, problem: ProblemSpec) -> Result<BoxSolver> {
//         (self.create_solver)(problem)
//     }
// }
// impl fmt::Debug for BoxSolverRecipe {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "BoxSolverRecipe {{ .. }}")
//     }
// }

pub trait Solver {
    fn specification(&self) -> SolverSpec;

    fn ask<R: Rng, G: IdGen>(&mut self, rng: R, idg: G) -> Result<Trial>;

    fn tell(&mut self, trial: Trial) -> Result<()>;
}

// TODO: SolverInstance

// pub struct BoxSolver {
//     spec: SolverSpec,
//     solver: Box<dyn FnMut(SolverArg) -> Result<Option<UnobservedObs>>>,
// }
// impl BoxSolver {
//     pub fn new<S>(mut inner: S) -> Self
//     where
//         S: 'static + Solver,
//     {
//         let spec = inner.specification();
//         let solver = Box::new(move |arg: SolverArg| match arg {
//             SolverArg::Ask(mut rng, mut idg) => track!(inner.ask(&mut rng, &mut idg)).map(Some),
//             SolverArg::Tell(obs) => track!(inner.tell(obs)).map(|_| None),
//         });
//         Self { spec, solver }
//     }
// }
// impl Solver for BoxSolver {
//     fn specification(&self) -> SolverSpec {
//         self.spec.clone()
//     }

//     fn ask<R: Rng, G: IdGen>(&mut self, mut rng: R, mut idg: G) -> Result<UnobservedObs> {
//         if let Some(obs) = track!((self.solver)(SolverArg::Ask(&mut rng, &mut idg)))? {
//             Ok(obs)
//         } else {
//             track_panic!(ErrorKind::Bug);
//         }
//     }

//     fn tell(&mut self, obs: ObservedObs) -> Result<()> {
//         if let None = track!((self.solver)(SolverArg::Tell(obs)))? {
//             Ok(())
//         } else {
//             track_panic!(ErrorKind::Bug);
//         }
//     }
// }
// impl fmt::Debug for BoxSolver {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "BoxSolver {{ name: {:?}, .. }}", self.spec.name)
//     }
// }

// enum SolverArg<'a> {
//     Ask(&'a mut dyn RngCore, &'a mut dyn IdGen),
//     Tell(ObservedObs),
// }

// #[derive(Debug, Serialize, Deserialize)]
// pub struct SolverRecipePlaceHolder {
//     #[serde(flatten)]
//     pub json: JsonValue,

//     #[serde(skip)]
//     pub recipe: Option<BoxSolverRecipe>,
// }
// impl SolverRecipePlaceHolder {
//     pub fn set_recipe<F>(&mut self, create_recipe: F) -> Result<()>
//     where
//         F: FnOnce(&JsonValue) -> Result<BoxSolverRecipe>,
//     {
//         let recipe = track!(create_recipe(&self.json))?;
//         self.recipe = Some(recipe);
//         Ok(())
//     }

//     pub fn create_solver(&self, problem: ProblemSpec) -> Result<BoxSolver> {
//         let recipe = track_assert_some!(self.recipe.as_ref(), ErrorKind::InvalidInput);
//         track!(recipe.create_solver(problem))
//     }
// }
// impl Clone for SolverRecipePlaceHolder {
//     fn clone(&self) -> Self {
//         Self {
//             json: self.json.clone(),
//             recipe: None,
//         }
//     }
// }
// impl FromStr for SolverRecipePlaceHolder {
//     type Err = Error;

//     fn from_str(s: &str) -> Result<Self> {
//         track!(json::parse_json(s))
//     }
// }
