use crate::filter::KurobakoFilterRecipe;
use kurobako_core::epi;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{ObservedObs, Solver, SolverRecipe, SolverSpec, UnobservedObs};
use kurobako_core::Result;
use kurobako_solvers::{optuna, random};
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::observation::IdGen;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct KurobakoSolverRecipe {
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    #[structopt(long, parse(try_from_str = "crate::json::parse_json"))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    filters: Vec<KurobakoFilterRecipe>,

    #[structopt(long)]
    #[serde(default, skip_serializing)]
    filter_end: bool,

    #[structopt(flatten)]
    #[serde(flatten)]
    inner: InnerRecipe,
}
impl SolverRecipe for KurobakoSolverRecipe {
    type Solver = KurobakoSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let inner = track!(self.inner.create_solver(problem))?;
        Ok(KurobakoSolver {
            tag: self.tag.clone(),
            inner,
        })
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
enum InnerRecipe {
    Random(random::RandomSolverRecipe),
    Optuna(optuna::OptunaSolverRecipe),
    Command(epi::solver::ExternalProgramSolverRecipe),
}
impl SolverRecipe for InnerRecipe {
    type Solver = InnerSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        match self {
            InnerRecipe::Random(r) => track!(r.create_solver(problem)).map(InnerSolver::Random),
            InnerRecipe::Optuna(r) => track!(r.create_solver(problem)).map(InnerSolver::Optuna),
            InnerRecipe::Command(r) => track!(r.create_solver(problem)).map(InnerSolver::Command),
        }
    }
}

#[derive(Debug)]
pub struct KurobakoSolver {
    tag: Option<String>,
    inner: InnerSolver,
}
impl Solver for KurobakoSolver {
    fn specification(&self) -> SolverSpec {
        let mut spec = self.inner.specification();
        if let Some(tag) = &self.tag {
            spec.name.push_str(&format!("#{}", tag));
        }
        spec
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        track!(self.inner.ask(rng, idg))
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        track!(self.inner.tell(obs))
    }
}

#[derive(Debug)]
pub enum InnerSolver {
    Random(random::RandomSolver),
    Optuna(optuna::OptunaSolver),
    Command(epi::solver::ExternalProgramSolver),
}
impl Solver for InnerSolver {
    fn specification(&self) -> SolverSpec {
        match self {
            InnerSolver::Random(s) => s.specification(),
            InnerSolver::Optuna(s) => s.specification(),
            InnerSolver::Command(s) => s.specification(),
        }
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        match self {
            InnerSolver::Random(s) => track!(s.ask(rng, idg)),
            InnerSolver::Optuna(s) => track!(s.ask(rng, idg)),
            InnerSolver::Command(s) => track!(s.ask(rng, idg)),
        }
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        match self {
            InnerSolver::Random(s) => track!(s.tell(obs)),
            InnerSolver::Optuna(s) => track!(s.tell(obs)),
            InnerSolver::Command(s) => track!(s.tell(obs)),
        }
    }
}
