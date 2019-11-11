use crate::problem::ProblemSpec;
use crate::solver::SolverSpec;
use crate::trial::{Params, Values};
use crate::ErrorKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolverMessage {
    SolverSpecCast {
        spec: SolverSpec,
    },
    CreateSolverCast {
        solver_id: u64,
        random_seed: u64,
        problem: ProblemSpec,
    },
    DropSolverCast {
        solver_id: u64,
    },
    AskCall {
        solver_id: u64,
        next_trial_id: u64,
    },
    AskReply {
        trial_id: u64,
        next_step: u64,
        params: Params,
        next_trial_id: u64,
    },
    TellCall {
        trial_id: u64,
        current_step: u64,
        values: Values,
    },
    TellReply,
    ErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
