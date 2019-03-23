#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

pub mod distribution;
pub mod problem;
pub mod problems;

mod serde_json_line;

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
