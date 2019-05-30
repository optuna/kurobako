use crate::epi::solver::{ExternalProgramSolver, ExternalProgramSolverRecipe};
use crate::json;
use crate::parameter::{ParamDomain, ParamValue};
use crate::problem::ProblemSpec;
use crate::recipe::Recipe;
use crate::solver::{ObservedObs, Solver, SolverRecipe, SolverSpec, UnobservedObs};
use crate::{Error, ErrorKind, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::Write as _;
use std::path::PathBuf;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};
use yamakan::observation::IdGen;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct EmbeddedScriptSolverRecipe {
    pub script: String,

    pub args: Vec<String>,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub params_domain: Vec<ParamDomain>,

    #[structopt(long)]
    pub interpreter: Option<PathBuf>,

    pub interpreter_args: Vec<String>,
}
impl Recipe for EmbeddedScriptSolverRecipe {
    fn get_free_params(&self) -> Result<Vec<ParamDomain>> {
        Ok(self.params_domain.clone())
    }

    fn bind_params(&mut self, params: Vec<(String, ParamValue)>) -> Result<()> {
        for (name, value) in params {
            if let Some(value) = value.try_to_string() {
                self.args.push(format!("--{}", name));
                self.args.push(value);
            }
        }
        Ok(())
    }
}
impl SolverRecipe for EmbeddedScriptSolverRecipe {
    type Solver = EmbeddedScriptSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
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

        let eppr = ExternalProgramSolverRecipe {
            path,
            args,
            params_domain: Vec::new(),
        };
        let inner = track!(eppr.create_solver(problem))?;
        Ok(EmbeddedScriptSolver { inner, temp })
    }
}

#[derive(Debug)]
pub struct EmbeddedScriptSolver {
    inner: ExternalProgramSolver,
    temp: TempPath,
}
impl Solver for EmbeddedScriptSolver {
    fn specification(&self) -> SolverSpec {
        self.inner.specification()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        track!(self.inner.ask(rng, idg))
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        track!(self.inner.tell(obs))
    }
}
