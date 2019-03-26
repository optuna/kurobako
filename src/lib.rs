#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

pub use kurobako_core::{Error, ErrorKind, Result};

pub use kurobako_core::distribution;
pub use kurobako_core::problem::{Evaluate, Problem, ProblemSpace, ProblemSpec};
pub use kurobako_core::ValueRange;

pub mod benchmark;
pub mod optimizer;
pub mod optimizer_suites;
pub mod problem_suites;
pub mod problems;
pub mod runner;
pub mod stats;
pub mod study;
pub mod summary;
pub mod time;
pub mod trial;

mod float;

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
        let a = serde_json::to_string(&self.0).expect("never fails");
        let b = serde_json::to_string(&other.0).expect("never fails");
        a.cmp(&b)
    }
}
