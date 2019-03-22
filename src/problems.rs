use crate::{Evaluate, Problem, ProblemSpace, ProblemSpec};
use failure::Fallible;
use yamakan::budget::Budget;

pub mod command;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub enum BuiltinProblemSpec {
    Command(command::CommandProblemSpec),
}
impl ProblemSpec for BuiltinProblemSpec {
    type Problem = BuiltinProblem;

    fn make_problem(&self) -> Fallible<Self::Problem> {
        match self {
            BuiltinProblemSpec::Command(p) => p.make_problem().map(BuiltinProblem::Command),
        }
    }
}

#[derive(Debug)]
pub enum BuiltinProblem {
    Command(command::CommandProblem),
}
impl Problem for BuiltinProblem {
    type Evaluator = BuiltinEvaluator;

    fn name(&self) -> &str {
        match self {
            BuiltinProblem::Command(p) => p.name(),
        }
    }

    fn problem_space(&self) -> ProblemSpace {
        match self {
            BuiltinProblem::Command(p) => p.problem_space(),
        }
    }

    fn evaluation_cost_hint(&self) -> usize {
        match self {
            BuiltinProblem::Command(p) => p.evaluation_cost_hint(),
        }
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Fallible<Self::Evaluator> {
        match self {
            BuiltinProblem::Command(p) => p.make_evaluator(params).map(BuiltinEvaluator::Command),
        }
    }
}

#[derive(Debug)]
pub enum BuiltinEvaluator {
    Command(command::CommandEvaluator),
}
impl Evaluate for BuiltinEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Fallible<f64> {
        match self {
            BuiltinEvaluator::Command(e) => e.evaluate(budget),
        }
    }
}
