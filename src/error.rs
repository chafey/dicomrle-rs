use std::fmt;
use std::io::Error as IoError;
use std::error::Error as StdError;

/// Errors that can occur while decoding an image.
#[derive(Debug)]
pub enum Error {
    /// The image is not formatted properly. The string contains detailed information about the
    /// error.
    Format(String),
    /// An I/O error occurred while decoding the image.
    Io(IoError),

    Std(Box<dyn StdError>)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Format(ref desc)      => write!(f, "{}", desc),
            Error::Io(ref err)           => err.fmt(f),
            Error::Std(ref err)     => err.fmt(f),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}

impl From<Box<dyn StdError>> for Error {
    fn from(err: Box<dyn StdError>) -> Error {
        Error::Std(err)
    }
}