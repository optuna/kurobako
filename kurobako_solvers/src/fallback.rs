use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::{Continuous, Discrete, Distribution, ParamDomain, ParamValue};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{
    ObservedObs, Solver, SolverCapabilities, SolverCapability, SolverRecipe, SolverSpec,
    UnobservedObs,
};
use kurobako_core::{ErrorKind, Result};
use rand::{self, Rng};
use rustats::range::Range;
use std::collections::HashMap;
use yamakan::budget::{Budget, Budgeted};
use yamakan::observation::{IdGen, Obs, ObsId};

#[derive(Debug)]
pub struct FallbackSolver<T> {
    inner: T,
    adaptors: Vec<Adaptor>,
    obss: HashMap<ObsId, Vec<ParamValue>>,
}
impl<T: Solver> FallbackSolver<T> {
    pub fn new<R>(recipe: R, problem: ProblemSpec) -> Result<Self>
    where
        R: SolverRecipe<Solver = T>,
    {
        let required_capabilities = problem.required_solver_capabilities();
        let mut inner = track!(recipe.create_solver(problem.clone()))?;
        let actual_capabilities = inner.specification().capabilities;

        let mut adaptors = Vec::new();
        let incapables = actual_capabilities.incapables(&required_capabilities);
        if !incapables.is_empty() {
            debug!("Incapables: {:?}", incapables);

            let adjuster = ProblemSpecAdjuster::new(incapables, &problem);
            let adjusted = track!(adjuster.adjust())?;
            inner = track!(recipe.create_solver(adjusted.0))?;
            adaptors = adjusted.1;
        }

        Ok(Self {
            inner,
            adaptors,
            obss: HashMap::new(),
        })
    }
}
impl<T: Solver> Solver for FallbackSolver<T> {
    fn specification(&self) -> SolverSpec {
        let mut spec = self.inner.specification();
        spec.capabilities = SolverCapabilities::all();
        spec
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        panic!()
    }

    fn tell(&mut self, _obs: ObservedObs) -> Result<()> {
        panic!()
    }
}

#[derive(Debug)]
struct ProblemSpecAdjuster {
    incapables: SolverCapabilities,
    problem: ProblemSpec,
    base_problem: ProblemSpec,
    adaptors: Vec<Adaptor>,
}
impl ProblemSpecAdjuster {
    fn new(incapables: SolverCapabilities, base_problem: &ProblemSpec) -> Self {
        Self {
            incapables,
            problem: ProblemSpec {
                params_domain: Vec::new(),
                values_domain: Vec::new(),
                ..base_problem.clone()
            },
            base_problem: base_problem.clone(),
            adaptors: Vec::new(),
        }
    }

    fn adjust(mut self) -> Result<(ProblemSpec, Vec<Adaptor>)> {
        if self.incapables.contains(SolverCapability::MultiObjective) {
            track_panic!(ErrorKind::Other, "Unimplemented: {:?}");
        } else {
            self.problem.values_domain = self.base_problem.values_domain.clone();
        }

        for param_domain in self.base_problem.params_domain.clone() {
            self.adjust_param_domain(&param_domain);
        }

        Ok((self.problem, self.adaptors))
    }

    fn adjust_param_domain(&mut self, param_domain: &ParamDomain) {
        match param_domain {
            ParamDomain::Continuous(p)
                if p.distribution == Distribution::LogUniform
                    && self.incapables.contains(SolverCapability::LogUniform) =>
            {
                panic!()
            }
            ParamDomain::Discrete(p) if self.incapables.contains(SolverCapability::Discrete) => {
                let adaptor = DiscreteAdaptor::new(p);
                self.problem
                    .params_domain
                    .push(ParamDomain::Continuous(adaptor.to.clone()));
                self.adaptors.push(Adaptor::Discrete(adaptor));
            }
            ParamDomain::Categorical(_)
                if self.incapables.contains(SolverCapability::Categorical) =>
            {
                if self.incapables.contains(SolverCapability::Discrete) {
                } else {
                }
                panic!()
            }
            ParamDomain::Conditional(_) => panic!(),
            _ => {
                self.adaptors.push(Adaptor::Noop);
                self.problem.params_domain.push(param_domain.clone());
            }
        }
    }
}

#[derive(Debug)]
enum Adaptor {
    Noop,
    Discrete(DiscreteAdaptor),
}

#[derive(Debug)]
struct DiscreteAdaptor {
    from: Discrete,
    to: Continuous,
}
impl DiscreteAdaptor {
    fn new(from: &Discrete) -> Self {
        Self {
            from: from.clone(),
            to: Continuous {
                name: from.name.clone(),
                range: unsafe {
                    Range::new(
                        FiniteF64::new_unchecked(from.range.low as f64),
                        FiniteF64::new_unchecked(from.range.high as f64),
                    )
                    .unwrap_or_else(|e| unreachable!("{}", e))
                },
                distribution: Distribution::Uniform,
            },
        }
    }

    fn backword(&self, value: ParamValue) -> Result<Vec<ParamValue>> {
        if let ParamValue::Continuous(v) = value {
            Ok(vec![ParamValue::Discrete(v.get().floor() as i64)])
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }

    fn forward(&self, value: ParamValue) -> Result<Vec<ParamValue>> {
        if let ParamValue::Discrete(v) = value {
            let n = v as f64 + rand::thread_rng().gen_range(0.0, 1.0);
            let n = track!(FiniteF64::new(n))?;
            Ok(vec![ParamValue::Continuous(n)])
        } else {
            track_panic!(ErrorKind::Bug);
        }
    }
}
