use kurobako_core::{Error, ErrorKind};
use trackable::error::ErrorKindExt;
use yamakan;

pub fn from_yamakan(f: yamakan::Error) -> Error {
    let kind = match f.kind() {
        yamakan::ErrorKind::Bug => ErrorKind::Bug,
        yamakan::ErrorKind::InvalidInput => ErrorKind::InvalidInput,
        yamakan::ErrorKind::IoError => ErrorKind::IoError,
        yamakan::ErrorKind::UnknownObservation | yamakan::ErrorKind::Other => ErrorKind::Other,
    };
    kind.takes_over(f).into()
}

pub fn into_yamakan(f: Error) -> yamakan::Error {
    let kind = match f.kind() {
        ErrorKind::Bug => yamakan::ErrorKind::Bug,
        ErrorKind::InvalidInput => yamakan::ErrorKind::InvalidInput,
        ErrorKind::IoError => yamakan::ErrorKind::IoError,
        _ => yamakan::ErrorKind::Other,
    };
    kind.takes_over(f).into()
}
