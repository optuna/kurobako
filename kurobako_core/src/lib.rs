#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

pub use error::{Error, ErrorKind};

pub mod distribution;
pub mod problem;
pub mod problems;

mod error;
mod serde_json_line;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ValueRange {
    pub min: f64,
    pub max: f64,
}
impl ValueRange {
    pub fn normalize(self, v: f64) -> f64 {
        (v - self.min) / (self.max - self.min)
    }
}
