use crate::problem::KurobakoProblemRecipe;
use crate::runner::StudyRunnerOptions;
use crate::solver::KurobakoSolverRecipe;
use crate::variable::Variable;
use kurobako_core::parameter::{ParamDomain, ParamValue};
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec, Values};
use kurobako_core::solver::{Solver, SolverRecipe, SolverSpec};
use kurobako_core::{json, Error, ErrorKind, Result};
use kurobako_solvers::random::RandomSolver;
use rand;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::{ObsId, SerialIdGenerator};

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

type Vars = Vec<ParamDomain>;
type Vals = HashMap<String, ParamValue>;

#[derive(Debug, Clone)]
struct ExamRecipeTemplate(json::JsonValue);
impl ExamRecipeTemplate {
    fn render(&self, vals: Vals) -> Result<ExamRecipe> {
        let mut recipe = self.0.get().clone();
        track!(Self::bind_vars(&mut recipe, &vals))?;
        debug!("Recipe: {}", recipe);
        let exam: ExamRecipe = track!(serde_json::from_value(recipe).map_err(Error::from))?;
        Ok(exam)
    }

    fn bind_vars(recipe: &mut serde_json::Value, vals: &Vals) -> Result<()> {
        match recipe {
            serde_json::Value::Array(a) => {
                for v in a {
                    track!(Self::bind_vars(v, vals))?;
                }
            }
            serde_json::Value::Object(o) => {
                for (k, v) in o {
                    if k.starts_with('$') {
                        let name = k.split_at(1).1;
                        let val = track_assert_some!(
                            vals.get(name),
                            ErrorKind::InvalidInput,
                            "Not Found: {:?}",
                            name
                        );
                        *v = track!(val.to_json_value())?;
                    } else {
                        track!(Self::bind_vars(v, vals))?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct ExamProblemRecipe {
    // TODO: flatten
    pub recipe: json::JsonValue,
}
impl ExamProblemRecipe {
    fn collect_vars(&self) -> Result<Vars> {
        let mut vars = Vec::new();
        track!(Self::collect_vars_recur(self.recipe.get(), &mut vars))?;
        Ok(vars)
    }

    fn collect_vars_recur(v: &serde_json::Value, vars: &mut Vars) -> Result<()> {
        match v {
            serde_json::Value::Array(a) => {
                for v in a {
                    track!(Self::collect_vars_recur(v, vars))?;
                }
                Ok(())
            }
            serde_json::Value::Object(o) => {
                for (k, v) in o {
                    if k.starts_with('$') {
                        let var: Variable =
                            track!(serde_json::from_value(v.clone()).map_err(Error::from))?;
                        let name = k.split_at(1).1;
                        vars.push(track!(var.to_param_domain(name))?);
                    } else {
                        track!(Self::collect_vars_recur(v, vars))?;
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
impl ProblemRecipe for ExamProblemRecipe {
    type Problem = ExamProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let vars = track!(self.collect_vars())?;
        let template = ExamRecipeTemplate(self.recipe.clone());

        let mut random = RandomSolver::new(vars.clone());
        let mut rng = rand::thread_rng(); // TODO
        let mut idg = SerialIdGenerator::new();
        let values = track!(random.ask(&mut rng, &mut idg))?;
        let vals = vars
            .iter()
            .zip(values.param.get().iter().cloned())
            .map(|(var, val)| (var.name().to_owned(), val))
            .collect();
        let exam = track!(template.render(vals))?;

        let problem = track!(exam.problem.create_problem())?.specification();
        let solver = track!(exam.solver.create_solver(problem.clone()))?.specification();
        Ok(ExamProblem {
            template,
            vars,
            exam,
            problem,
            solver,
        })
    }
}

#[derive(Debug)]
pub struct ExamProblem {
    template: ExamRecipeTemplate,
    vars: Vars,
    exam: ExamRecipe,
    problem: ProblemSpec,
    solver: SolverSpec,
}
impl Problem for ExamProblem {
    type Evaluator = ExamEvaluator;

    fn specification(&self) -> ProblemSpec {
        //        ProblemSpec{
        //  name: "exam".to_owned(), // TODO
        //  version: None,
        // params_domain: self.vars.values().cloned().collect(),
        // values_domain: Vec<MinMax<FiniteF64>>,
        // evaluation_expense: NonZeroU64,
        //  capabilities: EvaluatorCapabilities,
        //}
        panic!()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        // let exam: ExamRecipe =
        //     track!(serde_json::from_value(self.recipe.get().clone()).map_err(Error::from))?;

        panic!()
    }
}

#[derive(Debug)]
pub struct ExamEvaluator {}
impl Evaluate for ExamEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        panic!()
    }
}
