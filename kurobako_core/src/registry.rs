//! Registry of problem and solver factories.
use crate::json::JsonValue;
use crate::problem::{BoxProblemFactory, ProblemRecipe};
use crate::solver::{BoxSolverFactory, SolverRecipe};
use crate::{Error, Result};
use serde_json;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, Weak};

/// Factory registry.
#[derive(Debug)]
pub struct FactoryRegistry {
    problem: ProblemFactoryRegistry,
    solver: SolverFactoryRegistry,
}
impl FactoryRegistry {
    /// Makes a new `FactoryRegistry` instance.
    pub fn new<P, S>() -> Self
    where
        P: 'static + ProblemRecipe,
        S: 'static + SolverRecipe,
    {
        Self {
            problem: ProblemFactoryRegistry::new::<P>(),
            solver: SolverFactoryRegistry::new::<S>(),
        }
    }

    /// Gets or creates a problem factory associated with the given recipe JSON.
    pub fn get_or_create_problem_factory(
        &self,
        recipe_json: &JsonValue,
    ) -> Result<Arc<Mutex<BoxProblemFactory>>> {
        self.problem.get_or_create(recipe_json, self)
    }

    /// Gets or creates a solver factory associated with the given recipe JSON.
    pub fn get_or_create_solver_factory(
        &self,
        recipe_json: &JsonValue,
    ) -> Result<Arc<Mutex<BoxSolverFactory>>> {
        self.solver.get_or_create(recipe_json, self)
    }
}

struct ProblemFactoryRegistry {
    create_factory: Box<dyn Fn(&str, &FactoryRegistry) -> Result<BoxProblemFactory>>,
    factories: Mutex<HashMap<String, Weak<Mutex<BoxProblemFactory>>>>,
}
impl ProblemFactoryRegistry {
    fn new<T>() -> Self
    where
        T: 'static + ProblemRecipe,
    {
        let create_factory = Box::new(|json: &str, registry: &FactoryRegistry| {
            let recipe: T = track!(serde_json::from_str(json).map_err(Error::from))?;
            let factory = track!(recipe.create_factory(registry)).map(BoxProblemFactory::new)?;
            Ok(factory)
        });
        Self {
            create_factory,
            factories: Mutex::new(HashMap::new()),
        }
    }

    fn get_or_create(
        &self,
        recipe_json: &JsonValue,
        registry: &FactoryRegistry,
    ) -> Result<Arc<Mutex<BoxProblemFactory>>> {
        let json = track!(serde_json::to_string(recipe_json).map_err(Error::from))?;
        let factory = track!(self.factories.lock().map_err(Error::from))?
            .get(&json)
            .and_then(|s| s.upgrade());
        if let Some(factory) = factory {
            Ok(factory)
        } else {
            let factory = track!((self.create_factory)(&json, registry); json)?;
            let factory = Arc::new(Mutex::new(factory));
            track!(self.factories.lock().map_err(Error::from))?
                .insert(json, Arc::downgrade(&factory));
            Ok(factory)
        }
    }
}
impl fmt::Debug for ProblemFactoryRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ProblemFactoryRegistry {{ .. }}")
    }
}

struct SolverFactoryRegistry {
    create_factory: Box<dyn Fn(&str, &FactoryRegistry) -> Result<BoxSolverFactory>>,
    factories: Mutex<HashMap<String, Weak<Mutex<BoxSolverFactory>>>>,
}
impl SolverFactoryRegistry {
    fn new<T>() -> Self
    where
        T: 'static + SolverRecipe,
    {
        let create_factory = Box::new(|json: &str, registry: &FactoryRegistry| {
            let recipe: T = track!(serde_json::from_str(json).map_err(Error::from))?;
            let factory = track!(recipe.create_factory(registry)).map(BoxSolverFactory::new)?;
            Ok(factory)
        });
        Self {
            create_factory,
            factories: Mutex::new(HashMap::new()),
        }
    }

    fn get_or_create(
        &self,
        recipe_json: &JsonValue,
        registry: &FactoryRegistry,
    ) -> Result<Arc<Mutex<BoxSolverFactory>>> {
        let json = track!(serde_json::to_string(recipe_json).map_err(Error::from))?;
        let factory = track!(self.factories.lock().map_err(Error::from))?
            .get(&json)
            .and_then(|s| s.upgrade());
        if let Some(factory) = factory {
            Ok(factory)
        } else {
            let factory = track!((self.create_factory)(&json, registry); json)?;
            let factory = Arc::new(Mutex::new(factory));
            track!(self.factories.lock().map_err(Error::from))?
                .insert(json, Arc::downgrade(&factory));
            Ok(factory)
        }
    }
}
impl fmt::Debug for SolverFactoryRegistry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SolverFactoryRegistry {{ .. }}")
    }
}
