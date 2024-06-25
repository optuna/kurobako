use kurobako_core::domain::{self, Distribution, Domain, Range, VariableBuilder};
use kurobako_core::json::JsonRecipe;
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, BoxProblemFactory, Evaluator, Problem, ProblemFactory, ProblemRecipe,
    ProblemSpec,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Recipe to convert the distributions of continuous variables of a problem from uniform to log-uniform.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct LnProblemRecipe {
    /// Problem recipe JSON.
    pub problem: JsonRecipe,
}

impl ProblemRecipe for LnProblemRecipe {
    type Factory = LnProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let problem = track!(registry.create_problem_factory_from_json(&self.problem))?;
        Ok(LnProblemFactory { problem })
    }
}

#[derive(Debug)]
pub struct LnProblemFactory {
    problem: BoxProblemFactory,
}

impl ProblemFactory for LnProblemFactory {
    type Problem = LnProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let mut spec = track!(self.problem.specification())?;

        let mut transformed_vars = Vec::new();
        for var in spec.params_domain.variables() {
            if let (Range::Continuous { low, high }, Distribution::Uniform) =
                (var.range(), var.distribution())
            {
                transformed_vars.push(
                    domain::var(var.name())
                        .continuous(low.exp(), high.exp())
                        .log_uniform(),
                );
            } else {
                transformed_vars.push(VariableBuilder::from(var.clone()));
            }
        }
        spec.params_domain = track!(Domain::new(transformed_vars))?;

        Ok(spec)
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let problem = track!(self.problem.create_problem(rng))?;
        let spec = track!(self.specification())?;
        Ok(LnProblem { problem, spec })
    }
}

#[derive(Debug)]
pub struct LnProblem {
    problem: BoxProblem,
    spec: ProblemSpec,
}

impl Problem for LnProblem {
    type Evaluator = LnEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let params = self
            .spec
            .params_domain
            .variables()
            .iter()
            .zip(params.into_vec())
            .map(|(var, val)| {
                if let Range::Continuous { .. } = var.range() {
                    assert_eq!(var.distribution(), Distribution::LogUniform);
                    val.ln()
                } else {
                    val
                }
            })
            .collect::<Vec<_>>();

        let evaluator = track!(self.problem.create_evaluator(Params::new(params)))?;
        Ok(LnEvaluator { evaluator })
    }
}

#[derive(Debug)]
pub struct LnEvaluator {
    evaluator: BoxEvaluator,
}

impl Evaluator for LnEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        track!(self.evaluator.evaluate(next_step))
    }
}
