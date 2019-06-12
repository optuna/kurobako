// https://github.com/automl/nas_benchmarks/blob/master/tabular_benchmarks/fcnet_benchmark.py
use hdf5file::level2::DataObject;
use hdf5file::{self, Hdf5File};
use kurobako_core::parameter::{choices, int, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{Error, ErrorKind, Result};
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs::File;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::rc::Rc;
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

fn into_error(e: hdf5file::Error) -> Error {
    use trackable::error::ErrorKindExt as _;

    ErrorKind::Other.takes_over(e).into()
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct FcNetProblemRecipe {
    pub dataset_path: PathBuf,
}
impl ProblemRecipe for FcNetProblemRecipe {
    type Problem = FcNetProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let file = track!(File::open(&self.dataset_path).map_err(Error::from); self.dataset_path)?;
        let file = track!(Hdf5File::new(file).map_err(into_error))?;
        Ok(FcNetProblem {
            file: Rc::new(RefCell::new(file)),
            name: track_assert_some!(
                self.dataset_path.file_stem().and_then(|n| n.to_str()),
                ErrorKind::InvalidInput
            )
            .to_owned(),
        })
    }
}

#[derive(Debug)]
pub struct FcNetProblem {
    file: Rc<RefCell<Hdf5File<File>>>,
    name: String,
}
impl Problem for FcNetProblem {
    type Evaluator = FcNetEvaluator;

    fn specification(&self) -> ProblemSpec {
        let params_domain = vec![
            choices("activation_fn_1", &["tanh", "relu"]),
            choices("activation_fn_2", &["tanh", "relu"]),
            int("batch_size", 0, 4).unwrap(),
            int("dropout_1", 0, 3).unwrap(),
            int("dropout_2", 0, 3).unwrap(),
            int("init_lr", 0, 6).unwrap(),
            choices("lr_schedule", &["cosine", "const"]),
            int("n_units_1", 0, 6).unwrap(),
            int("n_units_2", 0, 6).unwrap(),
        ];

        ProblemSpec {
            name: self.name.clone(),
            version: None,
            params_domain,
            values_domain: unsafe {
                vec![MinMax::new_unchecked(
                    FiniteF64::new_unchecked(0.0),
                    FiniteF64::new_unchecked(1.0),
                )]
            },
            evaluation_expense: unsafe { NonZeroU64::new_unchecked(100) },
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        Ok(FcNetEvaluator {
            file: self.file.clone(),
            sample_index: id.get() as usize % 4,
        })
    }
}

#[derive(Debug)]
pub struct FcNetEvaluator {
    file: Rc<RefCell<Hdf5File<File>>>,
    sample_index: usize,
}
impl Evaluate for FcNetEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        const UNITS: [usize; 6] = [16, 32, 64, 128, 256, 512];
        const DROPOUTS: [&str; 3] = ["0.0", "0.3", "0.6"];

        fn index(p: &ParamValue) -> usize {
            p.as_discrete().unwrap() as usize
        }

        let key = format!(
            r#"{{"activation_fn_1": {:?}, "activation_fn_2": {:?}, "batch_size": {}, "dropout_1": {}, "dropout_2": {}, "init_lr": {}, "lr_schedule": {:?}, "n_units_1": {}, "n_units_2": {}}}"#,
            (["tanh", "relu"])[params[0].as_categorical().unwrap()],
            (["tanh", "relu"])[params[1].as_categorical().unwrap()],
            ([8, 16, 32, 64])[index(&params[2])],
            DROPOUTS[index(&params[3])],
            DROPOUTS[index(&params[4])],
            ([5.0 * 1e-4, 1e-3, 5.0 * 1e-3, 1e-2, 5.0 * 1e-2, 1e-1])[index(&params[5])],
            (["cosine", "const"])[params[6].as_categorical().unwrap()],
            UNITS[index(&params[7])],
            UNITS[index(&params[8])]
        );

        let data = track!(self
            .file
            .borrow_mut()
            .get_object(format!("/{}/valid_mse", key))
            .map_err(into_error))?;
        let DataObject::Float(data) = track_assert_some!(data, ErrorKind::InvalidInput; key);

        let value = data[[self.sample_index, budget.amount as usize - 1]];
        budget.consumption = budget.amount;
        Ok(vec![FiniteF64::new(value)?])
    }
}
