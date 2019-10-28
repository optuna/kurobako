//! A trial represents one ask-evaluate-tell cycle.
use crate::Result;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use std;
use std::cmp::Ordering;
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

/// Trial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trial {
    /// Trial identifier.
    pub id: TrialId,

    /// Evaluation budget.
    pub budget: Budget,

    /// Evaluation parameters.
    pub params: Params,

    /// Evaluated values.
    pub values: Values,
}
impl Trial {
    /// Makes a new unevaluated trial.
    pub fn new<G: IdGen>(mut idg: G, budget: Budget, params: Params) -> Result<Self> {
        let id = track!(idg.generate())?;
        Ok(Self {
            id,
            budget,
            params,
            values: Values::default(),
        })
    }
}

// TODO: move to `problem` (?)
/// Evaluation budget.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Budget {
    /// The amount of this budget.
    pub amount: u64,

    /// The consumption of this budget.
    ///
    /// Note that this value can exceed the budget amount.
    pub consumption: u64,
}
impl Budget {
    /// Makes a new `Budget` instance which has the given amount of budget.
    pub const fn new(amount: u64) -> Self {
        Self {
            consumption: 0,
            amount,
        }
    }

    /// Returns the remaining amount of this budget.
    ///
    /// # Errors
    ///
    /// If the consumption of the budget exceeded the budget amount, `Err(excess amount)` will be returned.
    pub fn remaining(&self) -> std::result::Result<u64, u64> {
        if self.consumption <= self.amount {
            Ok(self.amount - self.consumption)
        } else {
            Err(self.consumption - self.amount)
        }
    }

    /// Returns `true` if the consumption has exceeded the budget amount, otherwise `false`.
    pub fn is_consumed(&self) -> bool {
        self.consumption >= self.amount
    }
}

/// Trial ID generator.
pub trait IdGen {
    /// Generates a new identifier.
    fn generate(&mut self) -> Result<TrialId>;
}
impl<'a, T: IdGen + ?Sized> IdGen for &'a mut T {
    fn generate(&mut self) -> Result<TrialId> {
        (**self).generate()
    }
}
impl<T: IdGen + ?Sized> IdGen for Box<T> {
    fn generate(&mut self) -> Result<TrialId> {
        (**self).generate()
    }
}

/// Parameter values.
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
impl PartialOrd for Params {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ordered_floats().cmp(other.ordered_floats()))
    }
}
impl Ord for Params {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ordered_floats().cmp(other.ordered_floats())
    }
}
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

/// Evaluated values.
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
impl PartialOrd for Values {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.ordered_floats().cmp(other.ordered_floats()))
    }
}
impl Ord for Values {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ordered_floats().cmp(other.ordered_floats())
    }
}
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
