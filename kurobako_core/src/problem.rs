//! The interface of the problem for black-box optimization.
use crate::domain::{Distribution, Domain, Range, VariableBuilder};
use crate::registry::FactoryRegistry;
use crate::rng::ArcRng;
use crate::solver::{Capabilities, Capability};
use crate::trial::{Params, Values};
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use structopt::StructOpt;

/// `ProblemSpec` builder.
#[derive(Debug)]
pub struct ProblemSpecBuilder {
    name: String,
    attrs: BTreeMap<String, String>,
    params: Vec<VariableBuilder>,
    values: Vec<VariableBuilder>,
    steps: Vec<u64>,
}
impl ProblemSpecBuilder {
    /// Makes a new `ProblemSpecBuilder` instance.
    pub fn new(problem_name: &str) -> Self {
        Self {
            name: problem_name.to_owned(),
            attrs: BTreeMap::new(),
            params: Vec::new(),
            values: Vec::new(),
            steps: vec![1],
        }
    }

    /// Sets an attribute of this problem.
    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_owned(), value.to_owned());
        self
    }

    /// Adds a variable to the parameter domain of this problem.
    pub fn param(mut self, var: VariableBuilder) -> Self {
        self.params.push(var);
        self
    }

    /// Sets variables of the parameter domain of this problem.
    pub fn params(mut self, vars: Vec<VariableBuilder>) -> Self {
        self.params = vars;
        self
    }

    /// Adds a variable to the value domain of this problem.
    pub fn value(mut self, var: VariableBuilder) -> Self {
        self.values.push(var);
        self
    }

    /// Sets the evaluable steps of this problem.
    pub fn steps<I>(mut self, steps: I) -> Self
    where
        I: IntoIterator<Item = u64>,
    {
        self.steps = steps.into_iter().collect();
        self
    }

    /// Builds a `ProblemSpec` with the given settings.
    pub fn finish(self) -> Result<ProblemSpec> {
        let params_domain = track!(Domain::new(self.params))?;
        let values_domain = track!(Domain::new(self.values))?;
        let steps = track!(EvaluableSteps::new(self.steps))?;

        Ok(ProblemSpec {
            name: self.name,
            attrs: self.attrs,
            params_domain,
            values_domain,
            steps,
        })
    }
}

/// Problem specification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProblemSpec {
    /// Problem name.
    pub name: String,

    /// Problem attributes.
    #[serde(default)]
    pub attrs: BTreeMap<String, String>,

    /// Domain of the parameters.
    pub params_domain: Domain,

    /// Domain of the objective values.
    pub values_domain: Domain,

    /// List of steps.
    ///
    /// This problem can evaluate a given parameter set at a step in this list.
    pub steps: EvaluableSteps,
}
impl ProblemSpec {
    /// Returns the capabilities required to solver to handle this problem.
    pub fn requirements(&self) -> Capabilities {
        let mut c = Capabilities::empty();

        if self.values_domain.variables().len() > 1 {
            c.add_capability(Capability::MultiObjective);
        }

        for v in self.params_domain.variables() {
            if !v.conditions().is_empty() {
                c.add_capability(Capability::Conditional);
            }

            match (v.range(), v.distribution()) {
                (Range::Continuous { .. }, Distribution::Uniform) => {
                    c.add_capability(Capability::UniformContinuous);
                }
                (Range::Continuous { .. }, Distribution::LogUniform) => {
                    c.add_capability(Capability::LogUniformContinuous);
                }
                (Range::Discrete { .. }, Distribution::Uniform) => {
                    c.add_capability(Capability::UniformDiscrete);
                }
                (Range::Discrete { .. }, Distribution::LogUniform) => {
                    c.add_capability(Capability::LogUniformDiscrete);
                }
                (Range::Categorical { .. }, _) => {
                    c.add_capability(Capability::Categorical);
                }
            }
        }

        c
    }
}

/// Recipe of a problem.
pub trait ProblemRecipe: Clone + Send + StructOpt + Serialize + for<'a> Deserialize<'a> {
    /// The type of the factory creating problem instances from this recipe.
    type Factory: ProblemFactory;

    /// Creates a problem factory.
    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory>;
}

/// This trait allows creating instances of a problem.
pub trait ProblemFactory: Send {
    /// The type of the problem instance created by this factory.
    type Problem: Problem;

    /// Returns the specification of the problem created by this factory.
    fn specification(&self) -> Result<ProblemSpec>;

    /// Creates a problem instance.
    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem>;
}

enum ProblemFactoryCall {
    Specification,
    CreateProblem(ArcRng),
}

enum ProblemFactoryReturn {
    Specification(ProblemSpec),
    CreateProblem(BoxProblem),
}

/// Boxed problem factory.
pub struct BoxProblemFactory(
    Box<dyn Fn(ProblemFactoryCall) -> Result<ProblemFactoryReturn> + Send>,
);
impl BoxProblemFactory {
    /// Makes a new `BoxProblemFactory` instance.
    pub fn new<T>(problem: T) -> Self
    where
        T: 'static + ProblemFactory,
    {
        Self(Box::new(move |call| match call {
            ProblemFactoryCall::Specification => problem
                .specification()
                .map(ProblemFactoryReturn::Specification),
            ProblemFactoryCall::CreateProblem(rng) => problem
                .create_problem(rng)
                .map(BoxProblem::new)
                .map(ProblemFactoryReturn::CreateProblem),
        }))
    }
}
impl ProblemFactory for BoxProblemFactory {
    type Problem = BoxProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let v = track!((self.0)(ProblemFactoryCall::Specification))?;
        if let ProblemFactoryReturn::Specification(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let v = track!((self.0)(ProblemFactoryCall::CreateProblem(rng)))?;
        if let ProblemFactoryReturn::CreateProblem(v) = v {
            Ok(v)
        } else {
            unreachable!()
        }
    }
}
impl fmt::Debug for BoxProblemFactory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblemFactory {{ .. }}")
    }
}

/// Problem.
pub trait Problem: Send {
    /// The type of the evaluator of this problem.
    type Evaluator: Evaluator;

    /// Creates an evaluator that evaluates the given parameters.
    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator>;
}

/// Boxed problem.
pub struct BoxProblem(Box<dyn Fn(Params) -> Result<BoxEvaluator> + Send>);
impl BoxProblem {
    /// Makes a new `BoxProblem` instance.
    pub fn new<T>(problem: T) -> Self
    where
        T: 'static + Problem,
    {
        Self(Box::new(move |params| {
            problem.create_evaluator(params).map(BoxEvaluator::new)
        }))
    }
}
impl Problem for BoxProblem {
    type Evaluator = BoxEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        track!((self.0)(params))
    }
}
impl fmt::Debug for BoxProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblem {{ .. }}")
    }
}

/// This trait allows evaluating a parameter set of a problem.
pub trait Evaluator: Send {
    /// Procedes the evaluation of the parameter set given to `Problem::create_evaluator` method until reaches to `next_step..
    ///
    /// The first element of the result tuple means the current step of the evaluation.
    /// Although it's desirable that the current step matches to `next_step`,
    /// it's allowed to exceed `next_step`.
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)>;
}
impl<T: Evaluator + ?Sized> Evaluator for Box<T> {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        (**self).evaluate(next_step)
    }
}

/// Boxed evaluator.
pub struct BoxEvaluator(Box<dyn Evaluator>);
impl BoxEvaluator {
    /// Makes a new `BoxEvaluator` instance.
    pub fn new<T>(evaluator: T) -> Self
    where
        T: 'static + Evaluator,
    {
        Self(Box::new(evaluator))
    }
}
impl Evaluator for BoxEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        self.0.evaluate(next_step)
    }
}
impl fmt::Debug for BoxEvaluator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxEvaluator {{ .. }}")
    }
}

/// Evaluable steps of a problem.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EvaluableSteps(EvaluableStepsInner);
impl EvaluableSteps {
    /// Makes a new `EvaluableSteps` instance.
    pub fn new(steps: Vec<u64>) -> Result<Self> {
        track!(EvaluableStepsInner::new(steps)).map(Self)
    }

    /// Returns the last evaluation step.
    pub fn last(&self) -> u64 {
        self.0.last()
    }

    /// Returns an iterator that iterates over all the steps.
    pub fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = u64> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
enum EvaluableStepsInner {
    Max(u64),
    Steps(Vec<u64>),
}
impl EvaluableStepsInner {
    fn new(steps: Vec<u64>) -> Result<Self> {
        track_assert!(!steps.is_empty(), ErrorKind::InvalidInput);
        track_assert!(steps[0] > 0, ErrorKind::InvalidInput);

        for (a, b) in steps.iter().zip(steps.iter().skip(1)) {
            track_assert!(a < b, ErrorKind::InvalidInput);
        }

        let last = steps[steps.len() - 1];
        if last == steps.len() as u64 {
            Ok(Self::Max(last))
        } else {
            Ok(Self::Steps(steps))
        }
    }

    fn last(&self) -> u64 {
        match self {
            Self::Max(n) => *n,
            Self::Steps(ns) => ns[ns.len() - 1],
        }
    }

    fn iter<'a>(&'a self) -> impl 'a + Iterator<Item = u64> {
        match self {
            Self::Max(n) => itertools::Either::Left(0..=*n),
            Self::Steps(ns) => itertools::Either::Right(ns.iter().copied()),
        }
    }
}
