use crate::distribution::Distribution;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use std::ops::Range;
use structopt::StructOpt;
use yamakan::SearchSpace;

pub trait Problem: StructOpt + Serialize + for<'a> Deserialize<'a> {
    fn name(&self) -> &str;
    fn problem_space(&self) -> ProblemSpace;
    fn evaluate(&self, params: &[f64]) -> f64;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum ProblemSpec {
    Ackley(AckleyProblem),
}
impl Problem for ProblemSpec {
    fn name(&self) -> &str {
        match self {
            ProblemSpec::Ackley(x) => x.name(),
        }
    }

    fn problem_space(&self) -> ProblemSpace {
        match self {
            ProblemSpec::Ackley(x) => x.problem_space(),
        }
    }

    fn evaluate(&self, params: &[f64]) -> f64 {
        match self {
            ProblemSpec::Ackley(x) => x.evaluate(params),
        }
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct AckleyProblem {
    #[structopt(long, default_value = "2")]
    pub dim: usize,
}
impl Problem for AckleyProblem {
    fn name(&self) -> &str {
        "ackley"
    }

    fn problem_space(&self) -> ProblemSpace {
        ProblemSpace(
            (0..self.dim)
                .map(|_| Distribution::Uniform {
                    low: -10.0,
                    high: 30.0,
                })
                .collect(),
        )
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        let dim = self.dim as f64;
        let a = 20.0;
        let b = 0.2;
        let c = 2.0 * PI;
        let d = -a * (-b * (1.0 / dim * xs.iter().map(|&x| x.powi(2)).sum::<f64>()).sqrt()).exp();
        let e = (1.0 / dim * xs.iter().map(|&x| (x * c).cos()).sum::<f64>()).exp() + a + 1f64.exp();
        d - e
    }
}

#[derive(Debug)]
pub struct ProblemSpace(Vec<Distribution>);
impl ProblemSpace {
    pub fn distributions(&self) -> &[Distribution] {
        &self.0
    }
}
impl SearchSpace for ProblemSpace {
    type ExternalParam = Vec<f64>;
    type InternalParam = Vec<f64>;

    fn internal_range(&self) -> Range<Self::InternalParam> {
        Range {
            start: self.0.iter().map(|d| d.low()).collect(),
            end: self.0.iter().map(|d| d.high()).collect(),
        }
    }

    fn to_internal(&self, param: &Self::ExternalParam) -> Self::InternalParam {
        param.clone()
    }

    fn to_external(&self, param: &Self::InternalParam) -> Self::ExternalParam {
        param.clone()
    }
}
