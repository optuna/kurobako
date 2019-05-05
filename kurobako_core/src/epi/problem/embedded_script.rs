use crate::epi::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemRecipe,
};
use crate::parameter::ParamValue;
use crate::problem::{Evaluate, Evaluated, Problem, ProblemRecipe, ProblemSpec};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct EmbeddedScriptProblemRecipe {
    pub script: String,

    pub args: Vec<String>,

    #[structopt(long)]
    pub interpreter: Option<PathBuf>,

    pub interpreter_args: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub skip_lines: Option<usize>,
}
impl ProblemRecipe for EmbeddedScriptProblemRecipe {
    type Problem = EmbeddedScriptProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp.as_file_mut(), "{}", self.script).map_err(Error::from))?;
        let temp = temp.into_temp_path();

        let mut args = Vec::new();
        let path = if let Some(interpreter_path) = self.interpreter.clone() {
            args.extend(self.interpreter_args.clone());
            args.push(track_assert_some!(temp.to_str(), ErrorKind::InvalidInput).to_owned());
            interpreter_path
        } else {
            #[cfg(unix)]
            {
                use std::fs;
                use std::os::unix::fs::PermissionsExt as _;

                track!(
                    fs::set_permissions(&temp, fs::Permissions::from_mode(0o755))
                        .map_err(Error::from)
                )?;
            }
            temp.to_path_buf()
        };
        args.extend(self.args.clone());

        let eppr = ExternalProgramProblemRecipe {
            path,
            args,
            skip_lines: self.skip_lines,
        };
        let inner = track!(eppr.create_problem())?;
        Ok(EmbeddedScriptProblem { inner, temp })
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptProblem {
    inner: ExternalProgramProblem,
    temp: TempPath,
}
impl Problem for EmbeddedScriptProblem {
    type Evaluator = EmbeddedScriptEvaluator;

    fn specification(&self) -> ProblemSpec {
        self.inner.specification()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        track!(self.inner.create_evaluator(id)).map(EmbeddedScriptEvaluator)
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptEvaluator(ExternalProgramEvaluator);
impl Evaluate for EmbeddedScriptEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        track!(self.0.evaluate(params, budget))
    }
}
