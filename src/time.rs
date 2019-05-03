use chrono;
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Timestamp(f64);
impl Timestamp {
    pub fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    pub fn as_seconds(&self) -> f64 {
        self.0
    }
}

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
