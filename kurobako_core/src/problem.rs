use crate::parameter::{ParamDomain, ParamValue};
use crate::time::Seconds;
use crate::Result;
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

pub trait Evaluate {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated>;
}
impl<T: Evaluate + ?Sized> Evaluate for Box<T> {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        (**self).evaluate(params, budget)
    }
}

pub trait Problem {
    type Evaluator: Evaluate;

    fn specification(&self) -> ProblemSpec;
    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator>;
}

pub trait ProblemRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Problem: Problem;

    fn create_problem(&self) -> Result<Self::Problem>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ProblemSpec {
    pub name: String,

    #[serde(default)]
    pub version: Option<String>,

    pub params_domain: Vec<ParamDomain>,
    pub values_domain: Vec<MinMax<FiniteF64>>,
    pub evaluation_expense: NonZeroU64,

    #[serde(default)]
    pub capabilities: EvaluatorCapabilities,
}

// TODO: stripped ProblemSpec for solvers

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EvaluatorCapability {
    Concurrent,
    DynamicParamChange,
}

pub type EvaluatorCapabilities = BTreeSet<EvaluatorCapability>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluated {
    pub values: Vec<FiniteF64>,
    pub elapsed: Seconds,
}
impl Evaluated {
    pub const fn new(values: Vec<FiniteF64>, elapsed: Seconds) -> Self {
        Self { values, elapsed }
    }
}

pub struct BoxProblem {
    spec: ProblemSpec,
    create: Box<FnMut(ObsId) -> Result<BoxEvaluator>>,
}
impl BoxProblem {
    pub fn new<T>(mut problem: T) -> Self
    where
        T: Problem + 'static,
        T::Evaluator: 'static,
    {
        Self {
            spec: problem.specification(),
            create: Box::new(move |id| {
                let evaluator = track!(problem.create_evaluator(id))?;
                Ok(BoxEvaluator::new(evaluator))
            }),
        }
    }
}
impl Problem for BoxProblem {
    type Evaluator = BoxEvaluator;

    fn specification(&self) -> ProblemSpec {
        self.spec.clone()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        track!((self.create)(id))
    }
}
impl fmt::Debug for BoxProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblem {{ .. }}")
    }
}

pub struct BoxEvaluator(Box<(dyn Evaluate + 'static)>);
impl BoxEvaluator {
    pub fn new<T>(evaluator: T) -> Self
    where
        T: Evaluate + 'static,
    {
        Self(Box::new(evaluator))
    }
}
impl Evaluate for BoxEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        self.0.evaluate(params, budget)
    }
}
impl fmt::Debug for BoxEvaluator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxEvaluator {{ .. }}")
    }
}
