use crate::problem::KurobakoProblemRecipe;
use kurobako_core::problem::ProblemRecipe;
use kurobako_problems::sigopt::{Name, SigoptProblemRecipe};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub trait ProblemSuite {
    type ProblemRecipe: ProblemRecipe;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemRecipe>>;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoProblemSuite {
    Sigopt(SigoptProblemSuite),
}
impl ProblemSuite for KurobakoProblemSuite {
    type ProblemRecipe = KurobakoProblemRecipe;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemRecipe>> {
        match self {
            KurobakoProblemSuite::Sigopt(p) => {
                Box::new(p.problem_specs().map(KurobakoProblemRecipe::Sigopt))
            }
        }
    }
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum SigoptProblemSuite {
    Nonparametric,
    Auc,
}
impl ProblemSuite for SigoptProblemSuite {
    type ProblemRecipe = SigoptProblemRecipe;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemRecipe>> {
        use kurobako_problems::sigopt::Name::*;

        fn recipe(name: Name, dim: u32, res: Option<f64>) -> SigoptProblemRecipe {
            SigoptProblemRecipe {
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
        Box::new(specs.into_iter())
    }
}
