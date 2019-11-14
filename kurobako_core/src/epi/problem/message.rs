use crate::problem::ProblemSpec;
use crate::trial::{Params, Values};
use crate::ErrorKind;
use serde::{Deserialize, Serialize};

/// Messages that are used to communicate with external problems.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemMessage {
    ProblemSpecCast {
        spec: ProblemSpec,
    },
    CreateProblemCast {
        problem_id: u64,
        random_seed: u64,
    },
    DropProblemCast {
        problem_id: u64,
    },
    CreateEvaluatorCall {
        problem_id: u64,
        evaluator_id: u64,
        params: Params,
    },
    CreateEvaluatorReply,
    DropEvaluatorCast {
        problem_id: u64,
        evaluator_id: u64,
    },
    EvaluateCall {
        problem_id: u64,
        evaluator_id: u64,
        max_step: u64,
    },
    EvaluateReply {
        current_step: u64,
        values: Values,
    },
    ErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
