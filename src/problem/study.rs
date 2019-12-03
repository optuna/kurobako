use crate::study::StudyRecipe;
use crate::variable::Var;
use kurobako_core::json::{self, JsonRecipe};
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudyProblemRecipe {
    pub study: JsonRecipe,

    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub vars: Vec<Var>,
}
impl ProblemRecipe for StudyProblemRecipe {
    type Factory = StudyProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let study_json = self.study.clone();
        let study = track!(serde_json::from_value(study_json).map_err(Error::from))?;
        Ok(StudyProblemFactory {
            study,
            vars: self.vars.clone(),
        })
    }
}

#[derive(Debug)]
pub struct StudyProblemFactory {
    study: StudyRecipe,
    vars: Vec<Var>,
}
impl ProblemFactory for StudyProblemFactory {
    type Problem = StudyProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        panic!()
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        panic!()
    }
}

#[derive(Debug)]
pub struct StudyProblem {}
impl Problem for StudyProblem {
    type Evaluator = StudyEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        panic!()
    }
}

#[derive(Debug)]
pub struct StudyEvaluator {}
impl Evaluator for StudyEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        panic!()
    }
}
