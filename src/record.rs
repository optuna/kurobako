use serde::{Deserialize, Serialize};
use serde_json;
use std;
use std::hash::{Hash, Hasher};

pub use self::study::{RecipeAndSpec, StudyRecord};
pub use self::trial::{AskRecord, EvaluateRecord, TellRecord, TrialRecord};

mod study;
mod trial;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonValue(serde_json::Value);
impl JsonValue {
    pub fn new(value: serde_json::Value) -> Self {
        // TODO: non-nan check
        Self(value)
    }

    pub fn get(&self) -> &serde_json::Value {
        &self.0
    }
}
impl PartialEq for JsonValue {
    fn eq(&self, other: &Self) -> bool {
        JsonValueRef(&self.0).eq(&JsonValueRef(&other.0))
    }
}
impl Eq for JsonValue {}
impl Hash for JsonValue {
    fn hash<H: Hasher>(&self, h: &mut H) {
        JsonValueRef(&self.0).hash(h);
    }
}

struct JsonValueRef<'a>(&'a serde_json::Value);
impl<'a> PartialEq for JsonValueRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        use serde_json::Value::*;

        match (self.0, other.0) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Number(a), Number(b)) => {
                if let (Some(a), Some(b)) = (a.as_f64(), b.as_f64()) {
                    a == b
                } else if let (Some(a), Some(b)) = (a.as_u64(), b.as_u64()) {
                    a == b
                } else if let (Some(a), Some(b)) = (a.as_i64(), b.as_i64()) {
                    a == b
                } else {
                    false
                }
            }
            (String(a), String(b)) => a == b,
            (Array(a), Array(b)) => a.iter().map(JsonValueRef).eq(b.iter().map(JsonValueRef)),
            (Object(a), Object(b)) => a
                .iter()
                .map(|(k, v)| (k, JsonValueRef(v)))
                .eq(b.iter().map(|(k, v)| (k, JsonValueRef(v)))),
            _ => false,
        }
    }
}
impl<'a> Hash for JsonValueRef<'a> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        match self.0 {
            serde_json::Value::Null => {
                ().hash(h);
            }
            serde_json::Value::Bool(v) => {
                v.hash(h);
            }
            serde_json::Value::Number(v) => {
                if let Some(v) = v.as_f64() {
                    v.to_bits().hash(h);
                } else if let Some(v) = v.as_u64() {
                    v.hash(h);
                } else if let Some(v) = v.as_i64() {
                    v.hash(h);
                } else {
                    unreachable!();
                }
            }
            serde_json::Value::String(v) => {
                v.hash(h);
            }
            serde_json::Value::Array(v) => {
                for v in v {
                    JsonValueRef(v).hash(h);
                }
            }
            serde_json::Value::Object(v) => {
                for (k, v) in v {
                    (k, JsonValueRef(v)).hash(h);
                }
            }
        }
    }
}
