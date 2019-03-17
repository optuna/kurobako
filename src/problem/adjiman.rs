use super::{Problem, ProblemSpace};
use crate::distribution::Distribution;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct AdjimanProblem {}
impl Problem for AdjimanProblem {
    fn name(&self) -> &str {
        "adjiman"
    }

    fn problem_space(&self) -> ProblemSpace {
        ProblemSpace(vec![
            Distribution::uniform(-1.0, 2.0),
            Distribution::uniform(-1.0, 1.0),
        ])
    }

    fn evaluate(&self, x: &[f64]) -> f64 {
        x[0].cos() * x[1].sin() - x[0] / (x[1].powi(2) + 1.0)
    }
}
