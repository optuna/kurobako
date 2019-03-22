use crate::{Evaluate, Problem, ProblemSpace, ProblemSpec};
use failure::Fallible;
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

    fn make_problem(&self) -> Fallible<Self::Problem> {
        let name = self
            .path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or_else(|| format_err!("Invalid Path"))?
            .to_owned();

        let mut child = Command::new(&self.path)
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = child.stdin.take().ok_or_else(|| format_err!("No stdin"))?;
        let mut stdout = BufReader::new(
            child
                .stdout
                .take()
                .ok_or_else(|| format_err!("No stdout"))?,
        );
        let info: ProblemInfo = serde_json::from_reader(&mut stdout)?;
        Ok(CommandProblem {
            name,
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
    name: String,
    info: ProblemInfo,
    child: Child,
    stdin: Rc<RefCell<ChildStdin>>,
    stdout: Rc<RefCell<BufReader<ChildStdout>>>,
    next_eval_id: u32,
}
impl Problem for CommandProblem {
    type Evaluator = CommandEvaluator;

    fn name(&self) -> &str {
        &self.name
    }

    fn problem_space(&self) -> ProblemSpace {
        self.info.problem_space.clone()
    }

    fn evaluation_cost_hint(&self) -> usize {
        self.info.cost_hint
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Fallible<Self::Evaluator> {
        let m = StartEvalMessage {
            kind: "start_eval",
            eval_id: self.next_eval_id,
            params: Vec::from(params),
        };
        self.next_eval_id += 1;

        writeln!(
            &mut *self.stdin.borrow_mut(),
            "{}",
            serde_json::to_string(&m)?
        )?;
        Ok(CommandEvaluator {
            eval_id: m.eval_id,
            stdin: self.stdin.clone(),
            stdout: self.stdout.clone(),
        })
    }
}

#[derive(Debug)]
pub struct CommandEvaluator {
    eval_id: u32,
    stdin: Rc<RefCell<ChildStdin>>,
    stdout: Rc<RefCell<BufReader<ChildStdout>>>,
}
impl Evaluate for CommandEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Fallible<f64> {
        let m = EvalReqMessage {
            kind: "eval",
            eval_id: self.eval_id,
            budget: budget.amount(),
        };
        writeln!(
            &mut *self.stdin.borrow_mut(),
            "{}",
            serde_json::to_string(&m)?
        )?;

        let m: EvalResMessage = serde_json::from_reader(&mut *self.stdout.borrow_mut())?;
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
    cost_hint: usize,
    problem_space: ProblemSpace,
}

#[derive(Debug, Serialize)]
struct StartEvalMessage {
    kind: &'static str,
    eval_id: u32,
    params: Vec<f64>,
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
