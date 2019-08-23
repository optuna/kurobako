use crate::filter::KurobakoFilterRecipe;
use kurobako_core::epi;
use kurobako_core::filter::{BoxFilter, Filter as _, FilterRecipe as _};
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

    #[structopt(long, parse(try_from_str = "json::parse_json"))]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    filters: Vec<KurobakoFilterRecipe>,

    #[structopt(long)]
    #[serde(default, skip_serializing)]
    filters_end: bool,

    #[structopt(flatten)]
    #[serde(flatten)]
    inner: InnerRecipe,
}
impl SolverRecipe for KurobakoSolverRecipe {
    type Solver = KurobakoSolver;

    fn create_solver(&self, mut problem: ProblemSpec) -> Result<Self::Solver> {
        let mut filters = self
            .filters
            .iter()
            .map(|r| track!(r.create_filter()))
            .collect::<Result<Vec<_>>>()?;
        for f in &mut filters {
            track!(f.filter_problem_spec(&mut problem))?;
        }

        let inner = track!(self.inner.create_solver(problem))?;
        Ok(KurobakoSolver {
            tag: self.tag.clone(),
            inner,
            filters,
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
    filters: Vec<BoxFilter>,
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
        let mut obs = track!(self.inner.ask(rng, idg))?;
        for f in &mut self.filters {
            track!(f.filter_ask(rng, &mut obs))?;
        }
        Ok(obs)
    }

    fn tell(&mut self, mut obs: ObservedObs) -> Result<()> {
        let mut rng = rand::thread_rng(); // TODO
        for f in &mut self.filters {
            track!(f.filter_tell(&mut rng, &mut obs))?;
        }
        track!(self.inner.tell(obs))
    }
}
