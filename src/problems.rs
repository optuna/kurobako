use crate::{Evaluate, Problem, ProblemSpace, ProblemSpec, Result};
use kurobako_core::problems::command;
use kurobako_problems::problems::{nasbench, sigopt};
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use yamakan::budget::Budget;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum BuiltinProblemSpec {
    Command(command::CommandProblemSpec),
    Sigopt(sigopt::SigoptProblemSpec),
    Nasbench(nasbench::NasbenchProblemSpec),
}
impl ProblemSpec for BuiltinProblemSpec {
    type Problem = BuiltinProblem;

    fn make_problem(&self) -> Result<Self::Problem> {
        match self {
            BuiltinProblemSpec::Command(p) => track!(p.make_problem().map(BuiltinProblem::Command)),
            BuiltinProblemSpec::Sigopt(p) => track!(p.make_problem().map(BuiltinProblem::Sigopt)),
            BuiltinProblemSpec::Nasbench(p) => {
                track!(p.make_problem().map(BuiltinProblem::Nasbench))
            }
        }
    }
}

#[derive(Debug)]
pub enum BuiltinProblem {
    Command(command::CommandProblem),
    Sigopt(sigopt::SigoptProblem),
    Nasbench(nasbench::NasbenchProblem),
}
impl Problem for BuiltinProblem {
    type Evaluator = BuiltinEvaluator;

    fn problem_space(&self) -> ProblemSpace {
        match self {
            BuiltinProblem::Command(p) => p.problem_space(),
            BuiltinProblem::Sigopt(p) => p.problem_space(),
            BuiltinProblem::Nasbench(p) => p.problem_space(),
        }
    }

    fn evaluation_cost(&self) -> u64 {
        match self {
            BuiltinProblem::Command(p) => p.evaluation_cost(),
            BuiltinProblem::Sigopt(p) => p.evaluation_cost(),
            BuiltinProblem::Nasbench(p) => p.evaluation_cost(),
        }
    }

    fn value_range(&self) -> MinMax<f64> {
        match self {
            BuiltinProblem::Command(p) => p.value_range(),
            BuiltinProblem::Sigopt(p) => p.value_range(),
            BuiltinProblem::Nasbench(p) => p.value_range(),
        }
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Result<Option<Self::Evaluator>> {
        match self {
            BuiltinProblem::Command(p) => track!(p
                .make_evaluator(params)
                .map(|t| t.map(BuiltinEvaluator::Command))),
            BuiltinProblem::Sigopt(p) => track!(p
                .make_evaluator(params)
                .map(|t| t.map(BuiltinEvaluator::Sigopt))),
            BuiltinProblem::Nasbench(p) => track!(p
                .make_evaluator(params)
                .map(|t| t.map(BuiltinEvaluator::Nasbench))),
        }
    }
}

#[derive(Debug)]
pub enum BuiltinEvaluator {
    Command(command::CommandEvaluator),
    Sigopt(sigopt::SigoptEvaluator),
    Nasbench(nasbench::NasbenchEvaluator),
}
impl Evaluate for BuiltinEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Result<f64> {
        match self {
            BuiltinEvaluator::Command(e) => track!(e.evaluate(budget)),
            BuiltinEvaluator::Sigopt(e) => track!(e.evaluate(budget)),
            BuiltinEvaluator::Nasbench(e) => track!(e.evaluate(budget)),
        }
    }
}
