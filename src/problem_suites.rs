//! Built-in problem suites.
use crate::problem::KurobakoProblemRecipe;
use kurobako_problems::{hpobench, sigopt, surrogate, zdt};
use std::path::PathBuf;
use structopt::StructOpt;

/// Problem suite.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum ProblemSuite {
    Sigopt(SigoptProblemSuite),
    Hpobench(HpobenchProblemSuite),
    Zdt(ZdtProblemSuite),
    Surrogate(SurrogateProblemSuite),
}
impl ProblemSuite {
    /// Returns an iterator that iterates over the recipes included in the specified problem suite.
    pub fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        match self {
            Self::Sigopt(s) => s.recipes(),
            Self::Hpobench(s) => s.recipes(),
            Self::Zdt(s) => s.recipes(),
            Self::Surrogate(s) => s.recipes(),
        }
    }
}

/// Problem suite containing problems for all datasets defined in HPOBench.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum HpobenchProblemSuite {
    Fcnet { dataset_dir: PathBuf },
}
impl HpobenchProblemSuite {
    fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        match self {
            Self::Fcnet { dataset_dir } => {
                let recipe = |name| hpobench::HpobenchProblemRecipe {
                    dataset: dataset_dir.join(name),
                };
                let recipes = vec![
                    recipe("fcnet_naval_propulsion_data.hdf5"),
                    recipe("fcnet_parkinsons_telemonitoring_data.hdf5"),
                    recipe("fcnet_protein_structure_data.hdf5"),
                    recipe("fcnet_slice_localization_data.hdf5"),
                ];
                Box::new(recipes.into_iter().map(KurobakoProblemRecipe::from))
            }
        }
    }
}

/// Problem suite containing problems for all the ZDT functions.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct ZdtProblemSuite {}
impl ZdtProblemSuite {
    fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        Box::new(
            vec![
                zdt::Zdt::Function1,
                zdt::Zdt::Function2,
                zdt::Zdt::Function3,
                zdt::Zdt::Function4,
                zdt::Zdt::Function5,
                zdt::Zdt::Function6,
            ]
            .into_iter()
            .map(|zdt| KurobakoProblemRecipe::from(zdt::ZdtProblemRecipe { zdt })),
        )
    }
}

/// Problem suite defined in `https://github.com/sigopt/evalset`.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub enum SigoptProblemSuite {
    Nonparametric,
    Auc,
}
impl SigoptProblemSuite {
    fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        use kurobako_problems::sigopt::Name::{self, *};

        fn recipe(
            name: Name,
            dim: usize,
            int: Option<Vec<usize>>,
            res: Option<f64>,
        ) -> sigopt::SigoptProblemRecipe {
            sigopt::SigoptProblemRecipe {
                name,
                dim: Some(dim),
                res,
                int: int.unwrap_or_default(),
            }
        }

        let specs = match self {
            SigoptProblemSuite::Nonparametric => vec![
                recipe(Ackley, 11, None, None),
                recipe(Ackley, 3, None, Some(1.0)),
                recipe(Adjiman, 2, None, None),
                recipe(Alpine02, 2, Some(vec![0]), None),
                recipe(CarromTable, 2, Some(vec![0]), None),
                recipe(Csendes, 2, None, None),
                recipe(DeflectedCorrugatedSpring, 4, None, None),
                recipe(DeflectedCorrugatedSpring, 7, None, None),
                recipe(Easom, 2, None, None),
                recipe(Easom, 4, None, None),
                recipe(Easom, 5, None, None),
                recipe(Hartmann3, 3, Some(vec![0]), None),
                recipe(Hartmann6, 6, None, Some(10.0)),
                recipe(HelicalValley, 3, None, None),
                recipe(LennardJones6, 6, None, None),
                recipe(McCourt01, 7, None, Some(10.0)),
                recipe(McCourt03, 9, None, None),
                recipe(McCourt06, 5, None, None),
                recipe(McCourt07, 6, None, Some(12.0)),
                recipe(McCourt08, 4, None, None),
                recipe(McCourt09, 3, None, None),
                recipe(McCourt10, 8, None, None),
                recipe(McCourt11, 8, None, None),
                recipe(McCourt12, 7, None, None),
                recipe(McCourt13, 3, None, None),
                recipe(McCourt14, 3, None, None),
                recipe(McCourt16, 4, None, None),
                recipe(McCourt16, 4, None, Some(10.0)),
                recipe(McCourt17, 7, None, None),
                recipe(McCourt18, 8, None, None),
                recipe(McCourt19, 2, None, None),
                recipe(McCourt20, 2, None, None),
                recipe(McCourt23, 6, None, None),
                recipe(McCourt26, 3, None, None),
                recipe(McCourt28, 4, None, None),
                recipe(Michalewicz, 4, None, None),
                recipe(Michalewicz, 4, None, Some(20.0)),
                recipe(Michalewicz, 8, None, None),
                recipe(Mishra06, 2, None, None),
                recipe(Ned01, 2, None, None),
                recipe(OddSquare, 2, None, None),
                recipe(Parsopoulos, 2, Some(vec![0]), None),
                recipe(Pinter, 2, Some(vec![0, 1]), None),
                recipe(Plateau, 2, None, None),
                recipe(Problem03, 1, None, None),
                recipe(RosenbrockLog, 11, None, None),
                recipe(Sargan, 5, None, None),
                recipe(Sargan, 2, Some(vec![0]), None),
                recipe(Schwefel20, 2, None, None),
                recipe(Schwefel20, 2, Some(vec![0]), None),
                recipe(Schwefel36, 2, None, None),
                recipe(Shekel05, 4, None, None),
                recipe(Sphere, 7, Some(vec![0, 1, 2, 3, 4]), None),
                recipe(StyblinskiTang, 5, None, None),
                recipe(Tripod, 2, None, None),
                recipe(Xor, 9, None, None),
            ],
            SigoptProblemSuite::Auc => vec![
                recipe(Ackley, 3, None, None),
                recipe(Ackley, 5, None, None),
                recipe(Ackley, 11, None, None),
                recipe(Ackley, 3, None, Some(1.0)),
                recipe(Ackley, 11, Some(vec![0, 1, 2]), None),
                recipe(Branin02, 2, Some(vec![0]), None),
                recipe(Bukin06, 2, Some(vec![0]), None),
                recipe(CarromTable, 2, None, None),
                recipe(CarromTable, 2, Some(vec![0]), None),
                recipe(Deb02, 6, None, None),
                recipe(DeflectedCorrugatedSpring, 4, None, None),
                recipe(Easom, 4, None, None),
                recipe(Easom, 5, None, None),
                recipe(Exponential, 6, None, None),
                recipe(Hartmann3, 3, None, None),
                recipe(LennardJones6, 6, None, None),
                recipe(McCourt01, 7, None, Some(10.0)),
                recipe(McCourt02, 7, None, None),
                recipe(McCourt06, 5, None, Some(12.0)),
                recipe(McCourt07, 6, None, Some(12.0)),
                recipe(McCourt19, 2, None, None),
                recipe(McCourt22, 5, None, None),
                recipe(McCourt27, 3, None, None),
                recipe(Michalewicz, 4, None, None),
                recipe(Mishra06, 2, None, None),
                recipe(Ned01, 2, None, None),
                recipe(Plateau, 2, None, None),
                recipe(Rastrigin, 8, None, None),
                recipe(Rastrigin, 8, None, Some(0.1)),
                recipe(Sargan, 2, Some(vec![0]), None),
                recipe(Schwefel20, 2, None, None),
                recipe(Schwefel20, 2, Some(vec![0]), None),
                recipe(Shekel05, 4, None, None),
                recipe(Shekel07, 4, None, None),
                recipe(Sphere, 7, None, None),
                recipe(Sphere, 7, Some(vec![0, 1, 2, 3, 4]), None),
                recipe(StyblinskiTang, 5, None, None),
                recipe(Trid, 6, None, None),
                recipe(Tripod, 2, None, None),
                recipe(Weierstrass, 3, None, None),
                recipe(Xor, 9, None, None),
                recipe(YaoLiu, 5, None, None),
            ],
        };
        Box::new(specs.into_iter().map(KurobakoProblemRecipe::from))
    }
}

/// Problem suite containing surrogate problems defined under a directory.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
#[allow(missing_docs)]
pub struct SurrogateProblemSuite {
    /// Directory path under where target surrogated model directories exist.
    pub dir: PathBuf,

    /// Disable the in-memory model cache to reduce memory usage.
    #[structopt(long)]
    pub disable_cache: bool,
}
impl SurrogateProblemSuite {
    fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        let disable_cache = self.disable_cache;
        match std::fs::read_dir(&self.dir) {
            Err(e) => {
                eprintln!("Cannot read the directory {:?}: {}", self.dir, e);
                Box::new(std::iter::empty())
            }
            Ok(entries) => Box::new(entries.filter_map(move |entry| match entry {
                Err(e) => {
                    eprintln!("Wrong entry: {}", e);
                    None
                }
                Ok(entry) => {
                    if entry.path().is_dir() {
                        Some(KurobakoProblemRecipe::from(
                            surrogate::SurrogateProblemRecipe {
                                model: entry.path(),
                                disable_cache,
                            },
                        ))
                    } else {
                        None
                    }
                }
            })),
        }
    }
}
