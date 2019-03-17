use super::{Problem, ProblemSpace};
use crate::distribution::Distribution;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct Alpine01Problem {
    #[structopt(long, default_value = "2")]
    pub dim: usize,
}
impl Problem for Alpine01Problem {
    fn name(&self) -> &str {
        "alpine01"
    }

    fn problem_space(&self) -> ProblemSpace {
        ProblemSpace(vec![Distribution::uniform(-6.0, 10.0); self.dim])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x * x.sin() + 0.1 * x).sum()
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct Alpine02Problem {
    #[structopt(long, default_value = "2")]
    pub dim: usize,
}
impl Problem for Alpine02Problem {
    fn name(&self) -> &str {
        "alpine02"
    }

    fn problem_space(&self) -> ProblemSpace {
        ProblemSpace(vec![Distribution::uniform(0.0, 10.0); self.dim])
    }

    fn evaluate(&self, xs: &[f64]) -> f64 {
        xs.iter().map(|&x| x.sqrt() * x.sin()).product()
    }
}
