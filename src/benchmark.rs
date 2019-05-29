use crate::exam::ExamRecipe;
use crate::problem::KurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BenchmarkRecipe {
    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub solvers: Vec<KurobakoSolverRecipe>,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub problems: Vec<KurobakoProblemRecipe>,

    #[structopt(long, default_value = "10")]
    pub iterations: usize,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
}
impl BenchmarkRecipe {
    pub fn exams<'a>(&'a self) -> impl 'a + Iterator<Item = ExamRecipe> {
        self.problems.iter().flat_map(move |problem| {
            self.solvers.iter().flat_map(move |solver| {
                (0..self.iterations).map(move |_| ExamRecipe {
                    problem: problem.clone(),
                    solver: solver.clone(),
                    runner: self.runner.clone(),
                })
            })
        })
    }
}
