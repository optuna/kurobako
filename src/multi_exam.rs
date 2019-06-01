use crate::problem::KurobakoProblemRecipe;
use crate::runner::{StudyRunner, StudyRunnerOptions};
use crate::solver::{KurobakoSolver, KurobakoSolverRecipe};
use crate::variable::{VarPath, Variable};
use kurobako_core::parameter::{ParamDomain, ParamValue};
use kurobako_core::problem::{
    BoxProblem, Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::solver::{Solver, SolverRecipe, SolverSpec};
use kurobako_core::{json, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::cmp::Ordering;
use std::collections::BinaryHeap;
use std::collections::HashMap;
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct MultiExamRecipe {
    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub solver: KurobakoSolverRecipe,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub problems: Vec<KurobakoProblemRecipe>,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
    // TODO: Metric (raw or dominate-count, ...)
}
impl MultiExamRecipe {
    fn bind(&self, vals: &Vals) -> Result<Self> {
        let mut recipe = track!(serde_json::to_value(self.clone()).map_err(Error::from))?;
        track!(Self::bind_recur(&mut recipe, vals, &mut VarPath::new()))?;
        track!(serde_json::from_value(recipe).map_err(Error::from))
    }

    fn bind_recur(recipe: &mut serde_json::Value, vals: &Vals, path: &mut VarPath) -> Result<()> {
        if let Some(val) = vals.get(path) {
            *recipe = track!(val.to_json_value())?;
        } else {
            match recipe {
                serde_json::Value::Array(a) => {
                    for (i, v) in a.iter_mut().enumerate() {
                        path.push(i.to_string());
                        track!(Self::bind_recur(v, vals, path))?;
                        path.pop();
                    }
                }
                serde_json::Value::Object(o) => {
                    for (k, v) in o {
                        path.push(k.to_owned());
                        track!(Self::bind_recur(v, vals, path))?;
                        path.pop();
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}

type Vars = Vec<ParamDomain>;
type Vals = HashMap<VarPath, ParamValue>;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct MultiExamProblemRecipe {
    #[structopt(long)]
    pub recipe: json::JsonValue,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub vars: Vec<Variable>,
}
impl ProblemRecipe for MultiExamProblemRecipe {
    type Problem = MultiExamProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let exam: MultiExamRecipe =
            track!(serde_json::from_value(self.recipe.get().clone()).map_err(Error::from))?;
        let problems = exam
            .problems
            .iter()
            .map(|p| track!(p.create_problem()).map(|p| p.specification()))
            .collect::<Result<Vec<_>>>()?;

        // TODO
        let solver = track!(exam.solver.create_solver(problems[0].clone()))?.specification();

        let lcm = lcm(problems.iter().map(|p| p.evaluation_expense.get()));
        Ok(MultiExamProblem {
            exam,
            vars: self
                .vars
                .iter()
                .map(|v| track!(v.to_param_domain()))
                .collect::<Result<_>>()?,
            problems,
            solver,
            lcm,
        })
    }
}

// TODO
fn lcm<I>(ns: I) -> u64
where
    I: Iterator<Item = u64>,
{
    use std::collections::BTreeSet;

    ns.collect::<BTreeSet<_>>().into_iter().product()
}

#[derive(Debug)]
pub struct MultiExamProblem {
    exam: MultiExamRecipe,
    vars: Vars,
    problems: Vec<ProblemSpec>,
    solver: SolverSpec,
    lcm: u64,
}
impl Problem for MultiExamProblem {
    type Evaluator = MultiExamEvaluator;

    fn specification(&self) -> ProblemSpec {
        let evaluation_expense =
            NonZeroU64::new(self.exam.runner.budget as u64 * self.problems.len() as u64 * self.lcm)
                .unwrap_or_else(|| unimplemented!());
        ProblemSpec {
            name: format!("exam/{}/{}", self.solver.name, self.problems.len()), // TODO
            version: None,                                                      // TODO
            params_domain: self.vars.clone(),
            values_domain: self
                .problems
                .iter()
                .flat_map(|p| p.values_domain.clone().into_iter())
                .collect(),
            evaluation_expense,
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(MultiExamEvaluator::NotStarted {
            exam: self.exam.clone(),
            vars: self.vars.clone(),
            lcm: self.lcm,
        })
    }
}

#[derive(Debug)]
pub struct Runner {
    scale: u64,
    index: usize,
    inner: StudyRunner<KurobakoSolver, BoxProblem>,
}
impl Runner {
    fn new(index: usize, scale: u64, inner: StudyRunner<KurobakoSolver, BoxProblem>) -> Self {
        Self {
            index,
            scale,
            inner,
        }
    }

    fn consumption(&self) -> u64 {
        self.inner.study_budget.consumption * self.scale
    }
}
impl PartialOrd for Runner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.consumption().partial_cmp(&self.consumption())
    }
}
impl Ord for Runner {
    fn cmp(&self, other: &Self) -> Ordering {
        other.consumption().cmp(&self.consumption())
    }
}
impl PartialEq for Runner {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
impl Eq for Runner {}

#[derive(Debug)]
pub enum MultiExamEvaluator {
    NotStarted {
        exam: MultiExamRecipe,
        vars: Vars,
        lcm: u64,
    },
    Running {
        runners: BinaryHeap<Runner>,
    },
}
impl Evaluate for MultiExamEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        loop {
            let next = match self {
                MultiExamEvaluator::NotStarted { exam, vars, lcm } => {
                    let vals = vars
                        .iter()
                        .zip(params.iter())
                        .map(|(var, param)| {
                            let path = track!(var.name().parse())
                                .unwrap_or_else(|e| unreachable!("{}", e));
                            (path, param.clone())
                        })
                        .collect();
                    debug!("Before bind: {:?}", exam);
                    let exam = track!(exam.bind(&vals))?;
                    debug!("After bind: {:?}", exam);

                    let runners = exam
                        .problems
                        .iter()
                        .enumerate()
                        .map(|(i, p)| {
                            let inner = track!(StudyRunner::new(&exam.solver, p, &exam.runner))?;
                            let scale = *lcm / inner.study().problem.spec.evaluation_expense.get();
                            Ok(Runner::new(i, scale, inner))
                        })
                        .collect::<Result<_>>()?;
                    MultiExamEvaluator::Running { runners }
                }
                MultiExamEvaluator::Running { runners } => {
                    while !budget.is_consumed() {
                        let mut runner = runners.pop().unwrap_or_else(|| unreachable!());

                        let mut runner_budget = runner.inner.study_budget.clone();
                        let old_consumption = runner_budget.consumption;

                        runner_budget.amount = runner_budget.consumption + 1;
                        track!(runner.inner.run_once(&mut runner_budget))?;
                        loop {
                            trace!("Study: {:?}", runner.inner.study());
                            if runner.inner.study().best_value().is_some() {
                                budget.consumption +=
                                    (runner_budget.consumption - old_consumption) * runner.scale;
                                runners.push(runner);
                                break;
                            }

                            runner_budget.amount = runner_budget.consumption + 1;
                            track!(runner.inner.run_once(&mut runner_budget))?;
                        }
                    }

                    let mut vs = runners
                        .iter()
                        .map(|r| {
                            (
                                r.index,
                                r.inner
                                    .study()
                                    .best_value()
                                    .unwrap_or_else(|| unreachable!()),
                            )
                        })
                        .collect::<Vec<_>>();
                    vs.sort_by_key(|v| v.0);
                    let vs = vs.into_iter().map(|v| v.1).collect::<Vec<_>>();
                    debug!(
                        "Evaluated: budget={:?}, params={:?}, value={:?}",
                        budget, params, vs,
                    );
                    return Ok(vs);
                }
            };
            *self = next;
        }
    }
}
