use crate::parameter::{ParamDomain, ParamValue};
use crate::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

pub trait Recipe: Clone + StructOpt + Serialize + for<'a> Deserialize<'a> {
    fn get_free_params(&self) -> Result<Vec<ParamDomain>>;
    fn bind_params(&mut self, params: Vec<(String, ParamValue)>) -> Result<()>;
}
