use failure;
use std;

#[derive(Clone, Debug, Fail)]
pub enum Error {
    #[fail(display = "Failed to create the GELF backend")]
    BackendCreationFailed,
    #[fail(display = "'{}' is not a legal name for an additional GELF field", name)]
    IllegalNameForAdditional { name: String },
    #[fail(display = "Failed to create the GELF logger")]
    LoggerCreateFailed,
    #[fail(display = "Failed to create a GELF log message")]
    LogTransmitFailed,
    #[fail(display = "Failed to compress the message with '{}'", compression_method)]
    CompressMessageFailed { compression_method: &'static str },
    #[fail(display = "Failed to serialize the message to GELF json")]
    SerializeMessageFailed,
    #[fail(display = "Failed to chunk the message")]
    ChunkMessageFailed,
    #[fail(display = "Illegal chunk size: {}", size)]
    IllegalChunkSize { size: u16 },
}

pub type Result<T> = std::result::Result<T, failure::Error>;
