use crate::{Error, ErrorKind, Result};
use serde::{Deserialize, Serialize};
use serde_json::de::IoRead;
use serde_json::{self, Deserializer, StreamDeserializer};
use std::fmt;
use std::io::{Read, Write};
use std::marker::PhantomData;

pub struct JsonMessageSender<T, W> {
    writer: W,
    _message: PhantomData<T>,
}
impl<T, W> JsonMessageSender<T, W>
where
    T: Serialize,
    W: Write,
{
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            _message: PhantomData,
        }
    }

    pub fn send(&mut self, message: &T) -> Result<()> {
        track!(serde_json::to_writer(&mut self.writer, message).map_err(Error::from))?;
        track!(writeln!(self.writer).map_err(Error::from))?;
        track!(self.writer.flush().map_err(Error::from))?;
        Ok(())
    }
}
impl<T, W> fmt::Debug for JsonMessageSender<T, W> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JsonMessageSender {{ .. }}")
    }
}

pub struct JsonMessageReceiver<T, R: Read> {
    inner: StreamDeserializer<'static, IoRead<R>, T>,
}
impl<T, R> JsonMessageReceiver<T, R>
where
    T: for<'a> Deserialize<'a>,
    R: Read,
{
    pub fn new(reader: R) -> Self {
        let inner = Deserializer::from_reader(reader).into_iter();
        Self { inner }
    }

    pub fn recv(&mut self) -> Result<T> {
        match self.inner.next() {
            None => track_panic!(ErrorKind::UnexpectedEos),
            Some(result) => {
                let message = track!(result.map_err(Error::from))?;
                Ok(message)
            }
        }
    }
}
impl<T, R: Read> fmt::Debug for JsonMessageReceiver<T, R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "JsonMessageReceiver {{ .. }}")
    }
}
