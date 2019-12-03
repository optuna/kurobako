use kurobako_core::domain::{Range, VariableBuilder};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct Var {
    pub path: VarPath,

    #[structopt(long)]
    #[serde(default)]
    pub log_uniform: bool,

    #[structopt(flatten)]
    pub range: Range,
}
impl Var {
    pub fn to_domain_var(&self) -> VariableBuilder {
        let builder = VariableBuilder::new(&self.path.to_string()).range(self.range.clone());
        if self.log_uniform {
            builder.log_uniform()
        } else {
            builder
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
