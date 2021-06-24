//! Built-in solvers of [`kurobako`](https://github.com/optuna/kurobako).
#![warn(missing_docs)]

#[macro_use]
extern crate trackable;

pub mod asha;
pub mod nsga2;
pub mod optuna;
pub mod random;

mod error;
mod yamakan_utils;
