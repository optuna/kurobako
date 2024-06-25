//! `kurobako run` command.
use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use kurobako_core::problem::ProblemRecipe as _;
use kurobako_core::problem::{Evaluator as _, Problem as _, ProblemFactory as _};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use serde::Deserialize;
use serde::Serialize;
use std::io;
use structopt::StructOpt;
use serde_json::Error;
use std::io::Write;

/// Options of the `kurobako batch-evaluate` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct BatchEvaluateOpt {
    /// Evaluation target problem.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub problem: KurobakoProblemRecipe,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct EvalCall {
    params: Params,
    step: Option<u64>
}

#[derive(Debug, Clone, Serialize)]
struct EvalReply {
    values: Values,
}

impl BatchEvaluateOpt {
    /// Evaluates the given parameters.
    pub fn run(&self) -> Result<()> {
        let random_seed = self.seed.unwrap_or_else(rand::random);
        let rng = ArcRng::new(random_seed);
        let registry = FactoryRegistry::new::<KurobakoProblemRecipe, KurobakoSolverRecipe>();
        let problem_factory = track!(self.problem.create_factory(&registry))?;
        let problem_spec = track!(problem_factory.specification())?;

        let problem = track!(problem_factory.create_problem(rng))?;
        let mut writer = io::stdout();
        loop{
            let mut line = String::new();
            let n = io::stdin().read_line(&mut line)?;
            if n == 0 {
                break;
            }
            let EvalCall { params, step } = serde_json::from_str(&line).map_err(Error::from)?;

            track_assert_eq!(
                params.len(),
                problem_spec.params_domain.variables().len(),
                ErrorKind::InvalidInput
            );

            
            let evaluator_or_error = track!(problem.create_evaluator(params.clone()));

            let values = match evaluator_or_error {
                Ok(mut evaluator) => {
                    let s = step.unwrap_or_else(|| problem_spec.steps.last());
                    let (_, values) = track!(evaluator.evaluate(s))?;
                    values
                },
                Err(e) => {
                    if *e.kind() != ErrorKind::UnevaluableParams {
                        return Err(e);
                    } else {
                        Values::new(vec![])
                    }
                }
            };

            serde_json::to_writer(&mut writer, &EvalReply{values}).map_err(Error::from)?;
            writer.write("\n".as_bytes())?;
            writer.flush()?;
        }
        Ok(())
    }
}
