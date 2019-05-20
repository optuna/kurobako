use crate::problem::FullKurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;

fn parse_json<T>(json: &str) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let v = track!(serde_json::from_str(json).map_err(Error::from))?;
    Ok(v)
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BenchmarkSpec {
    #[structopt(long, parse(try_from_str = "parse_json"))]
    pub solvers: Vec<KurobakoSolverRecipe>,

    #[structopt(long, parse(try_from_str = "parse_json"))]
    pub problems: Vec<FullKurobakoProblemRecipe>,

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
    pub problem: &'a FullKurobakoProblemRecipe,
    pub runner: &'a StudyRunnerOptions,
}
