use crate::{Evaluate, Problem, ProblemSpace, ProblemSpec, Result, ValueRange};
use kurobako_core::problems::command;
use kurobako_problems::problems::sigopt;
use yamakan::budget::Budget;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum BuiltinProblemSpec {
    Command(command::CommandProblemSpec),
    Sigopt(sigopt::SigoptProblemSpec),
}
impl ProblemSpec for BuiltinProblemSpec {
    type Problem = BuiltinProblem;

    fn make_problem(&self) -> Result<Self::Problem> {
        match self {
            BuiltinProblemSpec::Command(p) => track!(p.make_problem().map(BuiltinProblem::Command)),
            BuiltinProblemSpec::Sigopt(p) => track!(p.make_problem().map(BuiltinProblem::Sigopt)),
        }
    }
}

#[derive(Debug)]
pub enum BuiltinProblem {
    Command(command::CommandProblem),
    Sigopt(sigopt::SigoptProblem),
}
impl Problem for BuiltinProblem {
    type Evaluator = BuiltinEvaluator;

    fn problem_space(&self) -> ProblemSpace {
        match self {
            BuiltinProblem::Command(p) => p.problem_space(),
            BuiltinProblem::Sigopt(p) => p.problem_space(),
        }
    }

    fn evaluation_cost_hint(&self) -> usize {
        match self {
            BuiltinProblem::Command(p) => p.evaluation_cost_hint(),
            BuiltinProblem::Sigopt(p) => p.evaluation_cost_hint(),
        }
    }

    fn value_range(&self) -> ValueRange {
        match self {
            BuiltinProblem::Command(p) => p.value_range(),
            BuiltinProblem::Sigopt(p) => p.value_range(),
        }
    }

    fn make_evaluator(&mut self, params: &[f64]) -> Result<Self::Evaluator> {
        match self {
            BuiltinProblem::Command(p) => {
                track!(p.make_evaluator(params).map(BuiltinEvaluator::Command))
            }
            BuiltinProblem::Sigopt(p) => {
                track!(p.make_evaluator(params).map(BuiltinEvaluator::Sigopt))
            }
        }
    }
}

#[derive(Debug)]
pub enum BuiltinEvaluator {
    Command(command::CommandEvaluator),
    Sigopt(sigopt::SigoptEvaluator),
}
impl Evaluate for BuiltinEvaluator {
    fn evaluate(&mut self, budget: &mut Budget) -> Result<f64> {
        match self {
            BuiltinEvaluator::Command(e) => track!(e.evaluate(budget)),
            BuiltinEvaluator::Sigopt(e) => track!(e.evaluate(budget)),
        }
    }
}
