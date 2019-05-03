use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Distribution {
    Uniform { low: f64, high: f64 },
}
impl Distribution {
    pub fn uniform(low: f64, high: f64) -> Self {
        Distribution::Uniform { low, high }
    }

    pub fn low(&self) -> f64 {
        match self {
            Distribution::Uniform { low, .. } => *low,
        }
    }

    pub fn high(&self) -> f64 {
        match self {
            Distribution::Uniform { high, .. } => *high,
        }
    }
}
