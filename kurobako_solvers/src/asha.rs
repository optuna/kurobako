use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{Distribution, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    ObservedObs, Solver, SolverCapabilities, SolverRecipe, SolverRecipePlaceHolder, SolverSpec,
    UnobservedObs,
};
use kurobako_core::{ErrorKind, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs};

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct AshaSolverRecipe {
    #[structopt(long, default_value = "0.01")]
    pub finish_rate: f64,

    #[structopt(long, default_value = "2")]
    pub reduction_factor: usize,

    pub base_solver: SolverRecipePlaceHolder,
}
impl SolverRecipe for AshaSolverRecipe {
    type Solver = AshaSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let base = track!(self.base_solver.create_solver(problem))?;
        Ok(AshaSolver {})
    }
}

#[derive(Debug)]
pub struct AshaSolver {}
impl Solver for AshaSolver {
    fn specification(&self) -> SolverSpec {
        // let mut spec = self.base.specification();
        // spec.name = format!("ASHA/{}", spec.name);
        // spec
        panic!()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        panic!()
    }

    fn tell(&mut self, _obs: ObservedObs) -> Result<()> {
        panic!()
    }
}
