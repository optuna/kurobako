//! The solver for `kurobako`.
use kurobako_core::epi;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{BoxSolver, BoxSolverFactory, SolverFactory, SolverRecipe, SolverSpec};
use kurobako_core::Result;
use kurobako_solvers::{asha, optuna, random};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Solver recipe.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct KurobakoSolverRecipe {
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[structopt(flatten)]
    #[serde(flatten)]
    inner: InnerRecipe,
}
impl SolverRecipe for KurobakoSolverRecipe {
    type Factory = KurobakoSolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let inner = track!(self.inner.create_factory(registry))?;
        Ok(KurobakoSolverFactory {
            name: self.name.clone(),
            inner,
        })
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
enum InnerRecipe {
    Command(epi::solver::ExternalProgramSolverRecipe),
    Random(random::RandomSolverRecipe),
    Asha(asha::AshaSolverRecipe),
    Optuna(optuna::OptunaSolverRecipe),
}
impl SolverRecipe for InnerRecipe {
    type Factory = BoxSolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        match self {
            Self::Random(r) => track!(r.create_factory(registry)).map(BoxSolverFactory::new),
            Self::Optuna(r) => track!(r.create_factory(registry)).map(BoxSolverFactory::new),
            Self::Asha(r) => track!(r.create_factory(registry)).map(BoxSolverFactory::new),
            Self::Command(r) => track!(r.create_factory(registry)).map(BoxSolverFactory::new),
        }
    }
}

/// Solver factory.
#[derive(Debug)]
pub struct KurobakoSolverFactory {
    name: Option<String>,
    inner: BoxSolverFactory,
}
impl SolverFactory for KurobakoSolverFactory {
    type Solver = BoxSolver;

    fn specification(&self) -> Result<SolverSpec> {
        let mut spec = track!(self.inner.specification())?;
        if let Some(name) = &self.name {
            spec.name = name.clone();
        }
        Ok(spec)
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        track!(self.inner.create_solver(rng, problem)).map(BoxSolver::new)
    }
}
