//! Domain of parameter and objective values.
use crate::{Error, ErrorKind, Result};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};
use structopt::StructOpt;

/// Domain.
///
/// A `Domain` instance consists of a vector of `Variable`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Domain(Vec<Variable>);

#[allow(clippy::len_without_is_empty)]
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

            vars.push(v);
        }
        Ok(Self(vars))
    }

    /// Returns a reference to the variables in this domain.
    pub fn variables(&self) -> &[Variable] {
        &self.0
    }

    /// Returns the number of variables in this domain.
    pub fn len(&self) -> usize {
        self.0.len()
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
    constraint: Option<Constraint>,
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
            constraint: None,
        }
    }

    /// Sets the name of this variable.
    pub fn name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
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

    /// Sets the range of this variable.
    pub fn range(mut self, range: Range) -> Self {
        self.range = range;
        self
    }

    /// Sets the evaluation constraint to this variable.
    pub fn constraint(mut self, constraint: Constraint) -> Self {
        self.constraint = Some(constraint);
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
                track_assert!(!choices.is_empty(), ErrorKind::InvalidInput; self)
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
            constraint: self.constraint,
        })
    }
}
impl From<Variable> for VariableBuilder {
    fn from(f: Variable) -> Self {
        Self {
            name: f.name,
            range: f.range,
            distribution: f.distribution,
            constraint: f.constraint,
        }
    }
}

/// A variable in a domain.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Variable {
    name: String,
    range: Range,
    distribution: Distribution,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    constraint: Option<Constraint>,
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

    /// Returns the constraint required to evaluate this variable.
    pub fn constraint(&self) -> Option<&Constraint> {
        self.constraint.as_ref()
    }
}

impl rand::distributions::Distribution<f64> for Variable {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> f64 {
        match &self.range {
            Range::Continuous { low, high } => match self.distribution {
                Distribution::Uniform => rng.gen_range(*low..*high),
                Distribution::LogUniform => rng.gen_range(low.log2()..high.log2()).exp2(),
            },
            Range::Discrete { low, high } => match self.distribution {
                Distribution::Uniform => rng.gen_range(*low..*high) as f64,
                Distribution::LogUniform => rng
                    .gen_range((*low as f64).log2()..(*high as f64).log2())
                    .exp2()
                    .floor(),
            },
            Range::Categorical { choices } => rng.gen_range(0..choices.len()) as f64,
        }
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

#[allow(clippy::trivially_copy_pass_by_ref)]
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
#[derive(Debug, Clone, Serialize, Deserialize, StructOpt)]
#[allow(missing_docs)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
#[structopt(rename_all = "kebab-case")]
pub enum Range {
    /// Continuous numerical range: `[low..high)`.
    Continuous {
        /// Lower bound of this range (inclusive).
        #[serde(skip_serializing_if = "is_not_finite", default = "neg_infinity")]
        low: f64,

        /// Upper bound of this range (exclusive).
        #[serde(skip_serializing_if = "is_not_finite", default = "infinity")]
        high: f64,
    },

    /// Discrete numerical range: `[low..high)`.
    Discrete {
        /// Lower bound of this range (inclusive).
        low: i64,

        /// Upper bound of this range (exclusive).
        high: i64,
    },

    /// Categorical range.
    Categorical {
        /// Possible choices.
        choices: Vec<String>,
    },
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

    /// Returns `true` if the given value is contained in this range.
    pub fn contains(&self, v: f64) -> bool {
        match self {
            Self::Continuous { low, high } => *low <= v && v < *high,
            Self::Discrete { low, high } => *low as f64 <= v && v < *high as f64,
            Self::Categorical { choices } => 0.0 <= v && v < choices.len() as f64,
        }
    }
}
impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Continuous { low: l0, high: h0 }, Self::Continuous { low: l1, high: h1 }) => {
                OrderedFloat(*l0) == OrderedFloat(*l1) && OrderedFloat(*h0) == OrderedFloat(*h1)
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

/// Evaluation constraint.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Constraint {
    lua_script: String,
}
impl Constraint {
    /// Makes a new `Constraint` instance.
    ///
    /// `lua_script` is the Lua script code that represents the constraint.
    /// In this script, you can access the variables that are located before
    /// the constrainted variable as global variables.
    /// This script must return a boolean value.
    pub fn new(lua_script: &str) -> Self {
        Self {
            lua_script: lua_script.to_owned(),
        }
    }

    /// Returns `Ok(true)` if this constraint is satisfied, otherwise `Ok(false)` or an error.
    pub fn is_satisfied(&self, vars: &[Variable], vals: &[f64]) -> Result<bool> {
        use rlua::Lua;

        let lua = Lua::new();
        lua.context(|lua_ctx| {
            let globals = lua_ctx.globals();

            for (var, &val) in vars.iter().zip(vals.iter()) {
                if !val.is_finite() {
                    continue;
                }

                if let Range::Categorical { choices } = &var.range {
                    let val = choices[val as usize].as_str();
                    track!(globals.set(var.name.as_str(), val).map_err(Error::from))?;
                } else {
                    track!(globals.set(var.name.as_str(), val).map_err(Error::from))?;
                }
            }

            lua_ctx.load(&self.lua_script).eval().map_err(Error::from)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trackable;

    #[test]
    fn constraint_test() -> trackable::result::TopLevelResult {
        let vars = vec![
            var("a").continuous(-10.0, 10.0).finish()?,
            var("b").discrete(0, 5).finish()?,
            var("c").categorical(&["foo", "bar", "baz"]).finish()?,
        ];

        let constraint = Constraint::new("(a + b) < 2");
        assert!(track!(constraint.is_satisfied(&vars, &[0.2, 1.0]))?);
        assert!(!track!(constraint.is_satisfied(&vars, &[1.1, 1.0]))?);

        let constraint = Constraint::new("c == \"bar\"");
        assert!(track!(constraint.is_satisfied(&vars, &[0.2, 1.0, 1.0]))?);
        assert!(!track!(constraint.is_satisfied(&vars, &[0.2, 1.0, 0.0]))?);
        assert!(!track!(constraint.is_satisfied(&vars, &[0.2, 1.0, 2.0]))?);

        Ok(())
    }
}
