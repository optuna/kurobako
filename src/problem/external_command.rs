use super::{Problem, ProblemSpace};
use crate::distribution::Distribution;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct ExternalCommandProblem {
    pub name: PathBuf,
    pub args: Vec<String>,
}
impl Problem for AdjimanProblem {
    fn name(&self) -> &str {
        self.name.file_stem().expect("TODO").to_str().expect("TODO")
    }

    fn problem_space(&self) -> ProblemSpace {
        panic!()
    }

    fn evaluate(&self, x: &[f64]) -> f64 {
        panic!()
    }
}
