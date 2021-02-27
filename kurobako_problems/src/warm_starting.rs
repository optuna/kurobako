//! A problem for warm-starting optimizations.
use kurobako_core::json::JsonRecipe;
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, BoxProblemFactory, Evaluator, Problem, ProblemFactory, ProblemRecipe,
    ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Recipe of `WarmStartingProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct WarmStartingProblemRecipe {
    /// Source problem recipe JSON.
    pub source: JsonRecipe,

    /// Target problem recipe JSON.
    pub target: JsonRecipe,
}

impl ProblemRecipe for WarmStartingProblemRecipe {
    type Factory = WarmStartingProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let source_factory = track!(registry.create_problem_factory_from_json(&self.source))?;
        let target_factory = track!(registry.create_problem_factory_from_json(&self.target))?;

        Ok(WarmStartingProblemFactory {
            source_factory,
            target_factory,
        })
    }
}

/// Factory of `WarmStartingProblem`.
#[derive(Debug)]
pub struct WarmStartingProblemFactory {
    source_factory: BoxProblemFactory,
    target_factory: BoxProblemFactory,
}
impl ProblemFactory for WarmStartingProblemFactory {
    type Problem = WarmStartingProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let source_spec = track!(self.source_factory.specification())?;
        let target_spec = track!(self.target_factory.specification())?;
        track_assert_eq!(
            source_spec.params_domain,
            target_spec.params_domain,
            ErrorKind::InvalidInput
        );
        track_assert_eq!(
            source_spec.values_domain,
            target_spec.values_domain,
            ErrorKind::InvalidInput
        );

        let spec = ProblemSpecBuilder::new(&format!("{} with warm starting", target_spec.name))
            .params(
                target_spec
                    .params_domain
                    .variables()
                    .iter()
                    .map(|p| p.clone().into())
                    .collect(),
            )
            .values(
                target_spec
                    .values_domain
                    .variables()
                    .iter()
                    .map(|p| p.clone().into())
                    .collect(),
            )
            .steps(std::iter::once(0).chain(target_spec.steps.iter()));
        track!(spec.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let source_spec = track!(self.source_factory.specification())?;
        let source_last_step = source_spec.steps.last();

        let source_problem = track!(self.source_factory.create_problem(rng.clone()))?;
        let target_problem = track!(self.target_factory.create_problem(rng))?;
        Ok(WarmStartingProblem {
            source_last_step,
            source_problem,
            target_problem,
        })
    }
}

/// Problem that uses a random forest surrogate model to evaluate parameters.
#[derive(Debug)]
pub struct WarmStartingProblem {
    source_last_step: u64,
    source_problem: BoxProblem,
    target_problem: BoxProblem,
}

impl Problem for WarmStartingProblem {
    type Evaluator = WarmStartingEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let source_evaluator = track!(self.source_problem.create_evaluator(params.clone()))?;
        let target_evaluator = track!(self.target_problem.create_evaluator(params))?;
        Ok(WarmStartingEvaluator {
            source_last_step: self.source_last_step,
            source_evaluator,
            target_evaluator,
        })
    }
}

/// Evaluator of `WarmStartingProblem`.
#[derive(Debug)]
pub struct WarmStartingEvaluator {
    source_last_step: u64,
    source_evaluator: BoxEvaluator,
    target_evaluator: BoxEvaluator,
}

impl Evaluator for WarmStartingEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        if next_step == 0 {
            let (_, values) = track!(self.source_evaluator.evaluate(self.source_last_step))?;
            Ok((0, values))
        } else {
            track!(self.target_evaluator.evaluate(next_step))
        }
    }
}
