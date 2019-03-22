use super::OptimizerBuilder;
use crate::ProblemSpace;
use failure::Error;
use rand::Rng;
use serde_json::{self, json};
use std::io::{BufRead as _, BufReader, Write as _};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use structopt::StructOpt;
use yamakan::Optimizer;

#[derive(Debug)]
pub struct ExternalCommandOptimizer {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}
impl Optimizer for ExternalCommandOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, _rng: &mut R) -> Self::Param {
        let mut line = String::new();
        self.stdout
            .read_line(&mut line)
            .unwrap_or_else(|e| panic!(e));
        let params = serde_json::from_str(&line).unwrap_or_else(|e| panic!(e));
        params
    }

    fn tell(&mut self, param: Self::Param, value: Self::Value) {
        let json = json!({"param": param, "value": value});
        serde_json::to_writer(&mut self.stdin, &json).unwrap_or_else(|e| panic!(e));
        writeln!(&mut self.stdin).unwrap_or_else(|e| panic!(e)); // TODO:
    }
}
impl Drop for ExternalCommandOptimizer {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct ExternalCommandOptimizerBuilder {
    pub name: PathBuf,
    pub args: Vec<String>,
}
impl OptimizerBuilder for ExternalCommandOptimizerBuilder {
    type Optimizer = ExternalCommandOptimizer;

    fn optimizer_name(&self) -> &str {
        self.name.file_stem().expect("TODO").to_str().expect("TODO")
    }

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        let mut child = Command::new(&self.name)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            //            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child.stdin.take().ok_or_else(|| format_err!("No stdin"))?;
        serde_json::to_writer(&mut stdin, problem_space)?;
        writeln!(&mut stdin)?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| format_err!("No stdout"))?;
        Ok(ExternalCommandOptimizer {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }
}
