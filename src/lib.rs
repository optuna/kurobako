#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

use serde::{Deserialize, Serialize};

pub mod benchmark;
pub mod optimizer;
pub mod plot;
pub mod problem_suites;
pub mod problems;
pub mod runner;
pub mod solver;
pub mod stats;
pub mod study;
pub mod time;
pub mod trial;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Name(serde_json::Value);
impl Name {
    pub fn new(v: serde_json::Value) -> Self {
        Name(v)
    }

    pub fn as_json(&self) -> &serde_json::Value {
        &self.0
    }
}
impl Eq for Name {}
impl PartialOrd for Name {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Name {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let a = serde_json::to_string(&self.0).unwrap_or_else(|e| panic!("never fails: {}", e));
        let b = serde_json::to_string(&other.0).unwrap_or_else(|e| panic!("never fails: {}", e));
        a.cmp(&b)
    }
}
