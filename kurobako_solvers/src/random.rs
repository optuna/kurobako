use kurobako_core::parameter::{Distribution, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    Asked, ObservedObs, Solver, SolverCapabilities, SolverRecipe, SolverSpec,
};
use kurobako_core::time::Elapsed;
use kurobako_core::Result;
use rand::Rng;
use rustats::num::FiniteF64;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
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
            capabilities: SolverCapabilities::all(),
        }
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<Asked> {
        let (obs, elapsed) = Elapsed::time(|| {
            let params = self
                .params_domain
                .iter()
                .map(|p| match p {
                    ParamDomain::Categorical(p) => {
                        ParamValue::Categorical(rng.gen_range(0, p.choices.len()))
                    }
                    ParamDomain::Conditional(p) => unimplemented!("Conditional: {:?}", p),
                    ParamDomain::Continuous(p) => {
                        assert_eq!(p.distribution, Distribution::LogUniform, "Unimplememented");

                        let n = rng.gen_range(p.range.low.get(), p.range.high.get());
                        ParamValue::Continuous(unsafe { FiniteF64::new_unchecked(n) })
                    }
                    ParamDomain::Discrete(p) => {
                        ParamValue::Discrete(rng.gen_range(p.range.low, p.range.high))
                    }
                })
                .collect();
            track!(Obs::new(idg, Budgeted::new(self.budget, params)))
        });
        Ok(Asked { obs: obs?, elapsed })
    }

    fn tell(&mut self, _obs: ObservedObs) -> Result<Elapsed> {
        Ok(Elapsed::zero())
    }
}
