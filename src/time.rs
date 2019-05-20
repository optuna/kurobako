use chrono;
use kurobako_core::num::FiniteF64;
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ElapsedSeconds(FiniteF64);
impl ElapsedSeconds {
    pub fn new(seconds: f64) -> Result<Self> {
        let seconds = track!(FiniteF64::new(seconds))?;
        Ok(Self(seconds))
    }

    pub const fn zero() -> Self {
        Self(unsafe { FiniteF64::new_unchecked(0.0) })
    }

    pub const fn get(&self) -> f64 {
        self.0.get()
    }

    pub fn to_duration(&self) -> Duration {
        let secs = self.0.get() as u64;
        let nanos = (self.0.get().fract() * 1_000_000_000.0) as u32;
        Duration::new(secs, nanos)
    }

    pub fn time<F, T>(f: F) -> (T, Self)
    where
        F: FnOnce() -> T,
    {
        let now = Instant::now();
        let result = f();
        (result, Self::from(now.elapsed()))
    }
}
impl From<Duration> for ElapsedSeconds {
    fn from(f: Duration) -> Self {
        let secs = f.as_secs() as f64;
        let micros = f.as_micros() as f64;
        Self(unsafe { FiniteF64::new_unchecked(secs + micros / 1_000_000.0) })
    }
}
