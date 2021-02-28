//! Subcommand to build Surrogate model.
use kurobako_core::domain;
use kurobako_core::problem::{ProblemSpec, ProblemSpecBuilder};
use kurobako_core::{Error, ErrorKind, Result};
use ordered_float::OrderedFloat;
use randomforest::criterion::Mse;
use randomforest::table::{ColumnType, TableBuilder};
use randomforest::{RandomForestRegressor, RandomForestRegressorOptions};
use std::collections::BTreeMap;
use std::io::{BufWriter, Write as _};
use std::num::NonZeroUsize;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use trackable::error::ErrorKindExt;

/// Options of the `kurobako dataset surrogate-optuna-study` command.
#[derive(Debug, Clone, structopt::StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub struct SurrogateOpt {
    /// Problem name.
    #[structopt(long)]
    pub problem_name: String,

    /// Optuna storage URI.
    #[structopt(long)]
    pub storage: String,

    /// Target study name (regexp).
    #[structopt(long, default_value = ".*")]
    pub target_study_name: String,

    /// Output directory.
    #[structopt(long, default_value = "surrogates/")]
    pub out: PathBuf,

    /// Objective value index.
    #[structopt(long, default_value = "0")]
    pub objective_index: usize,

    /// Max samples used for building each tree in a random forest.
    #[structopt(long, default_value = "1000")]
    pub max_samples: NonZeroUsize,

    /// Number of trees in a rando forest.
    #[structopt(long, default_value = "1000")]
    pub trees: NonZeroUsize,
}

impl SurrogateOpt {
    pub(crate) fn run(&self) -> Result<()> {
        let trials = track!(self.load_trials())?;

        track_assert!(!trials.is_empty(), ErrorKind::InvalidInput);
        for trial in &trials[1..] {
            track_assert_eq!(
                trials[0].distributions,
                trial.distributions,
                ErrorKind::InvalidInput,
                "Conditional search space is not yet supported"
            );
        }

        let model = track!(self.build_surrogate_model(&trials))?;
        let spec = track!(self.build_problem_spec(&model, &trials))?;
        track!(self.save_surrogate_model(&spec, &model))?;
        Ok(())
    }

    fn load_trials(&self) -> Result<Vec<Trial>> {
        let script = include_str!("../../scripts/dump-optuna-trials.py");
        let mut script_file = track!(NamedTempFile::new().map_err(Error::from))?;
        track!(write!(script_file.as_file_mut(), "{}", script).map_err(Error::from))?;
        let script_path = script_file.into_temp_path();

        let command = track!(std::process::Command::new("python3")
            .arg(track_assert_some!(script_path.to_str(), ErrorKind::Bug))
            .arg(&self.storage)
            .arg(&self.target_study_name)
            .output()
            .map_err(Error::from))?;
        if !command.stderr.is_empty() {
            eprint!("{}", String::from_utf8_lossy(&command.stderr));
        }
        track_assert!(command.status.success(), ErrorKind::Other; command.status);

        track!(serde_json::from_slice(&command.stdout).map_err(Error::from))
    }

    fn build_surrogate_model(&self, trials: &[Trial]) -> Result<Model> {
        let mut table = TableBuilder::new();
        let column_types = trials[0]
            .distributions
            .iter()
            .map(|(_, d)| {
                if matches!(d, Distribution::CategoricalDistribution { .. }) {
                    ColumnType::Categorical
                } else {
                    ColumnType::Numerical
                }
            })
            .collect::<Vec<_>>();
        table
            .set_feature_column_types(&column_types)
            .expect("unreachable");

        let mut rows = Vec::new();
        for trial in trials {
            rows.push((
                trial.params.values().copied().collect::<Vec<_>>(),
                track_assert_some!(
                    trial.values.get(self.objective_index).copied(),
                    ErrorKind::InvalidInput
                ),
            ));
        }

        let mut outliers = 0;
        let p95 = percentile(rows.iter().map(|x| x.1), 0.95);
        for r in rows.iter_mut() {
            if r.1 > p95 {
                r.1 = p95;
                outliers += 1;
            }
        }

        for r in rows {
            track!(table
                .add_row(&r.0, r.1)
                .map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
        }

        let table = track!(table.build().map_err(|e| ErrorKind::InvalidInput.cause(e)))?;
        let regressor = RandomForestRegressorOptions::new()
            .parallel()
            .max_samples(self.max_samples)
            .trees(self.trees)
            .fit(Mse, table);

        Ok(Model {
            regressor,
            samples: trials.len(),
            outliers,
        })
    }

    fn build_problem_spec(&self, model: &Model, trials: &[Trial]) -> Result<ProblemSpec> {
        let params = trials[0]
            .distributions
            .iter()
            .map(|(name, d)| {
                let v = domain::var(name);
                match d {
                    Distribution::UniformDistribution { low, high } => {
                        Ok(v.continuous(*low, *high))
                    }
                    Distribution::LogUniformDistribution { low, high } => {
                        Ok(v.continuous(*low, *high).log_uniform())
                    }
                    Distribution::DiscreteUniformDistribution { .. } => {
                        track_panic!(
                            ErrorKind::Other,
                            "unsupported: name={:?}, distribution={:?}",
                            name,
                            d
                        );
                    }
                    Distribution::IntUniformDistribution { low, high, step } => {
                        track_assert_eq!(
                            *step,
                            1,
                            ErrorKind::Other,
                            "unsupported: name={:?}, distribution={:?}",
                            name,
                            d
                        );
                        Ok(v.discrete(*low, *high))
                    }
                    Distribution::IntLogUniformDistribution { low, high, step } => {
                        track_assert_eq!(
                            *step,
                            1,
                            ErrorKind::Other,
                            "unsupported: name={:?}, distribution={:?}",
                            name,
                            d
                        );
                        Ok(v.discrete(*low, *high).log_uniform())
                    }
                    Distribution::CategoricalDistribution { choices } => {
                        Ok(v.categorical(choices.iter().map(|c| c.to_string())))
                    }
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let values = (0..trials[0].values.len())
            .map(|i| domain::var(&format!("Objective Value {}", i + 1)))
            .collect();
        track!(ProblemSpecBuilder::new(&self.problem_name)
            .params(params)
            .attr("samples", &model.samples.to_string())
            .attr("outliers", &model.outliers.to_string())
            .values(values)
            .finish())
    }

    fn save_surrogate_model(&self, spec: &ProblemSpec, model: &Model) -> Result<()> {
        let dir = self.out.join(format!("{}/", self.problem_name));
        track!(std::fs::create_dir_all(&dir).map_err(Error::from))?;

        let spec_path = dir.join("spec.json");
        let spec_file = track!(std::fs::File::create(&spec_path).map_err(Error::from))?;
        serde_json::to_writer(spec_file, &spec)?;

        let regressor_path = dir.join("model.bin");
        let regressor_file = track!(std::fs::File::create(&regressor_path).map_err(Error::from))?;
        model.regressor.serialize(BufWriter::new(regressor_file))?;

        eprintln!("Saved the surrogate model to the direcotry {:?}", dir);
        Ok(())
    }
}

#[derive(Debug, serde::Deserialize)]
struct Trial {
    params: BTreeMap<String, f64>,
    distributions: BTreeMap<String, Distribution>,
    values: Vec<f64>,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[allow(clippy::enum_variant_names)]
enum Distribution {
    UniformDistribution { low: f64, high: f64 },
    LogUniformDistribution { low: f64, high: f64 },
    DiscreteUniformDistribution { low: f64, high: f64, q: f64 },
    IntUniformDistribution { low: i64, high: i64, step: i64 },
    IntLogUniformDistribution { low: i64, high: i64, step: i64 },
    CategoricalDistribution { choices: Vec<serde_json::Value> },
}

fn percentile(xs: impl Iterator<Item = f64>, p: f64) -> f64 {
    let mut xs = xs.collect::<Vec<_>>();
    xs.sort_by_key(|x| OrderedFloat(*x));
    xs[(xs.len() as f64 * p) as usize]
}

#[derive(Debug)]
struct Model {
    regressor: RandomForestRegressor,
    samples: usize,
    outliers: usize,
}
