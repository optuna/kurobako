//! Problem interface for black-box optimization.
use crate::domain::{Domain, VariableBuilder};
use crate::repository::Repository;
use crate::trial::{Params, TrialId, Values};
use crate::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::num::NonZeroU64;
use structopt::StructOpt;

/// `ProblemSpec` builder.
#[derive(Debug)]
pub struct ProblemSpecBuilder {
    name: String,
    attrs: BTreeMap<String, String>,
    params: Vec<VariableBuilder>,
    values: Vec<VariableBuilder>,
    evaluation_steps: u64,
}
impl ProblemSpecBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            attrs: BTreeMap::new(),
            params: Vec::new(),
            values: Vec::new(),
            evaluation_steps: 1,
        }
    }

    pub fn attr(mut self, key: &str, value: &str) -> Self {
        self.attrs.insert(key.to_owned(), value.to_owned());
        self
    }

    pub fn param(mut self, var: VariableBuilder) -> Self {
        self.params.push(var);
        self
    }

    pub fn value(mut self, var: VariableBuilder) -> Self {
        self.values.push(var);
        self
    }

    pub fn evaluation_steps(mut self, steps: u64) -> Self {
        self.evaluation_steps = steps;
        self
    }

    pub fn finish(self) -> Result<ProblemSpec> {
        let params_domain = track!(Domain::new(self.params))?;
        let values_domain = track!(Domain::new(self.values))?;
        let evaluation_steps = track_assert_some!(
            NonZeroU64::new(self.evaluation_steps),
            ErrorKind::InvalidInput
        );

        Ok(ProblemSpec {
            name: self.name,
            attrs: self.attrs,
            params_domain,
            values_domain,
            evaluation_steps,
        })
    }
}

/// Problem specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    /// Number of steps to complete evaluating a parameter set.
    pub evaluation_steps: NonZeroU64,
}
// TODO
// impl ProblemSpec {
//     pub fn required_solver_capabilities(&self) -> SolverCapabilities {
//         let mut c = SolverCapabilities::empty();
//         if self.values_domain.len() > 1 {
//             c = c.multi_objective();
//         }
//         for p in &self.params_domain {
//             c = c.union(p.required_solver_capabilities());
//         }
//         c
//     }
// }

pub trait ProblemRecipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    type Factory: ProblemFactory;

    fn create_problem_factory(&self, repository: &mut Repository) -> Result<Self::Factory>;
}

pub trait ProblemFactory {
    type Problem: Problem;

    fn specification(&self) -> Result<ProblemSpec>;
    fn create_problem(&self, id: TrialId, params: Params) -> Result<Self::Problem>;
}

enum ProblemFactoryCall {
    Specification,
    CreateProblem { id: TrialId, params: Params },
}

enum ProblemFactoryReturn {
    Specification(ProblemSpec),
    CreateProblem(BoxProblem),
}

pub struct BoxProblemFactory(Box<dyn Fn(ProblemFactoryCall) -> Result<ProblemFactoryReturn>>);
impl BoxProblemFactory {
    pub fn new<T>(problem: T) -> Self
    where
        T: ProblemFactory + 'static,
        T::Problem: 'static,
    {
        Self(Box::new(move |call| match call {
            ProblemFactoryCall::Specification => problem
                .specification()
                .map(ProblemFactoryReturn::Specification),
            ProblemFactoryCall::CreateProblem { id, params } => problem
                .create_problem(id, params)
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

    fn create_problem(&self, id: TrialId, params: Params) -> Result<Self::Problem> {
        let v = track!((self.0)(ProblemFactoryCall::CreateProblem { id, params }))?;
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

pub trait Problem {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)>;
}
impl<T: Problem + ?Sized> Problem for Box<T> {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        (**self).evaluate(next_step)
    }
}

pub struct BoxProblem(Box<(dyn Problem + 'static)>);
impl BoxProblem {
    pub fn new<T>(problem: T) -> Self
    where
        T: Problem + 'static,
    {
        Self(Box::new(problem))
    }
}
impl Problem for BoxProblem {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        self.0.evaluate(next_step)
    }
}
impl fmt::Debug for BoxProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BoxProblem {{ .. }}")
    }
}
