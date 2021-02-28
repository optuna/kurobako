use crate::solver::KurobakoSolverRecipe;
use kurobako_core::solver::SolverSpec;
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolverRecord {
    pub recipe: KurobakoSolverRecipe,
    pub spec: SolverSpec,
}
impl SolverRecord {
    pub fn id(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.update(&track!(
            serde_json::to_vec(&self.recipe).map_err(Error::from)
        )?);
        hasher.update(&track!(serde_json::to_vec(&self.spec).map_err(Error::from))?);

        let mut id = String::with_capacity(64);
        for b in hasher.finalize().as_slice() {
            track_write!(&mut id, "{:02x}", b)?;
        }
        Ok(id)
    }
}
