//! JSON.
use crate::{Error, Result};
use serde::Deserialize;
use serde_json;
use std::io::Read;

/// JSON representation of a recipe.
pub type JsonRecipe = serde_json::Value;

/// Parses the given JSON string.
pub fn parse_json<T>(json: &str) -> Result<T>
where
    T: for<'a> Deserialize<'a>,
{
    let v = track!(serde_json::from_str(json).map_err(Error::from))?;
    Ok(v)
}

/// Loads entries from the given reader.
pub fn load<R, T>(reader: R) -> Result<Vec<T>>
where
    R: Read,
    T: for<'a> Deserialize<'a>,
{
    serde_json::Deserializer::from_reader(reader)
        .into_iter()
        .map(|json| track!(json.map_err(Error::from)))
        .collect()
}
