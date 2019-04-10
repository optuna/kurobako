use serde_json;
use trackable::error::{ErrorKind as TrackableErrorKind, ErrorKindExt};
use trackable::error::{Failure, TrackableError};
use yamakan;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorKind {
    InvalidInput,
    IoError,
    Other,
}
impl TrackableErrorKind for ErrorKind {}
