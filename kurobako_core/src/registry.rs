//! Registry of problem and solver factories.
// FIXME: Rename this module and structs.
use crate::json::JsonRecipe;
use crate::problem::{BoxProblemFactory, ProblemRecipe};
use crate::solver::{BoxSolverFactory, SolverRecipe};
use crate::{Error, Result};
use std::fmt;

/// Factory registry.
pub struct FactoryRegistry {
    create_problem_factory:
        Box<dyn Fn(&JsonRecipe, &FactoryRegistry) -> Result<BoxProblemFactory> + Send>,
    create_solver_factory:
        Box<dyn Fn(&JsonRecipe, &FactoryRegistry) -> Result<BoxSolverFactory> + Send>,
}
impl FactoryRegistry {
    /// Makes a new `FactoryRegistry` instance.
    pub fn new<P, S>() -> Self
    where
        P: 'static + ProblemRecipe,
        S: 'static + SolverRecipe,
    {
        let create_problem_factory = Box::new(|json: &JsonRecipe, registry: &FactoryRegistry| {
            let recipe: P = track!(serde_json::from_value(json.clone()).map_err(Error::from))?;
            let factory = track!(recipe.create_factory(registry)).map(BoxProblemFactory::new)?;
            Ok(factory)
        });
        let create_solver_factory = Box::new(|json: &JsonRecipe, registry: &FactoryRegistry| {
            let recipe: S = track!(serde_json::from_value(json.clone()).map_err(Error::from))?;
            let factory = track!(recipe.create_factory(registry)).map(BoxSolverFactory::new)?;
            Ok(factory)
        });
        Self {
            create_problem_factory,
            create_solver_factory,
        }
    }

    /// Creates a problem factory associated with the given recipe JSON.
    pub fn create_problem_factory_from_json(&self, json: &JsonRecipe) -> Result<BoxProblemFactory> {
        track!((self.create_problem_factory)(&json, self); json)
    }

    /// Creates a solver factory associated with the given recipe JSON.
    pub fn create_solver_factory_from_json(&self, json: &JsonRecipe) -> Result<BoxSolverFactory> {
        track!((self.create_solver_factory)(&json, self); json)
    }
}
impl fmt::Debug for FactoryRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FactoryRegistry {{ .. }}")
    }
}
