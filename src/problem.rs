//! The problem for `kurobako`.
use kurobako_core::epi::problem::ExternalProgramProblemRecipe;
use kurobako_core::problem::{
    BoxProblem, BoxProblemFactory, ProblemFactory, ProblemRecipe, ProblemSpec,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::Result;
use kurobako_problems::{hpobench, nasbench, sigopt};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

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

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "snake_case")]
enum InnerRecipe {
    Command(ExternalProgramProblemRecipe),
    Sigopt(sigopt::SigoptProblemRecipe),
    Nasbench(nasbench::NasbenchProblemRecipe),
    Hpobench(hpobench::HpobenchProblemRecipe),
}
impl ProblemRecipe for InnerRecipe {
    type Factory = BoxProblemFactory;

    fn create_factory(&self, registry: &FactoryRegistry) -> Result<Self::Factory> {
        match self {
            Self::Command(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Sigopt(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Nasbench(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
            Self::Hpobench(p) => track!(p.create_factory(registry).map(BoxProblemFactory::new)),
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
