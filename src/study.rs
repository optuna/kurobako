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
