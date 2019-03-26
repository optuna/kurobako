use crate::Result;
use serde::Deserialize;
use serde_json;
use std::io::BufRead;

pub fn from_reader<R, T>(mut reader: R) -> Result<T>
where
    R: BufRead,
    T: for<'a> Deserialize<'a>,
{
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let value = serde_json::from_str(&line)?;
    Ok(value)
}
