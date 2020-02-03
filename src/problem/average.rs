use kurobako_core::domain::VariableBuilder;
use kurobako_core::json::JsonRecipe;
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, BoxProblemFactory, Evaluator, Problem, ProblemFactory, ProblemRecipe,
    ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use rustats::fundamental::average;
use serde::{Deserialize, Serialize};
use std::cmp;
use structopt::StructOpt;

/// Recipe for aggregating (averaging) multiple problems.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct AverageProblemRecipe {
    /// Problem recipe JSONs.
    pub problems: Vec<JsonRecipe>,
}
impl ProblemRecipe for AverageProblemRecipe {
    type Factory = AverageProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        track_assert!(!self.problems.is_empty(), ErrorKind::InvalidInput);

        let problems = self
            .problems
            .iter()
            .map(|p| track!(registry.create_problem_factory_from_json(p)))
            .collect::<Result<Vec<_>>>()?;

        let mut specs = vec![track!(problems[0].specification())?];
        for p in problems.iter().skip(1) {
            let a = &specs[0];
            let b = track!(p.specification())?;

            track_assert_eq!(a.params_domain, b.params_domain, ErrorKind::InvalidInput);
            track_assert_eq!(
                a.values_domain
                    .variables()
                    .iter()
                    .map(|v| v.range())
                    .collect::<Vec<_>>(),
                b.values_domain
                    .variables()
                    .iter()
                    .map(|v| v.range())
                    .collect::<Vec<_>>(),
                ErrorKind::InvalidInput
            );
            track_assert_eq!(a.steps, b.steps, ErrorKind::InvalidInput);
            specs.push(b);
        }

        Ok(AverageProblemFactory { problems, specs })
    }
}

#[derive(Debug)]
pub struct AverageProblemFactory {
    problems: Vec<BoxProblemFactory>,
    specs: Vec<ProblemSpec>,
}
impl AverageProblemFactory {
    fn least_common_multiple_step(&self) -> u64 {
        let mut n = self.specs[0].steps.last();
        for spec in &self.specs[1..] {
            n = num::integer::lcm(n, spec.steps.last());
        }
        n
    }
}
impl ProblemFactory for AverageProblemFactory {
    type Problem = AverageProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let mut builder =
            ProblemSpecBuilder::new(&format!("Average of {} problems", self.problems.len())).attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            );

        for inner_spec in &self.specs {
            for (k, v) in &inner_spec.attrs {
                builder = builder.attr(&format!("{}.{}", inner_spec.name, k), v);
            }
        }

        let inner_spec = &self.specs[0];

        for v in inner_spec.values_domain.variables().iter().cloned() {
            builder = builder.value(VariableBuilder::from(v).name("Objective Value"));
        }

        builder = builder.steps(1..=self.least_common_multiple_step());

        track!(builder.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let problems = self
            .problems
            .iter()
            .map(|p| track!(p.create_problem(rng.clone())))
            .collect::<Result<Vec<_>>>()?;
        let lcm_step = self.least_common_multiple_step();
        Ok(AverageProblem {
            problems,
            step_scales: self
                .specs
                .iter()
                .map(|s| lcm_step / s.steps.last())
                .collect(),
        })
    }
}

#[derive(Debug)]
pub struct AverageProblem {
    problems: Vec<BoxProblem>,
    step_scales: Vec<u64>,
}
impl Problem for AverageProblem {
    type Evaluator = StudyEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let evaluators = self
            .problems
            .iter()
            .zip(self.step_scales.iter().cloned())
            .map(|(p, scale)| {
                Ok(EvaluatorState {
                    inner: track!(p.create_evaluator(params.clone()))?,
                    scale,
                    current_step: 0,
                    last_values: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(StudyEvaluator { evaluators })
    }
}

#[derive(Debug)]
pub struct StudyEvaluator {
    evaluators: Vec<EvaluatorState>,
}
impl Evaluator for StudyEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        loop {
            self.evaluators.sort_by_key(|e| e.current_step);
            let eval = &mut self.evaluators[0];
            let next_step = cmp::max(next_step, eval.current_step + 1);
            let (current_step, values) = track!(eval
                .inner
                .evaluate((next_step + eval.scale - 1) / eval.scale))?;
            eval.last_values = Some(values);
            eval.current_step = current_step * eval.scale;
            let current_step = eval.current_step;

            if self
                .evaluators
                .iter()
                .all(|e| e.current_step == current_step)
            {
                let dim = self.evaluators[0]
                    .last_values
                    .as_ref()
                    .unwrap_or_else(|| unreachable!())
                    .len();
                let values =
                    (0..dim)
                        .map(|i| {
                            average(self.evaluators.iter().map(|e| {
                                e.last_values.as_ref().unwrap_or_else(|| unreachable!())[i]
                            }))
                        })
                        .collect();
                return Ok((current_step, Values::new(values)));
            }
        }
    }
}

#[derive(Debug)]
struct EvaluatorState {
    inner: BoxEvaluator,
    scale: u64,
    current_step: u64,
    last_values: Option<Values>,
}
