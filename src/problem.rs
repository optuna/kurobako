use crate::problem_optuna;
use kurobako_core::epi::problem::ExternalProgramProblemRecipe;
use kurobako_core::problem::{BoxProblem, ProblemRecipe};
use kurobako_core::Result;
use kurobako_problems::{nasbench, sigopt};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum FullKurobakoProblemRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
    Optuna(problem_optuna::OptunaProblemRecipe),
}
impl ProblemRecipe for FullKurobakoProblemRecipe {
    type Problem = BoxProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            FullKurobakoProblemRecipe::Command(p) => {
                track!(p.create_problem().map(BoxProblem::new))
            }
            FullKurobakoProblemRecipe::Sigopt(p) => track!(p.create_problem().map(BoxProblem::new)),
            FullKurobakoProblemRecipe::Nasbench(p) => {
                track!(p.create_problem().map(BoxProblem::new))
            }
            FullKurobakoProblemRecipe::Optuna(p) => track!(p.create_problem().map(BoxProblem::new)),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoProblemRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
}
impl ProblemRecipe for KurobakoProblemRecipe {
    type Problem = BoxProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            KurobakoProblemRecipe::Command(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Sigopt(p) => track!(p.create_problem().map(BoxProblem::new)),
            KurobakoProblemRecipe::Nasbench(p) => track!(p.create_problem().map(BoxProblem::new)),
        }
    }
}
