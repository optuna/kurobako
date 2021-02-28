//! Datasets management.
use kurobako_core::{Error, Result};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod surrogate;

/// Options of the `kurobako dataset` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum DatasetOpt {
    /// Dataset management for `kurobako problem nasbench`.
    Nasbench(NasbenchOpt),

    /// Dataset management for `kurobako problem hpobench`.
    Hpobench(HpobenchOpt),

    /// Builds a surrogate model problem from Optuna studies.
    SurrogateOptunaStudy(self::surrogate::SurrogateOpt),
}

impl DatasetOpt {
    /// Runs the specified dataset management command.
    pub fn run(&self) -> Result<()> {
        match self {
            Self::Nasbench(opt) => track!(opt.run()),
            Self::Hpobench(opt) => {
                opt.run();
                Ok(())
            }
            Self::SurrogateOptunaStudy(opt) => {
                track!(opt.run())
            }
        }
    }
}

/// Options of the `kurobako dataset nasbench` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum NasbenchOpt {
    /// Shows the URL of the NASBench dataset.
    Url,

    /// Converts TFRecord (nasbench_*.tfrecord) file to the binary file for kurobako.
    Convert {
        /// Input file path.
        tfrecord_format_dataset_path: PathBuf,

        /// Output file path.
        binary_format_dataset_path: PathBuf,
    },
}

impl NasbenchOpt {
    fn run(&self) -> Result<()> {
        match self {
            Self::Url => {
                println!("https://storage.googleapis.com/nasbench/nasbench_full.tfrecord");
                Ok(())
            }
            Self::Convert {
                tfrecord_format_dataset_path,
                binary_format_dataset_path,
            } => {
                eprintln!(
                    "Converting {:?}. It may take several minutes.",
                    tfrecord_format_dataset_path
                );

                let file = track!(
                    std::fs::File::open(&tfrecord_format_dataset_path).map_err(Error::from);
                    tfrecord_format_dataset_path
                )?;
                let nasbench = track!(nasbench::NasBench::from_tfrecord_reader(
                    std::io::BufReader::new(file),
                    false
                ))?;

                let file =
                    track!(std::fs::File::create(binary_format_dataset_path).map_err(Error::from))?;
                track!(nasbench.to_writer(std::io::BufWriter::new(file)))?;

                eprintln!("Done!");
                Ok(())
            }
        }
    }
}

/// Options of the `kurobako dataset hpobench` command.
#[derive(Debug, Clone, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum HpobenchOpt {
    /// Shows the URL of the datasets for the FC-Net benchmark.
    Url,
}

impl HpobenchOpt {
    fn run(&self) {
        println!("http://ml4aad.org/wp-content/uploads/2019/01/fcnet_tabular_benchmarks.tar.gz");
    }
}
