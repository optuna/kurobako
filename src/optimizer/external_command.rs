use super::OptimizerBuilder;
use crate::{Error, ErrorKind, ProblemSpace};
use rand::Rng;
use serde_json::{self, json};
use std::fmt;
use std::io::{BufReader, Write as _};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use structopt::StructOpt;
use yamakan::Optimizer;

// #[derive(Debug)]
pub struct ExternalCommandOptimizer {
    child: Child,
    stdin: ChildStdin,
    stdout: serde_json::StreamDeserializer<
        'static,
        serde_json::de::IoRead<BufReader<ChildStdout>>,
        Vec<f64>,
    >,
}
impl fmt::Debug for ExternalCommandOptimizer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExternalCommandOptimizer {{ .. }}")
    }
}
impl Optimizer for ExternalCommandOptimizer {
    type Param = Vec<f64>;
    type Value = f64;

    fn ask<R: Rng>(&mut self, _rng: &mut R) -> Self::Param {
        let params = self.stdout.next().expect("TODO").expect("TODO");
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
        if self.child.kill().is_ok() {
            let _ = self.child.wait();
        }
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct ExternalCommandOptimizerBuilder {
    pub name: PathBuf,
    pub args: Vec<String>,
}
impl OptimizerBuilder for ExternalCommandOptimizerBuilder {
    type Optimizer = ExternalCommandOptimizer;

    fn build(&self, problem_space: &ProblemSpace) -> Result<Self::Optimizer, Error> {
        let mut child = track!(Command::new(&self.name)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(Error::from))?;

        let mut stdin = track_assert_some!(child.stdin.take(), ErrorKind::IoError);
        track!(serde_json::to_writer(&mut stdin, problem_space).map_err(Error::from))?;
        track!(writeln!(&mut stdin).map_err(Error::from))?;

        let stdout = track_assert_some!(child.stdout.take(), ErrorKind::InvalidInput);
        Ok(ExternalCommandOptimizer {
            child,
            stdin,
            stdout: serde_json::Deserializer::from_reader(BufReader::new(stdout)).into_iter(),
        })
    }
}
