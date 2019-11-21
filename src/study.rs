use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::str::FromStr;
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
    pub concurrency: NonZeroUsize,

    #[structopt(long, default_value = "random")]
    pub scheduling: Scheduling,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub enum Scheduling {
    Random,
    Fair,
}
impl Default for Scheduling {
    fn default() -> Self {
        Self::Random
    }
}
impl FromStr for Scheduling {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "random" => Ok(Self::Random),
            "fair" => Ok(Self::Fair),
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown scheduling type: {:?}", s),
        }
    }
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
    pub concurrency: NonZeroUsize,

    #[structopt(long, default_value = "random")]
    pub scheduling: Scheduling,

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
                        scheduling: self.scheduling,
                        seed,
                    };
                    studies.push(study);
                }
            }
        }
        studies.into_iter()
    }
}
