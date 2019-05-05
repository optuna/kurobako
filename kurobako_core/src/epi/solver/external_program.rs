use crate::epi::channel::{JsonMessageReceiver, JsonMessageSender};
use crate::parameter::ParamValue;
use crate::problem::ProblemSpec;
use crate::solver::{Asked, ObservedObs, Solver, SolverRecipe, SolverSpec};
use crate::time::Elapsed;
use crate::{Error, ErrorKind, Result};
use rand::Rng;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Instant;
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs, ObsId};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct ExternalProgramSolverRecipe {
    pub path: PathBuf,
    pub args: Vec<String>,
}
impl SolverRecipe for ExternalProgramSolverRecipe {
    type Solver = ExternalProgramSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let mut child = track!(Command::new(&self.path)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(Error::from))?;

        let stdin = BufWriter::new(track_assert_some!(child.stdin.take(), ErrorKind::IoError));
        let stdout = BufReader::new(track_assert_some!(child.stdout.take(), ErrorKind::IoError));

        let mut tx = JsonMessageSender::new(stdin);
        let mut rx = JsonMessageReceiver::new(stdout);
        let spec = match track!(rx.recv())? {
            SolverMessage::SolverSpecCast(m) => m,
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        };

        // TODO: check capabilities
        track!(tx.send(&SolverMessage::ProblemSpecCast(problem.clone())))?;

        Ok(ExternalProgramSolver {
            spec,
            child,
            tx,
            rx,
        })
    }
}

#[derive(Debug)]
pub struct ExternalProgramSolver {
    spec: SolverSpec,
    child: Child,
    tx: JsonMessageSender<SolverMessage, BufWriter<ChildStdin>>,
    rx: JsonMessageReceiver<SolverMessage, BufReader<ChildStdout>>,
}
impl Solver for ExternalProgramSolver {
    fn specification(&self) -> SolverSpec {
        self.spec.clone()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, _rng: &mut R, idg: &mut G) -> Result<Asked> {
        let id_hint = track!(idg.generate())?;
        let message = SolverMessage::AskRequest { id_hint };
        track!(self.tx.send(&message))?;

        let now = Instant::now();
        match track!(self.rx.recv())? {
            SolverMessage::AskReply {
                id,
                params,
                budget,
                elapsed,
            } => {
                let elapsed = elapsed.unwrap_or_else(|| Elapsed::from(now.elapsed()));
                let obs = Obs {
                    id,
                    param: Budgeted::new(budget, params),
                    value: (),
                };
                Ok(Asked { obs, elapsed })
            }
            SolverMessage::ErrorReply { kind, message } => {
                if let Some(message) = message {
                    track_panic!(kind, "{}", message);
                } else {
                    track_panic!(kind);
                }
            }
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        }
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<Elapsed> {
        let message = SolverMessage::TellRequest {
            id: obs.id,
            budget: obs.param.budget(),
            params: obs.param.into_inner(),
            values: obs.value,
        };
        track!(self.tx.send(&message))?;

        let now = Instant::now();
        match track!(self.rx.recv())? {
            SolverMessage::TellReply { elapsed } => {
                let elapsed = elapsed.unwrap_or_else(|| Elapsed::from(now.elapsed()));
                Ok(elapsed)
            }
            SolverMessage::ErrorReply { kind, message } => {
                if let Some(message) = message {
                    track_panic!(kind, "{}", message);
                } else {
                    track_panic!(kind);
                }
            }
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        }
    }
}
impl Drop for ExternalProgramSolver {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SolverMessage {
    SolverSpecCast(SolverSpec),
    ProblemSpecCast(ProblemSpec),
    AskRequest {
        id_hint: ObsId,
    },
    AskReply {
        id: ObsId,
        params: Vec<ParamValue>,
        budget: Budget,
        #[serde(default)]
        elapsed: Option<Elapsed>,
    },
    TellRequest {
        id: ObsId,
        params: Vec<ParamValue>,
        budget: Budget,
        values: Vec<FiniteF64>,
    },
    TellReply {
        #[serde(default)]
        elapsed: Option<Elapsed>,
    },
    ErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
