//! A problem based on the benchmark defined by [sigopt/evalset](https://github.com/sigopt/evalset).
use kurobako_core::epi::problem::{
    EmbeddedScriptEvaluator, EmbeddedScriptProblem, EmbeddedScriptProblemFactory,
    EmbeddedScriptProblemRecipe,
};
use kurobako_core::problem::{Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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
    pub dim: Option<u32>,

    /// Input resolution of the test function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub res: Option<f64>,
}
impl ProblemRecipe for SigoptProblemRecipe {
    type Factory = SigoptProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let script = include_str!("../scripts/sigopt_problem.py");
        let mut args = vec![format!("{:?}", self.name)];
        if let Some(dim) = self.dim {
            args.extend_from_slice(&["--dim".to_owned(), dim.to_string()]);
        }
        if let Some(res) = self.res {
            args.extend_from_slice(&["--res".to_owned(), res.to_string()]);
        }

        let recipe = EmbeddedScriptProblemRecipe {
            script: script.to_owned(),
            args,
        };
        let inner = track!(recipe.create_factory(registry))?;
        Ok(SigoptProblemFactory { inner })
    }
}

/// Factory of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptProblemFactory {
    inner: EmbeddedScriptProblemFactory,
}
impl ProblemFactory for SigoptProblemFactory {
    type Problem = SigoptProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        track!(self.inner.specification())
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        let inner = track!(self.inner.create_problem(rng))?;
        Ok(SigoptProblem { inner })
    }
}

/// Problem that uses the test functions defined in [sigopt/evalset](https://github.com/sigopt/evalset).
#[derive(Debug)]
pub struct SigoptProblem {
    inner: EmbeddedScriptProblem,
}
impl Problem for SigoptProblem {
    type Evaluator = SigoptEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        let inner = track!(self.inner.create_evaluator(params))?;
        Ok(SigoptEvaluator { inner })
    }
}

/// Evaluator of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptEvaluator {
    inner: EmbeddedScriptEvaluator,
}
impl Evaluator for SigoptEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        track!(self.inner.evaluate(next_step))
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
