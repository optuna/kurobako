//! Solver interface for black-box optimization.
use crate::problem::ProblemSpec;
use crate::registry::FactoryRegistry;
use crate::rng::ArcRng;
use crate::trial::{EvaluatedTrial, IdGen, UnevaluatedTrial};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use structopt::StructOpt;

pub use self::capability::{Capabilities, Capability};

mod capability;

/// Builder of `SolverSpec`.
#[derive(Debug)]
pub struct SolverSpecBuilder {
    name: String,
    attrs: BTreeMap<String, String>,
    capabilities: BTreeSet<Capability>,
}
impl SolverSpecBuilder {
    /// Makes a new `SolverSpecBuilder` instance.
    pub fn new(solver_name: &str) -> Self {
        Self {
            name: solver_name.to_owned(),
            attrs: BTreeMap::new(),
            capabilities: BTreeSet::new(),
        }
    }

    /// Adds an attribute to this solver.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_owned(), value.to_owned());
        self
    }

    /// Adds a capability to this solver.
    pub fn capable(mut self, capability: Capability) -> Self {
        self.capabilities.insert(capability);
        self
    }

    /// Builds a `SolverSpec` instance with the given settings.
    pub fn finish(self) -> SolverSpec {
        SolverSpec {
            name: self.name,
            attrs: self.attrs,
            capabilities: Capabilities::new(self.capabilities.into_iter()),
        }
    }
}

/// Solver specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverSpec {
    /// The name of this solver.
    pub name: String,

    /// The attributes of this solver.
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,

    /// The capability of this solver.
    #[serde(default)]
    pub capabilities: Capabilities,
}

pub trait SolverRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Factory: SolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory>;

    fn to_json(&self) -> SolverRecipeJson {
        unimplemented!()
    }
}

pub trait SolverFactory {
    type Solver: Solver;

    fn specification(&self) -> Result<SolverSpec>;
    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver>;
}

enum SolverFactoryCall<'a> {
    Specification,
    CreateSolver(ArcRng, &'a ProblemSpec),
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

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
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
