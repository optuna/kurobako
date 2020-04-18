use crate::epi::solver::{
    ExternalProgramSolver, ExternalProgramSolverFactory, ExternalProgramSolverRecipe,
};
use crate::problem::ProblemSpec;
use crate::registry::FactoryRegistry;
use crate::rng::ArcRng;
use crate::solver::{Solver, SolverFactory, SolverRecipe, SolverSpec};
use crate::trial::{EvaluatedTrial, IdGen, NextTrial};
use crate::{Error, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write as _;
use std::sync::Mutex;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

lazy_static! {
    static ref TEMP_FILES: Mutex<HashMap<String, TempPath>> = Mutex::new(HashMap::new());
}

/// Recipe for the solver that is implemented by an embedded script.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct EmbeddedScriptSolverRecipe {
    /// Embedded script code.
    pub script: String,

    /// Command line arguments that are passed to the script.
    pub args: Vec<String>,
}
impl SolverRecipe for EmbeddedScriptSolverRecipe {
    type Factory = EmbeddedScriptSolverFactory;

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
        let eppr = ExternalProgramSolverRecipe { path, args };
        let inner = track!(eppr.create_factory(registry))?;
        Ok(EmbeddedScriptSolverFactory { inner })
    }
}

/// Factory for the solver that is implemented by an embedded script.
#[derive(Debug)]
pub struct EmbeddedScriptSolverFactory {
    inner: ExternalProgramSolverFactory,
}
impl SolverFactory for EmbeddedScriptSolverFactory {
    type Solver = EmbeddedScriptSolver;

    fn specification(&self) -> Result<SolverSpec> {
        track!(self.inner.specification())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let inner = track!(self.inner.create_solver(rng, problem))?;
        Ok(EmbeddedScriptSolver { inner })
    }
}

/// Solver that is implemented by an embedded script.
#[derive(Debug)]
pub struct EmbeddedScriptSolver {
    inner: ExternalProgramSolver,
}
impl Solver for EmbeddedScriptSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        track!(self.inner.ask(idg))
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        track!(self.inner.tell(trial))
    }
}
