use chrono;
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

pub type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ElapsedSeconds(f64);
impl ElapsedSeconds {
    pub fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    pub const fn zero() -> Self {
        Self(0.0)
    }

    pub const fn get(self) -> f64 {
        self.0
    }

    pub fn to_duration(self) -> Duration {
        Duration::from_secs_f64(self.0)
    }

    pub fn time<F, T>(f: F) -> (T, Self)
    where
        F: FnOnce() -> T,
    {
        let now = Instant::now();
        let result = f();
        (result, Self::from(now.elapsed()))
    }

    pub fn try_time<F, T>(f: F) -> Result<(T, Self)>
    where
        F: FnOnce() -> Result<T>,
    {
        let now = Instant::now();
        let value = f()?;
        Ok((value, Self::from(now.elapsed())))
    }
}
impl From<Duration> for ElapsedSeconds {
    fn from(f: Duration) -> Self {
        Self(f.as_secs_f64())
    }
}
