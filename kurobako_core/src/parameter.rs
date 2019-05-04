use rustats::num::FiniteF64;
use rustats::range::Range;
use serde::{Deserialize, Serialize};
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
    Categorical(String),
    Conditional(Option<Box<ParamValue>>),
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Continuous {
    pub name: String,
    pub range: Range<FiniteF64>,
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
    Member { name: String, choices: Vec<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Distribution {
    Uniform,
    LogUniform,
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
