use crate::problem::KurobakoProblemRecipe;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProblemRecord {
    pub recipe: KurobakoProblemRecipe,
    pub spec: ProblemSpec,
}
impl ProblemRecord {
    pub fn id(&self) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(&track!(
            serde_json::to_vec(&self.recipe).map_err(Error::from)
        )?);
        hasher.input(&track!(serde_json::to_vec(&self.spec).map_err(Error::from))?);

        let mut id = String::with_capacity(64);
        for b in hasher.result().as_slice() {
            track_write!(&mut id, "{:02x}", b)?;
        }
        Ok(id)
    }
}
