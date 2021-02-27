//! The problem for `kurobako`.
use kurobako_core::epi::problem::ExternalProgramProblemRecipe;
use kurobako_core::problem::{
    BoxProblem, BoxProblemFactory, ProblemFactory, ProblemRecipe, ProblemSpec,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::Result;
use kurobako_problems::{hpobench, nasbench, sigopt, surrogate, warm_starting, zdt};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod average;
mod ln;
mod rank;
mod study;

/// Problem recipe.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct KurobakoProblemRecipe {
    #[structopt(long)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,

    #[structopt(flatten)]
    #[serde(flatten)]
    inner: InnerRecipe,
}
impl ProblemRecipe for KurobakoProblemRecipe {
    type Factory = KurobakoProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        let inner = track!(self.inner.create_factory(registry))?;
        Ok(KurobakoProblemFactory {
            name: self.name.clone(),
            inner,
        })
    }
}
impl From<hpobench::HpobenchProblemRecipe> for KurobakoProblemRecipe {
    fn from(f: hpobench::HpobenchProblemRecipe) -> Self {
        Self {
            name: None,
            inner: InnerRecipe::Hpobench(f),
        }
    }
}
impl From<sigopt::SigoptProblemRecipe> for KurobakoProblemRecipe {
    fn from(f: sigopt::SigoptProblemRecipe) -> Self {
        Self {
            name: None,
            inner: InnerRecipe::Sigopt(f),
        }
    }
}
impl From<zdt::ZdtProblemRecipe> for KurobakoProblemRecipe {
    fn from(f: zdt::ZdtProblemRecipe) -> Self {
        Self {
            name: None,
            inner: InnerRecipe::Zdt(f),
        }
    }
}
impl From<surrogate::SurrogateProblemRecipe> for KurobakoProblemRecipe {
    fn from(f: surrogate::SurrogateProblemRecipe) -> Self {
        Self {
            name: None,
            inner: InnerRecipe::Surrogate(f),
        }
    }
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "snake_case")]
enum InnerRecipe {
    Command(ExternalProgramProblemRecipe),
    /// Recipe of `SigoptProblem`.
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
    Hpobench(hpobench::HpobenchProblemRecipe),
    Zdt(zdt::ZdtProblemRecipe),
    Surrogate(surrogate::SurrogateProblemRecipe),
    Study(self::study::StudyProblemRecipe),
    Rank(self::rank::RankProblemRecipe),
    Average(self::average::AverageProblemRecipe),
    Ln(self::ln::LnProblemRecipe),
    WarmStarting(warm_starting::WarmStartingProblemRecipe),
}
impl ProblemRecipe for InnerRecipe {
    type Factory = BoxProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        match self {
            Self::Command(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Sigopt(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Nasbench(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Hpobench(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Zdt(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Surrogate(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Study(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Rank(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Average(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Ln(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::WarmStarting(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
        }
    }
}

/// Problem factory.
#[derive(Debug)]
pub struct KurobakoProblemFactory {
    name: Option<String>,
    inner: BoxProblemFactory,
}
impl ProblemFactory for KurobakoProblemFactory {
    type Problem = BoxProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let mut spec = track!(self.inner.specification())?;
        if let Some(name) = &self.name {
            spec.name = name.clone();
        }
        Ok(spec)
    }

    fn create_problem(&self, rng: ArcRng) -> Result<Self::Problem> {
        track!(self.inner.create_problem(rng)).map(BoxProblem::new)
    }
}
