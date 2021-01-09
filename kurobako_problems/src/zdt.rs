//! A two-objective problem that takes its name from its authors Zitzler, Deb and Thiele.
//!
//! # References
//!
//! - [Comparison of Multiob jective Evolutionary Algorithms: Empirical Results](https://www.mitpressjournals.org/doi/abs/10.1162/106365600568202)
//! - [A Benchmark Study of Multi-Objective Optimization Methods](http://www.redcedartech.com/pdfs/MO-SHERPA_paper.pdf)
use kurobako_core::domain::{self, Range};
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use std::f64;
use std::f64::consts::PI;
use structopt::StructOpt;

/// Recipe of `ZdtProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct ZdtProblemRecipe {
    #[structopt(flatten)]
    pub zdt: Zdt,
}

impl ProblemRecipe for ZdtProblemRecipe {
    type Factory = ZdtProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        Ok(ZdtProblemFactory { zdt: self.zdt })
    }
}

/// Factory of `ZdtProblem`.
#[derive(Debug)]
pub struct ZdtProblemFactory {
    zdt: Zdt,
}

impl ProblemFactory for ZdtProblemFactory {
    type Problem = ZdtProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let name = self.zdt.name();
        let mut spec = ProblemSpecBuilder::new(name)
            .attr(
                "version",
                &format!("kurobako_problems={}", env!("CARGO_PKG_VERSION")),
            )
            .attr(
                "paper",
                "Zitzler, Eckart, Kalyanmoy Deb, and Lothar Thiele. \"Comparison of multiobjective \
                 evolutionary algorithms: Empirical results.\" Evolutionary computation 8.2 (2000): 173-195."
            ).value(domain::var("f1")).value(domain::var("f2")).reference_point(Some(Params::new(vec![11.0, 11.0])));

        for (i, range) in self.zdt.ranges().into_iter().enumerate() {
            spec = spec.param(domain::var(&format!("x{}", i)).range(range));
        }
        track!(spec.finish())
    }

    fn create_problem(&self, _rng: ArcRng) -> Result<Self::Problem> {
        Ok(ZdtProblem { zdt: self.zdt })
    }
}

/// ZDT problem.
#[derive(Debug)]
pub struct ZdtProblem {
    zdt: Zdt,
}

impl Problem for ZdtProblem {
    type Evaluator = ZdtEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        Ok(ZdtEvaluator {
            params,
            zdt: self.zdt,
        })
    }
}

/// Evaluator of `ZdtProblem`.
#[derive(Debug)]
pub struct ZdtEvaluator {
    params: Params,
    zdt: Zdt,
}
impl Evaluator for ZdtEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let values = self.zdt.evaluate(self.params.get());
        Ok((next_step, Values::new(values)))
    }
}

/// ZDT functions.
#[derive(Debug, Clone, Copy, StructOpt, Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Zdt {
    /// This test function has a convex Pareto-optimal front.
    ///
    /// The solutions are uniformly distributed in the search space.
    #[structopt(name = "1")]
    #[serde(rename = "1")]
    Function1,

    /// This test function is the non-convex counterpart to the function 1.
    #[structopt(name = "2")]
    #[serde(rename = "2")]
    Function2,

    /// This test function represents the discreteness features.
    ///
    /// Its Pareto-optimal front consists of several non-contiguous convex parts,
    /// the search space is unbiased.
    #[structopt(name = "3")]
    #[serde(rename = "3")]
    Function3,

    /// This test function contains `21**9` local Pareto-optimal and
    /// therefore tests for the solver's ability to deal with multimodality.
    #[structopt(name = "4")]
    #[serde(rename = "4")]
    Function4,

    /// This test function describes a deceptive problem and distinguishes itself from the
    /// other test functions in that each parameter represents a binary string.
    #[structopt(name = "5")]
    #[serde(rename = "5")]
    Function5,

    /// This test function on T6 includes two diculties caused by the non-uniformity of the search space.
    #[structopt(name = "6")]
    #[serde(rename = "6")]
    Function6,
}

impl Zdt {
    fn name(self) -> &'static str {
        match self {
            Self::Function1 => "ZDT1",
            Self::Function2 => "ZDT2",
            Self::Function3 => "ZDT3",
            Self::Function4 => "ZDT4",
            Self::Function5 => "ZDT5",
            Self::Function6 => "ZDT6",
        }
    }

    fn ranges(self) -> Vec<Range> {
        match self {
            Self::Function1 | Self::Function2 | Self::Function3 => (0..30)
                .map(|_| Range::Continuous {
                    low: 0.0,
                    high: 1.0,
                })
                .collect(),
            Self::Function4 => std::iter::once((0.0, 1.0))
                .chain(std::iter::repeat((-5.0, 5.0)).take(9))
                .map(|(low, high)| Range::Continuous { low, high })
                .collect(),
            Self::Function5 => std::iter::once((0, ((1 << 30) - 1)))
                .chain(std::iter::repeat((0, ((1 << 5) - 1))).take(10))
                .map(|(low, high)| Range::Discrete { low, high })
                .collect(),
            Self::Function6 => (0..10)
                .map(|_| Range::Continuous {
                    low: 0.0,
                    high: 1.0,
                })
                .collect(),
        }
    }

    fn evaluate(self, xs: &[f64]) -> Vec<f64> {
        match self {
            Self::Function1 => self.evaluate_zdt1(xs),
            Self::Function2 => self.evaluate_zdt2(xs),
            Self::Function3 => self.evaluate_zdt3(xs),
            Self::Function4 => self.evaluate_zdt4(xs),
            Self::Function5 => self.evaluate_zdt5(xs),
            Self::Function6 => self.evaluate_zdt6(xs),
        }
    }

    fn evaluate_zdt1(self, xs: &[f64]) -> Vec<f64> {
        let n = xs.len() as f64;
        let f1 = xs[0];
        let g = 1.0 + 9.0 * xs.iter().skip(1).sum::<f64>() / (n - 1.0);
        let h = 1.0 - (f1 / g).sqrt();
        let f2 = g * h;
        vec![f1, f2]
    }

    fn evaluate_zdt2(self, xs: &[f64]) -> Vec<f64> {
        let n = xs.len() as f64;
        let f1 = xs[0];
        let g = 1.0 + 9.0 * xs.iter().skip(1).sum::<f64>() / (n - 1.0);
        let h = 1.0 - (f1 / g).powi(2);
        let f2 = g * h;
        vec![f1, f2]
    }

    fn evaluate_zdt3(self, xs: &[f64]) -> Vec<f64> {
        let n = xs.len() as f64;
        let f1 = xs[0];
        let g = 1.0 + 9.0 * xs.iter().skip(1).sum::<f64>() / (n - 1.0);
        let h = 1.0 - (f1 / g).sqrt() - (f1 / g) * (10.0 * PI * f1).sin();
        let f2 = g * h;
        vec![f1, f2]
    }

    fn evaluate_zdt4(self, xs: &[f64]) -> Vec<f64> {
        let n = xs.len() as f64;
        let f1 = xs[0];
        let g = 10.0 * (n - 1.0)
            + xs.iter()
                .skip(1)
                .map(|&x| x * x - 10.0 * (4.0 * PI * x).cos())
                .sum::<f64>();
        let h = 1.0 - (f1 / g).sqrt();
        let f2 = g * h;
        vec![f1, f2]
    }

    fn evaluate_zdt5(self, xs: &[f64]) -> Vec<f64> {
        let f1 = 1.0 + (xs[0] as i64).count_ones() as f64;
        let g = xs
            .iter()
            .skip(1)
            .map(|&x| (x as i64).count_ones())
            .map(|ones| if ones < 5 { 2 + ones } else { 1 } as f64)
            .sum::<f64>();
        let h = 1.0 / f1;
        let f2 = g * h;
        vec![f1, f2]
    }

    fn evaluate_zdt6(self, xs: &[f64]) -> Vec<f64> {
        let n = xs.len() as f64;
        let f1 = 1.0 - (-4.0 * xs[0]).exp() * (6.0 * PI * xs[0]).sin().powi(6);
        let g = 1.0 + 9.0 * (xs.iter().skip(1).sum::<f64>() / (n - 1.0)).powf(0.25);
        let h = 1.0 - (f1 / g).powi(2);
        let f2 = g * h;
        vec![f1, f2]
    }
}
