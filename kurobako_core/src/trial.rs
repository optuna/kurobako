//! A trial represents one ask-evaluate-tell cycle.
use crate::Result;
use serde::{Deserialize, Serialize};
use std;

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
    pub params: Vec<f64>,

    /// Evaluated values.
    pub values: Vec<f64>,
}
impl Trial {
    /// Makes a new unevaluated trial.
    pub fn new<G: IdGen>(mut idg: G, budget: Budget, params: Vec<f64>) -> Result<Self> {
        let id = track!(idg.generate())?;
        Ok(Self {
            id,
            budget,
            params,
            values: Vec::new(),
        })
    }
}

// TODO: move
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
