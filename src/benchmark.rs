use crate::problem::KurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BenchmarkSpec {
    #[structopt(long, parse(try_from_str = "crate::json::parse_json"))]
    pub solvers: Vec<KurobakoSolverRecipe>,

    #[structopt(long, parse(try_from_str = "crate::json::parse_json"))]
    pub problems: Vec<KurobakoProblemRecipe>,

    #[structopt(long, default_value = "10")]
    pub iterations: usize,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
}
impl BenchmarkSpec {
    pub fn len(&self) -> usize {
        self.solvers.len() * self.problems.len() * self.iterations
    }

    pub fn studies<'a>(&'a self) -> Box<(dyn Iterator<Item = StudySpec> + 'a)> {
        Box::new(self.problems.iter().flat_map(move |p| {
            self.solvers.iter().flat_map(move |s| {
                (0..self.iterations).map(move |_| StudySpec {
                    problem: p,
                    solver: s,
                    runner: &self.runner,
                })
            })
        }))
    }
}

#[derive(Debug)]
pub struct StudySpec<'a> {
    pub solver: &'a KurobakoSolverRecipe,
    pub problem: &'a KurobakoProblemRecipe,
    pub runner: &'a StudyRunnerOptions,
}
