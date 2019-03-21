use crate::{Problem, ProblemSpace, ProblemSpec};
use failure::Fallible;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub struct CommandProblemSpec {}
impl ProblemSpec for CommandProblemSpec {
    type Problem = CommandProblem;

    fn build(&self, params: &[f64]) -> Fallible<Self::Problem> {
        panic!()
    }
}

#[derive(Debug)]
pub struct CommandProblem {
    name: String,
}
impl Problem for CommandProblem {
    fn name(&self) -> &str {
        &self.name
    }

    fn problem_space(&self) -> ProblemSpace {
        panic!()
    }
}
