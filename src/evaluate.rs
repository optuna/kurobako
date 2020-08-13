//! `kurobako evaluate` command.
use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use kurobako_core::problem::{
    Evaluator as _, Problem as _, ProblemFactory as _, ProblemRecipe as _,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Options of the `kurobako evaluate` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct EvaluateOpt {
    /// Evaluation target problem.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub problem: KurobakoProblemRecipe,

    /// Parameters to be evaluated.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub params: Params,

    /// Evaluation step. If omitted, the maximum step of the problem is used.
    #[structopt(long)]
    pub step: Option<u64>,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}

impl EvaluateOpt {
    /// Evaluates the given parameters.
    pub fn evaluate(&self) -> Result<Evaluated> {
        let random_seed = self.seed.unwrap_or_else(rand::random);
        let rng = ArcRng::new(random_seed);
        let registry = FactoryRegistry::new::<KurobakoProblemRecipe, KurobakoSolverRecipe>();
        let problem_factory = track!(self.problem.create_factory(&registry))?;
        let problem_spec = track!(problem_factory.specification())?;
        track_assert_eq!(
            self.params.len(),
            problem_spec.params_domain.variables().len(),
            ErrorKind::InvalidInput
        );

        let problem = track!(problem_factory.create_problem(rng))?;

        let mut evaluator = track!(problem.create_evaluator(self.params.clone()))?;
        let step = self.step.unwrap_or_else(|| problem_spec.steps.last());
        let (current_step, values) = track!(evaluator.evaluate(step))?;

        Ok(Evaluated {
            values,
            step: current_step,
            seed: random_seed,
        })
    }
}

/// Evaluated result.
#[derive(Debug, Serialize, Deserialize)]
pub struct Evaluated {
    /// Evaluated values.
    pub values: Values,

    /// Current evaluation step.
    pub step: u64,

    /// Random seed.
    pub seed: u64,
}
