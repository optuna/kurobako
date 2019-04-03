use crate::problem::{Evaluate, Problem, ProblemSpace, ProblemSpec};
use crate::serde_json_line;
use crate::{Error, ErrorKind, Result, ValueRange};
use std::cell::RefCell;
use std::io::{BufReader, Write as _};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::rc::Rc;
use yamakan::budget::Budget;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct CommandProblemSpec {
    pub path: PathBuf,
    pub args: Vec<String>,
}
impl ProblemSpec for CommandProblemSpec {
    type Problem = CommandProblem;

    fn make_problem(&self) -> Result<Self::Problem> {
        let mut child = Command::new(&self.path)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = track_assert_some!(child.stdin.take(), ErrorKind::IoError);
        let mut stdout =
            BufReader::new(track_assert_some!(child.stdout.take(), ErrorKind::IoError));
        let info: ProblemInfo = track!(serde_json_line::from_reader(&mut stdout))?;
        Ok(CommandProblem {
            info,
            child,
            stdin: Rc::new(RefCell::new(stdin)),
            stdout: Rc::new(RefCell::new(stdout)),
            next_eval_id: 0,
        })
    }
}

#[derive(Debug)]
pub struct CommandProblem {
    info: ProblemInfo,
    child: Child,
    stdin: Rc<RefCell<ChildStdin>>,
    stdout: Rc<RefCell<BufReader<ChildStdout>>>,
    next_eval_id: u32,
}
impl Problem for CommandProblem {
    type Evaluator = CommandEvaluator;

    fn problem_space(&self) -> ProblemSpace {
        self.info.problem_space.clone()
    }

    fn evaluation_cost(&self) -> u64 {
        self.info.cost
    }

    fn value_range(&self) -> ValueRange {
        self.info.value_range
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Result<Option<Self::Evaluator>> {
        let m = StartEvalMessage {
            kind: "start_eval",
            eval_id: self.next_eval_id,
            params: Vec::from(params),
        };
        self.next_eval_id += 1;

        let json = track!(serde_json::to_string(&m).map_err(Error::from))?;
        track!(writeln!(&mut *self.stdin.borrow_mut(), "{}", json).map_err(Error::from))?;

        let res: StartEvalResMessage =
            serde_json_line::from_reader(&mut *self.stdout.borrow_mut())?;
        if !res.ok {
            // invalid params
            Ok(None)
        } else {
            Ok(Some(CommandEvaluator {
                eval_id: m.eval_id,
                stdin: self.stdin.clone(),
                stdout: self.stdout.clone(),
            }))
        }
    }
}
impl Drop for CommandProblem {
    fn drop(&mut self) {
        if self.child.kill().is_ok() {
            let _ = self.child.wait(); // for preventing the child process becomes a zombie.
        }
    }
}

#[derive(Debug)]
pub struct CommandEvaluator {
    eval_id: u32,
    stdin: Rc<RefCell<ChildStdin>>,
    stdout: Rc<RefCell<BufReader<ChildStdout>>>,
}
impl Evaluate for CommandEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Result<f64> {
        let m = EvalReqMessage {
            kind: "eval",
            eval_id: self.eval_id,
            budget: budget.remaining(),
        };
        writeln!(
            &mut *self.stdin.borrow_mut(),
            "{}",
            serde_json::to_string(&m)?
        )?;

        let m: EvalResMessage = serde_json_line::from_reader(&mut *self.stdout.borrow_mut())?;
        budget.consume(m.cost);
        Ok(m.value)
    }
}
impl Drop for CommandEvaluator {
    fn drop(&mut self) {
        let m = FinishEvalMessage {
            kind: "finish_eval",
            eval_id: self.eval_id,
        };
        if let Ok(m) = serde_json::to_string(&m) {
            let _ = writeln!(&mut *self.stdin.borrow_mut(), "{}", m);
        }
    }
}

#[derive(Debug, Deserialize)]
struct ProblemInfo {
    cost: u64,
    problem_space: ProblemSpace,
    value_range: ValueRange,
}

#[derive(Debug, Serialize)]
struct StartEvalMessage {
    kind: &'static str,
    eval_id: u32,
    params: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StartEvalResMessage {
    ok: bool,
}

#[derive(Debug, Serialize)]
struct FinishEvalMessage {
    kind: &'static str,
    eval_id: u32,
}

#[derive(Debug, Serialize)]
struct EvalReqMessage {
    kind: &'static str,
    eval_id: u32,
    budget: u64,
}

#[derive(Debug, Deserialize)]
struct EvalResMessage {
    value: f64,
    cost: u64,
}
