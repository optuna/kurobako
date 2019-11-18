//! Solver interface for black-box optimization.
use crate::problem::ProblemSpec;
use crate::registry::FactoryRegistry;
use crate::rng::ArcRng;
use crate::trial::{AskedTrial, EvaluatedTrial, IdGen};
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

    /// Sets the given capabilities to this solver.
    pub fn capabilities(mut self, capabilities: Capabilities) -> Self {
        self.capabilities = capabilities.iter().collect();
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

/// Recipe of a solver.
pub trait SolverRecipe: Clone + Send + StructOpt + Serialize + for<'a> Deserialize<'a> {
    /// The type of he factory creating solver instances from this recipe.
    type Factory: SolverFactory;

    /// Creates a solver factory.
    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory>;
}

/// This trait allows creating instances of a solver.
pub trait SolverFactory: Send {
    /// The type of the solver instance created by this factory.
    type Solver: Solver;

    /// Returns the specification of the solver created by this factory.
    fn specification(&self) -> Result<SolverSpec>;

    /// Creates a solver instance.
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

/// Boxed solver factory.
pub struct BoxSolverFactory(Box<dyn Fn(SolverFactoryCall) -> Result<SolverFactoryReturn> + Send>);
impl BoxSolverFactory {
    /// Makes a new `BoxSolverFactory` instance.
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

/// Solver.
pub trait Solver: Send {
    /// Asks the next trial to be evaluated.
    fn ask(&mut self, idg: &mut IdGen) -> Result<AskedTrial>;

    /// Tells the evaluation result of a trial.
    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()>;
}

/// Boxed solver.
pub struct BoxSolver(Box<dyn Solver>);
impl BoxSolver {
    /// Makes a new `BoxSolver` instance.
    pub fn new<S>(solver: S) -> Self
    where
        S: 'static + Solver,
    {
        Self(Box::new(solver))
    }
}
impl Solver for BoxSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<AskedTrial> {
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
