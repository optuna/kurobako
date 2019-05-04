use rustats;
use serde::{Deserialize, Serialize};
use serde_json;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
use trackable::error::{Failure, TrackableError};
use yamakan;

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
impl From<serde_json::error::Error> for Error {
    fn from(f: serde_json::error::Error) -> Self {
        if let serde_json::error::Category::Io = f.classify() {
            ErrorKind::IoError.cause(f).into()
        } else {
            ErrorKind::InvalidInput.cause(f).into()
        }
    }
}
impl From<yamakan::Error> for Error {
    fn from(f: yamakan::Error) -> Self {
        let original_kind = f.kind().clone();
        let kind = match original_kind {
            yamakan::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
            yamakan::ErrorKind::IoError => ErrorKind::IoError,
            _ => ErrorKind::Other,
        };
        track!(kind.takes_over(f); original_kind).into()
    }
}
impl From<rustats::Error> for Error {
    fn from(f: rustats::Error) -> Self {
        let original_kind = f.kind().clone();
        let kind = match original_kind {
            rustats::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
            rustats::ErrorKind::IoError => ErrorKind::IoError,
            _ => ErrorKind::Other,
        };
        track!(kind.takes_over(f); original_kind).into()
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

    /// Other error.
    Other,
}
impl TrackableErrorKind for ErrorKind {}
