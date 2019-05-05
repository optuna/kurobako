use kurobako_core::epi;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{Asked, ObservedObs, Solver, SolverRecipe, SolverSpec};
use kurobako_core::time::Elapsed;
use kurobako_core::Result;
use kurobako_solvers::random;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::observation::IdGen;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoSolverRecipe {
    Random(random::RandomSolverRecipe),
    Command(epi::solver::ExternalProgramSolverRecipe),
}
impl SolverRecipe for KurobakoSolverRecipe {
    type Solver = KurobakoSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        match self {
            KurobakoSolverRecipe::Random(r) => {
                track!(r.create_solver(problem)).map(KurobakoSolver::Random)
            }
            KurobakoSolverRecipe::Command(r) => {
                track!(r.create_solver(problem)).map(KurobakoSolver::Command)
            }
        }
    }
}

#[derive(Debug)]
pub enum KurobakoSolver {
    Random(random::RandomSolver),
    Command(epi::solver::ExternalProgramSolver),
}
impl Solver for KurobakoSolver {
    fn specification(&self) -> SolverSpec {
        match self {
            KurobakoSolver::Random(s) => s.specification(),
            KurobakoSolver::Command(s) => s.specification(),
        }
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<Asked> {
        match self {
            KurobakoSolver::Random(s) => track!(s.ask(rng, idg)),
            KurobakoSolver::Command(s) => track!(s.ask(rng, idg)),
        }
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<Elapsed> {
        match self {
            KurobakoSolver::Random(s) => track!(s.tell(obs)),
            KurobakoSolver::Command(s) => track!(s.tell(obs)),
        }
    }
}
