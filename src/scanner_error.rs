use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    io::{self, ErrorKind},
    num::{ParseFloatError, ParseIntError},
};

#[derive(Debug)]
/// The possible errors of `Scanner`, `ScannerAscii`, `ScannerStr`, `ScannerU8Slice`, `ScannerU8SliceAscii` structs.
pub enum ScannerError {
    IOError(io::Error),
    ParseIntError(ParseIntError),
    ParseFloatError(ParseFloatError),
}

impl From<io::Error> for ScannerError {
    #[inline]
    fn from(err: io::Error) -> ScannerError {
        ScannerError::IOError(err)
    }
}

impl From<ErrorKind> for ScannerError {
    #[inline]
    fn from(kind: ErrorKind) -> ScannerError {
        ScannerError::IOError(kind.into())
    }
}

impl From<ParseIntError> for ScannerError {
    #[inline]
    fn from(err: ParseIntError) -> ScannerError {
        ScannerError::ParseIntError(err)
    }
}

impl From<ParseFloatError> for ScannerError {
    #[inline]
    fn from(err: ParseFloatError) -> ScannerError {
        ScannerError::ParseFloatError(err)
    }
}

impl Display for ScannerError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            ScannerError::IOError(err) => Display::fmt(&err, f),
            ScannerError::ParseIntError(err) => Display::fmt(&err, f),
            ScannerError::ParseFloatError(err) => Display::fmt(&err, f),
        }
    }
}

impl Error for ScannerError {}
