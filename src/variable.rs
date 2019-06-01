use kurobako_core::parameter::{self, Distribution};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[structopt(rename_all = "kebab-case")]
pub enum Variable {
    Float(Float),
    Int(Int),
    //Bool,
}
impl Variable {
    pub fn to_param_domain(&self, name: &str) -> Result<parameter::ParamDomain> {
        match self {
            Variable::Float(v) => track!(v.to_param_domain(name)),
            Variable::Int(v) => track!(v.to_param_domain(name)),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct Float {
    pub low: f64,
    pub high: f64,

    #[structopt(long, default_value = "uniform")]
    #[serde(default, skip_serializing_if = "Distribution::is_uniform")]
    pub distribution: Distribution,
    // TODO: condition
}
impl Float {
    pub fn to_param_domain(&self, name: &str) -> Result<parameter::ParamDomain> {
        match self.distribution {
            Distribution::Uniform => track!(parameter::uniform(name, self.low, self.high)),
            Distribution::LogUniform => track!(parameter::log_uniform(name, self.low, self.high)),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct Int {
    pub low: i64,
    pub high: i64,
}
impl Int {
    pub fn to_param_domain(&self, name: &str) -> Result<parameter::ParamDomain> {
        track!(parameter::int(name, self.low, self.high))
    }
}
