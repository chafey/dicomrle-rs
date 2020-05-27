use std::fmt;
use std::io::Error as IoError;

/// Errors that can occur while decoding an image.
#[derive(Debug)]
pub enum Error {
    /// The image is not formatted properly. The string contains detailed information about the
    /// error.
    Format(String),
    /// An I/O error occurred while decoding the image.
    Io(IoError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Format(ref desc)      => write!(f, "invalid RLE format: {}", desc),
            Error::Io(ref err)           => err.fmt(f),
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error::Io(err)
    }
}