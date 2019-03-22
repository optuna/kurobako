use super::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder, OptimizerBuilder};
use crate::ProblemSpace;
use failure::Error;
use rand::Rng;
use std::fs;
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::NamedTempFile;
use yamakan::Optimizer;

#[derive(Debug)]
pub struct OptunaOptimizer {
    inner: ExternalCommandOptimizer,
    tempfile: NamedTempFile,
}
impl Optimizer for OptunaOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, rng: &mut R) -> Self::Param {
        self.inner.ask(rng)
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        self.inner.tell(param, value)
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct OptunaOptimizerBuilder {}
impl OptimizerBuilder for OptunaOptimizerBuilder {
    type Optimizer = OptunaOptimizer;

    fn optimizer_name(&self) -> &str {
        "optuna"
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        let python_code = include_str!("../../contrib/optimizers/optuna_optimizer.py");
        let mut temp = NamedTempFile::new()?;
        write!(temp.as_file_mut(), "{}", python_code)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o755))?;
        }

        let builder = ExternalCommandOptimizerBuilder {
            name: temp.path().to_path_buf(),
            args: vec![],
        };

        builder.build(problem_space).map(|inner| OptunaOptimizer {
            inner,
            tempfile: temp,
        })
    }
}
