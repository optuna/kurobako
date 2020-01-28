//! A solver based on [**A**synchronous **S**uccessive **H**alving **A**lgorithm][ASHA].
//!
//! [ASHA]: https://arxiv.org/abs/1810.05934
use crate::error::{from_yamakan, into_yamakan};
use kurobako_core::json::JsonRecipe;
use kurobako_core::num::OrderedFloat;
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::{ArcRng, Rng};
use kurobako_core::solver::{
    BoxSolver, BoxSolverFactory, Capability, Solver, SolverFactory, SolverRecipe, SolverSpec,
    SolverSpecBuilder,
};
use kurobako_core::trial::{EvaluatedTrial, IdGen, NextTrial, TrialId, Values};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::f64;
use structopt::StructOpt;
use yamakan::optimizers::asha::{AshaOptimizer, AshaOptimizerBuilder};
use yamakan::{self, Budget, MfObs, MultiFidelityOptimizer, Obs, ObsId, Optimizer, Ranked};

/// Recipe of `AshaSolver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct AshaSolverRecipe {
    /// Rate to determine the value of `min_step`.
    ///
    /// The value of `min_step` will be set to `problem.steps.last() * min_step_rate`.
    /// If `min_step` is given, this field is ignored.
    #[structopt(long, default_value = "0.01")]
    pub min_step_rate: f64,

    /// Minimum resource parameter of AHSA.
    #[structopt(long)]
    pub min_step: Option<u64>,

    /// Reduction factor parameter of ASHA.
    #[structopt(long, default_value = "2")]
    pub reduction_factor: usize,

    /// If this flag is set, ASHA assumes that problems don't support checkpointing.
    #[structopt(long)]
    pub without_checkpoint: bool,

    /// Recipe of the base solver.
    pub base_solver: JsonRecipe,
}
impl SolverRecipe for AshaSolverRecipe {
    type Factory = AshaSolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let base = track!(registry.create_solver_factory_from_json(&self.base_solver))?;
        Ok(AshaSolverFactory {
            min_step_rate: self.min_step_rate,
            min_step: self.min_step,
            reduction_factor: self.reduction_factor,
            without_checkpoint: self.without_checkpoint,
            base,
        })
    }
}

/// Factory of `AshaSolver`.
#[derive(Debug)]
pub struct AshaSolverFactory {
    min_step_rate: f64,
    min_step: Option<u64>,
    reduction_factor: usize,
    without_checkpoint: bool,
    base: BoxSolverFactory,
}
impl SolverFactory for AshaSolverFactory {
    type Solver = AshaSolver;

    fn specification(&self) -> Result<SolverSpec> {
        let mut base = track!(self.base.specification())?;
        base.capabilities
            .remove_capability(Capability::MultiObjective);

        let spec = SolverSpecBuilder::new(&format!("ASHA with {}", base.name))
            .attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            )
            .attr(
                "paper",
                "Li, Liam, et al. \"Massively parallel hyperparameter tuning.\" \
                 arXiv preprint arXiv:1810.05934 (2018).",
            )
            .capabilities(base.capabilities);
        Ok(spec.finish())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let max_budget = problem.steps.last();
        let min_budget = if let Some(v) = self.min_step {
            v
        } else {
            (max_budget as f64 * self.min_step_rate) as u64
        };

        let base = track!(self.base.create_solver(rng.clone(), problem))?;

        let mut builder = AshaOptimizerBuilder::new();
        track!(builder
            .reduction_factor(self.reduction_factor)
            .map_err(from_yamakan))?;
        if self.without_checkpoint {
            builder.without_checkpoint();
        }
        let optimizer = track!(builder
            .finish(BaseOptimizer::new(max_budget, base), min_budget, max_budget)
            .map_err(from_yamakan))?;

        Ok(AshaSolver {
            optimizer,
            rng,
            trials: HashMap::new(),
            max_budget,
        })
    }
}

/// A solver based on [**A**synchronous **S**uccessive **H**alving **A**lgorithm][ASHA].
///
/// [ASHA]: https://arxiv.org/abs/1810.05934
#[derive(Debug)]
pub struct AshaSolver {
    optimizer: AshaOptimizer<OrderedFloat<f64>, BaseOptimizer>,
    rng: ArcRng,
    trials: HashMap<TrialId, NextTrial>,
    max_budget: u64,
}
impl Solver for AshaSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        let mut idg = YamakanIdGen(idg);
        let obs = track!(self
            .optimizer
            .ask(&mut self.rng, &mut idg)
            .map_err(from_yamakan))?;

        let mut trial = obs.param.clone();
        trial.id = TrialId::new(obs.id.get());

        self.trials.insert(trial.id, obs.param);
        Ok(trial)
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        let param = track_assert_some!(self.trials.remove(&trial.id), ErrorKind::Bug);
        let value = if trial.values.is_empty() {
            OrderedFloat(f64::NAN)
        } else {
            OrderedFloat(trial.values[0])
        };

        let obs = MfObs {
            id: ObsId::new(trial.id.get()),
            budget: Budget {
                amount: self.max_budget,
                consumption: trial.current_step,
            },
            param,
            value,
        };
        track!(self.optimizer.tell(obs).map_err(from_yamakan))
    }
}

#[derive(Debug)]
struct YamakanIdGen<'a>(&'a mut IdGen);
impl<'a> yamakan::IdGen for YamakanIdGen<'a> {
    fn generate(&mut self) -> Result<ObsId, yamakan::Error> {
        Ok(ObsId::new(self.0.generate().get()))
    }
}

#[derive(Debug)]
struct BaseOptimizer {
    max_budget: u64,
    solver: BoxSolver,
    idg: IdGen,
    idmap: HashMap<TrialId, ObsId>,
}
impl BaseOptimizer {
    fn new(max_budget: u64, solver: BoxSolver) -> Self {
        Self {
            max_budget,
            solver,
            idg: IdGen::new(),
            idmap: HashMap::new(),
        }
    }
}
impl Optimizer for BaseOptimizer {
    type Param = NextTrial;
    type Value = Ranked<OrderedFloat<f64>>;

    #[allow(clippy::map_entry)]
    fn ask<R: Rng, G: yamakan::IdGen>(
        &mut self,
        _rng: R,
        mut idg: G,
    ) -> Result<Obs<Self::Param>, yamakan::Error> {
        let trial = track!(self.solver.ask(&mut self.idg).map_err(into_yamakan))?;
        if !self.idmap.contains_key(&trial.id) {
            self.idmap.insert(trial.id, track!(idg.generate())?);
        }
        Ok(Obs {
            id: self.idmap[&trial.id],
            param: trial,
            value: (),
        })
    }

    fn tell(&mut self, obs: Obs<Self::Param, Self::Value>) -> Result<(), yamakan::Error> {
        let value = obs.value.value.0;
        let values = if value.is_nan() {
            Values::new(Vec::new())
        } else {
            Values::new(vec![value])
        };

        let trial = EvaluatedTrial {
            id: obs.param.id,
            values,
            current_step: self.max_budget - obs.value.rank,
        };
        track!(self.solver.tell(trial).map_err(into_yamakan))?;

        Ok(())
    }
}
