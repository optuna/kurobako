//! Domain of parameter and objective values.
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std;

/// Domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Domain(Vec<Variable>);
impl Domain {
    /// Makes a new `Domain` instance.
    pub fn new(variables: Vec<VariableBuilder>) -> Result<Self> {
        let mut vars = Vec::<Variable>::new();
        for v in variables.into_iter() {
            let v = track!(v.finish())?;

            track_assert!(
                vars.iter().all(|var| v.name != var.name),
                ErrorKind::InvalidInput,
                "Duplicate name: {:?}",
                v.name
            );

            for c in &v.conditions {
                track!(c.validate(&vars))?;
            }

            vars.push(v);
        }
        Ok(Self(vars))
    }

    /// Returns an iterator visiting variables in this domain.
    pub fn variables<'a>(&'a self) -> impl 'a + Iterator<Item = &'a Variable> {
        self.0.iter()
    }
}

pub fn var(name: &str) -> VariableBuilder {
    VariableBuilder::new(name)
}

/// `Variable` builder.
#[derive(Debug)]
pub struct VariableBuilder {
    name: String,
    range: Range,
    distribution: Distribution,
    conditions: Vec<Condition>,
}
impl VariableBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            range: Range::Continuous {
                low: std::f64::NEG_INFINITY,
                high: std::f64::INFINITY,
            },
            distribution: Distribution::Uniform,
            conditions: Vec::new(),
        }
    }

    pub fn uniform(mut self) -> Self {
        self.distribution = Distribution::Uniform;
        self
    }

    pub fn log_uniform(mut self) -> Self {
        self.distribution = Distribution::LogUniform;
        self
    }

    pub fn continuous(mut self, low: f64, high: f64) -> Self {
        self.range = Range::Continuous { low, high };
        self
    }

    pub fn discrete(mut self, low: i64, high: i64) -> Self {
        self.range = Range::Discrete { low, high };
        self
    }

    pub fn categorical<I, T>(mut self, choices: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        self.range = Range::Categorical {
            choices: choices.into_iter().map(|c| c.as_ref().to_owned()).collect(),
        };
        self
    }

    pub fn boolean(self) -> Self {
        self.categorical(&["false", "true"])
    }

    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    pub fn finish(self) -> Result<Variable> {
        match &self.range {
            Range::Continuous { low, high } => track_assert!(low < high, ErrorKind::InvalidInput; self),
            Range::Discrete { low, high } => track_assert!(low < high, ErrorKind::InvalidInput; self),
            Range::Categorical { choices } => track_assert!(choices.len() > 0, ErrorKind::InvalidInput; self),
        }

        if self.distribution == Distribution::LogUniform {
            match self.range {
                Range::Continuous { low, .. } if 0.0 < low => {}
                Range::Discrete { low, .. } if 0 < low => {}
                _ => track_panic!(ErrorKind::InvalidInput; self),
            }
        }

        Ok(Variable {
            name: self.name,
            range: self.range,
            distribution: self.distribution,
            conditions: self.conditions,
        })
    }
}

/// A variable in a domain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    range: Range,
    distribution: Distribution,
    conditions: Vec<Condition>,
}
impl Variable {
    /// Returns the name of this variable.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the value range of this variable.
    pub fn range(&self) -> &Range {
        &self.range
    }

    /// Returns the prior distribution of the value of this variable.
    pub fn distribution(&self) -> Distribution {
        self.distribution
    }

    /// Returns the conditions required to evaluate this variable.
    pub fn conditions(&self) -> &[Condition] {
        &self.conditions
    }
}

/// Distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
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
    Category { name: String, value: String },
}
impl Condition {
    pub fn category(name: &str, value: &str) -> Self {
        Self::Category {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }

    fn validate(&self, preceding_variables: &[Variable]) -> Result<()> {
        let Condition::Category { name, value } = self;

        for v in preceding_variables {
            if name != &v.name {
                continue;
            }

            if let Range::Categorical { choices } = &v.range {
                if choices.iter().find(|&c| c == value).is_some() {
                    return Ok(());
                }
            }
        }

        track_panic!(ErrorKind::InvalidInput; self);
    }
}
