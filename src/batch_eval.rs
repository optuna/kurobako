//! `kurobako run` command.
use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::epi::channel::{MessageReceiver, MessageSender};
use kurobako_core::problem::ProblemRecipe as _;
use kurobako_core::problem::{
    Evaluator as _, Problem as _, ProblemFactory as _,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Values, Params};
use kurobako_core::{ErrorKind, Result};
use structopt::StructOpt;
use std::io;
use serde::Serialize;
use serde::Deserialize;
use kurobako_core::json;



/// Options of the `kurobako evaluate` command.
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
/// Messages that are used to communicate with external solvers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BatchEvaluateMessage {
    EvalCall {
        /// Parameters to be evaluated.
        params: Params,
        step: Option<u64>,
    },
    EvalReply {
        values: Values,
    },
    EvalEnd,
}


impl BatchEvaluateOpt {
    /// Evaluates the given parameters.
    pub fn run(&self) -> Result<()> {
        let mut rx: MessageReceiver<BatchEvaluateMessage, _> = MessageReceiver::new(io::stdin());
        let mut tx: MessageSender<BatchEvaluateMessage, _> = MessageSender::new(io::stdout());
        let random_seed = self.seed.unwrap_or_else(rand::random);
        let rng = ArcRng::new(random_seed);
        let registry = FactoryRegistry::new::<KurobakoProblemRecipe, KurobakoSolverRecipe>();
        let problem_factory = track!(self.problem.create_factory(&registry))?;
        let problem_spec = track!(problem_factory.specification())?;
        
        let problem = track!(problem_factory.create_problem(rng))?;
        
        loop{
            match track!(rx.recv())? {
                BatchEvaluateMessage::EvalCall { params, step } => {
                    track_assert_eq!(
                        params.len(),
                        problem_spec.params_domain.variables().len(),
                        ErrorKind::InvalidInput
                    );
                    let mut evaluator = track!(problem.create_evaluator(params.clone()))?;
                    let s = step.unwrap_or_else(|| problem_spec.steps.last());
                    let (_, values) = track!(evaluator.evaluate(s))?;
                    track!(tx.send(&BatchEvaluateMessage::EvalReply { values }))?;
                },
                BatchEvaluateMessage::EvalEnd => {
                    break;
                }
                m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
            };
        };
        Ok(())
    }
}

