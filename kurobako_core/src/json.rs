//! JSON.
use crate::{Error, Result};
use serde::Deserialize;
use serde_json;

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
