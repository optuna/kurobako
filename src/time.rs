use chrono;

pub type DateTime = chrono::DateTime<chrono::Local>;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Timestamp(f64);
