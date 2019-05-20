use kurobako_core::epi::problem::{
    EmbeddedScriptEvaluator, EmbeddedScriptProblem, EmbeddedScriptProblemRecipe,
};
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec, Values};
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct LightgbmProblemRecipe {
    pub training_data_path: PathBuf,
    pub validation_data_path: PathBuf,

    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub num_boost_round: Option<usize>,

    #[structopt(long, default_value = "auc")]
    #[serde(default, skip_serializing_if = "Metric::is_default")]
    pub metric: Metric,
}
impl ProblemRecipe for LightgbmProblemRecipe {
    type Problem = LightgbmProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let script = include_str!("../contrib/lightgbm_problem.py");
        let mut args = vec![
            track_assert_some!(self.training_data_path.to_str(), ErrorKind::InvalidInput)
                .to_owned(),
            track_assert_some!(self.validation_data_path.to_str(), ErrorKind::InvalidInput)
                .to_owned(),
        ];
        if let Some(round) = self.num_boost_round {
            args.extend(vec!["--num-boost-round".to_owned(), round.to_string()]);
        }

        let recipe = EmbeddedScriptProblemRecipe {
            script: script.to_owned(),
            args,
            interpreter: None, // TODO: env!("KUROBAKO_PYTHON")
            interpreter_args: Vec::new(),
            skip_lines: None,
        };
        let inner = track!(recipe.create_problem())?;
        Ok(LightgbmProblem(inner))
    }
}

#[derive(Debug)]
pub struct LightgbmProblem(EmbeddedScriptProblem);
impl Problem for LightgbmProblem {
    type Evaluator = LightgbmEvaluator;

    fn specification(&self) -> ProblemSpec {
        self.0.specification()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        track!(self.0.create_evaluator(id)).map(LightgbmEvaluator)
    }
}

#[derive(Debug)]
pub struct LightgbmEvaluator(EmbeddedScriptEvaluator);
impl Evaluate for LightgbmEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        track!(self.0.evaluate(params, budget))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Metric {
    Auc,
}
impl Metric {
    pub fn is_default(&self) -> bool {
        *self == Metric::Auc
    }
}
impl Default for Metric {
    fn default() -> Self {
        Metric::Auc
    }
}
impl FromStr for Metric {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s {
            "auc" => Metric::Auc,
            _ => track_panic!(ErrorKind::InvalidInput, "Unknown metric: {:?}", s),
        })
    }
}
