use crate::problem::KurobakoProblemRecipe;
use kurobako_problems::hpobench;
use kurobako_problems::sigopt;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum ProblemSuite {
    Sigopt(SigoptProblemSuite),
    Hpobench(HpobenchProblemSuite),
}
impl ProblemSuite {
    pub fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        match self {
            Self::Sigopt(s) => s.recipes(),
            Self::Hpobench(s) => s.recipes(),
        }
    }
}

/// Problem suite containing problems for all datasets defined in HPOBench.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
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

/// Problem suite defined in `https://github.com/sigopt/evalset`.
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
pub enum SigoptProblemSuite {
    Nonparametric,
    Auc,
}
impl SigoptProblemSuite {
    fn recipes(&self) -> Box<dyn Iterator<Item = KurobakoProblemRecipe>> {
        use kurobako_problems::sigopt::Name::{self, *};

        fn recipe(name: Name, dim: u32, res: Option<f64>) -> sigopt::SigoptProblemRecipe {
            sigopt::SigoptProblemRecipe {
                name,
                dim: Some(dim),
                res,
            }
        }

        let specs = match self {
            SigoptProblemSuite::Nonparametric => vec![
                recipe(Ackley, 11, None),
                recipe(Ackley, 3, Some(1.0)),
                recipe(Adjiman, 2, None),
                recipe(Alpine02, 2, None),
                recipe(CarromTable, 2, None),
                recipe(Csendes, 2, None),
                recipe(DeflectedCorrugatedSpring, 4, None),
                recipe(DeflectedCorrugatedSpring, 7, None),
                recipe(Easom, 2, None),
                recipe(Easom, 4, None),
                recipe(Easom, 5, None),
                recipe(Hartmann3, 3, None),
                recipe(Hartmann6, 6, Some(10.0)),
                recipe(HelicalValley, 3, None),
                recipe(LennardJones6, 6, None),
                recipe(McCourt01, 7, Some(10.0)),
                recipe(McCourt03, 9, None),
                recipe(McCourt06, 5, None),
                recipe(McCourt07, 6, Some(12.0)),
                recipe(McCourt08, 4, None),
                recipe(McCourt09, 3, None),
                recipe(McCourt10, 8, None),
                recipe(McCourt11, 8, None),
                recipe(McCourt12, 7, None),
                recipe(McCourt13, 3, None),
                recipe(McCourt14, 3, None),
                recipe(McCourt16, 4, None),
                recipe(McCourt16, 4, Some(10.0)),
                recipe(McCourt17, 7, None),
                recipe(McCourt18, 8, None),
                recipe(McCourt19, 2, None),
                recipe(McCourt20, 2, None),
                recipe(McCourt23, 6, None),
                recipe(McCourt26, 3, None),
                recipe(McCourt28, 4, None),
                recipe(Michalewicz, 4, None),
                recipe(Michalewicz, 4, Some(20.0)),
                recipe(Michalewicz, 8, None),
                recipe(Mishra06, 2, None),
                recipe(Ned01, 2, None),
                recipe(OddSquare, 2, None),
                recipe(Parsopoulos, 2, None),
                recipe(Pinter, 2, None),
                recipe(Plateau, 2, None),
                recipe(Problem03, 1, None),
                recipe(RosenbrockLog, 11, None),
                recipe(Sargan, 5, None),
                recipe(Sargan, 2, None),
                recipe(Schwefel20, 2, None),
                recipe(Schwefel36, 2, None),
                recipe(Shekel05, 4, None),
                recipe(Sphere, 7, None),
                recipe(StyblinskiTang, 5, None),
                recipe(Tripod, 2, None),
                recipe(Xor, 9, None),
            ],
            SigoptProblemSuite::Auc => vec![
                recipe(Ackley, 3, None),
                recipe(Ackley, 5, None),
                recipe(Ackley, 11, None),
                recipe(Ackley, 3, Some(1.0)),
                recipe(Branin02, 2, None),
                recipe(Bukin06, 2, None),
                recipe(CarromTable, 2, None),
                recipe(Deb02, 6, None),
                recipe(DeflectedCorrugatedSpring, 4, None),
                recipe(Easom, 4, None),
                recipe(Easom, 5, None),
                recipe(Exponential, 6, None),
                recipe(Hartmann3, 3, None),
                recipe(LennardJones6, 6, None),
                recipe(McCourt01, 7, Some(10.0)),
                recipe(McCourt02, 7, None),
                recipe(McCourt06, 5, Some(12.0)),
                recipe(McCourt07, 6, Some(12.0)),
                recipe(McCourt19, 2, None),
                recipe(McCourt22, 5, None),
                recipe(McCourt27, 3, None),
                recipe(Michalewicz, 4, None),
                recipe(Mishra06, 2, None),
                recipe(Ned01, 2, None),
                recipe(Plateau, 2, None),
                recipe(Rastrigin, 8, None),
                recipe(Rastrigin, 8, Some(0.1)),
                recipe(Sargan, 2, None),
                recipe(Schwefel20, 2, None),
                recipe(Shekel05, 4, None),
                recipe(Shekel07, 4, None),
                recipe(Sphere, 7, None),
                recipe(StyblinskiTang, 5, None),
                recipe(Trid, 6, None),
                recipe(Tripod, 2, None),
                recipe(Weierstrass, 3, None),
                recipe(Xor, 9, None),
                recipe(YaoLiu, 5, None),
            ],
        };
        Box::new(specs.into_iter().map(KurobakoProblemRecipe::from))
    }
}
