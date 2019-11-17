use crate::filter::KurobakoFilterRecipe;
use kurobako_core::epi;
use kurobako_core::json;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    BoxSolver, BoxSolverRecipe, ObservedObs, Solver, SolverRecipe, SolverSpec, UnobservedObs,
};
use kurobako_core::{Error, Result};
use kurobako_solvers::{asha, nelder_mead, optuna, random};
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use serde_json;
use structopt::StructOpt;
use yamakan::observation::IdGen;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct KurobakoSolverRecipe {
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    tag: Option<String>,

    #[structopt(flatten)]
    #[serde(flatten)]
    inner: InnerRecipe,
}
impl SolverRecipe for KurobakoSolverRecipe {
    type Solver = KurobakoSolver;

    fn create_solver(&self, mut problem: ProblemSpec) -> Result<Self::Solver> {
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
    Asha(asha::AshaSolverRecipe),
    NelderMead(nelder_mead::NelderMeadSolverRecipe),
    Command(epi::solver::ExternalProgramSolverRecipe),
}
impl SolverRecipe for InnerRecipe {
    type Solver = BoxSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        match self {
            InnerRecipe::Random(r) => track!(r.create_solver(problem)).map(BoxSolver::new),
            InnerRecipe::Optuna(r) => track!(r.create_solver(problem)).map(BoxSolver::new),
            InnerRecipe::Asha(r) => {
                let mut r = r.clone();
                track!(r.base_solver.set_recipe(|json| {
                    let recipe: KurobakoSolverRecipe =
                        track!(serde_json::from_value(json.get().clone()).map_err(Error::from))?;
                    Ok(BoxSolverRecipe::new(recipe))
                }))?;
                track!(r.create_solver(problem)).map(BoxSolver::new)
            }
            InnerRecipe::NelderMead(r) => track!(r.create_solver(problem)).map(BoxSolver::new),
            InnerRecipe::Command(r) => track!(r.create_solver(problem)).map(BoxSolver::new),
        }
    }
}

#[derive(Debug)]
pub struct KurobakoSolver {
    tag: Option<String>,
    inner: BoxSolver,
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

    fn tell(&mut self, mut obs: ObservedObs) -> Result<()> {
        track!(self.inner.tell(obs))
    }
}
