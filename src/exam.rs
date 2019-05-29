use crate::problem::KurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use kurobako_core::json;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct ExamRecipe {
    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub solver: KurobakoSolverRecipe,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub problem: KurobakoProblemRecipe,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
}
