//! Time related components.
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

/// Datetime.
pub type DateTime = chrono::DateTime<chrono::Local>;

/// Elapsed seconds.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct ElapsedSeconds(f64);
impl ElapsedSeconds {
    /// Makes a new `ElapsedSeconds` instance.
    pub fn new(seconds: f64) -> Self {
        Self(seconds)
    }

    /// Makes a `ElapsedSeconds` instance that represents the zero elapsed seconds.
    pub const fn zero() -> Self {
        Self(0.0)
    }

    /// Returns the elapsed seconds value.
    pub const fn get(self) -> f64 {
        self.0
    }

    /// Converts the elapsed seconds to `Duration`.
    pub fn to_duration(self) -> Duration {
        Duration::from_secs_f64(self.0)
    }

    /// Executes the given function, and returns the result and elapsed time.
    pub fn time<F, T>(f: F) -> (T, Self)
    where
        F: FnOnce() -> T,
    {
        let now = Instant::now();
        let result = f();
        (result, Self::from(now.elapsed()))
    }

    /// Executes the given function that may fail, and returns the result and elapsed time.
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
