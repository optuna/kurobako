//! Domain of parameter and objective values.
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};

/// Domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain(Vec<Variable>);

/// Variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub range: Range,
    pub distribution: Distribution,
    pub conditions: Vec<Condition>,
}

/// Distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Distribution {
    Uniform,
    LogUniform,
}

/// Variable range.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Range {
    /// Continuous numerical range: `[low..high)`.
    Continuous { low: f64, high: f64 },

    /// Discrete numerical range: `[low..high)`.
    Discrete { low: i64, high: i64 },

    /// Categorical range.
    Categorical { choices: Vec<String> },
}
impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Continuous { low: l0, high: h0 }, Self::Continuous { low: l1, high: h1 }) => {
                l0 == l1 && h0 == h1
            }
            (Self::Discrete { low: l0, high: h0 }, Self::Discrete { low: l1, high: h1 }) => {
                l0 == l1 && h0 == h1
            }
            (Self::Categorical { choices: c0 }, Self::Categorical { choices: c1 }) => c0 == c1,
            _ => false,
        }
    }
}
impl Eq for Range {}

/// Evaluation condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Condition {
    Member { name: String, choices: Vec<String> },
}
