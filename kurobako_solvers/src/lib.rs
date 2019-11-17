//! Built-in solvers of [`kurobako`](https://github.com/sile/kurobako).
#![warn(missing_docs)]

#[macro_use]
extern crate trackable;

pub mod asha;
// pub mod optuna;
pub mod random;

mod error;
