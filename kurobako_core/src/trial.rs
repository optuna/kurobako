//! A trial that represents one ask-evaluate-tell cycle.
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Trial Identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrialId(u64);
impl TrialId {
    /// Makes a new trial identifier.
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the value of this identifier.
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A trial that has a parameter set to be evaluated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NextTrial<P = Params> {
    /// The identifier of this trial.
    pub id: TrialId,

    /// The parameters to be evaluated.
    pub params: P,

    /// The next evaluation step.
    ///
    /// The evaluator needs to evaluate the parameters until this step reaches.
    /// If this is `None`, it means that this trial doesn't need to be evaluated anymore.
    pub next_step: Option<u64>,
}
impl NextTrial<Params> {
    /// Makes an `EvaluatedTrial` instance with the given values and step.
    pub fn evaluated(&self, values: Values, current_step: u64) -> EvaluatedTrial {
        EvaluatedTrial {
            id: self.id,
            values,
            current_step,
        }
    }

    /// Makes an `EvaluatedTrial` instance that indicates this trial couldn't be evaluated.
    pub fn unevaluable(&self) -> EvaluatedTrial {
        self.evaluated(Values::new(Vec::new()), self.next_step.unwrap_or(0))
    }
}

/// A trial that has an evaluated values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluatedTrial {
    /// The identifier of this trial.
    pub id: TrialId,

    /// The evaluated objective values.
    ///
    /// If the parameters couldn't be evaluated for any reasons, this becomes empty.
    pub values: Values,

    /// The current evaluation step.
    pub current_step: u64,
}

/// Trial ID generator.
#[derive(Debug)]
pub struct IdGen {
    next: u64,
}
impl IdGen {
    /// Makes a new `IdGen` instance.
    pub const fn new() -> Self {
        Self { next: 0 }
    }

    /// Generates a new identifier.
    pub fn generate(&mut self) -> TrialId {
        let id = TrialId(self.next);
        self.next += 1;
        id
    }

    /// Peeks the next identifier.
    pub fn peek_id(&self) -> TrialId {
        TrialId(self.next)
    }
}

/// Parameter values.
///
/// Note that if a parameter is conditional and the condition didn't hold,
/// the value of the parameter is set to NaN.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Params(Vec<f64>);
impl Params {
    /// Makes a new `Params` instance.
    pub const fn new(params: Vec<f64>) -> Self {
        Self(params)
    }

    /// Converts into `Vec<f64>`.
    pub fn into_vec(self) -> Vec<f64> {
        self.0
    }

    fn ordered_floats<'a>(&'a self) -> impl 'a + Iterator<Item = OrderedFloat<f64>> {
        self.0.iter().copied().map(OrderedFloat)
    }
}
impl PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        self.ordered_floats().eq(other.ordered_floats())
    }
}
impl Eq for Params {}
impl Hash for Params {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for x in self.ordered_floats() {
            x.hash(hasher);
        }
    }
}
impl Deref for Params {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Evaluated values (a.k.a. objective values).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Values(Vec<f64>);
impl Values {
    /// Makes a new `Values` instance.
    pub const fn new(values: Vec<f64>) -> Self {
        Self(values)
    }

    /// Converts into `Vec<f64>`.
    pub fn into_vec(self) -> Vec<f64> {
        self.0
    }

    fn ordered_floats<'a>(&'a self) -> impl 'a + Iterator<Item = OrderedFloat<f64>> {
        self.0.iter().copied().map(OrderedFloat)
    }
}
impl PartialEq for Values {
    fn eq(&self, other: &Self) -> bool {
        self.ordered_floats().eq(other.ordered_floats())
    }
}
impl Eq for Values {}
impl Hash for Values {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        for x in self.ordered_floats() {
            x.hash(hasher);
        }
    }
}
impl Deref for Values {
    type Target = [f64];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
