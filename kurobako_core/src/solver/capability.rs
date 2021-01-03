use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Solver capabilities.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Capabilities(BTreeSet<Capability>);
impl Capabilities {
    /// Makes a `Capabilities` instance that has the given capabilities.
    pub fn new<I>(capabilities: I) -> Self
    where
        I: Iterator<Item = Capability>,
    {
        Self(capabilities.collect())
    }

    /// Makes a `Capabilities` instance that has the all capabilities.
    pub fn all() -> Self {
        let all = [
            Capability::UniformContinuous,
            Capability::UniformDiscrete,
            Capability::LogUniformContinuous,
            Capability::LogUniformDiscrete,
            Capability::Categorical,
            Capability::Conditional,
            Capability::MultiObjective,
            Capability::Concurrent,
        ]
        .iter()
        .copied()
        .collect();
        Self(all)
    }

    /// Makes a `Capabilities` instance that has no capabilities.
    pub fn empty() -> Self {
        Self(BTreeSet::new())
    }

    /// Returns `true` if this instance has no capabilities.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if this instance has the given capability.
    pub fn is_capable(&self, c: Capability) -> bool {
        self.0.contains(&c)
    }

    /// Iterates over the capabilities required by `required` but not owned by this instance.
    pub fn incapables<'a>(&'a self, required: &'a Self) -> impl 'a + Iterator<Item = Capability> {
        required.0.difference(&self.0).copied()
    }

    /// Iterates over all the capabilities that this instance has.
    pub fn iter(&self) -> impl '_ + Iterator<Item = Capability> {
        self.0.iter().copied()
    }

    /// Adds the given capability to this instance.
    pub fn add_capability(&mut self, c: Capability) -> &mut Self {
        self.0.insert(c);
        self
    }

    /// Removes the given capability from this instance.
    pub fn remove_capability(&mut self, c: Capability) -> &mut Self {
        self.0.remove(&c);
        self
    }
}

/// Solver capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[allow(missing_docs)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Capability {
    UniformContinuous,
    UniformDiscrete,
    LogUniformContinuous,
    LogUniformDiscrete,
    Categorical,

    /// Conditional search space.
    ///
    /// If a problem has one or more constrainted parameters, the search space of the problem is conditional.
    Conditional,

    MultiObjective,
    Concurrent,
}
