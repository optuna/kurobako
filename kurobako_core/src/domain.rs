//! Domain of parameter and objective values.
use crate::{ErrorKind, Result};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std;
use std::hash::{Hash, Hasher};

/// Domain.
///
/// A `Domain` instance consists of a vector of `Variable`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Domain(Vec<Variable>);
impl Domain {
    /// Makes a new `Domain` instance.
    pub fn new(variables: Vec<VariableBuilder>) -> Result<Self> {
        track_assert!(!variables.is_empty(), ErrorKind::InvalidInput);

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

    /// Returns a reference to the variables in this domain.
    pub fn variables(&self) -> &[Variable] {
        &self.0
    }
}

/// Returns a `VariableBuilder` which was initialized with the given variable name.
///
/// This is equivalent to `VariableBuilder::new(name)`.
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
    /// Makes a new `VariableBuilder` with the given variable name.
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

    /// Sets the distribution of this variable to `Distribution::Uniform`.
    ///
    /// Note that `Distribution::Uniform` is the default distribution.
    pub fn uniform(mut self) -> Self {
        self.distribution = Distribution::Uniform;
        self
    }

    /// Sets the distribution of this variable to `Distribution::LogUniform`.
    pub fn log_uniform(mut self) -> Self {
        self.distribution = Distribution::LogUniform;
        self
    }

    /// Sets the range of this variable to the given continuous numerical range.
    pub fn continuous(mut self, low: f64, high: f64) -> Self {
        self.range = Range::Continuous { low, high };
        self
    }

    /// Sets the range of this variable to the given discrete numerical range.
    pub fn discrete(mut self, low: i64, high: i64) -> Self {
        self.range = Range::Discrete { low, high };
        self
    }

    /// Sets the range of this variable to the given categorical range.
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

    /// Sets the range of this variable to boolean.
    ///
    /// This is equivalent to `self.categorical(&["false", "true"])`.
    pub fn boolean(self) -> Self {
        self.categorical(&["false", "true"])
    }

    /// Adds an evaluation condition to this variable.
    pub fn condition(mut self, condition: Condition) -> Self {
        self.conditions.push(condition);
        self
    }

    /// Builds a `Variable` instance with the given settings.
    pub fn finish(self) -> Result<Variable> {
        match &self.range {
            Range::Continuous { low, high } => {
                track_assert!(low < high, ErrorKind::InvalidInput; self)
            }
            Range::Discrete { low, high } => {
                track_assert!(low < high, ErrorKind::InvalidInput; self)
            }
            Range::Categorical { choices } => {
                track_assert!(choices.len() > 0, ErrorKind::InvalidInput; self)
            }
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
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    range: Range,
    distribution: Distribution,
    #[serde(default)]
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
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Distribution {
    Uniform,
    LogUniform,
}

fn is_not_finite(x: &f64) -> bool {
    !x.is_finite()
}

fn neg_infinity() -> f64 {
    std::f64::NEG_INFINITY
}

fn infinity() -> f64 {
    std::f64::INFINITY
}

/// Variable range.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Range {
    /// Continuous numerical range: `[low..high)`.
    Continuous {
        #[serde(skip_serializing_if = "is_not_finite", default = "neg_infinity")]
        low: f64,

        #[serde(skip_serializing_if = "is_not_finite", default = "infinity")]
        high: f64,
    },

    /// Discrete numerical range: `[low..high)`.
    Discrete { low: i64, high: i64 },

    /// Categorical range.
    Categorical { choices: Vec<String> },
}
impl Range {
    /// Returns the inclusive lower bound of this range.
    pub fn low(&self) -> f64 {
        match self {
            Self::Continuous { low, .. } => *low,
            Self::Discrete { low, .. } => *low as f64,
            Self::Categorical { .. } => 0.0,
        }
    }

    /// Returns the exclusive upper bound of this range.
    pub fn high(&self) -> f64 {
        match self {
            Self::Continuous { high, .. } => *high,
            Self::Discrete { high, .. } => *high as f64,
            Self::Categorical { choices } => choices.len() as f64,
        }
    }

    fn contains(&self, v: f64) -> bool {
        match self {
            Self::Continuous { low, high } => *low <= v && v < *high,
            Self::Discrete { low, high } => *low as f64 <= v && v < *high as f64,
            Self::Categorical { choices } => 0.0 <= v && v < choices.len() as f64,
        }
    }
}
impl Eq for Range {}
impl Hash for Range {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Continuous { low, high } => {
                OrderedFloat(*low).hash(state);
                OrderedFloat(*high).hash(state);
            }
            Self::Discrete { low, high } => {
                low.hash(state);
                high.hash(state);
            }
            Self::Categorical { choices } => {
                choices.hash(state);
            }
        }
    }
}

/// Evaluation condition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Condition {
    /// This condition holds if the value of the variable named `target` is equal to `value`.
    Eq { target: String, value: f64 },
}
impl Condition {
    fn validate(&self, preceding_variables: &[Variable]) -> Result<()> {
        let Condition::Eq { target, value } = self;

        for v in preceding_variables {
            if target != &v.name {
                continue;
            }

            track_assert!(v.range.contains(*value), ErrorKind::InvalidInput; self);
        }

        track_panic!(ErrorKind::InvalidInput; self);
    }
}
impl Eq for Condition {}
impl Hash for Condition {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let Condition::Eq { target, value } = self;
        target.hash(state);
        OrderedFloat(*value).hash(state);
    }
}
