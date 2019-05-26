use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{Distribution, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    ObservedObs, Solver, SolverCapabilities, SolverRecipe, SolverSpec, UnobservedObs,
};
use kurobako_core::{ErrorKind, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs};

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct RandomSolverRecipe {}
impl SolverRecipe for RandomSolverRecipe {
    type Solver = RandomSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        Ok(RandomSolver {
            params_domain: problem.params_domain,
            budget: Budget::new(problem.evaluation_expense.get()),
        })
    }
}

#[derive(Debug)]
pub struct RandomSolver {
    params_domain: Vec<ParamDomain>,
    budget: Budget,
}
impl Solver for RandomSolver {
    fn specification(&self) -> SolverSpec {
        SolverSpec {
            name: "random".to_owned(),
            version: Some(env!("CARGO_PKG_VERSION").to_owned()),
            capabilities: SolverCapabilities::empty()
                .categorical()
                .discrete()
                .multi_objective(),
        }
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        let mut params = Vec::new();
        for p in &self.params_domain {
            let v = match p {
                ParamDomain::Categorical(p) => {
                    ParamValue::Categorical(rng.gen_range(0, p.choices.len()))
                }
                ParamDomain::Conditional(_) => {
                    track_panic!(ErrorKind::Bug);
                }
                ParamDomain::Continuous(p) => {
                    track_assert_eq!(p.distribution, Distribution::Uniform, ErrorKind::Bug);

                    let n = rng.gen_range(p.range.low.get(), p.range.high.get());
                    ParamValue::Continuous(unsafe { FiniteF64::new_unchecked(n) })
                }
                ParamDomain::Discrete(p) => {
                    ParamValue::Discrete(rng.gen_range(p.range.low, p.range.high))
                }
            };
            params.push(v);
        }
        let obs = track!(Obs::new(idg, Budgeted::new(self.budget, params)))?;
        Ok(obs)
    }

    fn tell(&mut self, _obs: ObservedObs) -> Result<()> {
        Ok(())
    }
}
