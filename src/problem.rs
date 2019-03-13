use crate::distribution::Distribution;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;
use structopt::StructOpt;

pub trait Problem: StructOpt + Serialize + for<'a> Deserialize<'a> {
    const NAME: &'static str;

    fn search_space(&self) -> Vec<Distribution>;
    fn evaluate(&self, params: &[f64]) -> f64;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct AckleyProblem {
    #[structopt(long, default_value = "2")]
    pub dim: usize,
}
impl Problem for AckleyProblem {
    const NAME: &'static str = "ackley";

    fn search_space(&self) -> Vec<Distribution> {
        (0..self.dim)
            .map(|_| Distribution::Uniform {
                low: -10.0,
                high: 30.0,
            })
            .collect()
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
