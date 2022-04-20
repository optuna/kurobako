//! Study.
use crate::problem::KurobakoProblemRecipe;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroUsize;
use std::str::FromStr;
use structopt::StructOpt;

/// Recipe of a study.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
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

/// Logical threads scheduling policy for executing a study.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(missing_docs)]
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
impl fmt::Display for Scheduling {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Random => write!(f, "random"),
            Self::Fair => write!(f, "fair"),
        }
    }
}

/// Recipe of multiple studies.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudiesRecipe {
    /// Solver recipe JSONs.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub solvers: Vec<KurobakoSolverRecipe>,

    /// Problem recipe JSONs.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub problems: Vec<KurobakoProblemRecipe>,

    /// Number of execution times of each study.
    #[structopt(long, default_value = "10")]
    pub repeats: usize,

    /// Budget of a study execution.
    #[structopt(long, default_value = "20")]
    pub budget: u64,

    /// Concurrency of a study execution.
    #[structopt(long, default_value = "1")]
    pub concurrency: NonZeroUsize,

    /// Scheduling policy of logical threads.
    ///
    /// This option is ignored when `concurrency` is less then `2`.
    #[structopt(long, default_value = "random")]
    pub scheduling: Scheduling,

    /// Random seed.
    #[structopt(long)]
    pub seed: Option<u64>,
}
impl StudiesRecipe {
    /// Returns a iterator that iterates over the study recipes specified by this recipe.
    pub fn studies(&self) -> impl Iterator<Item = StudyRecipe> {
        let mut studies = Vec::new();
        for problem in &self.problems {
            for i in 0..self.repeats {
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
