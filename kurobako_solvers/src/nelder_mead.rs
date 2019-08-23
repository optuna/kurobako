use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{Distribution, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    ObservedObs, Solver, SolverCapabilities, SolverRecipe, SolverSpec, UnobservedObs,
};
use kurobako_core::{ErrorKind, Result};
use rand::{self, Rng};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::IdGen;
use yamakan::optimizers::nelder_mead::NelderMeadOptimizer;
use yamakan::parameters::F64;
use yamakan::Optimizer as _;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub struct NelderMeadSolverRecipe {}
impl SolverRecipe for NelderMeadSolverRecipe {
    type Solver = NelderMeadSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let mut rng = rand::thread_rng(); // TODO
        let param_space = problem
            .params_domain
            .iter()
            .map(|p| {
                if let ParamDomain::Continuous(c) = p {
                    track_assert_eq!(
                        c.distribution,
                        Distribution::Uniform,
                        ErrorKind::InvalidInput
                    );
                    Ok(F64::from(c.range))
                } else {
                    track_panic!(ErrorKind::InvalidInput, "Unsupported: {:?}", p)
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let optimizer = track!(NelderMeadOptimizer::new(param_space, &mut rng))?;
        let budget = Budget::new(problem.evaluation_expense.get());
        Ok(NelderMeadSolver { optimizer, budget })
    }
}

#[derive(Debug)]
pub struct NelderMeadSolver {
    optimizer: NelderMeadOptimizer<F64, Vec<FiniteF64>>,
    budget: Budget,
}
impl Solver for NelderMeadSolver {
    fn specification(&self) -> SolverSpec {
        SolverSpec {
            name: "NelderMead".to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            capabilities: SolverCapabilities::empty(),
        }
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        let obs = track!(self.optimizer.ask(rng, idg))?;
        let obs = obs.map_param(|p| {
            Budgeted::new(
                self.budget,
                p.into_iter()
                    .map(|p| unsafe { ParamValue::Continuous(FiniteF64::new_unchecked(p)) })
                    .collect(),
            )
        });
        Ok(obs)
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        let obs = obs.map_param(|p| {
            p.into_inner()
                .into_iter()
                .map(|p| p.to_f64())
                .collect::<Vec<_>>()
        });
        track!(self.optimizer.tell(obs))?;
        Ok(())
    }
}
