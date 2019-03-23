use serde_json::Value as JsonValue;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BenchmarkSpec {
    #[structopt(long)]
    pub optimizers: JsonValue,

    #[structopt(long)]
    pub problems: JsonValue, // TODO: Vec<BuiltinProblemSpec> (impl FromStr)

    #[structopt(long, default_value = "20")]
    pub budget: usize,

    #[structopt(long, default_value = "10")]
    pub iterations: usize,
}
