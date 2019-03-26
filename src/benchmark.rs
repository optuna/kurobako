use crate::optimizer::OptimizerSpec;
use crate::problems::BuiltinProblemSpec;
use crate::runner::RunSpec;
use crate::{Error, Result};
use serde::Deserialize;
use serde_json;

fn parse_json<T>(json: &str) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let v = track!(serde_json::from_str(json).map_err(Error::from))?;
    Ok(v)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimizerSpecs(Vec<OptimizerSpec>);

#[derive(Debug, Serialize, Deserialize)]
pub struct BuiltinProblemSpecs(Vec<BuiltinProblemSpec>);

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BenchmarkSpec {
    #[structopt(long, parse(try_from_str = "parse_json"))]
    pub optimizers: OptimizerSpecs,

    #[structopt(long, parse(try_from_str = "parse_json"))]
    pub problems: BuiltinProblemSpecs,

    #[structopt(long, default_value = "20")]
    pub budget: usize,

    #[structopt(long, default_value = "10")]
    pub iterations: usize,
}
impl BenchmarkSpec {
    pub fn len(&self) -> usize {
        self.optimizers.0.len() * self.problems.0.len() * self.iterations
    }

    pub fn run_specs<'a>(&'a self) -> Box<(dyn Iterator<Item = RunSpec> + 'a)> {
        Box::new(self.problems.0.iter().flat_map(move |p| {
            self.optimizers.0.iter().flat_map(move |o| {
                (0..self.iterations).map(move |_| RunSpec {
                    problem: p,
                    optimizer: o,
                    budget: self.budget,
                })
            })
        }))
    }
}
