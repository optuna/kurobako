use super::{ExternalCommandOptimizer, ExternalCommandOptimizerBuilder, OptimizerBuilder};
use crate::{Error, ProblemSpace};
use rand::Rng;
use std::fs;
use std::io::Write as _;
use structopt::StructOpt;
use tempfile::{NamedTempFile, TempPath};
use yamakan;
use yamakan::budget::Budgeted;
use yamakan::observation::{IdGen, Obs, ObsId};
use yamakan::optimizers::Optimizer;

#[derive(Debug)]
pub struct OptunaOptimizer {
    inner: ExternalCommandOptimizer,
    temp: TempPath,
}
impl Optimizer for OptunaOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGen>(
        &mut self,
        rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Obs<Self::Param, ()>> {
        track!(self.inner.ask(rng, idgen))
    }

    fn tell(&mut self, obs: Obs<Self::Param, Self::Value>) -> yamakan::Result<()> {
        track!(self.inner.tell(obs))
    }

    fn forget(&mut self, _id: ObsId) -> yamakan::Result<()> {
        unimplemented!()
    }
}

#[derive(Debug, Default, StructOpt, Serialize, Deserialize)]
pub struct OptunaOptimizerBuilder {
    #[structopt(long)]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}
impl OptimizerBuilder for OptunaOptimizerBuilder {
    type Optimizer = OptunaOptimizer;

    fn build(
        &self,
        problem_space: &ProblemSpace,
        eval_cost: u64,
    ) -> Result<Self::Optimizer, Error> {
        let python_code = include_str!("../../contrib/optimizers/optuna_optimizer.py");
        let mut temp = NamedTempFile::new()?;
        write!(temp.as_file_mut(), "{}", python_code)?;
        let temp = temp.into_temp_path();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(&temp, fs::Permissions::from_mode(0o755))?;
        }

        let builder = ExternalCommandOptimizerBuilder {
            name: temp.to_path_buf(),
            args: vec![],
        };

        builder
            .build(problem_space, eval_cost)
            .map(|inner| OptunaOptimizer { inner, temp })
    }
}
