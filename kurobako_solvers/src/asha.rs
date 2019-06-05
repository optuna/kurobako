use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    BoxSolver, ObservedObs, Solver, SolverCapability, SolverRecipe, SolverRecipePlaceHolder,
    SolverSpec, UnobservedObs, YamakanSolver,
};
use kurobako_core::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::observation::IdGen;
use yamakan::optimizers::asha::{AshaOptimizer, AshaOptimizerBuilder};
use yamakan::Optimizer as _;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct AshaSolverRecipe {
    #[structopt(long, default_value = "0.01")]
    pub finish_rate: f64,

    // TODO: integratio with finish_rate
    #[structopt(long)]
    pub min_resource: Option<u64>,

    #[structopt(long, default_value = "2")]
    pub reduction_factor: usize,

    #[structopt(long)]
    pub without_checkpoint: bool,

    pub base_solver: SolverRecipePlaceHolder,
}
impl SolverRecipe for AshaSolverRecipe {
    type Solver = AshaSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let max_budget = problem.evaluation_expense.get();
        let mut min_budget = max_budget;
        while min_budget > 1 {
            min_budget = ((min_budget as f64) / (self.reduction_factor as f64)).ceil() as u64;
            if (min_budget as f64) / (max_budget as f64) < self.finish_rate {
                break;
            }
        }
        if let Some(min) = self.min_resource {
            min_budget = min;
        }
        debug!(
            "ASHA options: min_budget={}, max_budget={}, reduction_factor={}",
            min_budget, max_budget, self.reduction_factor
        );

        let base = track!(self.base_solver.create_solver(problem))?;

        let mut builder = AshaOptimizerBuilder::new();
        track!(builder.reduction_factor(self.reduction_factor))?;
        if self.without_checkpoint {
            builder.without_checkpoint();
        }
        let optimizer = track!(builder.finish(YamakanSolver::new(base), min_budget, max_budget))?;

        Ok(AshaSolver { optimizer })
    }
}

#[derive(Debug)]
pub struct AshaSolver {
    optimizer: AshaOptimizer<YamakanSolver<BoxSolver>, Vec<ParamValue>>,
}
impl Solver for AshaSolver {
    fn specification(&self) -> SolverSpec {
        let mut spec = self.optimizer.inner().inner().specification();
        spec.name = format!("ASHA/{}", spec.name);
        spec.capabilities.remove(SolverCapability::MultiObjective);
        spec
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        let obs = track!(self.optimizer.ask(rng, idg))?;
        Ok(obs)
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        track!(self.optimizer.tell(obs))?;
        Ok(())
    }
}
