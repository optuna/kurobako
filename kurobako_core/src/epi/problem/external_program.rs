use crate::epi::channel::{JsonMessageReceiver, JsonMessageSender};
use crate::parameter::ParamValue;
use crate::problem::{
    Evaluate, Evaluated, EvaluatorCapabilities, EvaluatorCapability, Problem, ProblemRecipe,
    ProblemSpec,
};
use crate::time::Seconds;
use crate::{Error, ErrorKind, Result};
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;
use std::time::Instant;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct ExternalProgramProblemRecipe {
    pub path: PathBuf,
    pub args: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub skip_lines: Option<usize>,
}
impl ProblemRecipe for ExternalProgramProblemRecipe {
    type Problem = ExternalProgramProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let mut child = track!(Command::new(&self.path)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(Error::from))?;

        let stdin = BufWriter::new(track_assert_some!(child.stdin.take(), ErrorKind::IoError));
        let mut stdout =
            BufReader::new(track_assert_some!(child.stdout.take(), ErrorKind::IoError));
        for _ in 0..self.skip_lines.unwrap_or(0) {
            let mut s = String::new();
            track!(stdout.read_line(&mut s).map_err(Error::from))?;
            debug!("Skipped line ({:?}): {}", self.path, s);
        }

        let tx = Rc::new(RefCell::new(JsonMessageSender::new(stdin)));
        let rx = Rc::new(RefCell::new(JsonMessageReceiver::new(stdout)));
        let spec = match track!(rx.borrow_mut().recv())? {
            ProblemMessage::ProblemSpecCast(m) => m,
            m => track_panic!(ErrorKind::InvalidInput, "Unexpected message: {:?}", m),
        };

        Ok(ExternalProgramProblem {
            spec,
            child,
            tx,
            rx,
        })
    }
}

#[derive(Debug)]
pub struct ExternalProgramProblem {
    spec: ProblemSpec,
    child: Child,
    tx: Rc<RefCell<JsonMessageSender<ProblemMessage, BufWriter<ChildStdin>>>>,
    rx: Rc<RefCell<JsonMessageReceiver<ProblemMessage, BufReader<ChildStdout>>>>,
}
impl Problem for ExternalProgramProblem {
    type Evaluator = ExternalProgramEvaluator;

    fn specification(&self) -> ProblemSpec {
        self.spec.clone()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        let m = ProblemMessage::CreateEvaluatorCast { id };
        track!(self.tx.borrow_mut().send(&m))?;
        Ok(ExternalProgramEvaluator {
            id,
            tx: self.tx.clone(),
            rx: self.rx.clone(),
            capabilities: self.spec.capabilities.clone(),
            prev_params: None,
            prev_values: None,
        })
    }
}
impl Drop for ExternalProgramProblem {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}

#[derive(Debug)]
pub struct ExternalProgramEvaluator {
    id: ObsId,
    tx: Rc<RefCell<JsonMessageSender<ProblemMessage, BufWriter<ChildStdin>>>>,
    rx: Rc<RefCell<JsonMessageReceiver<ProblemMessage, BufReader<ChildStdout>>>>,
    capabilities: EvaluatorCapabilities,
    prev_params: Option<Vec<ParamValue>>,
    prev_values: Option<Vec<FiniteF64>>,
}
impl ExternalProgramEvaluator {
    // TODO: Move to `kurobako` crate
    fn check_capabilities(&self, params: &[ParamValue]) -> Result<()> {
        if !self
            .capabilities
            .contains(&EvaluatorCapability::DynamicParamChange)
        {
            if let Some(prev_params) = &self.prev_params {
                track_assert_eq!(
                    params,
                    &prev_params[..],
                    ErrorKind::Incapable,
                    "{:?}",
                    EvaluatorCapability::DynamicParamChange
                );
            }
        }
        if !self.capabilities.contains(&EvaluatorCapability::Concurrent) {
            track_assert_eq!(
                Rc::strong_count(&self.tx),
                2,
                ErrorKind::Incapable,
                "{:?}",
                EvaluatorCapability::Concurrent
            );
        }
        Ok(())
    }
}

impl Evaluate for ExternalProgramEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Evaluated> {
        track!(self.check_capabilities(params))?;
        self.prev_params = Some(Vec::from(params));

        if budget.is_consumed() {
            if let Some(prev_values) = self.prev_values.clone() {
                return Ok(Evaluated::new(prev_values, Seconds::zero()));
            }
        }

        let m = ProblemMessage::EvaluateCall {
            id: self.id,
            params: Vec::from(params),
            budget: *budget,
        };
        track!(self.tx.borrow_mut().send(&m))?;

        let now = Instant::now();
        match track!(self.rx.borrow_mut().recv())? {
            ProblemMessage::EvaluateOkReply {
                values,
                budget: consumed_budget,
                elapsed,
            } => {
                track_assert_eq!(
                    consumed_budget.amount,
                    budget.amount,
                    ErrorKind::InvalidInput
                );
                // TODO
                // track_assert!(consumed_budget.is_consumed(), ErrorKind::InvalidInput; consumed_budget);
                budget.consumption = consumed_budget.consumption;

                let elapsed = elapsed.unwrap_or_else(|| Seconds::from(now.elapsed()));
                self.prev_values = Some(values.clone());

                Ok(Evaluated::new(values, elapsed))
            }
            ProblemMessage::EvaluateErrorReply { kind, message } => {
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
impl Drop for ExternalProgramEvaluator {
    fn drop(&mut self) {
        let m = ProblemMessage::DropEvaluatorCast { id: self.id };
        let _ = self.tx.borrow_mut().send(&m);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemMessage {
    ProblemSpecCast(ProblemSpec),
    CreateEvaluatorCast {
        id: ObsId,
    },
    DropEvaluatorCast {
        id: ObsId,
    },
    EvaluateCall {
        id: ObsId,
        params: Vec<ParamValue>,
        budget: Budget,
    },
    EvaluateOkReply {
        values: Vec<FiniteF64>,
        budget: Budget,
        #[serde(default)]
        elapsed: Option<Seconds>,
    },
    EvaluateErrorReply {
        kind: ErrorKind,
        #[serde(default)]
        message: Option<String>,
    },
}
