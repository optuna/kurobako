use rustats::range::Range;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDomain(Vec<Param>);
impl ParamDomain {
    pub const fn new(domain: Vec<Param>) -> Self {
        Self(domain)
    }

    pub fn get(&self) -> &[Param] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParamValue {
    Continuous(f64),
    Discrete(i64),
    Categorical(String),
    Conditional(Option<Box<ParamValue>>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Param {
    Continuous(Continuous),
    Discrete(Discrete),
    Categorical(Categorical),
    Conditional(Conditional),
}
impl Param {
    pub fn name(&self) -> &str {
        match self {
            Param::Continuous(p) => &p.name,
            Param::Discrete(p) => &p.name,
            Param::Categorical(p) => &p.name,
            Param::Conditional(p) => p.param.name(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Continuous {
    pub name: String,
    pub range: Range<f64>,
    pub distribution: Distribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Discrete {
    pub name: String,
    pub range: Range<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Categorical {
    pub name: String,
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conditional {
    pub condition: Condition,
    pub param: Box<Unconditional>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub fn boolean(name: &str) -> Param {
    choices(name, &["false", "true"])
}

pub fn choices<I, C>(name: &str, choices: I) -> Param
where
    I: IntoIterator<Item = C>,
    C: Display,
{
    Param::Categorical(Categorical {
        name: name.to_owned(),
        choices: choices.into_iter().map(|c| c.to_string()).collect(),
    })
}
