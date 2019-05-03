use crate::problems::BuiltinProblemSpec;
use crate::ProblemSpec;
use kurobako_problems::problems::sigopt::SigoptProblemSpec;
use serde::{Deserialize, Serialize};

pub trait ProblemSuite {
    type ProblemSpec: ProblemSpec;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemSpec>>;
}

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum BuiltinProblemSuite {
    Sigopt(SigoptProblemSuite),
}
impl ProblemSuite for BuiltinProblemSuite {
    type ProblemSpec = BuiltinProblemSpec;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemSpec>> {
        match self {
            BuiltinProblemSuite::Sigopt(p) => {
                Box::new(p.problem_specs().map(BuiltinProblemSpec::Sigopt))
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
    type ProblemSpec = SigoptProblemSpec;

    fn problem_specs(&self) -> Box<dyn Iterator<Item = Self::ProblemSpec>> {
        use kurobako_problems::problems::sigopt::SigoptProblemSpec::*;
        let specs = match self {
            SigoptProblemSuite::Nonparametric => vec![
                Ackley { dim: 11, res: None },
                Ackley {
                    dim: 3,
                    res: Some(1.0),
                },
                Adjiman { dim: 2, res: None },
                Alpine02 { dim: 2, res: None },
                CarromTable { dim: 2, res: None },
                Csendes { dim: 2, res: None },
                DeflectedCorrugatedSpring { dim: 4, res: None },
                DeflectedCorrugatedSpring { dim: 7, res: None },
                Easom { dim: 2, res: None },
                Easom { dim: 4, res: None },
                Easom { dim: 5, res: None },
                Hartmann3 { dim: 3, res: None },
                Hartmann6 {
                    dim: 6,
                    res: Some(10.0),
                },
                HelicalValley { dim: 3, res: None },
                LennardJones6 { dim: 6, res: None },
                McCourt01 {
                    dim: 7,
                    res: Some(10.0),
                },
                McCourt03 { dim: 9, res: None },
                McCourt06 { dim: 5, res: None },
                McCourt07 {
                    dim: 6,
                    res: Some(12.0),
                },
                McCourt08 { dim: 4, res: None },
                McCourt09 { dim: 3, res: None },
                McCourt10 { dim: 8, res: None },
                McCourt11 { dim: 8, res: None },
                McCourt12 { dim: 7, res: None },
                McCourt13 { dim: 3, res: None },
                McCourt14 { dim: 3, res: None },
                McCourt16 { dim: 4, res: None },
                McCourt16 {
                    dim: 4,
                    res: Some(10.0),
                },
                McCourt17 { dim: 7, res: None },
                McCourt18 { dim: 8, res: None },
                McCourt19 { dim: 2, res: None },
                McCourt20 { dim: 2, res: None },
                McCourt23 { dim: 6, res: None },
                McCourt26 { dim: 3, res: None },
                McCourt28 { dim: 4, res: None },
                Michalewicz { dim: 4, res: None },
                Michalewicz {
                    dim: 4,
                    res: Some(20.0),
                },
                Michalewicz { dim: 8, res: None },
                Mishra06 { dim: 2, res: None },
                Ned01 { dim: 2, res: None },
                OddSquare { dim: 2, res: None },
                Parsopoulos { dim: 2, res: None },
                Pinter { dim: 2, res: None },
                Plateau { dim: 2, res: None },
                Problem03 { dim: 1, res: None },
                RosenbrockLog { dim: 11, res: None },
                Sargan { dim: 5, res: None },
                Sargan { dim: 2, res: None },
                Schwefel20 { dim: 2, res: None },
                Schwefel36 { dim: 2, res: None },
                Shekel05 { dim: 4, res: None },
                Sphere { dim: 7, res: None },
                StyblinskiTang { dim: 5, res: None },
                Tripod { dim: 2, res: None },
                Xor { dim: 9, res: None },
            ],
            SigoptProblemSuite::Auc => vec![
                Ackley { dim: 3, res: None },
                Ackley { dim: 5, res: None },
                Ackley { dim: 11, res: None },
                Ackley {
                    dim: 3,
                    res: Some(1.0),
                },
                Branin02 { dim: 2, res: None },
                Bukin06 { dim: 2, res: None },
                CarromTable { dim: 2, res: None },
                Deb02 { dim: 6, res: None },
                DeflectedCorrugatedSpring { dim: 4, res: None },
                Easom { dim: 4, res: None },
                Easom { dim: 5, res: None },
                Exponential { dim: 6, res: None },
                Hartmann3 { dim: 3, res: None },
                LennardJones6 { dim: 6, res: None },
                McCourt01 {
                    dim: 7,
                    res: Some(10.0),
                },
                McCourt02 { dim: 7, res: None },
                McCourt06 {
                    dim: 5,
                    res: Some(12.0),
                },
                McCourt07 {
                    dim: 6,
                    res: Some(12.0),
                },
                McCourt19 { dim: 2, res: None },
                McCourt22 { dim: 5, res: None },
                McCourt27 { dim: 3, res: None },
                Michalewicz { dim: 4, res: None },
                Mishra06 { dim: 2, res: None },
                Ned01 { dim: 2, res: None },
                Plateau { dim: 2, res: None },
                Rastrigin { dim: 8, res: None },
                Rastrigin {
                    dim: 8,
                    res: Some(0.1),
                },
                Sargan { dim: 2, res: None },
                Schwefel20 { dim: 2, res: None },
                Shekel05 { dim: 4, res: None },
                Shekel07 { dim: 4, res: None },
                Sphere { dim: 7, res: None },
                StyblinskiTang { dim: 5, res: None },
                Trid { dim: 6, res: None },
                Tripod { dim: 2, res: None },
                Weierstrass { dim: 3, res: None },
                Xor { dim: 9, res: None },
                YaoLiu { dim: 5, res: None },
            ],
        };
        Box::new(specs.into_iter())
    }
}
