use kurobako_core::parameter::{self, Distribution};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VarPath(Vec<String>);
impl VarPath {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn push(&mut self, s: String) {
        self.0.push(s);
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }
}
impl fmt::Display for VarPath {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join("."))
    }
}
impl FromStr for VarPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(Self(s.split('.').map(|s| s.to_owned()).collect()))
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[structopt(rename_all = "kebab-case")]
pub enum Variable {
    Float(Float),
    Int(Int),
    //Bool,
}
impl Variable {
    // TODO: TryFrom or TryInto
    pub fn to_param_domain(&self) -> Result<parameter::ParamDomain> {
        match self {
            Variable::Float(v) => track!(v.to_param_domain()),
            Variable::Int(v) => track!(v.to_param_domain()),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct Float {
    pub path: VarPath,

    pub low: f64,
    pub high: f64,

    #[structopt(long, default_value = "uniform")]
    #[serde(default, skip_serializing_if = "Distribution::is_uniform")]
    pub distribution: Distribution,
    // TODO: condition
}
impl Float {
    pub fn to_param_domain(&self) -> Result<parameter::ParamDomain> {
        match self.distribution {
            Distribution::Uniform => track!(parameter::uniform(
                &self.path.to_string(),
                self.low,
                self.high
            )),
            Distribution::LogUniform => track!(parameter::log_uniform(
                &self.path.to_string(),
                self.low,
                self.high
            )),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct Int {
    pub path: VarPath, // e.g., "solver.optuna.tpe-gamma-factor"
    // TODO: short-name
    pub low: i64,
    pub high: i64,
}
impl Int {
    pub fn to_param_domain(&self) -> Result<parameter::ParamDomain> {
        track!(parameter::int(&self.path.to_string(), self.low, self.high))
    }
}
