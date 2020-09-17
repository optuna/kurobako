//! A solver based on random search.
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{
    Capabilities, Solver, SolverFactory, SolverRecipe, SolverSpec, SolverSpecBuilder,
};
use kurobako_core::trial::{EvaluatedTrial, IdGen, NextTrial, Params};
use kurobako_core::{ErrorKind, Result};
use rand::distributions::Distribution as _;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !b
}

/// Recipe of `RandomSolver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct RandomSolverRecipe {
    /// If this flag is set, this solver asks evaluators to evaluate parameters at every intermediate step.
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "is_false")]
    ask_all_steps: bool,
}
impl SolverRecipe for RandomSolverRecipe {
    type Factory = RandomSolverFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        Ok(RandomSolverFactory {
            ask_all_steps: self.ask_all_steps,
        })
    }
}

/// Factory of `RandomSolver`.
#[derive(Debug)]
pub struct RandomSolverFactory {
    ask_all_steps: bool,
}
impl SolverFactory for RandomSolverFactory {
    type Solver = RandomSolver;

    fn specification(&self) -> Result<SolverSpec> {
        let spec = SolverSpecBuilder::new("Random")
            .attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            )
            .capabilities(Capabilities::all());
        Ok(spec.finish())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        Ok(RandomSolver {
            problem: problem.clone(),
            rng,
            current_step: if self.ask_all_steps { Some(0) } else { None },
        })
    }
}

/// Solver based on random search.
#[derive(Debug)]
pub struct RandomSolver {
    rng: ArcRng,
    problem: ProblemSpec,
    current_step: Option<u64>,
}
impl Solver for RandomSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        let mut params = Vec::new();
        for p in self.problem.params_domain.variables() {
            let param = p.sample(&mut self.rng);
            params.push(param);
        }

        let next_step = if let Some(current_step) = self.current_step {
            let step = self.problem.steps.iter().find(|&s| s > current_step);
            track_assert_some!(step, ErrorKind::Bug)
        } else {
            self.problem.steps.last()
        };
        Ok(NextTrial {
            id: idg.generate(),
            params: Params::new(params),
            next_step: Some(next_step),
        })
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        if let Some(step) = &mut self.current_step {
            *step = trial.current_step;
        }
        Ok(())
    }
}
