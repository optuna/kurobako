//! Repository for active problems and solvers.
use crate::problem::ProblemSpec;
use crate::solver::{BoxSolverFactory, BoxSolverRecipe, SolverRecipeJson};
use crate::Result;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex, Weak};

pub struct Repository {
    json_to_solver_recipe: Box<dyn Fn(&SolverRecipeJson) -> Result<BoxSolverRecipe>>,
    solvers: HashMap<SolverRecipeJson, Weak<Mutex<BoxSolverFactory>>>,
}
impl Repository {
    pub fn new<F>(json_to_solver_recipe: F) -> Self
    where
        F: 'static + Fn(&SolverRecipeJson) -> Result<BoxSolverRecipe>,
    {
        Self {
            json_to_solver_recipe: Box::new(json_to_solver_recipe),
            solvers: HashMap::new(),
        }
    }

    pub fn create_solver_if_absent(
        &mut self,
        recipe_json: &SolverRecipeJson,
        problem: &ProblemSpec,
    ) -> Result<Arc<Mutex<BoxSolverFactory>>> {
        if let Some(solver) = self.solvers.get(recipe_json).and_then(|s| s.upgrade()) {
            Ok(solver)
        } else {
            let recipe = track!((self.json_to_solver_recipe)(recipe_json); recipe_json)?;
            let solver = Arc::new(Mutex::new(track!(
                recipe.create_solver_factory(problem, self)
            )?));
            self.solvers
                .insert(recipe_json.clone(), Arc::downgrade(&solver));
            Ok(solver)
        }
    }
}
impl fmt::Debug for Repository {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repository {{ .. }}")
    }
}
