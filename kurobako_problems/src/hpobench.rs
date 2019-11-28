//! A problem based on the benchmark described in [Tabular Benchmarks for Joint Architecture and Hyperparameter Optimization][paper].
//!
//! [paper]: https://arxiv.org/abs/1905.04970
use hdf5file::{self, DataObject, Hdf5File};
use kurobako_core::domain;
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::{ArcRng, Rng};
use kurobako_core::trial::{Params, Values};
use kurobako_core::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use std::f64;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;
use trackable::error::ErrorKindExt as _;

/// Recipe of `HpobenchProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct HpobenchProblemRecipe {
    /// Path of the FC-Net dataset.
    pub dataset: PathBuf,
}
impl ProblemRecipe for HpobenchProblemRecipe {
    type Factory = HpobenchProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let file = track!(Hdf5File::open_file(&self.dataset).map_err(into_error))?;
        Ok(HpobenchProblemFactory {
            file: Arc::new(Mutex::new(file)),
            path: self.dataset.clone(),
        })
    }
}

/// Factory of `HpobenchProblem`.
#[derive(Debug)]
pub struct HpobenchProblemFactory {
    file: Arc<Mutex<Hdf5File>>,
    path: PathBuf,
}
impl ProblemFactory for HpobenchProblemFactory {
    type Problem = HpobenchProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let name = track_assert_some!(
            self.path.file_stem().and_then(|n| n.to_str()),
            ErrorKind::InvalidInput
        );
        let name = match name {
            "fcnet_naval_propulsion_data" => "HPO-Bench-Naval",
            "fcnet_parkinsons_telemonitoring_data" => "HPO-Bench-Parkinson",
            "fcnet_protein_structure_data" => "HPO-Bench-Protein",
            "fcnet_slice_localization_data" => "HPO-Bench-Slice",
            _ => name,
        };

        let spec = ProblemSpecBuilder::new(name)
            .attr(
                "version",
                &format!("kurobako_problems={}", env!("CARGO_PKG_VERSION")),
            )
            .attr(
                "paper",
                "Klein, Aaron, and Frank Hutter. \"Tabular Benchmarks \
                 for Joint Architecture and Hyperparameter Optimization.\" \
                 arXiv preprint arXiv:1905.04970 (2019).",
            )
            .attr("github", "https://github.com/automl/nas_benchmarks")
            .param(domain::var("activation_fn_1").categorical(&["tanh", "relu"]))
            .param(domain::var("activation_fn_2").categorical(&["tanh", "relu"]))
            .param(domain::var("batch_size").discrete(0, 4))
            .param(domain::var("dropout_1").discrete(0, 3))
            .param(domain::var("dropout_2").discrete(0, 3))
            .param(domain::var("init_lr").discrete(0, 6))
            .param(domain::var("lr_schedule").categorical(&["cosine", "const"]))
            .param(domain::var("n_units_1").discrete(0, 6))
            .param(domain::var("n_units_2").discrete(0, 6))
            .value(domain::var("Validation MSE").continuous(0.0, f64::INFINITY))
            .steps(1..=100);

        track!(spec.finish())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        Ok(HpobenchProblem {
            file: Arc::clone(&self.file),
            rng,
        })
    }
}

/// FC-Net problem.
#[derive(Debug)]
pub struct HpobenchProblem {
    file: Arc<Mutex<Hdf5File>>,
    rng: ArcRng,
}
impl Problem for HpobenchProblem {
    type Evaluator = HpobenchEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        const UNITS: [usize; 6] = [16, 32, 64, 128, 256, 512];
        const DROPOUTS: [&str; 3] = ["0.0", "0.3", "0.6"];

        let key = format!(
            r#"{{"activation_fn_1": {:?}, "activation_fn_2": {:?}, "batch_size": {}, "dropout_1": {}, "dropout_2": {}, "init_lr": {}, "lr_schedule": {:?}, "n_units_1": {}, "n_units_2": {}}}"#,
            (["tanh", "relu"])[params[0] as usize],
            (["tanh", "relu"])[params[1] as usize],
            ([8, 16, 32, 64])[params[2] as usize],
            DROPOUTS[params[3] as usize],
            DROPOUTS[params[4] as usize],
            ([5.0 * 1e-4, 1e-3, 5.0 * 1e-3, 1e-2, 5.0 * 1e-2, 1e-1])[params[5] as usize],
            (["cosine", "const"])[params[6] as usize],
            UNITS[params[7] as usize],
            UNITS[params[8] as usize]
        );

        let sample_index = track!(self.rng.with_lock(|rng| rng.gen::<usize>() % 4))?;
        Ok(HpobenchEvaluator {
            file: Arc::clone(&self.file),
            key: format!("/{}/valid_mse", key),
            sample_index,
        })
    }
}

/// Evaluator of `HpobenchProblem`.
#[derive(Debug)]
pub struct HpobenchEvaluator {
    file: Arc<Mutex<Hdf5File>>,
    key: String,
    sample_index: usize,
}
impl Evaluator for HpobenchEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        let mut file = track!(self.file.lock().map_err(Error::from))?;
        let data = track!(file.get_object(&self.key).map_err(into_error))?;
        let DataObject::Float(data) = track_assert_some!(data, ErrorKind::InvalidInput; self.key);

        let value = data[[self.sample_index, next_step as usize - 1]];
        Ok((next_step, Values::new(vec![value])))
    }
}

fn into_error(e: hdf5file::Error) -> Error {
    ErrorKind::Other.takes_over(e).into()
}
