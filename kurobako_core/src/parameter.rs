use crate::{Error, ErrorKind, Result};
use rustats::num::FiniteF64;
use rustats::range::Range;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt::Display;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamDomain {
    Continuous(Continuous),
    Discrete(Discrete),
    Categorical(Categorical),
    Conditional(Conditional),
}
impl ParamDomain {
    pub fn name(&self) -> &str {
        match self {
            ParamDomain::Continuous(p) => &p.name,
            ParamDomain::Discrete(p) => &p.name,
            ParamDomain::Categorical(p) => &p.name,
            ParamDomain::Conditional(p) => p.param.name(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParamValue {
    Continuous(FiniteF64),
    Discrete(i64),
    Categorical(usize),
    Conditional(Option<Box<ParamValue>>), // TODO: Conditional(Option<Box<UnconditionalValue>>)
}
impl ParamValue {
    pub fn as_discrete(&self) -> Option<i64> {
        if let ParamValue::Discrete(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_continuous(&self) -> Option<FiniteF64> {
        if let ParamValue::Continuous(v) = self {
            Some(*v)
        } else {
            None
        }
    }

    pub fn as_categorical(&self) -> Option<usize> {
        if let ParamValue::Categorical(v) = self {
            Some(*v)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Unconditional {
    Continuous(Continuous),
    Discrete(Discrete),
    Categorical(Categorical),
}
impl Unconditional {
    pub fn name(&self) -> &str {
        match self {
            Unconditional::Continuous(p) => &p.name,
            Unconditional::Discrete(p) => &p.name,
            Unconditional::Categorical(p) => &p.name,
        }
    }
}
impl TryFrom<ParamDomain> for Unconditional {
    type Error = Error;

    fn try_from(f: ParamDomain) -> Result<Self> {
        Ok(match f {
            ParamDomain::Categorical(p) => Unconditional::Categorical(p),
            ParamDomain::Conditional(_) => track_panic!(ErrorKind::InvalidInput),
            ParamDomain::Continuous(p) => Unconditional::Continuous(p),
            ParamDomain::Discrete(p) => Unconditional::Discrete(p),
        })
    }
}
// TODO: Implement PriorDistribution

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuous {
    pub name: String,
    pub range: Range<FiniteF64>,

    #[serde(default)]
    pub distribution: Distribution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Discrete {
    pub name: String,
    pub range: Range<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Categorical {
    pub name: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: Condition,
    pub param: Box<Unconditional>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Condition {
    // can refer to only preceeding parameters
    Member { name: String, choices: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")] // TODO: remove kebab-case
pub enum Distribution {
    Uniform,
    LogUniform,
}
impl Default for Distribution {
    fn default() -> Self {
        Distribution::Uniform
    }
}

pub fn when(condition: Condition, param: ParamDomain) -> Result<ParamDomain> {
    let param = Box::new(track!(Unconditional::try_from(param))?);
    Ok(ParamDomain::Conditional(Conditional { condition, param }))
}

pub fn category_eq(name: &str, value: &str) -> Condition {
    Condition::Member {
        name: name.to_owned(),
        choices: vec![value.to_owned()],
    }
}

pub fn boolean(name: &str) -> ParamDomain {
    choices(name, &["false", "true"])
}

pub fn choices<I, C>(name: &str, choices: I) -> ParamDomain
where
    I: IntoIterator<Item = C>,
    C: Display,
{
    ParamDomain::Categorical(Categorical {
        name: name.to_owned(),
        choices: choices.into_iter().map(|c| c.to_string()).collect(),
    })
}

pub fn uniform(name: &str, low: f64, high: f64) -> Result<ParamDomain> {
    let low = track!(FiniteF64::new(low))?;
    let high = track!(FiniteF64::new(high))?;
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Continuous(Continuous {
        name: name.to_owned(),
        range,
        distribution: Distribution::Uniform,
    }))
}

pub fn log_uniform(name: &str, low: f64, high: f64) -> Result<ParamDomain> {
    let low = track!(FiniteF64::new(low))?;
    let high = track!(FiniteF64::new(high))?;
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Continuous(Continuous {
        name: name.to_owned(),
        range,
        distribution: Distribution::LogUniform,
    }))
}

pub fn int(name: &str, low: i64, high: i64) -> Result<ParamDomain> {
    let range = track!(Range::new(low, high))?;
    Ok(ParamDomain::Discrete(Discrete {
        name: name.to_owned(),
        range,
    }))
}
