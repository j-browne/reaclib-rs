use std::{
    convert::Infallible,
    io,
    num::{ParseFloatError, ParseIntError},
};
use thiserror::Error;

/// A reading or parsing error.
#[derive(Error, Debug, Clone, Eq, PartialEq)]
#[non_exhaustive]
#[allow(clippy::module_name_repetitions)]
pub enum ReaclibError {
    #[error("read error")]
    Io(io::ErrorKind),
    #[error("int parsing error")]
    ParseInt(#[from] ParseIntError),
    #[error("float parsing error")]
    ParseFloat(#[from] ParseFloatError),
    #[error("no chapter set")]
    ChapterUnset,
    #[error("unknown chapter: {0}")]
    UnknownChapter(u8),
    #[error("unknown resonance: {0}")]
    UnknownResonance(String),
    #[error("line too short")]
    TooShortLine,
    #[error("too few lines in a set")]
    TooFewLines,
    #[error("string indexing error")]
    StrIndex,
}

impl From<io::Error> for ReaclibError {
    fn from(e: io::Error) -> Self {
        Self::Io(e.kind())
    }
}

impl From<io::ErrorKind> for ReaclibError {
    fn from(k: io::ErrorKind) -> Self {
        Self::Io(k)
    }
}

impl From<Infallible> for ReaclibError {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    const fn test_send() {
        const fn assert_send<T: Send>() {}
        assert_send::<ReaclibError>();
    }

    #[test]
    const fn test_sync() {
        const fn assert_sync<T: Sync>() {}
        assert_sync::<ReaclibError>();
    }
}
