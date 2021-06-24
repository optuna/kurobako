//! Built-in problems of [`kurobako`](https://github.com/optuna/kurobako).
#![warn(missing_docs)]

#[macro_use]
extern crate trackable;

pub mod hpobench;
pub mod nasbench;
pub mod sigopt;
pub mod surrogate;
pub mod warm_starting;
pub mod zdt;
