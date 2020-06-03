use std::fmt;

/// Errors that can occur while decoding an image.
#[derive(Debug)]
pub enum Error {
    /// The image is not formatted properly. The string contains detailed information about the
    /// error.
    Format(String)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Format(ref desc)      => write!(f, "{}", desc)
        }
    }
}