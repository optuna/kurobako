//! Solver interface for black-box optimization.
use crate::problem::ProblemSpec;
use crate::trial::{IdGen, Trial};
use crate::{Error, Result};
use rand::{Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::{Arc, Mutex};
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

    fn create_solver(&self, problem: &ProblemSpec) -> Result<Self::Solver>;
}

pub struct BoxSolverRecipe {
    create_solver: Box<dyn Fn(&ProblemSpec) -> Result<BoxSolver>>,
}
impl BoxSolverRecipe {
    pub fn new<R>(recipe: R) -> Self
    where
        R: 'static + SolverRecipe,
    {
        let create_solver = Box::new(move |problem: &ProblemSpec| {
            track!(recipe.create_solver(problem)).map(BoxSolver::new)
        });
        Self { create_solver }
    }

    pub fn create_solver(&self, problem: &ProblemSpec) -> Result<BoxSolver> {
        (self.create_solver)(problem)
    }
}
impl fmt::Debug for BoxSolverRecipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolverRecipe {{ .. }}")
    }
}

pub trait Solver {
    type Optimizer: Optimizer;

    fn specification(&self) -> Result<SolverSpec>;
    fn create_optimizer(&self) -> Result<Self::Optimizer>;
}

enum BoxSolverCall {
    Specification,
    CreateOptimizer,
}

enum BoxSolverReturn {
    Specification(SolverSpec),
    CreateOptimizer(BoxOptimizer),
}

pub struct BoxSolver(Box<dyn Fn(BoxSolverCall) -> Result<BoxSolverReturn>>);
impl BoxSolver {
    pub fn new<S>(inner: S) -> Self
    where
        S: 'static + Solver,
    {
        let solver = Box::new(move |call| match call {
            BoxSolverCall::Specification => {
                inner.specification().map(BoxSolverReturn::Specification)
            }
            BoxSolverCall::CreateOptimizer => inner
                .create_optimizer()
                .map(BoxOptimizer::new)
                .map(BoxSolverReturn::CreateOptimizer),
        });
        Self(solver)
    }
}
impl Solver for BoxSolver {
    type Optimizer = BoxOptimizer;

    fn specification(&self) -> Result<SolverSpec> {
        let v = track!((self.0)(BoxSolverCall::Specification))?;
        if let BoxSolverReturn::Specification(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }

    fn create_optimizer(&self) -> Result<Self::Optimizer> {
        let v = track!((self.0)(BoxSolverCall::CreateOptimizer))?;
        if let BoxSolverReturn::CreateOptimizer(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }
}
impl fmt::Debug for BoxSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolver {{ .. }}")
    }
}

pub trait Optimizer {
    fn ask<R: Rng, G: IdGen>(&mut self, rng: R, idg: G) -> Result<Trial>;
    fn tell(&mut self, trial: Trial) -> Result<()>;
}

enum BoxOptimizerCall<'a> {
    Ask {
        rng: &'a mut dyn RngCore,
        idg: &'a mut dyn IdGen,
    },
    Tell {
        trial: Trial,
    },
}

enum BoxOptimizerReturn {
    Ask(Trial),
    Tell(()),
}

pub struct BoxOptimizer(Box<dyn FnMut(BoxOptimizerCall) -> Result<BoxOptimizerReturn>>);
impl BoxOptimizer {
    pub fn new<O>(mut inner: O) -> Self
    where
        O: 'static + Optimizer,
    {
        let optimizer = Box::new(move |call: BoxOptimizerCall| match call {
            BoxOptimizerCall::Ask { rng, idg } => inner.ask(rng, idg).map(BoxOptimizerReturn::Ask),
            BoxOptimizerCall::Tell { trial } => inner.tell(trial).map(BoxOptimizerReturn::Tell),
        });
        Self(optimizer)
    }
}
impl Optimizer for BoxOptimizer {
    fn ask<R: Rng, G: IdGen>(&mut self, mut rng: R, mut idg: G) -> Result<Trial> {
        let v = track!((self.0)(BoxOptimizerCall::Ask {
            rng: &mut rng,
            idg: &mut idg
        }))?;
        if let BoxOptimizerReturn::Ask(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }

    fn tell(&mut self, trial: Trial) -> Result<()> {
        let v = track!((self.0)(BoxOptimizerCall::Tell { trial }))?;
        if let BoxOptimizerReturn::Tell(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }
}
impl fmt::Debug for BoxOptimizer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxOptimizer {{ .. }}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SolverRecipeJson {}

#[derive(Clone)]
pub struct SolverRepository {
    json_to_recipe: Arc<Mutex<Box<dyn Fn(&SolverRecipeJson) -> Result<BoxSolverRecipe>>>>,
    solvers: Arc<Mutex<HashMap<SolverRecipeJson, Arc<Mutex<BoxSolver>>>>>,
}
impl SolverRepository {
    pub fn set_recipe_deserializer<F>(&self, f: F) -> Result<()>
    where
        F: 'static + Fn(&SolverRecipeJson) -> Result<BoxSolverRecipe>,
    {
        let mut json_to_recipe = track!(self.json_to_recipe.lock().map_err(Error::from))?;
        *json_to_recipe = Box::new(f);
        Ok(())
    }

    pub fn get(
        &self,
        recipe_json: &SolverRecipeJson,
        problem: &ProblemSpec,
    ) -> Result<Arc<Mutex<BoxSolver>>> {
        let mut solvers = track!(self.solvers.lock().map_err(Error::from))?;
        if let Some(solver) = solvers.get(recipe_json).cloned() {
            Ok(solver)
        } else {
            let json_to_recipe = track!(self.json_to_recipe.lock().map_err(Error::from))?;
            let recipe = track!(json_to_recipe(recipe_json); recipe_json)?;
            let solver = Arc::new(Mutex::new(track!(recipe.create_solver(problem))?));
            solvers.insert(recipe_json.clone(), Arc::clone(&solver));
            Ok(solver)
        }
    }
}
impl fmt::Debug for SolverRepository {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SolverRepository {{ .. }}")
    }
}
