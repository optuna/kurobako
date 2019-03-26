use super::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder, OptimizerBuilder};
use crate::{Error, ProblemSpace, Result};
use rand::Rng;
use std::fs;
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};
use yamakan::Optimizer;

#[derive(Debug)]
pub struct GpyoptOptimizer {
    inner: ExternalCommandOptimizer,
    temp: TempPath,
}
impl Optimizer for GpyoptOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        self.inner.ask(rng)
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        self.inner.tell(param, value)
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct GpyoptOptimizerBuilder {}
impl OptimizerBuilder for GpyoptOptimizerBuilder {
    type Optimizer = GpyoptOptimizer;

    fn optimizer_name(&self) -> &str {
        "gpyopt"
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer> {
        let python_code = include_str!("../../contrib/optimizers/gpyopt_optimizer.py");
        let mut temp = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(temp.as_file_mut(), "{}", python_code).map_err(Error::from))?;

        let temp = temp.into_temp_path();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            track!(
                fs::set_permissions(&temp, fs::Permissions::from_mode(0o755)).map_err(Error::from)
            )?;
        }

        let builder = ExternalCommandOptimizerBuilder {
            name: temp.to_path_buf(),
            args: vec![],
        };

        track!(builder.build(problem_space)).map(|inner| GpyoptOptimizer { inner, temp })
    }
}
