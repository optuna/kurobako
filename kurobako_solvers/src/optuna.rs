//! A solver based on [Optuna](https://github.com/optuna/optuna).
use kurobako_core::epi::solver::{
    EmbeddedScriptSolver, EmbeddedScriptSolverFactory, EmbeddedScriptSolverRecipe,
};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{Solver, SolverFactory, SolverRecipe, SolverSpec};
use kurobako_core::trial::{EvaluatedTrial, IdGen, NextTrial};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

fn add_arg(args: &mut Vec<String>, key: &str, val: &str) {
    args.push(key.to_owned());
    args.push(val.to_owned());
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !(*b)
}

mod defaults {
    macro_rules! define {
        ($val_fn:ident, $pred_fn:ident, $type:ty, $val:expr) => {
            #[allow(clippy::ptr_arg, clippy::float_cmp)]
            pub fn $pred_fn(x: &$type) -> bool {
                x == &$val
            }

            pub fn $val_fn() -> $type {
                $val
            }
        };
    }

    define!(loglevel, is_loglevel, String, "warning".to_owned());
}

/// Recipe of `OptunaSolver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[allow(missing_docs)]
#[structopt(rename_all = "kebab-case")]
pub struct OptunaSolverRecipe {
    /// Log level.
    #[structopt(
        long,
        default_value = "warning",
        possible_values = &["debug", "info", "warning", "error"]
    )]
    #[serde(skip_serializing_if = "defaults::is_loglevel")]
    #[serde(default = "defaults::loglevel")]
    pub loglevel: String,

    /// Sampler class name (e.g., "TPESampler").
    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sampler: Option<String>,

    /// Sampler arguments (e.g., "{\"seed\": 10}").
    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub sampler_kwargs: Option<String>,

    /// Pruner class name (e.g., "MedianPruner").
    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pruner: Option<String>,

    /// Pruner arguments (e.g., "{\"n_warmup_steps\": 10}").
    #[structopt(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub pruner_kwargs: Option<String>,

    /// Sets optimization direction to "maximize".
    ///
    /// The sign of all evaluated values ​​is reversed before being passed to Optuna.
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub maximize: bool,

    /// If this is `true`, `Trial.suggest_discrete_uniform()` is used for sampling discrete parameters instead of `Trial.suggest_int()`.
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub use_discrete_uniform: bool,
}
impl OptunaSolverRecipe {
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        add_arg(&mut args, "--loglevel", &self.loglevel);
        if let Some(v) = &self.sampler {
            add_arg(&mut args, "--sampler", v);
        }
        if let Some(v) = &self.sampler_kwargs {
            add_arg(&mut args, "--sampler-kwargs", v);
        }
        if let Some(v) = &self.pruner {
            add_arg(&mut args, "--pruner", v);
        }
        if let Some(v) = &self.pruner_kwargs {
            add_arg(&mut args, "--pruner-kwargs", v);
        }
        if self.maximize {
            args.push("--direction".to_owned());
            args.push("maximize".to_owned());
        }
        if self.use_discrete_uniform {
            args.push("--use-discrete-uniform".to_owned());
        }
        args
    }
}
impl SolverRecipe for OptunaSolverRecipe {
    type Factory = OptunaSolverFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let script = include_str!("../scripts/optuna_solver.py");
        let args = self.build_args();
        let recipe = EmbeddedScriptSolverRecipe {
            script: script.to_owned(),
            args,
        };
        let inner = track!(recipe.create_factory(registry))?;
        Ok(OptunaSolverFactory { inner })
    }
}

/// Factory of `OptunaSolver`.
#[derive(Debug)]
pub struct OptunaSolverFactory {
    inner: EmbeddedScriptSolverFactory,
}
impl SolverFactory for OptunaSolverFactory {
    type Solver = OptunaSolver;

    fn specification(&self) -> Result<SolverSpec> {
        track!(self.inner.specification())
    }

    fn create_solver(&self, rng: ArcRng, problem: &ProblemSpec) -> Result<Self::Solver> {
        let inner = track!(self.inner.create_solver(rng, problem))?;
        Ok(OptunaSolver { inner })
    }
}

/// Solver that uses [Optuna](https://github.com/optuna/optuna) as the backend.
#[derive(Debug)]
pub struct OptunaSolver {
    inner: EmbeddedScriptSolver,
}
impl Solver for OptunaSolver {
    fn ask(&mut self, idg: &mut IdGen) -> Result<NextTrial> {
        track!(self.inner.ask(idg))
    }

    fn tell(&mut self, trial: EvaluatedTrial) -> Result<()> {
        track!(self.inner.tell(trial))
    }
}
