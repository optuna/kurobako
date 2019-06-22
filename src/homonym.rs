use crate::problem::KurobakoProblemRecipe;
use crate::record::{load_studies, StudyRecord};
use kurobako_core::num::FiniteF64;
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{
    BoxEvaluator, BoxProblem, Evaluate, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{json, Error, ErrorKind, Result};
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::num::NonZeroU64;
use std::path::PathBuf;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
pub struct HomonymProblemRecipe {
    #[structopt(long)]
    pub problems: Vec<json::JsonValue>,

    #[structopt(long)]
    pub baseline: PathBuf,
}
impl ProblemRecipe for HomonymProblemRecipe {
    type Problem = HomonymProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let recipes: Vec<KurobakoProblemRecipe> = self
            .problems
            .iter()
            .map(|p| track!(serde_json::from_value(p.get().clone()).map_err(Error::from)))
            .collect::<Result<_>>()?;
        let specs = recipes
            .iter()
            .map(|p| track!(p.create_problem()).map(|p| p.specification()))
            .collect::<Result<Vec<_>>>()?;
        track_assert!(!specs.is_empty(), ErrorKind::InvalidInput);
        for p in &specs[1..] {
            track_assert_eq!(
                p.params_domain,
                specs[0].params_domain,
                ErrorKind::InvalidInput
            );
            track_assert_eq!(
                p.evaluation_expense,
                specs[0].evaluation_expense,
                ErrorKind::InvalidInput
            );
            track_assert_eq!(
                p.capabilities,
                specs[0].capabilities,
                ErrorKind::InvalidInput
            );
        }

        let studies = track!(load_studies(&self.baseline))?;
        debug!("Studies: {}", studies.len());

        // TODO
        let mut recipe_to_studies = HashMap::<_, Vec<_>>::new();
        for study in studies {
            recipe_to_studies
                .entry(study.problem.recipe.clone())
                .or_default()
                .push(study);
        }

        let mut baselines = Vec::new();
        for p in &recipes {
            let p =
                json::JsonValue::new(track!(serde_json::to_value(p.clone()).map_err(Error::from))?); // TODO
            if let Some(s) = recipe_to_studies.get(&p).cloned() {
                debug!("Baseline: n={}, recipe={}", s.len(), p.get());
                baselines.push(s);
            } else {
                track_panic!(ErrorKind::InvalidInput, "No baseline studies: {:?}", p);
            }
        }

        let problems = recipes
            .iter()
            .map(|r| track!(r.create_problem()))
            .collect::<Result<_>>()?;
        Ok(HomonymProblem {
            problems,
            specs,
            baselines,
        })
    }
}

#[derive(Debug)]
pub struct HomonymProblem {
    problems: Vec<BoxProblem>,
    specs: Vec<ProblemSpec>,
    baselines: Vec<Vec<StudyRecord>>,
}
impl Problem for HomonymProblem {
    type Evaluator = HomonymEvaluator;

    fn specification(&self) -> ProblemSpec {
        let base = self.specs[0].evaluation_expense.get();
        ProblemSpec {
            name: format!("homonym/{}", self.specs.len()),
            version: None, // TODO
            params_domain: self.specs[0].params_domain.clone(),
            values_domain: vec![unsafe {
                MinMax::new_unchecked(FiniteF64::new_unchecked(0.0), FiniteF64::new_unchecked(1.0))
            }],
            evaluation_expense: unsafe {
                NonZeroU64::new_unchecked(base * self.problems.len() as u64)
            },
            capabilities: self.specs[0].capabilities.clone(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        let evaluators = self
            .problems
            .iter_mut()
            .map(|p| track!(p.create_evaluator(id)))
            .collect::<Result<_>>()?;
        Ok(HomonymEvaluator {
            evaluators,
            consumptions: vec![0; self.problems.len()],
            baselines: self.baselines.clone(),
        })
    }
}

#[derive(Debug)]
pub struct HomonymEvaluator {
    evaluators: Vec<BoxEvaluator>,
    consumptions: Vec<u64>,
    baselines: Vec<Vec<StudyRecord>>,
}
impl Evaluate for HomonymEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        let mut values = vec![0.0; self.evaluators.len()];

        while !budget.is_consumed() {
            for ((evaluator, consumption), value) in self
                .evaluators
                .iter_mut()
                .zip(self.consumptions.iter_mut())
                .zip(values.iter_mut())
            {
                let mut evaluator_budget = Budget {
                    amount: *consumption + 1,
                    consumption: *consumption,
                };
                let vs = track!(evaluator.evaluate(params, &mut evaluator_budget))?;
                *value = vs[0].get(); // TODO: support multi-objective

                let delta = evaluator_budget.consumption - *consumption;
                *consumption += delta;
                budget.consumption += delta;
            }
        }

        debug!(
            "Evaluated: budget={:?}, params={:?}, value={:?}",
            budget, params, values,
        );

        let ranking_sum = values
            .iter()
            .zip(self.consumptions.iter())
            .zip(self.baselines.iter())
            .map(|((&v, &budget), studies)| {
                let mut total = 0;
                let mut smalers = 0;
                for study in studies {
                    for trial in study.intermediate_trials(budget) {
                        total += 1;
                        if v > trial.evaluate.values[0].get() {
                            smalers += 1;
                        }
                    }
                }
                smalers as f64 / total as f64
            })
            .sum::<f64>(); // TODO: remove duplicated baseline studies
        let score = ranking_sum / values.len() as f64;
        debug!("Ranking: {} ({})", ranking_sum, score);

        return Ok(vec![track!(FiniteF64::new(score))?]);
    }
}
