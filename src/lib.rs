#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

pub mod distribution;
pub mod optimizer;
pub mod problem;
pub mod runner;
pub mod study;
pub mod summary;
pub mod time;
pub mod trial;

mod float;
