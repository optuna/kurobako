//! A problem that uses a random forest surrogate model to evaluate parameters.
//!
//! # References
//!
//! - [Surrogate Benchmarks for Hyperparameter Optimization][paper]
//!
//! [paper]: http://ceur-ws.org/Vol-1201/paper-06.pdf
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::{ArcRng, Rng};
use kurobako_core::trial::{Params, Values};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;

/// Recipe of `SurrogateProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct SurrogateProblemRecipe {
    /// Path to a surrogate model file (JSON).
    pub model: PathBuf,
}
impl ProblemRecipe for SurrogateProblemRecipe {
    type Factory = SurrogateProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        todo!()
    }
}

/// Factory of `SurrogateProblem`.
#[derive(Debug)]
pub struct SurrogateProblemFactory {}
impl ProblemFactory for SurrogateProblemFactory {
    type Problem = SurrogateProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        todo!()
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        todo!()
    }
}

/// Problem that uses a random forest surrogate model to evaluate parameters.
#[derive(Debug)]
pub struct SurrogateProblem {}

impl Problem for SurrogateProblem {
    type Evaluator = SurrogateEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        todo!()
    }
}

/// Evaluator of `SurrogateProblem`.
#[derive(Debug)]
pub struct SurrogateEvaluator {}

impl Evaluator for SurrogateEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        todo!()
    }
}
