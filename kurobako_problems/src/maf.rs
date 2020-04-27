//! https://link.springer.com/article/10.1007/s40747-017-0039-7
use kurobako_core::domain;
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

/// Recipe of `MafProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct MafProblemRecipe {
    #[structopt(flatten)]
    pub maf: Maf,

    #[structopt(long)] // TODO: default, check objectives >= 1
    pub objectives: usize,
}

impl ProblemRecipe for MafProblemRecipe {
    type Factory = MafProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        Ok(MafProblemFactory {
            maf: self.maf,
            objectives: self.objectives,
        })
    }
}

/// Factory of `MafProblem`.
#[derive(Debug)]
pub struct MafProblemFactory {
    maf: Maf,
    objectives: usize,
}

impl ProblemFactory for MafProblemFactory {
    type Problem = MafProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let name = format!("{} [objectives={}]", self.maf.name(), self.objectives);
        let mut spec = ProblemSpecBuilder::new(&name)
            .attr(
                "version",
                &format!("kurobako_problems={}", env!("CARGO_PKG_VERSION")),
            )
            .attr(
                "paper",
                "Cheng, Ran, et al. \"A benchmark test suite for evolutionary many-objective optimization.\" \
                 Complex & Intelligent Systems 3.1 (2017): 67-81."
            );
        for i in 0..self.maf.decision_variables(self.objectives) {
            spec = spec.param(domain::var(&format!("x{}", i)).continuous(0.0, 1.0));
        }
        for i in 0..self.objectives {
            spec = spec.param(domain::var(&format!("objective{}", i)));
        }
        track!(spec.finish())
    }

    fn create_problem(&self, _rng: ArcRng) -> Result<Self::Problem> {
        Ok(MafProblem {
            maf: self.maf,
            objectives: self.objectives,
        })
    }
}

/// TODO
#[derive(Debug)]
pub struct MafProblem {
    maf: Maf,
    objectives: usize,
}

impl Problem for MafProblem {
    type Evaluator = MafEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        Ok(MafEvaluator {
            params,
            maf: self.maf,
            objectives: self.objectives,
        })
    }
}

/// Evaluator of `MafProblem`.
#[derive(Debug)]
pub struct MafEvaluator {
    params: Params,
    maf: Maf,
    objectives: usize,
}
impl Evaluator for MafEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let values = self.maf.evaluate(self.objectives, self.params.get());
        Ok((next_step, Values::new(values)))
    }
}

/// TODO
#[derive(Debug, Clone, Copy, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub enum Maf {
    /// Modified inverted DTLZ1.
    Maf1,

    /// DTLZ2BZ.
    Maf2,

    /// Convex DTLZ3.
    Maf3,

    Maf4,
    Maf5,
    Maf6,
    Maf7,
    Maf8,
    Maf9,
    Maf10,
    Maf11,
    Maf12,
    Maf13,
    Maf14,
    Maf15,
}

impl Maf {
    fn name(&self) -> &'static str {
        match self {
            Self::Maf1 => "MaF1 (modified inverted DTLZ1)",
            Self::Maf2 => "MaF2 (DTLZ2BZ)",
            _ => todo!(),
        }
    }

    fn decision_variables(&self, objectives: usize) -> usize {
        match self {
            Self::Maf1 => objectives + 9,
            Self::Maf2 => objectives + 9,
            Self::Maf3 => objectives + 9,
            _ => todo!(),
        }
    }

    fn evaluate(&self, objectives: usize, xs: &[f64]) -> Vec<f64> {
        match self {
            Self::Maf1 => self.evaluate_maf1(objectives, xs),
            Self::Maf2 => self.evaluate_maf2(objectives, xs),
            _ => todo!(),
        }
    }

    fn evaluate_maf1(&self, objectives: usize, xs: &[f64]) -> Vec<f64> {
        let g = xs
            .iter()
            .skip(objectives - 1)
            .map(|&x| (x - 0.5).powi(2))
            .sum::<f64>();
        (0..objectives)
            .rev()
            .map(|i| (i == (objectives - 1), i))
            .map(|(first, i)| {
                let v = xs.iter().take(i).product::<f64>() * if first { 1.0 } else { 1.0 - xs[i] };
                (1.0 - v) * (1.0 + g)
            })
            .collect()
    }

    fn evaluate_maf2(&self, objectives: usize, xs: &[f64]) -> Vec<f64> {
        let g = |i: usize| -> f64 {
            let k = 10.0;
            let m = objectives as f64;
            let interval = (k / m).floor() as usize;
            let j_start = objectives + i * interval;
            let j_end = if i != objectives - 1 {
                objectives + (i + 1) * interval
            } else {
                xs.len()
            };

            (&xs[j_start..j_end])
                .iter()
                .map(|&x| (x / 2.0 + 1.0 / 4.0).powi(2))
                .sum()
        };

        fn theta(x: f64) -> f64 {
            (PI / 2.0) * (x / 2.0 + 1.0 / 4.0)
        }

        (0..objectives)
            .map(|i| {
                let n = objectives - 1 - i;
                let v = xs.iter().take(n).map(|&x| theta(x).cos()).product::<f64>()
                    * if i == 0 { 1.0 } else { theta(xs[n]).sin() };
                v * (1.0 + g(i))
            })
            .collect()
    }

    fn evaluate_maf3(&self, objectives: usize, xs: &[f64]) -> Vec<f64> {
        todo!()
    }
}
