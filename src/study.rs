use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudyRecipe {
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub solver: KurobakoSolverRecipe,

    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub problem: KurobakoProblemRecipe,

    #[structopt(long, default_value = "20")]
    pub budget: u64,

    #[structopt(long, default_value = "1")]
    pub concurrency: usize,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudiesRecipe {
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub solvers: Vec<KurobakoSolverRecipe>,

    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub problems: Vec<KurobakoProblemRecipe>,

    #[structopt(long, default_value = "10")]
    pub repeats: usize,

    #[structopt(long, default_value = "20")]
    pub budget: u64,

    #[structopt(long, default_value = "1")]
    pub concurrency: usize,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}
impl StudiesRecipe {
    pub fn studies(&self) -> impl Iterator<Item = StudyRecipe> {
        let mut studies = Vec::new();
        for i in 0..self.repeats {
            for problem in &self.problems {
                for solver in &self.solvers {
                    let seed = self.seed.map(|s| s + i as u64);
                    let study = StudyRecipe {
                        solver: solver.clone(),
                        problem: problem.clone(),
                        budget: self.budget,
                        concurrency: self.concurrency,
                        seed,
                    };
                    studies.push(study);
                }
            }
        }
        studies.into_iter()
    }
}
