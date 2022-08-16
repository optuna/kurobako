//! **R**andom **N**number **G**enerator.
use crate::Result;
use rand::rngs::StdRng;
use rand::{Error, RngCore, SeedableRng};
use std::sync::{Arc, Mutex};

pub use rand::Rng;

/// The random number generator for `kurobako`.
#[derive(Debug, Clone)]
pub struct ArcRng(Arc<Mutex<StdRng>>);
impl ArcRng {
    /// Makes a new `ArcRng` with the given random seed.
    pub fn new(seed: u64) -> Self {
        let mut seed256 = [0; 32];
        seed256[0..8].copy_from_slice(&seed.to_be_bytes());

        let inner = StdRng::from_seed(seed256);
        Self(Arc::new(Mutex::new(inner)))
    }

    /// Acquires the lock of this instance and invokes `f` with the internal RNG.
    pub fn with_lock<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut StdRng) -> T,
    {
        let mut rng = track!(self.0.lock().map_err(crate::Error::from))?;
        Ok(f(&mut rng))
    }
}
impl RngCore for ArcRng {
    fn next_u32(&mut self) -> u32 {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.lock().unwrap_or_else(|e| panic!("{}", e)).next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0
            .lock()
            .unwrap_or_else(|e| panic!("{}", e))
            .fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0
            .lock()
            .unwrap_or_else(|e| panic!("{}", e))
            .try_fill_bytes(dest)
    }
}
