use crate::epi::solver::{
    ExternalProgramSolver, ExternalProgramSolverFactory, ExternalProgramSolverRecipe,
};
use crate::problem::ProblemSpec;
use crate::repository::Repository;
use crate::rng::ArcRng;
use crate::solver::{Solver, SolverFactory, SolverRecipe, SolverSpec};
use crate::trial::{EvaluatedTrial, IdGen, UnevaluatedTrial};
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};

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

    fn create_factory(&self, repository: &mut Repository) -> Result<Self::Factory> {
        let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp.as_file_mut(), "{}", self.script).map_err(Error::from))?;
        let temp = temp.into_temp_path();

        let mut args = Vec::new();
        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt as _;

            track!(
                fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)).map_err(Error::from)
            )?;
        }
        let path = temp.to_path_buf();
        args.extend(self.args.clone());

        let eppr = ExternalProgramSolverRecipe { path, args };
        let inner = track!(eppr.create_factory(repository))?;
        Ok(EmbeddedScriptSolverFactory { inner, temp })
    }
}

/// Factory for the solver that is implemented by an embedded script.
#[derive(Debug)]
pub struct EmbeddedScriptSolverFactory {
    inner: ExternalProgramSolverFactory,
    temp: TempPath,
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
    fn ask(&mut self, idg: &mut IdGen) -> Result<UnevaluatedTrial> {
        track!(self.inner.ask(idg))
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        track!(self.inner.tell(trial))
    }
}
