use kurobako_core::epi::solver::{EmbeddedScriptSolver, EmbeddedScriptSolverRecipe};
use kurobako_core::problem::ProblemSpec;
use kurobako_core::solver::{ObservedObs, Solver, SolverRecipe, SolverSpec, UnobservedObs};
use kurobako_core::Result;
use rand::Rng;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;
use yamakan::observation::IdGen;

fn add_arg(args: &mut Vec<String>, key: &str, val: &str) {
    args.push(key.to_owned());
    args.push(val.to_owned());
}

fn is_false(b: &bool) -> bool {
    *b == false
}

mod defaults {
    macro_rules! define {
        ($val_fn:ident, $pred_fn:ident, $type:ty, $val:expr) => {
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
    define!(tpe_gamma_factor, is_tpe_gamma_factor, f64, 0.25);
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
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct OptunaSolverRecipe {
    #[structopt(
        long,
        default_value = "warning",
        raw(possible_values = "&[\"debug\", \"info\", \"warning\", \"error\"]")
    )]
    #[serde(skip_serializing_if = "defaults::is_loglevel")]
    #[serde(default = "defaults::loglevel")]
    pub loglevel: String,

    #[structopt(
        long,
        default_value = "tpe",
        raw(possible_values = "&[\"tpe\", \"random\", \"skopt\"]")
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

    #[structopt(long, default_value = "0.25")]
    #[serde(skip_serializing_if = "defaults::is_tpe_gamma_factor")]
    #[serde(default = "defaults::tpe_gamma_factor")]
    pub tpe_gamma_factor: f64,

    #[structopt(
        long,
        default_value = "GP",
        raw(possible_values = "&[\"GP\", \"RF\", \"ET\", \"GBRT\"]")
    )]
    #[serde(skip_serializing_if = "defaults::is_skopt_base_estimator")]
    #[serde(default = "defaults::skopt_base_estimator")]
    pub skopt_base_estimator: String,

    #[structopt(
        long,
        default_value = "median",
        raw(possible_values = "&[\"median\", \"asha\", \"none\"]")
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
        add_arg(
            &mut args,
            "--tpe-gamma-factor",
            &self.tpe_gamma_factor.to_string(),
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
        if self.maximize {
            args.push("--direction".to_owned());
            args.push("maximize".to_owned());
        }
        args
    }
}
impl SolverRecipe for OptunaSolverRecipe {
    type Solver = OptunaSolver;

    fn create_solver(&self, problem: ProblemSpec) -> Result<Self::Solver> {
        let script = include_str!("../contrib/optuna_solver.py");
        let args = self.build_args();
        let recipe = EmbeddedScriptSolverRecipe {
            script: script.to_owned(),
            args,
            interpreter: None, // TODO: env!("KUROBAKO_PYTHON"),
            interpreter_args: Vec::new(),
        };
        let inner = track!(recipe.create_solver(problem))?;
        Ok(OptunaSolver(inner))
    }
}

#[derive(Debug)]
pub struct OptunaSolver(EmbeddedScriptSolver);
impl Solver for OptunaSolver {
    fn specification(&self) -> SolverSpec {
        self.0.specification()
    }

    fn ask<R: Rng, G: IdGen>(&mut self, rng: &mut R, idg: &mut G) -> Result<UnobservedObs> {
        track!(self.0.ask(rng, idg))
    }

    fn tell(&mut self, obs: ObservedObs) -> Result<()> {
        track!(self.0.tell(obs))
    }
}
