use crate::filters;
use kurobako_core::filter::{BoxFilter, FilterRecipe};
use kurobako_core::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub enum KurobakoFilterRecipe {
    GaussianNoise(filters::GaussianNoiseFilterRecipe),
    DiscreteToContinuous(filters::DiscreteToContinuousFilterRecipe),
}
impl FilterRecipe for KurobakoFilterRecipe {
    type Filter = BoxFilter;

    fn create_filter(&self) -> Result<Self::Filter> {
        match self {
            KurobakoFilterRecipe::GaussianNoise(r) => track!(r.create_filter()).map(BoxFilter::new),
            KurobakoFilterRecipe::DiscreteToContinuous(r) => {
                track!(r.create_filter()).map(BoxFilter::new)
            }
        }
    }
}
