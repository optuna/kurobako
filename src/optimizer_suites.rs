use crate::optimizer::{OptimizerSpec, OptunaOptimizerBuilder, RandomOptimizerBuilder};

pub trait OptimizerSuite {
    //    type OptimizerSpec: OptimizerSpec;

    fn suite(&self) -> Box<dyn Iterator<Item = OptimizerSpec>>;
}

// TODO
#[derive(Debug, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct BuiltinOptimizerSuite {}
impl OptimizerSuite for BuiltinOptimizerSuite {
    fn suite(&self) -> Box<dyn Iterator<Item = OptimizerSpec>> {
        let suite = vec![
            OptimizerSpec::Random(RandomOptimizerBuilder::default()),
            OptimizerSpec::Optuna(OptunaOptimizerBuilder::default()),
        ];
        Box::new(suite.into_iter())
    }
}
