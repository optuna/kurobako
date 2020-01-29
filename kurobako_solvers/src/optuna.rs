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
    define!(sampler, is_sampler, String, "tpe".to_owned());
    define!(tpe_startup_trials, is_tpe_startup_trials, usize, 10);
    define!(tpe_ei_candidates, is_tpe_ei_candidates, usize, 24);
    define!(tpe_prior_weight, is_tpe_prior_weight, f64, 1.0);
    define!(
        skopt_base_estimator,
        is_skopt_base_estimator,
        String,
        "GP".to_owned()
    );
    define!(pruner, is_pruner, String, "median".to_owned());
    define!(median_startup_trials, is_median_startup_trials, usize, 5);
    define!(median_warmup_steps, is_median_warmup_steps, usize, 0);
    define!(asha_min_resource, is_asha_min_resource, usize, 1);
    define!(asha_reduction_factor, is_asha_reduction_factor, usize, 4);
    define!(hyperband_min_resource, is_hyperband_min_resource, usize, 1);
    define!(
        hyperband_reduction_factor,
        is_hyperband_reduction_factor,
        usize,
        3
    );
    define!(hyperband_n_brackets, is_hyperband_n_brackets, usize, 4);
}

/// Recipe of `OptunaSolver`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[allow(missing_docs)]
#[structopt(rename_all = "kebab-case")]
pub struct OptunaSolverRecipe {
    #[structopt(
        long,
        default_value = "warning",
        possible_values = &["debug", "info", "warning", "error"]
    )]
    #[serde(skip_serializing_if = "defaults::is_loglevel")]
    #[serde(default = "defaults::loglevel")]
    pub loglevel: String,

    #[structopt(
        long,
        default_value = "tpe",
        possible_values = &["tpe", "random", "skopt", "cma-es"]
    )]
    #[serde(skip_serializing_if = "defaults::is_sampler")]
    #[serde(default = "defaults::sampler")]
    pub sampler: String,

    #[structopt(long, default_value = "10")]
    #[serde(skip_serializing_if = "defaults::is_tpe_startup_trials")]
    #[serde(default = "defaults::tpe_startup_trials")]
    pub tpe_startup_trials: usize,

    #[structopt(long, default_value = "24")]
    #[serde(skip_serializing_if = "defaults::is_tpe_ei_candidates")]
    #[serde(default = "defaults::tpe_ei_candidates")]
    pub tpe_ei_candidates: usize,

    #[structopt(long, default_value = "1.0")]
    #[serde(skip_serializing_if = "defaults::is_tpe_prior_weight")]
    #[serde(default = "defaults::tpe_prior_weight")]
    pub tpe_prior_weight: f64,

    #[structopt(
        long,
        default_value = "GP",
        possible_values = &["GP", "RF", "ET", "GBRT"]
    )]
    #[serde(skip_serializing_if = "defaults::is_skopt_base_estimator")]
    #[serde(default = "defaults::skopt_base_estimator")]
    pub skopt_base_estimator: String,

    #[structopt(
        long,
        default_value = "median",
        possible_values = &["median", "asha", "nop", "hyperband"]
    )]
    #[serde(skip_serializing_if = "defaults::is_pruner")]
    #[serde(default = "defaults::pruner")]
    pub pruner: String,

    #[structopt(long, default_value = "5")]
    #[serde(skip_serializing_if = "defaults::is_median_startup_trials")]
    #[serde(default = "defaults::median_startup_trials")]
    pub median_startup_trials: usize,

    #[structopt(long, default_value = "0")]
    #[serde(skip_serializing_if = "defaults::is_median_warmup_steps")]
    #[serde(default = "defaults::median_warmup_steps")]
    pub median_warmup_steps: usize,

    #[structopt(long, default_value = "1")]
    #[serde(skip_serializing_if = "defaults::is_asha_min_resource")]
    #[serde(default = "defaults::asha_min_resource")]
    pub asha_min_resource: usize,

    #[structopt(long, default_value = "4")]
    #[serde(skip_serializing_if = "defaults::is_asha_reduction_factor")]
    #[serde(default = "defaults::asha_reduction_factor")]
    pub asha_reduction_factor: usize,

    #[structopt(long, default_value = "1")]
    #[serde(skip_serializing_if = "defaults::is_hyperband_min_resource")]
    #[serde(default = "defaults::hyperband_min_resource")]
    pub hyperband_min_resource: usize,

    #[structopt(long, default_value = "3")]
    #[serde(skip_serializing_if = "defaults::is_hyperband_reduction_factor")]
    #[serde(default = "defaults::hyperband_reduction_factor")]
    pub hyperband_reduction_factor: usize,

    #[structopt(long, default_value = "4")]
    #[serde(skip_serializing_if = "defaults::is_hyperband_n_brackets")]
    #[serde(default = "defaults::hyperband_n_brackets")]
    pub hyperband_n_brackets: usize,

    #[structopt(long)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub maximize: bool,
}
impl OptunaSolverRecipe {
    fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();
        add_arg(&mut args, "--loglevel", &self.loglevel);
        add_arg(&mut args, "--sampler", &self.sampler);
        add_arg(
            &mut args,
            "--tpe-startup-trials",
            &self.tpe_startup_trials.to_string(),
        );
        add_arg(
            &mut args,
            "--tpe-ei-candidates",
            &self.tpe_ei_candidates.to_string(),
        );
        add_arg(
            &mut args,
            "--tpe-prior-weight",
            &self.tpe_prior_weight.to_string(),
        );
        add_arg(&mut args, "--pruner", &self.pruner);
        add_arg(
            &mut args,
            "--median-startup-trials",
            &self.median_startup_trials.to_string(),
        );
        add_arg(
            &mut args,
            "--median-warmup-steps",
            &self.median_warmup_steps.to_string(),
        );
        add_arg(
            &mut args,
            "--asha-min-resource",
            &self.asha_min_resource.to_string(),
        );
        add_arg(
            &mut args,
            "--asha-reduction-factor",
            &self.asha_reduction_factor.to_string(),
        );
        add_arg(
            &mut args,
            "--hyperband-min-resource",
            &self.hyperband_min_resource.to_string(),
        );
        add_arg(
            &mut args,
            "--hyperband-reduction-factor",
            &self.hyperband_reduction_factor.to_string(),
        );
        add_arg(
            &mut args,
            "--hyperband-n-brackets",
            &self.hyperband_n_brackets.to_string(),
        );
        if self.maximize {
            args.push("--direction".to_owned());
            args.push("maximize".to_owned());
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
