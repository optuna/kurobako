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
use std::collections::HashMap;
use std::num::NonZeroU64;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct ExamRecipe {
    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub solver: KurobakoSolverRecipe,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub problem: KurobakoProblemRecipe,

    #[serde(flatten)]
    #[structopt(flatten)]
    pub runner: StudyRunnerOptions,
    // TODO: Metric (best or auc, ...)
}
impl ExamRecipe {
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
pub struct ExamProblemRecipe {
    #[structopt(long)]
    pub recipe: json::JsonValue,

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    pub vars: Vec<Variable>,
}
impl ProblemRecipe for ExamProblemRecipe {
    type Problem = ExamProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let exam: ExamRecipe =
            track!(serde_json::from_value(self.recipe.get().clone()).map_err(Error::from))?;
        let problem = track!(exam.problem.create_problem())?.specification();
        let solver = track!(exam.solver.create_solver(problem.clone()))?.specification();

        Ok(ExamProblem {
            exam,
            vars: self
                .vars
                .iter()
                .map(|v| track!(v.to_param_domain()))
                .collect::<Result<_>>()?,
            problem,
            solver,
        })
    }
}

#[derive(Debug)]
pub struct ExamProblem {
    exam: ExamRecipe,
    vars: Vars,
    problem: ProblemSpec,
    solver: SolverSpec,
}
impl Problem for ExamProblem {
    type Evaluator = ExamEvaluator;

    fn specification(&self) -> ProblemSpec {
        let evaluation_expense =
            NonZeroU64::new(self.exam.runner.budget as u64 * self.problem.evaluation_expense.get())
                .unwrap_or_else(|| unimplemented!());
        ProblemSpec {
            name: format!("exam/{}/{}", self.solver.name, self.problem.name),
            version: None, // TODO
            params_domain: self.vars.clone(),
            values_domain: self.problem.values_domain.clone(), // TODO
            evaluation_expense,
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(ExamEvaluator::NotStarted {
            exam: self.exam.clone(),
            vars: self.vars.clone(),
        })
    }
}

#[derive(Debug)]
pub enum ExamEvaluator {
    NotStarted {
        exam: ExamRecipe,
        vars: Vars,
    },
    Running {
        runner: StudyRunner<KurobakoSolver, BoxProblem>,
    },
}
impl Evaluate for ExamEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        loop {
            let next = match self {
                ExamEvaluator::NotStarted { exam, vars } => {
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

                    let runner =
                        track!(StudyRunner::new(&exam.solver, &exam.problem, &exam.runner))?;
                    ExamEvaluator::Running { runner }
                }
                ExamEvaluator::Running { runner } => {
                    track!(runner.run_once(budget))?;
                    loop {
                        trace!("Study: {:?}", runner.study());
                        if let Some(v) = runner.study().best_value() {
                            debug!(
                                "Evaluated: budget={:?}, params={:?}, value={:?}",
                                budget,
                                params,
                                v.get()
                            );
                            return Ok(vec![v]);
                        }

                        budget.amount = budget.consumption + 1;
                        track!(runner.run_once(budget))?;
                    }
                }
            };
            *self = next;
        }
    }
}
