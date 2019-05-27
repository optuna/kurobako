use kurobako_core::{Error, Result};
use serde::Deserialize;
use serde_json;

pub fn parse_json<T>(json: &str) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let v = track!(serde_json::from_str(json).map_err(Error::from))?;
    Ok(v)
}
