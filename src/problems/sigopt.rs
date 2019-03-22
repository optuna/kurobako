use super::command::{CommandEvaluator, CommandProblem, CommandProblemSpec};
use crate::{Evaluate, Problem, ProblemSpace, ProblemSpec};
use failure::Fallible;
use std::fs;
use std::io::Write as _;
use tempfile::NamedTempFile;
use yamakan::budget::Budget;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum SigoptProblemSpec {
    Ackley {
        dim: u32,

        #[serde(skip_serializing_if = "Option::is_none")]
        #[structopt(long)]
        res: Option<u32>,
    },
}
impl SigoptProblemSpec {
    pub fn name(&self) -> &'static str {
        match self {
            SigoptProblemSpec::Ackley { .. } => "Ackley",
        }
    }

    pub fn dim(&self) -> u32 {
        match *self {
            SigoptProblemSpec::Ackley { dim, .. } => dim,
        }
    }

    pub fn res(&self) -> Option<u32> {
        match *self {
            SigoptProblemSpec::Ackley { res, .. } => res,
        }
    }
}
impl ProblemSpec for SigoptProblemSpec {
    type Problem = SigoptProblem;

    fn make_problem(&self) -> Fallible<Self::Problem> {
        let python_code = include_str!("../../contrib/problems/sigopt_problem.py");

        let mut temp = NamedTempFile::new()?;
        write!(temp.as_file_mut(), "{}", python_code)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o755))?;
        }

        let mut args = vec![self.name().to_owned(), self.dim().to_string()];
        if let Some(res) = self.res() {
            args.extend_from_slice(&["--res".to_owned(), res.to_string()]);
        }

        let spec = CommandProblemSpec {
            path: temp.path().to_path_buf(),
            args,
        };

        Ok(SigoptProblem {
            inner: spec.make_problem()?,
            tempfile: temp,
        })
    }
}

#[derive(Debug)]
pub struct SigoptProblem {
    inner: CommandProblem,
    tempfile: NamedTempFile,
}
impl Problem for SigoptProblem {
    type Evaluator = SigoptEvaluator;

    fn problem_space(&self) -> ProblemSpace {
        self.inner.problem_space()
    }

    fn evaluation_cost_hint(&self) -> usize {
        self.inner.evaluation_cost_hint()
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Fallible<Self::Evaluator> {
        Ok(SigoptEvaluator {
            inner: self.inner.make_evaluator(params)?,
        })
    }
}

#[derive(Debug)]
pub struct SigoptEvaluator {
    inner: CommandEvaluator,
}
impl Evaluate for SigoptEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Fallible<f64> {
        self.inner.evaluate(budget)
    }
}
