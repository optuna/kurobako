use kurobako_core::num::FiniteF64;
use serde::{Deserialize, Serialize};
use serde_json;
use std;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

pub use self::benchmark::{BenchmarkRecord, ProblemRecord};
pub use self::study::{Id, RecipeAndSpec, StudyRecord};
pub use self::trial::{AskRecord, EvaluateRecord, TellRecord, TrialRecord};

mod benchmark;
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
impl PartialOrd for JsonValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        JsonValueRef(&self.0).partial_cmp(&JsonValueRef(&other.0))
    }
}
impl Ord for JsonValue {
    fn cmp(&self, other: &Self) -> Ordering {
        JsonValueRef(&self.0).cmp(&JsonValueRef(&other.0))
    }
}
impl Hash for JsonValue {
    fn hash<H: Hasher>(&self, h: &mut H) {
        JsonValueRef(&self.0).hash(h);
    }
}

struct JsonValueRef<'a>(&'a serde_json::Value);
impl<'a> PartialOrd for JsonValueRef<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<'a> Ord for JsonValueRef<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        use serde_json::Value::*;

        let a = Type::new(self.0);
        let b = Type::new(other.0);
        if a != b {
            return a.cmp(&b);
        }

        match (self.0, other.0) {
            (Null, Null) => Ordering::Equal,
            (Bool(a), Bool(b)) => a.cmp(&b),
            (Number(a), Number(b)) => Num::new(a).cmp(&Num::new(b)),
            (String(a), String(b)) => a.cmp(&b),
            (Array(a), Array(b)) => a.iter().map(JsonValueRef).cmp(b.iter().map(JsonValueRef)),
            (Object(a), Object(b)) => a
                .iter()
                .map(|(k, v)| (k, JsonValueRef(v)))
                .cmp(b.iter().map(|(k, v)| (k, JsonValueRef(v)))),
            (_, _) => unreachable!(),
        }
    }
}
impl<'a> PartialEq for JsonValueRef<'a> {
    fn eq(&self, other: &Self) -> bool {
        use serde_json::Value::*;

        match (self.0, other.0) {
            (Null, Null) => true,
            (Bool(a), Bool(b)) => a == b,
            (Number(a), Number(b)) => Num::new(a) == Num::new(b),
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
impl<'a> Eq for JsonValueRef<'a> {}
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

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Type {
    Null,
    Bool,
    Number,
    String,
    Array,
    Object,
}
impl Type {
    fn new(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Type::Null,
            serde_json::Value::Bool(_) => Type::Bool,
            serde_json::Value::Number(_) => Type::Number,
            serde_json::Value::String(_) => Type::String,
            serde_json::Value::Array(_) => Type::Array,
            serde_json::Value::Object(_) => Type::Object,
        }
    }
}

#[derive(PartialOrd, Ord, PartialEq, Eq)]
enum Num {
    F64(FiniteF64),
    U64(u64),
    I64(i64),
}
impl Num {
    fn new(n: &serde_json::Number) -> Self {
        if let Some(n) = n.as_f64() {
            Num::F64(FiniteF64::new(n).unwrap_or_else(|e| panic!("{}", e)))
        } else if let Some(n) = n.as_u64() {
            Num::U64(n)
        } else if let Some(n) = n.as_i64() {
            Num::I64(n)
        } else {
            unreachable!()
        }
    }
}
