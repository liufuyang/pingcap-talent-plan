use std::io;

use failure::Fail;

/// Error type for kvs
#[derive(Fail, Debug)]
pub enum KvsError {
    /// io error
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),
    /// serde error
    #[fail(display = "{}", _0)]
    Serde(#[cause] serde_json::Error),
    /// no key error
    #[fail(display = "NO_KEY_ERROR")]
    NO_KEY_ERROR,
}

impl From<io::Error> for KvsError {
    fn from(err: io::Error) -> KvsError {
        KvsError::Io(err)
    }
}

impl From<serde_json::error::Error> for KvsError {
    fn from(err: serde_json::error::Error) -> KvsError {
        KvsError::Serde(err)
    }
}

/// Result type for kvs
pub type Result<T> = std::result::Result<T, KvsError>;
