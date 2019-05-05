use kurobako_core::epi::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemRecipe,
};
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Evaluated, Problem, ProblemRecipe, ProblemSpec};
use kurobako_core::Result;
use kurobako_problems::{nasbench, sigopt};
use serde::{Deserialize, Serialize};
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum BuiltinProblemRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
}
impl ProblemRecipe for BuiltinProblemRecipe {
    type Problem = BuiltinProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        match self {
            BuiltinProblemRecipe::Command(p) => {
                track!(p.create_problem().map(BuiltinProblem::Command))
            }
            BuiltinProblemRecipe::Sigopt(p) => {
                track!(p.create_problem().map(BuiltinProblem::Sigopt))
            }
            BuiltinProblemRecipe::Nasbench(p) => {
                track!(p.create_problem().map(BuiltinProblem::Nasbench))
            }
        }
    }
}

#[derive(Debug)]
pub enum BuiltinProblem {
    Command(ExternalProgramProblem),
    Sigopt(sigopt::SigoptProblem),
    Nasbench(nasbench::NasbenchProblem),
}
impl Problem for BuiltinProblem {
    type Evaluator = BuiltinEvaluator;

    fn specification(&self) -> ProblemSpec {
        match self {
            BuiltinProblem::Command(p) => p.specification(),
            BuiltinProblem::Sigopt(p) => p.specification(),
            BuiltinProblem::Nasbench(p) => p.specification(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        match self {
            BuiltinProblem::Command(p) => {
                track!(p.create_evaluator(id).map(BuiltinEvaluator::Command))
            }
            BuiltinProblem::Sigopt(p) => {
                track!(p.create_evaluator(id).map(BuiltinEvaluator::Sigopt))
            }
            BuiltinProblem::Nasbench(p) => {
                track!(p.create_evaluator(id).map(BuiltinEvaluator::Nasbench))
            }
        }
    }
}

#[derive(Debug)]
pub enum BuiltinEvaluator {
    Command(ExternalProgramEvaluator),
    Sigopt(sigopt::SigoptEvaluator),
    Nasbench(nasbench::NasbenchEvaluator),
}
impl Evaluate for BuiltinEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        match self {
            BuiltinEvaluator::Command(e) => track!(e.evaluate(params, budget)),
            BuiltinEvaluator::Sigopt(e) => track!(e.evaluate(params, budget)),
            BuiltinEvaluator::Nasbench(e) => track!(e.evaluate(params, budget)),
        }
    }
}
