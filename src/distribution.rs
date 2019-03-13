#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Distribution {
    Uniform { low: f64, high: f64 },
}
impl Distribution {
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
