use crate::Result;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct Seconds(FiniteF64);
impl Seconds {
    pub fn new(seconds: f64) -> Result<Self> {
        let seconds = track!(FiniteF64::new(seconds))?;
        Ok(Self(seconds))
    }

    pub const fn zero() -> Self {
        Seconds(unsafe { FiniteF64::new_unchecked(0.0) })
    }

    pub const fn get(&self) -> f64 {
        self.0.get()
    }

    pub fn to_duration(&self) -> Duration {
        let secs = self.0.get() as u64;
        let nanos = (self.0.get().fract() * 1_000_000_000.0) as u32;
        Duration::new(secs, nanos)
    }
}
impl From<Duration> for Seconds {
    fn from(f: Duration) -> Self {
        let secs = f.as_secs() as f64;
        let micros = f.as_micros() as f64;
        Self(unsafe { FiniteF64::new_unchecked(secs + micros / 1_000_000.0) })
    }
}
