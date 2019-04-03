use crate::{Error, Result};
use kurobako_core::problem::{Evaluate, Problem, ProblemSpace, ProblemSpec};
use kurobako_core::problems::command::{CommandEvaluator, CommandProblem, CommandProblemSpec};
use kurobako_core::ValueRange;
use std::fs;
use std::io::Write as _;
use tempfile::{NamedTempFile, TempPath};
use yamakan::budget::Budget;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct NasbenchProblemSpec {
    pub dataset_path: String, // TODO: PathBuf
}
impl ProblemSpec for NasbenchProblemSpec {
    type Problem = NasbenchProblem;

    fn make_problem(&self) -> Result<Self::Problem> {
        let python_code = include_str!("../../contrib/nasbench_problem.py");

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

        let args = vec!["--dataset-path".to_owned(), self.dataset_path.clone()];

        let spec = CommandProblemSpec {
            path: temp.to_path_buf(),
            args,
            skip_lines: Some(2),
        };

        let inner = track!(spec.make_problem())?;
        Ok(NasbenchProblem { inner, temp })
    }
}

#[derive(Debug)]
pub struct NasbenchProblem {
    inner: CommandProblem,
    temp: TempPath,
}
impl Problem for NasbenchProblem {
    type Evaluator = NasbenchEvaluator;

    fn problem_space(&self) -> ProblemSpace {
        self.inner.problem_space()
    }

    fn evaluation_cost(&self) -> u64 {
        self.inner.evaluation_cost()
    }

    fn value_range(&self) -> ValueRange {
        self.inner.value_range()
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Result<Option<Self::Evaluator>> {
        let inner = track!(self.inner.make_evaluator(params))?;
        Ok(inner.map(|inner| NasbenchEvaluator { inner }))
    }
}

#[derive(Debug)]
pub struct NasbenchEvaluator {
    inner: CommandEvaluator,
}
impl Evaluate for NasbenchEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Result<f64> {
        track!(self.inner.evaluate(budget))
    }
}
