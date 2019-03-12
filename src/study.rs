use crate::time::DateTime;
use crate::trial::TrialRecord;
use serde_json::{Map as JsonMap, Value as JsonValue};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyRecord {
    pub optimizer: NameAndOptions,
    pub problem: NameAndOptions,
    pub budget: usize,
    pub start_time: DateTime,
    pub end_time: DateTime,
    pub trials: Vec<TrialRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameAndOptions {
    pub name: String,
    pub options: JsonMap<String, JsonValue>,
}
impl NameAndOptions {
    fn options_json(&self) -> String {
        match serde_json::to_string(&self.options) {
            Err(e) => panic!(
                "Can not serialize to JSON: options={:?}, error={}",
                self.options, e
            ),
            Ok(v) => v,
        }
    }
}
impl PartialEq for NameAndOptions {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.options_json() == other.options_json()
    }
}
impl Eq for NameAndOptions {}
impl PartialOrd for NameAndOptions {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for NameAndOptions {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name
            .cmp(&other.name)
            .then_with(|| self.options_json().cmp(&other.options_json()))
    }
}
impl Hash for NameAndOptions {
    fn hash<H: Hasher>(&self, h: &mut H) {
        self.name.hash(h);
        self.options_json().hash(h);
    }
}
