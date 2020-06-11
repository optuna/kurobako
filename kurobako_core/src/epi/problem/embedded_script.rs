use crate::epi::problem::{
    ExternalProgramEvaluator, ExternalProgramProblem, ExternalProgramProblemFactory,
    ExternalProgramProblemRecipe,
};
use crate::problem::{Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec};
use crate::registry::FactoryRegistry;
use crate::rng::ArcRng;
use crate::trial::{Params, Values};
use crate::{Error, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write as _;
use std::sync::Mutex;
use std::time::Duration;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

lazy_static! {
    static ref TEMP_FILES: Mutex<HashMap<String, TempPath>> = Mutex::new(HashMap::new());
}

/// Recipe for the problem implemented by an embedded script.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct EmbeddedScriptProblemRecipe {
    /// Embedded script code.
    pub script: String,

    /// Command line arguments that are passed to the script.
    pub args: Vec<String>,
}
impl ProblemRecipe for EmbeddedScriptProblemRecipe {
    type Factory = EmbeddedScriptProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let path = {
            let mut temp_files = track!(TEMP_FILES.lock().map_err(Error::from))?;
            if let Some(path) = temp_files.get(&self.script).map(|p| p.to_path_buf()) {
                path
            } else {
                let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
                track!(write!(temp.as_file_mut(), "{}", self.script).map_err(Error::from))?;
                let temp = temp.into_temp_path();

                #[cfg(unix)]
                {
                    use std::fs;
                    use std::os::unix::fs::PermissionsExt as _;

                    track!(
                        fs::set_permissions(&temp, fs::Permissions::from_mode(0o755))
                            .map_err(Error::from)
                    )?;
                }

                let path = temp.to_path_buf();
                temp_files.insert(self.script.clone(), temp);
                path
            }
        };

        let args = self.args.clone();
        let eppr = ExternalProgramProblemRecipe { path, args };
        let inner = track!(eppr.create_factory(registry))?;

        Ok(EmbeddedScriptProblemFactory { inner })
    }
}

/// Factory for the problem implemented by an embedded script.
#[derive(Debug)]
pub struct EmbeddedScriptProblemFactory {
    inner: ExternalProgramProblemFactory,
}
impl ProblemFactory for EmbeddedScriptProblemFactory {
    type Problem = EmbeddedScriptProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        track!(self.inner.specification())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let inner = track!(self.inner.create_problem(rng))?;
        Ok(EmbeddedScriptProblem { inner })
    }
}

/// Problem that is implemented by an embedded script.
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

/// Evaluator that is implemented by an embedded script.
#[derive(Debug)]
pub struct EmbeddedScriptEvaluator {
    inner: ExternalProgramEvaluator,
}
impl Evaluator for EmbeddedScriptEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        track!(self.inner.evaluate(next_step))
    }

    // TODO: delete
    fn elapsed(&self) -> Option<Duration> {
        self.inner.elapsed()
    }
}
