use crate::epi::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemFactory,
    ExternalProgramProblemRecipe,
};
use crate::problem::{Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec};
use crate::repository::Repository;
use crate::trial::{Params, Values};
use crate::{Error, Result};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct EmbeddedScriptProblemRecipe {
    pub script: String,
    pub args: Vec<String>,
}
impl ProblemRecipe for EmbeddedScriptProblemRecipe {
    type Factory = EmbeddedScriptProblemFactory;

    fn create_factory(&self, repository: &mut Repository) -> Result<Self::Factory> {
        let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp.as_file_mut(), "{}", self.script).map_err(Error::from))?;
        let temp = temp.into_temp_path();

        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt as _;

            track!(
                fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)).map_err(Error::from)
            )?;
        }

        let path = temp.to_path_buf();
        let args = self.args.clone();
        let eppr = ExternalProgramProblemRecipe { path, args };
        let inner = track!(eppr.create_factory(repository))?;

        Ok(EmbeddedScriptProblemFactory { inner, temp })
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptProblemFactory {
    inner: ExternalProgramProblemFactory,
    temp: TempPath,
}
impl ProblemFactory for EmbeddedScriptProblemFactory {
    type Problem = EmbeddedScriptProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        track!(self.inner.specification())
    }

    fn create_problem(&self, rng: StdRng) -> Result<Self::Problem> {
        let inner = track!(self.inner.create_problem(rng))?;
        Ok(EmbeddedScriptProblem { inner })
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptProblem {
    inner: ExternalProgramProblem,
}
impl Problem for EmbeddedScriptProblem {
    type Evaluator = EmbeddedScriptEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let inner = track!(self.inner.create_evaluator(params))?;
        Ok(EmbeddedScriptEvaluator { inner })
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptEvaluator {
    inner: ExternalProgramEvaluator,
}
impl Evaluator for EmbeddedScriptEvaluator {
    fn evaluate(&mut self, max_step: u64) -> Result<(u64, Values)> {
        track!(self.inner.evaluate(max_step))
    }
}
