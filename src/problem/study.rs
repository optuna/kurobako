use crate::runner::StudyRunner;
use crate::study::StudyRecipe;
use crate::variable::Var;
use kurobako_core::domain::Range;
use kurobako_core::json::{self, JsonRecipe};
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::solver::{SolverFactory as _, SolverRecipe as _, SolverSpec};
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

/// Recipe for problem based on a parameterized study.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct StudyProblemRecipe {
    /// Study recipe JSON.
    pub study: JsonRecipe,

    /// Variable JSONs.
    #[structopt(long, parse(try_from_str = json::parse_json))]
    pub vars: Vec<Var>,
}
impl ProblemRecipe for StudyProblemRecipe {
    type Factory = StudyProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let study_json = self.study.clone();
        let study: StudyRecipe = track!(serde_json::from_value(study_json).map_err(Error::from))?;

        let problem = track!(study.problem.create_factory(registry))?;
        let solver = track!(study.solver.create_factory(registry))?;
        Ok(StudyProblemFactory {
            problem: track!(problem.specification())?,
            solver: track!(solver.specification())?,
            budget: study.budget,
            study: self.study.clone(),
            vars: self.vars.clone(),
        })
    }
}

#[derive(Debug)]
pub struct StudyProblemFactory {
    problem: ProblemSpec,
    solver: SolverSpec,
    budget: u64,
    study: JsonRecipe,
    vars: Vec<Var>,
}
impl ProblemFactory for StudyProblemFactory {
    type Problem = StudyProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let mut spec = ProblemSpecBuilder::new(&format!(
            "Study: problem={}, solver={}",
            self.problem.name, self.solver.name
        ));

        spec = spec.attr(
            "version",
            &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
        );

        for v in &self.vars {
            spec = spec.param(v.to_domain_var());
        }

        for v in self.problem.values_domain.variables() {
            spec = spec.value(v.clone().into());
        }

        let steps = self.problem.steps.last() * self.budget;
        spec = spec.steps(1..=steps); // FIXME: optimize

        track!(spec.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        Ok(StudyProblem {
            study: self.study.clone(),
            vars: self.vars.clone(),
            rng,
        })
    }
}

#[derive(Debug)]
pub struct StudyProblem {
    study: JsonRecipe,
    vars: Vec<Var>,
    rng: ArcRng,
}
impl StudyProblem {
    fn bind(&self, vals: &[f64]) -> Result<JsonRecipe> {
        let mut json = self.study.clone();
        for (var, val) in self.vars.iter().zip(vals.iter().copied()) {
            track!(self.bind_var(&mut json, var, val))?;
        }
        Ok(json)
    }

    fn bind_var(&self, mut json: &mut JsonRecipe, var: &Var, val: f64) -> Result<()> {
        for c in var.path.components() {
            match json {
                serde_json::Value::Object(ref mut x) => {
                    json = x
                        .entry(c)
                        .or_insert_with(|| serde_json::Value::Object(Default::default()));
                }
                serde_json::Value::Array(ref mut x) => {
                    let index: usize = track!(c.parse().map_err(Error::from))?;
                    json = track_assert_some!(x.get_mut(index), ErrorKind::InvalidInput; index);
                }
                _ => track_panic!(ErrorKind::InvalidInput; json, var, val),
            }
        }

        match &var.range {
            Range::Continuous { .. } => {
                let n = track_assert_some!(serde_json::Number::from_f64(val), ErrorKind::InvalidInput; val);
                *json = serde_json::Value::Number(n);
            }
            Range::Discrete { .. } => {
                let n = serde_json::Number::from(val as i64);
                *json = serde_json::Value::Number(n);
            }
            Range::Categorical { choices } => {
                *json = serde_json::Value::String(choices[val as usize].clone());
            }
        }
        Ok(())
    }
}
impl Problem for StudyProblem {
    type Evaluator = StudyEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let study = track!(self.bind(params.get()))?;
        let study: StudyRecipe =
            track!(serde_json::from_value(study.clone()).map_err(Error::from); study)?;
        let mut runner = track!(StudyRunner::new(&study))?;
        track!(runner.run_init())?;
        Ok(StudyEvaluator { runner })
    }
}

#[derive(Debug)]
pub struct StudyEvaluator {
    runner: StudyRunner,
}
impl Evaluator for StudyEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        loop {
            track!(self.runner.run_once())?;
            if self.runner.current_step() < next_step {
                continue;
            }

            if let Some(values) = self.runner.best_values().cloned() {
                let current_step = self.runner.current_step();
                return Ok((current_step, values));
            }

            track_assert!(
                self.runner.current_step() < self.runner.max_step(),
                ErrorKind::Other
            );
        }
    }
}
