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
use kurobako_core::{Error, Result};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct RankProblemRecipe {
    pub problem: JsonRecipe,
    pub baselines: Vec<PathBuf>,
}
impl RankProblemRecipe {
    fn inner_problem_id(&self, factory: &Arc<Mutex<BoxProblemFactory>>) -> Result<String> {
        let recipe = track!(serde_json::from_value(self.problem.clone()).map_err(Error::from))?;
        let spec = track!(track!(factory.lock().map_err(Error::from))?.specification())?;
        let record = ProblemRecord { recipe, spec };
        track!(record.id())
    }

    fn load_baseline_studies(&self, inner_problem_id: &str) -> Result<Vec<StudyRecord>> {
        let mut studies = Vec::new();
        for path in &self.baselines {
            let file = track!(File::open(path).map_err(Error::from); path)?;
            let temp_studies: Vec<StudyRecord> = track!(json::load(BufReader::new(file)); path)?;
            for study in temp_studies {
                if track!(study.problem.id())? == inner_problem_id {
                    studies.push(study);
                }
            }
        }
        Ok(studies)
    }
}
impl ProblemRecipe for RankProblemRecipe {
    type Factory = RankProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let inner_factory =
            track!(registry.get_or_create_problem_factory_from_json(&self.problem))?;
        let inner_problem_id = track!(self.inner_problem_id(&inner_factory))?;
        let studies = track!(self.load_baseline_studies(&inner_problem_id))?;
        let baseline = Arc::new(Baseline::new(&studies));
        Ok(RankProblemFactory {
            inner_factory,
            baseline,
            baseline_studies: studies.len(),
        })
    }
}

#[derive(Debug)]
pub struct RankProblemFactory {
    inner_factory: Arc<Mutex<BoxProblemFactory>>,
    baseline: Arc<Baseline>,
    baseline_studies: usize,
}
impl ProblemFactory for RankProblemFactory {
    type Problem = RankProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let inner_spec =
            track!(track!(self.inner_factory.lock().map_err(Error::from))?.specification())?;
        let mut spec = ProblemSpecBuilder::new(&inner_spec.name)
            .attr(
                "version",
                &format!("kurobako_solvers={}", env!("CARGO_PKG_VERSION")),
            )
            .attr("baseline_study_count", &self.baseline_studies.to_string())
            .params(
                inner_spec
                    .params_domain
                    .variables()
                    .iter()
                    .map(|p| p.clone().into())
                    .collect(),
            )
            .steps(inner_spec.steps.iter());

        for (k, v) in &inner_spec.attrs {
            spec = spec.attr(&format!("inner.{}", k), v);
        }

        for v in inner_spec.values_domain.variables().iter() {
            spec = spec.value(
                domain::var(&format!("1 - percentile_rank({}) / 100", v.name()))
                    .continuous(0.0, 1.0),
            );
        }

        track!(spec.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let inner_problem =
            track!(track!(self.inner_factory.lock().map_err(Error::from))?.create_problem(rng))?;
        Ok(RankProblem {
            inner_problem,
            baseline: Arc::clone(&self.baseline),
        })
    }
}

#[derive(Debug)]
pub struct RankProblem {
    inner_problem: BoxProblem,
    baseline: Arc<Baseline>,
}
impl Problem for RankProblem {
    type Evaluator = StudyEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let inner_evaluator = track!(self.inner_problem.create_evaluator(params))?;
        Ok(StudyEvaluator {
            inner_evaluator,
            baseline: Arc::clone(&self.baseline),
        })
    }
}

#[derive(Debug)]
pub struct StudyEvaluator {
    inner_evaluator: BoxEvaluator,
    baseline: Arc<Baseline>,
}
impl Evaluator for StudyEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let (current_step, values) = track!(self.inner_evaluator.evaluate(next_step))?;
        let ranks = self.baseline.rank_values(current_step, &values);
        Ok((current_step, ranks))
    }
}

#[derive(Debug)]
struct Baseline {
    step_to_values: HashMap<u64, Vec<Values>>,
}
impl Baseline {
    fn new(studies: &[StudyRecord]) -> Self {
        let mut step_to_values: HashMap<_, Vec<_>> = HashMap::new();
        for study in studies {
            for trial in &study.trials {
                let mut step = 0;
                for eval in &trial.evaluations {
                    step += eval.elapsed_steps();
                    step_to_values
                        .entry(step)
                        .or_default()
                        .push(eval.values.clone());
                }
            }
        }
        Self { step_to_values }
    }

    fn rank_values(&self, step: u64, values: &Values) -> Values {
        let count = self.step_to_values.get(&step).map_or(0, |vs| vs.len()) + 1;
        let mut ranks = vec![0; values.len()];
        for vs in self
            .step_to_values
            .get(&step)
            .iter()
            .flat_map(|vs| vs.iter())
        {
            for ((rank, a), b) in ranks.iter_mut().zip(vs.iter()).zip(values.iter()) {
                if a < b {
                    *rank += 1;
                }
            }
        }

        Values::new(
            ranks
                .into_iter()
                .map(|rank| rank as f64 / count as f64)
                .collect(),
        )
    }
}
