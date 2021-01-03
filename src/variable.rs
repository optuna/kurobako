//! `kurobako var` command.
use kurobako_core::domain::{Range, VariableBuilder};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use structopt::StructOpt;

/// Variable.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct Var {
    /// Path of the target variable on a recipe JSON.
    pub path: VarPath,

    /// Makes the distribution of the variable log scale.
    #[structopt(long)]
    #[serde(default)]
    pub log_uniform: bool,

    #[structopt(flatten)]
    #[allow(missing_docs)]
    pub range: Range,
}
impl Var {
    /// Converts to `VariableBuilder`.
    pub fn to_domain_var(&self) -> VariableBuilder {
        let builder = VariableBuilder::new(&self.path.to_string()).range(self.range.clone());
        if self.log_uniform {
            builder.log_uniform()
        } else {
            builder
        }
    }
}

/// Path of a variable.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VarPath(Vec<String>);
impl VarPath {
    /// Makes a new `VarPath` instance.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Appends a component to this path.
    pub fn push(&mut self, s: String) {
        self.0.push(s);
    }

    /// Pops a last component of this path
    pub fn pop(&mut self) {
        self.0.pop();
    }

    /// Returns an iterator that iterates over the components in this path.
    pub fn components(&self) -> impl '_ + Iterator<Item = &str> {
        self.0.iter().map(|x| x.as_str())
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
