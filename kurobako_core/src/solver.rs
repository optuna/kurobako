//! Solver interface for black-box optimization.
use crate::problem::ProblemSpec;
use crate::repository::Repository;
use crate::trial::{EvaluatedTrial, IdGen, UnevaluatedTrial};
use crate::Result;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
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
    type Factory: SolverFactory;

    fn create_factory(&self, repository: &mut Repository) -> Result<Self::Factory>;

    fn to_json(&self) -> SolverRecipeJson {
        unimplemented!()
    }
}

pub struct BoxSolverRecipe {
    create: Box<dyn Fn(&mut Repository) -> Result<BoxSolverFactory>>,
}
impl BoxSolverRecipe {
    pub fn new<R>(recipe: R) -> Self
    where
        R: 'static + SolverRecipe,
    {
        let create = Box::new(move |repository: &mut Repository| {
            track!(recipe.create_factory(repository)).map(BoxSolverFactory::new)
        });
        Self { create }
    }

    pub fn create_factory(&self, repository: &mut Repository) -> Result<BoxSolverFactory> {
        (self.create)(repository)
    }
}
impl fmt::Debug for BoxSolverRecipe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolverRecipe {{ .. }}")
    }
}

pub trait SolverFactory {
    type Solver: Solver;

    fn specification(&self) -> Result<SolverSpec>;
    fn create_solver(&self, rng: StdRng, problem: &ProblemSpec) -> Result<Self::Solver>;
}

enum SolverFactoryCall<'a> {
    Specification,
    CreateSolver(StdRng, &'a ProblemSpec),
}

enum SolverFactoryReturn {
    Specification(SolverSpec),
    CreateSolver(BoxSolver),
}

pub struct BoxSolverFactory(Box<dyn Fn(SolverFactoryCall) -> Result<SolverFactoryReturn>>);
impl BoxSolverFactory {
    pub fn new<S>(inner: S) -> Self
    where
        S: 'static + SolverFactory,
    {
        let solver = Box::new(move |call: SolverFactoryCall| match call {
            SolverFactoryCall::Specification => inner
                .specification()
                .map(SolverFactoryReturn::Specification),
            SolverFactoryCall::CreateSolver(rng, problem) => inner
                .create_solver(rng, problem)
                .map(BoxSolver::new)
                .map(SolverFactoryReturn::CreateSolver),
        });
        Self(solver)
    }
}
impl SolverFactory for BoxSolverFactory {
    type Solver = BoxSolver;

    fn specification(&self) -> Result<SolverSpec> {
        let v = track!((self.0)(SolverFactoryCall::Specification))?;
        if let SolverFactoryReturn::Specification(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }

    fn create_solver(&self, rng: StdRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let v = track!((self.0)(SolverFactoryCall::CreateSolver(rng, problem)))?;
        if let SolverFactoryReturn::CreateSolver(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }
}
impl fmt::Debug for BoxSolverFactory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolverFactory {{ .. }}")
    }
}

pub trait Solver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<UnevaluatedTrial>;
    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()>;
}

pub struct BoxSolver(Box<dyn Solver>);
impl BoxSolver {
    pub fn new<S>(solver: S) -> Self
    where
        S: 'static + Solver,
    {
        Self(Box::new(solver))
    }
}
impl Solver for BoxSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<UnevaluatedTrial> {
        track!(self.0.ask(idg))
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        track!(self.0.tell(trial))
    }
}
impl fmt::Debug for BoxSolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxSolver {{ .. }}")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SolverRecipeJson {}
