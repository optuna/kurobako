use crate::epi::channel::{MessageReceiver, MessageSender};
use crate::epi::problem::ProblemMessage;
use crate::problem::{Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec};
use crate::registry::FactoryRegistry;
use crate::rng::{ArcRng, Rng as _};
use crate::trial::{Params, Values};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

/// Recipe for the problem implemented by an external program.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct ExternalProgramProblemRecipe {
    /// The path of the external program.
    pub path: PathBuf,

    /// The command line arguments that are passed to the program.
    pub args: Vec<String>,
}
impl ProblemRecipe for ExternalProgramProblemRecipe {
    type Factory = ExternalProgramProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let mut child = track!(Command::new(&self.path)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(Error::from))?;

        let stdin = track_assert_some!(child.stdin.take(), ErrorKind::IoError);
        let stdout = track_assert_some!(child.stdout.take(), ErrorKind::IoError);

        let tx = MessageSender::new(stdin);
        let mut rx = MessageReceiver::new(stdout);
        let spec = match track!(rx.recv())? {
            ProblemMessage::ProblemSpecCast { spec } => spec,
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        };

        Ok(ExternalProgramProblemFactory {
            spec,
            child,
            tx: Arc::new(Mutex::new(tx)),
            rx: Arc::new(Mutex::new(rx)),
            next_problem_id: AtomicU64::new(0),
            next_evaluator_id: Arc::new(AtomicU64::new(0)),
        })
    }
}

/// Factory for the problem implemented by an external program.
#[derive(Debug)]
pub struct ExternalProgramProblemFactory {
    spec: ProblemSpec,
    child: Child,
    tx: Arc<Mutex<MessageSender<ProblemMessage, ChildStdin>>>,
    rx: Arc<Mutex<MessageReceiver<ProblemMessage, ChildStdout>>>,
    next_problem_id: AtomicU64,
    next_evaluator_id: Arc<AtomicU64>,
}
impl ProblemFactory for ExternalProgramProblemFactory {
    type Problem = ExternalProgramProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        Ok(self.spec.clone())
    }

    fn create_problem(&self, mut rng: ArcRng) -> Result<Self::Problem> {
        let problem_id = self.next_problem_id.fetch_add(1, atomic::Ordering::SeqCst);
        let m = ProblemMessage::CreateProblemCast {
            problem_id,
            random_seed: rng.gen(),
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        Ok(ExternalProgramProblem {
            problem_id,
            tx: Arc::clone(&self.tx),
            rx: Arc::clone(&self.rx),
            next_evaluator_id: Arc::clone(&self.next_evaluator_id),
        })
    }
}
impl Drop for ExternalProgramProblemFactory {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}

/// Problem that is implemented by an external program.
#[derive(Debug)]
pub struct ExternalProgramProblem {
    problem_id: u64,
    tx: Arc<Mutex<MessageSender<ProblemMessage, ChildStdin>>>,
    rx: Arc<Mutex<MessageReceiver<ProblemMessage, ChildStdout>>>,
    next_evaluator_id: Arc<AtomicU64>,
}
impl Problem for ExternalProgramProblem {
    type Evaluator = ExternalProgramEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let evaluator_id = self
            .next_evaluator_id
            .fetch_add(1, atomic::Ordering::SeqCst);
        let m = ProblemMessage::CreateEvaluatorCall {
            problem_id: self.problem_id,
            evaluator_id,
            params: params.into(),
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        let mut rx = track!(self.rx.lock().map_err(Error::from))?;
        match track!(rx.recv())? {
            ProblemMessage::CreateEvaluatorReply => {}
            ProblemMessage::ErrorReply { kind, message } => {
                if let Some(message) = message {
                    track_panic!(kind, "{}", message);
                } else {
                    track_panic!(kind);
                }
            }
            m => {
                track_panic!(ErrorKind::Other, "Unexpected message: {:?}", m);
            }
        }

        Ok(ExternalProgramEvaluator {
            evaluator_id,
            tx: Arc::clone(&self.tx),
            rx: Arc::clone(&self.rx),
        })
    }
}
impl Drop for ExternalProgramProblem {
    fn drop(&mut self) {
        let problem_id = self.problem_id;
        let m = ProblemMessage::DropProblemCast { problem_id };
        if let Ok(mut tx) = self.tx.lock() {
            let _ = tx.send(&m);
        }
    }
}

/// Evaluator that is implemented by an external program.
#[derive(Debug)]
pub struct ExternalProgramEvaluator {
    evaluator_id: u64,
    tx: Arc<Mutex<MessageSender<ProblemMessage, ChildStdin>>>,
    rx: Arc<Mutex<MessageReceiver<ProblemMessage, ChildStdout>>>,
}
impl Evaluator for ExternalProgramEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let evaluator_id = self.evaluator_id;
        let m = ProblemMessage::EvaluateCall {
            evaluator_id,
            next_step,
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        let mut rx = track!(self.rx.lock().map_err(Error::from))?;
        match track!(rx.recv())? {
            ProblemMessage::EvaluateReply {
                current_step,
                values,
            } => Ok((current_step, values)),
            ProblemMessage::ErrorReply { kind, message } => {
                if let Some(message) = message {
                    track_panic!(kind, "{}", message);
                } else {
                    track_panic!(kind);
                }
            }
            m => {
                track_panic!(ErrorKind::Other, "Unexpected message: {:?}", m);
            }
        }
    }
}
impl Drop for ExternalProgramEvaluator {
    fn drop(&mut self) {
        let m = ProblemMessage::DropEvaluatorCast {
            evaluator_id: self.evaluator_id,
        };
        if let Ok(mut tx) = self.tx.lock() {
            let _ = tx.send(&m);
        }
    }
}
