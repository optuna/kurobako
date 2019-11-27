//! **E**xternal **P**rogram **I**nterface.
use crate::trial::Params;
use serde::{Deserialize, Serialize};
use std::f64::NAN;

pub mod channel;
pub mod problem;
pub mod solver;

/// `Params` for transfer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamsForTransfer(Vec<Option<f64>>);
impl From<Params> for ParamsForTransfer {
    fn from(f: Params) -> Self {
        Self(
            f.into_vec()
                .into_iter()
                .map(|v| if v.is_finite() { Some(v) } else { None })
                .collect(),
        )
    }
}
impl From<ParamsForTransfer> for Params {
    fn from(f: ParamsForTransfer) -> Self {
        Params::new(
            f.0.into_iter()
                .map(|v| if let Some(v) = v { v } else { NAN })
                .collect(),
        )
    }
}
