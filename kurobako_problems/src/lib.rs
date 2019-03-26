#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

pub mod problems;

pub use kurobako_core::{Error, ErrorKind, Result};
