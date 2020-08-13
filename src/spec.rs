//! `kurobako spec` command.
use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use kurobako_core::problem::{ProblemFactory as _, ProblemRecipe as _, ProblemSpec};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::solver::{SolverFactory as _, SolverRecipe as _, SolverSpec};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Options of the `kurobako spec` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SpecOpt {
    /// Show the specification of the given problem.
    Problem {
        /// Problem recipe (JSON).
        #[structopt(parse(try_from_str = json::parse_json))]
        problem: KurobakoProblemRecipe,
    },

    /// Show the specification of the given solver.
    Solver {
        /// Solver recipe (JSON).
        #[structopt(parse(try_from_str = json::parse_json))]
        solver: KurobakoSolverRecipe,
    },
}

impl SpecOpt {
    /// Returns the specification of the given problem or solver.
    pub fn get_spec(&self) -> Result<Spec> {
        let registry = FactoryRegistry::new::<KurobakoProblemRecipe, KurobakoSolverRecipe>();
        match self {
            Self::Problem { problem } => {
                let problem_factory = track!(problem.create_factory(&registry))?;
                let problem_spec = track!(problem_factory.specification())?;
                Ok(Spec::Problem(problem_spec))
            }
            Self::Solver { solver } => {
                let solver_factory = track!(solver.create_factory(&registry))?;
                let solver_spec = track!(solver_factory.specification())?;
                Ok(Spec::Solver(solver_spec))
            }
        }
    }
}

/// Specification.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spec {
    /// Problem specification.
    Problem(ProblemSpec),

    /// Solver specification.
    Solver(SolverSpec),
}
