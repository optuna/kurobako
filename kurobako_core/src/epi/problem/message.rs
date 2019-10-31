use crate::problem::ProblemSpec;
// use crate::trial::TrialId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemMessage {
    ProblemSpecCast { spec: ProblemSpec },
    // CreateProblemCast {
    //     id: TrialId,
    // },
    // DropProblemCast {
    //     id: TrialId,
    // },
    // EvaluateCall {
    //     id: TrialId,
    //     params: Vec<ParamValue>,
    //     budget: Budget,
    // },
    // EvaluateOkReply {
    //     values: Vec<FiniteF64>,
    //     budget: Budget,
    // },
    // EvaluateErrorReply {
    //     kind: ErrorKind,
    //     #[serde(default)]
    //     message: Option<String>,
    // },
}
