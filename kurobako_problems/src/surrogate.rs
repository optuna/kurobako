//! A problem that uses a random forest surrogate model to evaluate parameters.
//!
//! # References
//!
//! - [Surrogate Benchmarks for Hyperparameter Optimization][paper]
//!
//! [paper]: http://ceur-ws.org/Vol-1201/paper-06.pdf
use kurobako_core::problem::{Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, Result};
use lazy_static::lazy_static;
use randomforest::RandomForestRegressor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

lazy_static! {
    static ref CACHE: Mutex<HashMap<PathBuf, Arc<RandomForestRegressor>>> =
        Mutex::new(HashMap::new());
}

/// Recipe of `SurrogateProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct SurrogateProblemRecipe {
    /// Directory path where a problem spec and a surrogate model files exist.
    pub model: PathBuf,

    /// Disable the in-memory model cache to reduce memory usage.
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "is_false")]
    pub disable_cache: bool,
}

impl SurrogateProblemRecipe {
    fn load_model(&self, model_path: &Path) -> Result<Arc<RandomForestRegressor>> {
        let model_file = track!(std::fs::File::open(model_path).map_err(Error::from); model_path)?;
        let model = RandomForestRegressor::deserialize(BufReader::new(model_file))?;
        Ok(Arc::new(model))
    }
}

impl ProblemRecipe for SurrogateProblemRecipe {
    type Factory = SurrogateProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let spec_path = self.model.join("spec.json");
        let spec_file = track!(std::fs::File::open(&spec_path).map_err(Error::from); spec_path)?;
        let spec: ProblemSpec = track!(serde_json::from_reader(spec_file).map_err(Error::from))?;

        let model_path = self.model.join("model.bin");
        let model = if self.disable_cache {
            track!(self.load_model(&model_path))?
        } else {
            let mut cache = track!(CACHE.lock().map_err(Error::from))?;
            if let Some(model) = cache.get(&model_path) {
                Arc::clone(model)
            } else {
                let model = track!(self.load_model(&model_path))?;
                cache.insert(model_path, Arc::clone(&model));
                model
            }
        };

        Ok(SurrogateProblemFactory { spec, model })
    }
}

/// Factory of `SurrogateProblem`.
#[derive(Debug)]
pub struct SurrogateProblemFactory {
    spec: ProblemSpec,
    model: Arc<RandomForestRegressor>,
}
impl ProblemFactory for SurrogateProblemFactory {
    type Problem = SurrogateProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        Ok(self.spec.clone())
    }

    fn create_problem(&self, _rng: ArcRng) -> Result<Self::Problem> {
        Ok(SurrogateProblem {
            model: Arc::clone(&self.model),
        })
    }
}

/// Problem that uses a random forest surrogate model to evaluate parameters.
#[derive(Debug)]
pub struct SurrogateProblem {
    model: Arc<RandomForestRegressor>,
}

impl Problem for SurrogateProblem {
    type Evaluator = SurrogateEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        Ok(SurrogateEvaluator {
            params,
            model: Arc::clone(&self.model),
        })
    }
}

/// Evaluator of `SurrogateProblem`.
#[derive(Debug)]
pub struct SurrogateEvaluator {
    params: Params,
    model: Arc<RandomForestRegressor>,
}

impl Evaluator for SurrogateEvaluator {
    fn evaluate(&mut self, _next_step: u64) -> Result<(u64, Values)> {
        let value = self.model.predict(self.params.get());
        Ok((1, Values::new(vec![value])))
    }
}

fn is_false(&b: &bool) -> bool {
    !b
}
