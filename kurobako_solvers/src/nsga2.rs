//! A solver based on nsga2 search.
use crate::error::from_yamakan;
use crate::yamakan_utils::{KurobakoDomain, YamakanIdGen};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{
    Capabilities, Capability, Solver, SolverFactory, SolverRecipe, SolverSpec, SolverSpecBuilder,
};
use kurobako_core::trial::{EvaluatedTrial, IdGen, NextTrial, Params, TrialId};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use structopt::StructOpt;
use yamakan::domains::VecDomain;
use yamakan::optimizers::nsga2;
use yamakan::{Obs, ObsId, Optimizer};

type Nsga2Strategy = nsga2::Nsga2Strategy<
    VecDomain<KurobakoDomain>,
    nsga2::RandomGenerator,
    nsga2::TournamentSelector,
    nsga2::ExchangeVec,
    nsga2::ReplaceVec,
>;

type Nsga2Optimizer = nsga2::Nsga2Optimizer<VecDomain<KurobakoDomain>, Nsga2Strategy>;

/// Recipe of `Nsga2Solver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct Nsga2SolverRecipe {
    /// Population size.
    #[structopt(long, default_value = "20")]
    population: usize,

    /// Tournament size.
    #[structopt(long, default_value = "2")]
    tournament: usize,

    /// Cossover probability of each parameter.
    #[structopt(long, default_value = "0.5")]
    crossover: f64,

    /// Mutation probability of each parameter.
    #[structopt(long, default_value = "0.3")]
    mutation: f64,
}
impl SolverRecipe for Nsga2SolverRecipe {
    type Factory = Nsga2SolverFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        Ok(Nsga2SolverFactory {
            population: self.population,
            tournament: self.tournament,
            crossover: self.crossover,
            mutation: self.mutation,
        })
    }
}

/// Factory of `Nsga2Solver`.
#[derive(Debug)]
pub struct Nsga2SolverFactory {
    population: usize,
    tournament: usize,
    crossover: f64,
    mutation: f64,
}
impl SolverFactory for Nsga2SolverFactory {
    type Solver = Nsga2Solver;

    fn specification(&self) -> Result<SolverSpec> {
        let spec = SolverSpecBuilder::new("NSGA-II")
            .attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            )
            .capabilities(
                Capabilities::all()
                    .remove_capability(Capability::Conditional)
                    .clone(),
            );
        Ok(spec.finish())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let params_domain = problem
            .params_domain
            .variables()
            .iter()
            .map(|v| KurobakoDomain::new(v.range().clone(), v.distribution()))
            .collect();
        let params_domain = VecDomain(params_domain);
        let selector =
            track!(nsga2::TournamentSelector::new(self.tournament).map_err(from_yamakan))?;
        let crossover = track!(nsga2::ExchangeVec::new(self.crossover).map_err(from_yamakan))?;
        let mutator = track!(nsga2::ReplaceVec::new(self.mutation).map_err(from_yamakan))?;
        let strategy = Nsga2Strategy::new(nsga2::RandomGenerator, selector, crossover, mutator);
        let optimizer = track!(
            Nsga2Optimizer::new(params_domain, self.population, strategy).map_err(from_yamakan)
        )?;

        Ok(Nsga2Solver {
            problem: problem.clone(),
            rng,
            optimizer,
            evaluatings: HashMap::new(),
        })
    }
}

/// Solver based on nsga2 search.
#[derive(Debug)]
pub struct Nsga2Solver {
    rng: ArcRng,
    problem: ProblemSpec,
    optimizer: Nsga2Optimizer,
    evaluatings: HashMap<TrialId, Vec<f64>>,
}
impl Solver for Nsga2Solver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        let mut idg = YamakanIdGen(idg);
        let obs = track!(self
            .optimizer
            .ask(&mut self.rng, &mut idg)
            .map_err(from_yamakan))?;

        let trial = NextTrial {
            id: TrialId::new(obs.id.get()),
            params: Params::new(obs.param.clone()),
            next_step: Some(self.problem.steps.last()),
        };
        self.evaluatings.insert(trial.id, obs.param);

        Ok(trial)
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        let param = track_assert_some!(self.evaluatings.remove(&trial.id), ErrorKind::InvalidInput);
        let obs = Obs {
            id: ObsId::new(trial.id.get()),
            param,
            value: trial.values.into_vec(),
        };
        if !obs.value.is_empty() {
            track!(self.optimizer.tell(obs).map_err(from_yamakan))
        } else {
            // Unevaluable params.
            Ok(())
        }
    }
}
