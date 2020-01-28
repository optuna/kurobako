use crate::epi::channel::{MessageReceiver, MessageSender};
use crate::epi::solver::SolverMessage;
use crate::problem::ProblemSpec;
use crate::registry::FactoryRegistry;
use crate::rng::{ArcRng, Rng as _};
use crate::solver::{Solver, SolverFactory, SolverRecipe, SolverSpec};
use crate::trial::{EvaluatedTrial, IdGen, NextTrial};
use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex};
use std::thread_local;
use structopt::StructOpt;

thread_local! {
    static FACTORIES: RefCell<HashMap<Vec<u8>, ExternalProgramSolverFactory>> =
        RefCell::new(HashMap::new());
}

/// Recipe for the solver that is implemented by an external program.
#[derive(Debug, Clone, PartialEq, Eq, Hash, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct ExternalProgramSolverRecipe {
    /// The path of the external program.
    pub path: PathBuf,

    /// The command line arguments that are passed to the program.
    pub args: Vec<String>,
}
impl ExternalProgramSolverRecipe {
    fn create_new_factory(
        &self,
        _registry: &FactoryRegistry,
    ) -> Result<ExternalProgramSolverFactory> {
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
            SolverMessage::SolverSpecCast { spec } => spec,
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        };

        Ok(ExternalProgramSolverFactory(Arc::new(
            ExternalProgramSolverFactoryInner {
                spec,
                child,
                tx: Arc::new(Mutex::new(tx)),
                rx: Arc::new(Mutex::new(rx)),
                next_solver_id: AtomicU64::new(0),
            },
        )))
    }

    fn cache_key(&self) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.input(&track!(fs::read(&self.path).map_err(Error::from); self.path)?);
        for arg in &self.args {
            hasher.input(arg.as_bytes());
        }
        Ok(hasher.result().to_vec())
    }
}
impl SolverRecipe for ExternalProgramSolverRecipe {
    type Factory = ExternalProgramSolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        FACTORIES.with(|f| {
            let mut f = f.borrow_mut();
            let key = track!(self.cache_key())?;
            if !f.contains_key(&key) {
                eprintln!("Create new solver: {:?}", self);
                f.insert(key.clone(), track!(self.create_new_factory(registry))?);
            }
            Ok(f[&key].clone())
        })
    }
}

/// Factory for the solver that is implemented by an external program.
#[derive(Debug, Clone)]
pub struct ExternalProgramSolverFactory(Arc<ExternalProgramSolverFactoryInner>);
impl SolverFactory for ExternalProgramSolverFactory {
    type Solver = ExternalProgramSolver;

    fn specification(&self) -> Result<SolverSpec> {
        track!(self.0.specification())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        track!(self.0.create_solver(rng, problem))
    }
}

#[derive(Debug)]
struct ExternalProgramSolverFactoryInner {
    spec: SolverSpec,
    child: Child,
    tx: Arc<Mutex<MessageSender<SolverMessage, ChildStdin>>>,
    rx: Arc<Mutex<MessageReceiver<SolverMessage, ChildStdout>>>,
    next_solver_id: AtomicU64,
}
impl SolverFactory for ExternalProgramSolverFactoryInner {
    type Solver = ExternalProgramSolver;

    fn specification(&self) -> Result<SolverSpec> {
        Ok(self.spec.clone())
    }

    fn create_solver(&self, mut rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let solver_id = self.next_solver_id.fetch_add(1, atomic::Ordering::SeqCst);
        let m = SolverMessage::CreateSolverCast {
            solver_id,
            random_seed: rng.gen(),
            problem: problem.clone(),
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        Ok(ExternalProgramSolver {
            solver_id,
            tx: Arc::clone(&self.tx),
            rx: Arc::clone(&self.rx),
        })
    }
}
impl Drop for ExternalProgramSolverFactoryInner {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}

/// Solver that is implemented by an external program.
#[derive(Debug)]
pub struct ExternalProgramSolver {
    solver_id: u64,
    tx: Arc<Mutex<MessageSender<SolverMessage, ChildStdin>>>,
    rx: Arc<Mutex<MessageReceiver<SolverMessage, ChildStdout>>>,
}
impl Solver for ExternalProgramSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        let m = SolverMessage::AskCall {
            solver_id: self.solver_id,
            next_trial_id: idg.peek_id().get(),
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        let mut rx = track!(self.rx.lock().map_err(Error::from))?;
        match track!(rx.recv())? {
            SolverMessage::AskReply {
                trial,
                next_trial_id,
            } => {
                track_assert!(
                    idg.peek_id().get() <= next_trial_id,
                    ErrorKind::InvalidInput; idg.peek_id().get(), next_trial_id
                );
                while idg.peek_id().get() < next_trial_id {
                    idg.generate();
                }

                Ok(trial)
            }
            SolverMessage::ErrorReply { kind, message } => {
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

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        let m = SolverMessage::TellCall {
            solver_id: self.solver_id,
            trial,
        };
        let mut tx = track!(self.tx.lock().map_err(Error::from))?;
        track!(tx.send(&m))?;

        let mut rx = track!(self.rx.lock().map_err(Error::from))?;
        match track!(rx.recv())? {
            SolverMessage::TellReply => Ok(()),
            SolverMessage::ErrorReply { kind, message } => {
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
impl Drop for ExternalProgramSolver {
    fn drop(&mut self) {
        let solver_id = self.solver_id;
        let m = SolverMessage::DropSolverCast { solver_id };
        if let Ok(mut tx) = self.tx.lock() {
            let _ = tx.send(&m);
        }
    }
}
