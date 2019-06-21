#[macro_use]
extern crate log;
#[macro_use]
extern crate trackable;

pub mod benchmark;
pub mod exam;
pub mod filter;
pub mod filters;
pub mod homonym;
pub mod markdown;
pub mod multi_exam;
pub mod plot;
pub mod plot_scatter; // TODO: merge with plot
pub mod problem;
pub mod problem_suites;
pub mod record;
pub mod runner;
pub mod select;
pub mod solver;
pub mod stats;
pub mod time;
pub mod variable;

mod rankings;
