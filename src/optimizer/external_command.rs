use super::OptimizerBuilder;
use crate::{Error, ErrorKind, ProblemSpace};
use rand::Rng;
use serde_json::{self, json};
use std::fmt;
use std::io::{BufReader, Write as _};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use structopt::StructOpt;
use trackable::error::ErrorKindExt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGenerator, Observation};
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
    need_tell: bool,
}
impl fmt::Debug for ExternalCommandOptimizer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ExternalCommandOptimizer {{ .. }}")
    }
}
impl Optimizer for ExternalCommandOptimizer {
    type Param = Budgeted<Vec<f64>>;
    type Value = f64;

    fn ask<R: Rng, G: IdGenerator>(
        &mut self,
        _rng: &mut R,
        idgen: &mut G,
    ) -> yamakan::Result<Observation<Self::Param, ()>> {
        if self.need_tell {
            let json = json!({});
            track!(serde_json::to_writer(&mut self.stdin, &json)
                .map_err(|e| yamakan::ErrorKind::IoError.cause(e)))?;
            track!(writeln!(&mut self.stdin).map_err(yamakan::Error::from))?;
        }

        let params = track_assert_some!(
            self.stdout.next(),
            yamakan::ErrorKind::IoError,
            "Unexpected EOS"
        );
        let params = track!(params.map_err(|e| yamakan::ErrorKind::InvalidInput.cause(e)))?;
        self.need_tell = true;

        let budget = Budget::new(::std::u64::MAX); // TODO
        let params = Budgeted::new(budget, params);
        track!(Observation::new(idgen, params))
    }

    fn tell(&mut self, obs: Observation<Self::Param, Self::Value>) -> yamakan::Result<()> {
        self.need_tell = false;

        // TODO: pass budget info
        let json = json!({"param": obs.param.get(), "value": obs.value});
        track!(serde_json::to_writer(&mut self.stdin, &json)
            .map_err(|e| yamakan::ErrorKind::IoError.cause(e)))?;
        track!(writeln!(&mut self.stdin).map_err(yamakan::Error::from))?;
        Ok(())
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

    fn build(
        &self,
        problem_space: &ProblemSpace,
        _eval_cost: u64,
    ) -> Result<Self::Optimizer, Error> {
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
            need_tell: false,
        })
    }
}
