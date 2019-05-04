//! The core crate for [`kurobako`](https://github.com/sile/kurobako).
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};

pub mod parameter;
pub mod problem;
//pub mod problems;

mod error;
mod serde_json_line;

/// This crate specific `Result` type.
pub type Result<T> = std::result::Result<T, Error>;
