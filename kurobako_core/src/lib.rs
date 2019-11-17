//! The core crate of [`kurobako`](https://github.com/sile/kurobako).
#![warn(missing_docs)]

#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};

pub mod domain;
pub mod epi;
pub mod json;
pub mod num;
pub mod problem;
pub mod registry;
pub mod rng;
pub mod solver;
pub mod trial;

mod error;

/// This crate specific `Result` type.
pub type Result<T, E = Error> = std::result::Result<T, E>;
