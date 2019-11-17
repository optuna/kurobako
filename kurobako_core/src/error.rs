use serde::{Deserialize, Serialize};
use serde_json;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
use trackable::error::{Failure, TrackableError};

/// This crate specific `Error` type.
#[derive(Debug, Clone, TrackableError)]
pub struct Error(TrackableError<ErrorKind>);
impl From<Failure> for Error {
    fn from(f: Failure) -> Self {
        ErrorKind::Other.takes_over(f).into()
    }
}
impl From<std::io::Error> for Error {
    fn from(f: std::io::Error) -> Self {
        ErrorKind::IoError.cause(f).into()
    }
}
impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(f: std::sync::PoisonError<T>) -> Self {
        ErrorKind::Other.cause(f.to_string()).into()
    }
}
impl From<serde_json::error::Error> for Error {
    fn from(f: serde_json::error::Error) -> Self {
        if let serde_json::error::Category::Io = f.classify() {
            ErrorKind::IoError.cause(f).into()
        } else {
            ErrorKind::InvalidInput.cause(f).into()
        }
    }
}

/// Possible error kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorKind {
    /// Invalid input was given.
    InvalidInput,

    /// I/O error.
    IoError,

    /// Unexpected end-of-stream.
    UnexpectedEos,

    /// Incapable feature was required.
    Incapable,

    /// Unevaluable parameter set was passed.
    UnevaluableParams,

    /// Implementation bug.
    Bug,

    /// Other error.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
