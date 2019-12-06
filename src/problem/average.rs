use crate::record::{ProblemRecord, StudyRecord};
use kurobako_core::domain;
use kurobako_core::json::{self, JsonRecipe};
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, BoxProblemFactory, Evaluator, Problem, ProblemFactory, ProblemRecipe,
    ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct AverageProblemRecipe {
    pub problems: Vec<JsonRecipe>,
}
impl ProblemRecipe for AverageProblemRecipe {
    type Factory = AverageProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        track_assert!(!self.problems.is_empty(), ErrorKind::InvalidInput);

        let problems = self
            .problems
            .iter()
            .map(|p| track!(registry.get_or_create_problem_factory_from_json(p)))
            .collect::<Result<Vec<_>>>()?;

        for p in problems.iter().skip(1) {
            let a = track!(spec(&problems[0]))?;
            let b = track!(spec(p))?;

            track_assert_eq!(a.params_domain, b.params_domain, ErrorKind::InvalidInput);
            track_assert_eq!(a.values_domain, b.values_domain, ErrorKind::InvalidInput);
            track_assert_eq!(a.steps, b.steps, ErrorKind::InvalidInput);
        }

        Ok(AverageProblemFactory { problems })
    }
}

#[derive(Debug)]
pub struct AverageProblemFactory {
    problems: Vec<Arc<Mutex<BoxProblemFactory>>>,
}
impl ProblemFactory for AverageProblemFactory {
    type Problem = AverageProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let mut builder =
            ProblemSpecBuilder::new(&format!("Average of {} problems", self.problems.len())).attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            );

        for p in &self.problems {
            let inner_spec = track!(spec(p))?;
            for (k, v) in inner_spec.attrs {
                builder = builder.attr(&format!("{}.{}", inner_spec.name, k), &v);
            }
        }

        let inner_spec = track!(spec(&self.problems[0]))?;

        for v in inner_spec.values_domain.variables().iter().cloned() {
            builder = builder.value(v.into());
        }

        // TODO:
        builder = builder.steps(inner_spec.steps.iter());

        track!(builder.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let problems = self
            .problems
            .iter()
            .map(|p| track!(track!(p.lock().map_err(Error::from))?.create_problem(rng.clone())))
            .collect::<Result<Vec<_>>>()?;
        Ok(AverageProblem { problems })
    }
}

#[derive(Debug)]
pub struct AverageProblem {
    problems: Vec<BoxProblem>,
}
impl Problem for AverageProblem {
    type Evaluator = StudyEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let evaluators = self
            .problems
            .iter()
            .map(|p| track!(p.create_evaluator(params.clone())))
            .collect::<Result<Vec<_>>>()?;
        Ok(StudyEvaluator { evaluators })
    }
}

#[derive(Debug)]
pub struct StudyEvaluator {
    evaluators: Vec<BoxEvaluator>,
}
impl Evaluator for StudyEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let mut current_step = None;
        for evaluator in &self.evaluators {
            let (step, values) = track!(evaluator.evaluate(next_step))?;
            if current_step.is_none() {
                current_step = Some(step);
            }
        }
        panic!()
    }
}

fn spec(factory: &Arc<Mutex<BoxProblemFactory>>) -> Result<ProblemSpec> {
    let factory = track!(factory.lock().map_err(Error::from))?;
    track!(factory.specification())
}
