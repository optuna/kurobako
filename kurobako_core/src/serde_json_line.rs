use crate::{Error, Result};
use serde::Deserialize;
use serde_json;
use std::io::BufRead;

pub fn from_reader<R, T>(mut reader: R) -> Result<T>
where
    R: BufRead,
    T: for<'a> Deserialize<'a>,
{
    let mut line = String::new();
    track!(reader.read_line(&mut line).map_err(Error::from))?;
    let value = track!(serde_json::from_str(&line).map_err(Error::from); line)?;
    Ok(value)
}
