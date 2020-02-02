//! A problem based on the benchmark defined by [sigopt/evalset].
//!
//! # Note
//!
//! Currently, only a part of the functions defined in [sigopt/evalset] are implemented.
//! If you want to use an unimplemented function, please create an issue or PR.
//!
//! [sigopt/evalset]: https://github.com/sigopt/evalset
use self::functions::TestFunction;
use kurobako_core::domain;
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod bessel;
mod functions;

/// Recipe of `SigoptProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct SigoptProblemRecipe {
    /// Test function name.
    #[structopt(subcommand)]
    pub name: Name,

    /// Dimension of the test function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub dim: Option<usize>,

    /// Input resolution of the test function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub res: Option<f64>,

    /// List of the dimensions which should only accept integer values.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[structopt(long)]
    pub int: Vec<usize>,
}
impl ProblemRecipe for SigoptProblemRecipe {
    type Factory = SigoptProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let test_function = self.name.to_test_function();
        Ok(SigoptProblemFactory {
            name: self.name,
            dim: self
                .dim
                .unwrap_or_else(|| test_function.default_dimension()),
            res: self.res,
            int: self.int.clone(),
        })
    }
}

/// Factory of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptProblemFactory {
    name: Name,
    dim: usize,
    res: Option<f64>,
    int: Vec<usize>,
}
impl ProblemFactory for SigoptProblemFactory {
    type Problem = SigoptProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let test_function = self.name.to_test_function();

        let problem_name = if let Some(res) = self.res {
            format!(
                "sigopt/evalset/{:?}(dim={}, res={})",
                self.name, self.dim, res
            )
        } else {
            format!("sigopt/evalset/{:?}(dim={})", self.name, self.dim)
        };
        let paper = "Dewancker, Ian, et al. \"A strategy for ranking optimization methods using multiple criteria.\" Workshop on Automatic Machine Learning. 2016.";

        let mut spec = ProblemSpecBuilder::new(&problem_name)
            .attr(
                "version",
                &format!("kurobako_problems={}", env!("CARGO_PKG_VERSION")),
            )
            .attr("paper", paper)
            .attr("github", "https://github.com/sigopt/evalset");

        for (i, (low, high)) in track!(test_function.bounds(self.dim))?
            .into_iter()
            .enumerate()
        {
            let var = domain::var(&format!("p{}", i));
            if self.int.contains(&i) {
                let low = low.ceil() as i64;
                let high = high.floor() as i64;
                spec = spec.param(var.discrete(low, high));
            } else {
                spec = spec.param(var.continuous(low, high));
            }
        }

        track!(spec.value(domain::var("Objective Value")).finish())
    }

    fn create_problem(&self, _rng: ArcRng) -> Result<Self::Problem> {
        Ok(SigoptProblem {
            name: self.name,
            res: self.res,
        })
    }
}

/// Problem that uses the test functions defined in [sigopt/evalset](https://github.com/sigopt/evalset).
#[derive(Debug)]
pub struct SigoptProblem {
    name: Name,
    res: Option<f64>,
}
impl Problem for SigoptProblem {
    type Evaluator = SigoptEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        Ok(SigoptEvaluator {
            res: self.res,
            test_function: self.name.to_test_function(),
            params,
        })
    }
}

/// Evaluator of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptEvaluator {
    res: Option<f64>,
    test_function: Box<dyn TestFunction>,
    params: Params,
}
impl Evaluator for SigoptEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        track_assert_eq!(next_step, 1, ErrorKind::Bug);

        let mut value = self.test_function.evaluate(self.params.get());
        if let Some(res) = self.res {
            value = (value * res).floor() / res;
        }

        Ok((1, Values::new(vec![value])))
    }
}

/// Test function name.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, StructOpt, Serialize, Deserialize,
)]
#[allow(missing_docs)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Name {
    Ackley,
    Adjiman,
    Alpine01,
    Alpine02,
    ArithmeticGeometricMean,
    BartelsConn,
    Beale,
    Bird,
    Bohachevsky,
    BoxBetts,
    Branin01,
    Branin02,
    Brent,
    Brown,
    Bukin06,
    CarromTable,
    Chichinadze,
    Cigar,
    Cola,
    Corana,
    CosineMixture,
    CrossInTray,
    Csendes,
    Cube,
    Damavandi,
    Deb01,
    Deb02,
    Deceptive,
    DeflectedCorrugatedSpring,
    Dolan,
    DropWave,
    Easom,
    EggCrate,
    EggHolder,
    ElAttarVidyasagarDutta,
    Exponential,
    Franke,
    FreudensteinRoth,
    Gear,
    Giunta,
    GoldsteinPrice,
    Griewank,
    Hansen,
    Hartmann3,
    Hartmann4,
    Hartmann6,
    HelicalValley,
    HimmelBlau,
    HolderTable,
    Hosaki,
    HosakiExpanded,
    JennrichSampson,
    Judge,
    Keane,
    Langermann,
    LennardJones6,
    Leon,
    Levy03,
    Levy05,
    Levy13,
    Matyas,
    McCormick,
    McCourt01,
    McCourt02,
    McCourt03,
    McCourt04,
    McCourt05,
    McCourt06,
    McCourt07,
    McCourt08,
    McCourt09,
    McCourt10,
    McCourt11,
    McCourt12,
    McCourt13,
    McCourt14,
    McCourt15,
    McCourt16,
    McCourt17,
    McCourt18,
    McCourt19,
    McCourt20,
    McCourt21,
    McCourt22,
    McCourt23,
    McCourt24,
    McCourt25,
    McCourt26,
    McCourt27,
    McCourt28,
    MegaDomain01,
    MegaDomain02,
    MegaDomain03,
    MegaDomain04,
    MegaDomain05,
    Michalewicz,
    MieleCantrell,
    Mishra02,
    Mishra06,
    Mishra08,
    Mishra10,
    ManifoldMin,
    MixtureOfGaussians01,
    MixtureOfGaussians02,
    MixtureOfGaussians03,
    MixtureOfGaussians04,
    MixtureOfGaussians05,
    MixtureOfGaussians06,
    Ned01,
    Ned03,
    OddSquare,
    Parsopoulos,
    Pavianini,
    Penalty01,
    Penalty02,
    PenHolder,
    Perm01,
    Perm02,
    Pinter,
    Plateau,
    Powell,
    PowellTripleLog,
    PowerSum,
    Price,
    Qing,
    Quadratic,
    Rastrigin,
    RippleSmall,
    RippleBig,
    RosenbrockLog,
    RosenbrockModified,
    Salomon,
    Sargan,
    Schaffer,
    SchmidtVetters,
    Schwefel01,
    Schwefel06,
    Schwefel20,
    Schwefel22,
    Schwefel26,
    Schwefel36,
    Shekel05,
    Shekel07,
    Shekel10,
    Shubert01,
    Shubert03,
    SineEnvelope,
    SixHumpCamel,
    Sphere,
    Step,
    StretchedV,
    StyblinskiTang,
    SumPowers,
    TestTubeHolder,
    ThreeHumpCamel,
    Trefethen,
    Trid,
    Tripod,
    Ursem01,
    Ursem03,
    Ursem04,
    UrsemWaves,
    VenterSobiezcczanskiSobieski,
    Watson,
    Weierstrass,
    Wolfe,
    XinSheYang02,
    XinSheYang03,
    Xor,
    YaoLiu,
    ZeroSum,
    Zimmerman,
    Problem02,
    Problem03,
    Problem04,
    Problem05,
    Problem06,
    Problem07,
    Problem09,
    Problem10,
    Problem11,
    Problem12,
    Problem13,
    Problem14,
    Problem15,
    Problem18,
    Problem20,
    Problem21,
    Problem22,
}
impl Name {
    fn to_test_function(self) -> Box<dyn TestFunction> {
        match self {
            Self::Ackley => Box::new(functions::Ackley),
            _ => todo!(),
        }
    }
}
