use kurobako_core::epi::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemRecipe,
};
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Evaluated, Problem, ProblemRecipe, ProblemSpec};
use kurobako_core::Result;
use kurobako_problems::{nasbench, sigopt};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoProblemRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
}
impl ProblemRecipe for KurobakoProblemRecipe {
    type Problem = KurobakoProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            KurobakoProblemRecipe::Command(p) => {
                track!(p.create_problem().map(KurobakoProblem::Command))
            }
            KurobakoProblemRecipe::Sigopt(p) => {
                track!(p.create_problem().map(KurobakoProblem::Sigopt))
            }
            KurobakoProblemRecipe::Nasbench(p) => {
                track!(p.create_problem().map(KurobakoProblem::Nasbench))
            }
        }
    }
}

#[derive(Debug)]
pub enum KurobakoProblem {
    Command(ExternalProgramProblem),
    Sigopt(sigopt::SigoptProblem),
    Nasbench(nasbench::NasbenchProblem),
}
impl Problem for KurobakoProblem {
    type Evaluator = KurobakoEvaluator;

    fn specification(&self) -> ProblemSpec {
        match self {
            KurobakoProblem::Command(p) => p.specification(),
            KurobakoProblem::Sigopt(p) => p.specification(),
            KurobakoProblem::Nasbench(p) => p.specification(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        match self {
            KurobakoProblem::Command(p) => {
                track!(p.create_evaluator(id).map(KurobakoEvaluator::Command))
            }
            KurobakoProblem::Sigopt(p) => {
                track!(p.create_evaluator(id).map(KurobakoEvaluator::Sigopt))
            }
            KurobakoProblem::Nasbench(p) => {
                track!(p.create_evaluator(id).map(KurobakoEvaluator::Nasbench))
            }
        }
    }
}

#[derive(Debug)]
pub enum KurobakoEvaluator {
    Command(ExternalProgramEvaluator),
    Sigopt(sigopt::SigoptEvaluator),
    Nasbench(nasbench::NasbenchEvaluator),
}
impl Evaluate for KurobakoEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        match self {
            KurobakoEvaluator::Command(e) => track!(e.evaluate(params, budget)),
            KurobakoEvaluator::Sigopt(e) => track!(e.evaluate(params, budget)),
            KurobakoEvaluator::Nasbench(e) => track!(e.evaluate(params, budget)),
        }
    }
}
