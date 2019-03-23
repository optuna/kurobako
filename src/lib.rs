#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

pub use self::problem::{Evaluate, Problem, ProblemSpace, ProblemSpec};

pub mod distribution;
pub mod optimizer;
pub mod problems;
pub mod runner;
pub mod study;
pub mod summary;
pub mod time;
pub mod trial;

mod float;
mod problem;
mod serde_json_line;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: f64,
    pub max: f64,
}
