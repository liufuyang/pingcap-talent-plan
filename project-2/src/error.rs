use failure::Fail;
use std::io;

/// Error type for kvs
#[derive(Fail, Debug)]
pub enum KvsError {
    #[fail(display = "KVS_ERROR")]
    KVS_ERROR
}


/// Result type for kvs
pub type Result<T> = std::result::Result<T, KvsError>;