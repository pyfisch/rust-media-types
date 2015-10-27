use std::error;
use std::fmt::{self, Display};
use std::str::Utf8Error;
use std::string::FromUtf8Error;

use charsets;


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Defines an Error type for media types.
pub enum Error {
    /// Parsing the given string as a media type failed.
    Invalid,
    /// The media type does not have this parameter.
    NotFound,
    /// Decoding a string as UTF-8 (or ASCII) failed.
    Utf8Error(Utf8Error),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Invalid => "given media type is invalid",
            Error::NotFound => "given parameter not found",
            Error::Utf8Error(_) => "decoding as UTF-8 failed",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        if let Error::Utf8Error(ref error) = *self {
            Some(error)
        } else {
            None
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(error::Error::description(self))
    }
}

impl From<charsets::Error> for Error {
    fn from(_: charsets::Error) -> Error {
        Error::Invalid
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error::Utf8Error(err.utf8_error())
    }
}

/// Result type used for this library.
pub type Result<T> = ::std::result::Result<T, Error>;
