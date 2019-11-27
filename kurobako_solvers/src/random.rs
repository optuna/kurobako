//! A solver based on random search.
use kurobako_core::domain::{Distribution, Range};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::{ArcRng, Rng};
use kurobako_core::solver::{
    Capabilities, Solver, SolverFactory, SolverRecipe, SolverSpec, SolverSpecBuilder,
};
use kurobako_core::trial::{AskedTrial, EvaluatedTrial, IdGen, Params};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Recipe of `RandomSolver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct RandomSolverRecipe {}
impl SolverRecipe for RandomSolverRecipe {
    type Factory = RandomSolverFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        Ok(RandomSolverFactory {})
    }
}

/// Factory of `RandomSolver`.
#[derive(Debug)]
pub struct RandomSolverFactory {}
impl SolverFactory for RandomSolverFactory {
    type Solver = RandomSolver;

    fn specification(&self) -> Result<SolverSpec> {
        let spec = SolverSpecBuilder::new("Random")
            .attr(
                "version",
                &format!("kurobako={}", env!("CARGO_PKG_VERSION")),
            )
            .capabilities(Capabilities::all());
        Ok(spec.finish())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        Ok(RandomSolver {
            problem: problem.clone(),
            rng,
        })
    }
}

/// Solver based on random search.
#[derive(Debug)]
pub struct RandomSolver {
    rng: ArcRng,
    problem: ProblemSpec,
}
impl Solver for RandomSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<AskedTrial> {
        let mut params = Vec::new();
        for p in self.problem.params_domain.variables() {
            let param = match p.range() {
                Range::Continuous { low, high } => match p.distribution() {
                    Distribution::Uniform => self.rng.gen_range(low, high),
                    Distribution::LogUniform => self.rng.gen_range(low.log2(), high.log2()).exp2(),
                },
                Range::Discrete { low, high } => match p.distribution() {
                    Distribution::Uniform => self.rng.gen_range(low, high) as f64,
                    Distribution::LogUniform => self
                        .rng
                        .gen_range((*low as f64).log2(), (*high as f64).log2())
                        .exp2()
                        .floor(),
                },
                Range::Categorical { choices } => self.rng.gen_range(0, choices.len()) as f64,
            };
            params.push(param);
        }

        Ok(AskedTrial {
            id: idg.generate(),
            params: Params::new(params),
            next_step: Some(self.problem.steps.last()),
        })
    }

    fn tell(&mut self, _trial: EvaluatedTrial) -> Result<()> {
        Ok(())
    }
}
