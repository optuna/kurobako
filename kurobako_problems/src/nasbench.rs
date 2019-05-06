use kurobako_core::epi::problem::{
    EmbeddedScriptEvaluator, EmbeddedScriptProblem, EmbeddedScriptProblemRecipe,
};
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Evaluated, Problem, ProblemRecipe, ProblemSpec};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct NasbenchProblemRecipe {
    pub dataset_path: PathBuf,
}
impl ProblemRecipe for NasbenchProblemRecipe {
    type Problem = NasbenchProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let script = include_str!("../contrib/nasbench_problem.py");
        let args = vec![
            "--dataset-path".to_owned(),
            track_assert_some!(self.dataset_path.to_str(), ErrorKind::InvalidInput).to_owned(),
        ];
        let recipe = EmbeddedScriptProblemRecipe {
            script: script.to_owned(),
            args,
            interpreter: None, // TODO: env!("KUROBAKO_PYTHON")
            interpreter_args: Vec::new(),
            skip_lines: Some(2),
        };
        let inner = track!(recipe.create_problem())?;
        Ok(NasbenchProblem(inner))
    }
}

#[derive(Debug)]
pub struct NasbenchProblem(EmbeddedScriptProblem);
impl Problem for NasbenchProblem {
    type Evaluator = NasbenchEvaluator;

    fn specification(&self) -> ProblemSpec {
        self.0.specification()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        track!(self.0.create_evaluator(id)).map(NasbenchEvaluator)
    }
}

#[derive(Debug)]
pub struct NasbenchEvaluator(EmbeddedScriptEvaluator);
impl Evaluate for NasbenchEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        track!(self.0.evaluate(params, budget))
    }
}
