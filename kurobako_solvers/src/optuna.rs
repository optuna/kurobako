use kurobako_core::epi::solver::{EmbeddedScriptSolver, EmbeddedScriptSolverRecipe};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{Asked, ObservedObs, Solver, SolverRecipe, SolverSpec};
use kurobako_core::time::Elapsed;
use kurobako_core::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use structopt::StructOpt;
use yamakan::observation::IdGen;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct OptunaSolverRecipe {
    // TODO: tpe options, asha/median options, loglevel
    #[structopt(long)]
    pub python: Option<PathBuf>,
}
impl SolverRecipe for OptunaSolverRecipe {
    type Solver = OptunaSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let script = include_str!("../contrib/optuna_solver.py");
        let args = vec![];
        let recipe = EmbeddedScriptSolverRecipe {
            script: script.to_owned(),
            args,
            interpreter: self.python.clone(),
            interpreter_args: Vec::new(),
        };
        let inner = track!(recipe.create_solver(problem))?;
        Ok(OptunaSolver(inner))
    }
}

#[derive(Debug)]
pub struct OptunaSolver(EmbeddedScriptSolver);
impl Solver for OptunaSolver {
    fn specification(&self) -> SolverSpec {
        self.0.specification()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<Asked> {
        track!(self.0.ask(rng, idg))
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<Elapsed> {
        track!(self.0.tell(obs))
    }
}
