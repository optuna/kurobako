use chrono;
use std::time::Instant;

pub type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Timestamp(f64);

#[derive(Debug)]
pub struct Stopwatch(Instant);
impl Stopwatch {
    pub fn new() -> Self {
        Self(Instant::now())
    }

    pub fn elapsed(&self) -> Timestamp {
        let d = self.0.elapsed();
        Timestamp((d.as_secs() as f64) + (d.subsec_micros() as f64) / 1_000_000.0)
    }
}
