/// Errors that can occur while decoding an image.
#[derive(Debug)]
pub enum Error {
    /// The image is not formatted properly. The string contains detailed information about the
    /// error.
    Format(String)
}

