//! The problems defined in [A Generic Test Suite for Evolutionary Multifidelity Optimization].
//!
//! [A Generic Test Suite for Evolutionary Multifidelity Optimization]: https://ieeexplore.ieee.org/document/8054707
use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{self, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{ErrorKind, Result};
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::num::{NonZeroU64, NonZeroUsize};
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct MfbProblemRecipe {
    pub problem_number: usize,

    #[structopt(long, default_value = "8")]
    pub dimensions: usize,

    #[structopt(long, default_value = "100")]
    pub fidelity_levels: u64,
}
impl ProblemRecipe for MfbProblemRecipe {
    type Problem = MfbProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        track_assert!(
            1 <= self.problem_number && self.problem_number <= 13,
            ErrorKind::InvalidInput; self.problem_number
        );
        track_assert!(self.fidelity_levels <= 10_000, ErrorKind::InvalidInput);

        let dimensions =
            track_assert_some!(NonZeroUsize::new(self.dimensions), ErrorKind::InvalidInput);
        let fidelity_levels = track_assert_some!(
            NonZeroU64::new(self.fidelity_levels),
            ErrorKind::InvalidInput
        );

        let mfb = Mfb::new(dimensions, self.problem_number);
        Ok(MfbProblem {
            fidelity_levels,
            mfb,
        })
    }
}

#[derive(Debug)]
pub struct MfbProblem {
    fidelity_levels: NonZeroU64,
    mfb: Mfb,
}
impl Problem for MfbProblem {
    type Evaluator = MfbEvaluator;

    fn specification(&self) -> ProblemSpec {
        ProblemSpec {
            name: format!("MFB{}", self.mfb.no),
            version: None,
            params_domain: (0..self.mfb.f.dimensions.get())
                .map(|i| {
                    parameter::uniform(&format!("x{}", i), -1.0, 1.0)
                        .unwrap_or_else(|e| unreachable!("{}", e))
                })
                .collect(),
            values_domain: vec![unsafe {
                MinMax::new_unchecked(FiniteF64::new_unchecked(0.0), FiniteF64::new_unchecked(3.0))
            }],
            evaluation_expense: self.fidelity_levels,
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(MfbEvaluator {
            mfb: self.mfb.clone(),
            fidelity_levels: self.fidelity_levels,
        })
    }
}

#[derive(Debug)]
pub struct MfbEvaluator {
    mfb: Mfb,
    fidelity_levels: NonZeroU64,
}
impl Evaluate for MfbEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        let max_cost = budget.amount * 10_000 / self.fidelity_levels.get();

        // TODO: optimize
        let phi = self
            .mfb
            .phis()
            .take_while(|&phi| self.mfb.c.cost(phi) <= max_cost)
            .last();
        let phi = track_assert_some!(phi, ErrorKind::InvalidInput) as f64;

        let xs = params
            .iter()
            .map(|p| {
                Ok(track_assert_some!(
                    p.as_continuous().map(|p| p.get()),
                    ErrorKind::InvalidInput
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let v = self.mfb.f.evaluate(&xs);
        let e = self.mfb.e.error(&xs, phi);

        budget.consumption = budget.amount;
        Ok(vec![track!(FiniteF64::new(v + e))?])
    }
}

#[derive(Debug, Clone)]
struct Mfb {
    no: usize,
    f: ModifiedRastrigin,
    e: FidelityError,
    c: Cost,
}
impl Mfb {
    fn new(dimensions: NonZeroUsize, no: usize) -> Self {
        let f = ModifiedRastrigin { dimensions };
        let (e, c) = match no {
            1 => (FidelityError::Resolution1, Cost::Linear),
            2 => (FidelityError::Resolution2, Cost::Linear),
            3 => (FidelityError::Resolution3, Cost::NonLinear),
            4 => (FidelityError::Resolution1, Cost::NonLinear),
            5 => (FidelityError::Resolution2, Cost::NonLinear),
            6 => (FidelityError::Resolution1, Cost::Linear),
            7 => (FidelityError::Resolution4, Cost::Linear),
            8 => (FidelityError::Stochastic1, Cost::Linear),
            9 => (FidelityError::Stochastic2, Cost::NonLinear),
            10 => (FidelityError::Stochastic3, Cost::Linear),
            11 => (FidelityError::Stochastic4, Cost::NonLinear),
            12 => (FidelityError::Instability1, Cost::Linear),
            13 => (FidelityError::Instability2, Cost::NonLinear),
            _ => unreachable!(),
        };
        Self { no, f, e, c }
    }

    fn phis(&self) -> Box<dyn Iterator<Item = u64>> {
        match self.no {
            4 => Box::new((0..=10).map(|i| i * 1_000)),
            5 => Box::new(vec![1_000, 3_000, 10_000].into_iter()),
            6 => Box::new(vec![1_000, 10_000].into_iter()),
            _ => Box::new(0..=10_000),
        }
    }
}

#[derive(Debug, Clone)]
struct ModifiedRastrigin {
    dimensions: NonZeroUsize,
}
impl ModifiedRastrigin {
    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter()
            .map(|&x| x.powi(2) + 1.0 - (10.0 * PI * x).cos())
            .sum()
    }
}

#[derive(Debug, Clone)]
enum Cost {
    Linear,
    NonLinear,
}
impl Cost {
    fn cost(&self, phi: u64) -> u64 {
        match self {
            Cost::Linear => phi,
            Cost::NonLinear => (0.001 * phi as f64).powi(4).round() as u64,
        }
    }
}

#[derive(Debug, Clone)]
enum FidelityError {
    Resolution1,
    Resolution2,
    Resolution3,
    Resolution4,
    Stochastic1,
    Stochastic2,
    Stochastic3,
    Stochastic4,
    Instability1,
    Instability2,
}
impl FidelityError {
    fn error(&self, xs: &[f64], phi: f64) -> f64 {
        panic!()
    }
}
