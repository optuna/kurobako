use crate::problem::KurobakoProblemRecipe;
use kurobako_core::problem::ProblemRecipe;
use kurobako_problems::sigopt::SigoptProblemRecipe;
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
        use kurobako_problems::sigopt::SigoptProblemRecipe::*;
        let specs = match self {
            SigoptProblemSuite::Nonparametric => vec![
                Ackley {
                    dim: 11,
                    res: None,
                    python: None,
                },
                Ackley {
                    dim: 3,
                    res: Some(1.0),
                    python: None,
                },
                Adjiman {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Alpine02 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                CarromTable {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Csendes {
                    dim: 2,
                    res: None,
                    python: None,
                },
                DeflectedCorrugatedSpring {
                    dim: 4,
                    res: None,
                    python: None,
                },
                DeflectedCorrugatedSpring {
                    dim: 7,
                    res: None,
                    python: None,
                },
                Easom {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Easom {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Easom {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Hartmann3 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                Hartmann6 {
                    dim: 6,
                    res: Some(10.0),
                    python: None,
                },
                HelicalValley {
                    dim: 3,
                    res: None,
                    python: None,
                },
                LennardJones6 {
                    dim: 6,
                    res: None,
                    python: None,
                },
                McCourt01 {
                    dim: 7,
                    res: Some(10.0),
                    python: None,
                },
                McCourt03 {
                    dim: 9,
                    res: None,
                    python: None,
                },
                McCourt06 {
                    dim: 5,
                    res: None,
                    python: None,
                },
                McCourt07 {
                    dim: 6,
                    res: Some(12.0),
                    python: None,
                },
                McCourt08 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                McCourt09 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                McCourt10 {
                    dim: 8,
                    res: None,
                    python: None,
                },
                McCourt11 {
                    dim: 8,
                    res: None,
                    python: None,
                },
                McCourt12 {
                    dim: 7,
                    res: None,
                    python: None,
                },
                McCourt13 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                McCourt14 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                McCourt16 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                McCourt16 {
                    dim: 4,
                    res: Some(10.0),
                    python: None,
                },
                McCourt17 {
                    dim: 7,
                    res: None,
                    python: None,
                },
                McCourt18 {
                    dim: 8,
                    res: None,
                    python: None,
                },
                McCourt19 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                McCourt20 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                McCourt23 {
                    dim: 6,
                    res: None,
                    python: None,
                },
                McCourt26 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                McCourt28 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Michalewicz {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Michalewicz {
                    dim: 4,
                    res: Some(20.0),
                    python: None,
                },
                Michalewicz {
                    dim: 8,
                    res: None,
                    python: None,
                },
                Mishra06 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Ned01 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                OddSquare {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Parsopoulos {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Pinter {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Plateau {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Problem03 {
                    dim: 1,
                    res: None,
                    python: None,
                },
                RosenbrockLog {
                    dim: 11,
                    res: None,
                    python: None,
                },
                Sargan {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Sargan {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Schwefel20 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Schwefel36 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Shekel05 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Sphere {
                    dim: 7,
                    res: None,
                    python: None,
                },
                StyblinskiTang {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Tripod {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Xor {
                    dim: 9,
                    res: None,
                    python: None,
                },
            ],
            SigoptProblemSuite::Auc => vec![
                Ackley {
                    dim: 3,
                    res: None,
                    python: None,
                },
                Ackley {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Ackley {
                    dim: 11,
                    res: None,
                    python: None,
                },
                Ackley {
                    dim: 3,
                    res: Some(1.0),
                    python: None,
                },
                Branin02 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Bukin06 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                CarromTable {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Deb02 {
                    dim: 6,
                    res: None,
                    python: None,
                },
                DeflectedCorrugatedSpring {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Easom {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Easom {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Exponential {
                    dim: 6,
                    res: None,
                    python: None,
                },
                Hartmann3 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                LennardJones6 {
                    dim: 6,
                    res: None,
                    python: None,
                },
                McCourt01 {
                    dim: 7,
                    res: Some(10.0),
                    python: None,
                },
                McCourt02 {
                    dim: 7,
                    res: None,
                    python: None,
                },
                McCourt06 {
                    dim: 5,
                    res: Some(12.0),
                    python: None,
                },
                McCourt07 {
                    dim: 6,
                    res: Some(12.0),
                    python: None,
                },
                McCourt19 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                McCourt22 {
                    dim: 5,
                    res: None,
                    python: None,
                },
                McCourt27 {
                    dim: 3,
                    res: None,
                    python: None,
                },
                Michalewicz {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Mishra06 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Ned01 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Plateau {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Rastrigin {
                    dim: 8,
                    res: None,
                    python: None,
                },
                Rastrigin {
                    dim: 8,
                    res: Some(0.1),
                    python: None,
                },
                Sargan {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Schwefel20 {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Shekel05 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Shekel07 {
                    dim: 4,
                    res: None,
                    python: None,
                },
                Sphere {
                    dim: 7,
                    res: None,
                    python: None,
                },
                StyblinskiTang {
                    dim: 5,
                    res: None,
                    python: None,
                },
                Trid {
                    dim: 6,
                    res: None,
                    python: None,
                },
                Tripod {
                    dim: 2,
                    res: None,
                    python: None,
                },
                Weierstrass {
                    dim: 3,
                    res: None,
                    python: None,
                },
                Xor {
                    dim: 9,
                    res: None,
                    python: None,
                },
                YaoLiu {
                    dim: 5,
                    res: None,
                    python: None,
                },
            ],
        };
        Box::new(specs.into_iter())
    }
}
