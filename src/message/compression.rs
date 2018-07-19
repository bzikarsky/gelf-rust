use failure;
use libflate::gzip;
use libflate::zlib;
use std::io;

use errors::{Error, Result};
use message::WireMessage;

/// MessageCompression represents all possible compression algorithms in GELF.
#[derive(PartialEq, Clone, Copy)]
pub enum MessageCompression {
    None,
    Gzip,
    Zlib,
}

impl MessageCompression {
    /// Return the default compression algorithm.
    pub fn default() -> MessageCompression {
        MessageCompression::Gzip
    }

    /// Compress a serialized message with the defined algorithm.
    pub fn compress(&self, message: &WireMessage) -> Result<Vec<u8>> {
        let json = message.to_gelf()?;

        Ok(match *self {
            MessageCompression::None => json.into_bytes(),
            MessageCompression::Gzip => {
                let mut cursor = io::Cursor::new(json);
                gzip::Encoder::new(Vec::new())
                    .and_then(|mut encoder| {
                        io::copy(&mut cursor, &mut encoder)
                            .and_then(|_| encoder.finish().into_result())
                    })
                    .map_err(|e| {
                        failure::Error::from(e).context(Error::CompressMessageFailed {
                            compression_method: "gzip",
                        })
                    })?
            }
            MessageCompression::Zlib => {
                let mut cursor = io::Cursor::new(json);
                zlib::Encoder::new(Vec::new())
                    .and_then(|mut encoder| {
                        io::copy(&mut cursor, &mut encoder)
                            .and_then(|_| encoder.finish().into_result())
                    })
                    .map_err(|e| {
                        failure::Error::from(e).context(Error::CompressMessageFailed {
                            compression_method: "zlib",
                        })
                    })?
            }
        })
    }
}
