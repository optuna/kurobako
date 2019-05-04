use crate::parameter::ParamValue;
use crate::problem::ProblemSpec;
use crate::ErrorKind;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemSpecNotification(pub ProblemSpec);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EvaluateRequest {
    Create {
        id: ObsId,
    },
    Drop {
        id: ObsId,
    },
    Evaluate {
        id: ObsId,
        params: Vec<ParamValue>,
        budget: NonZeroU64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum EvaluateResponse {
    Ok {
        id: ObsId,
        values: Vec<FiniteF64>,
        overrun: u64,
    },
    Error {
        id: ObsId,
        kind: ErrorKind,
        message: Option<String>,
    },
}
