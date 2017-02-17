use std::io;

use libflate::gzip;
use libflate::zlib;

use errors::{Result, ErrorKind, ResultExt};

#[derive(PartialEq, Clone, Copy)]
pub enum MessageCompression {
    None,
    Gzip,
    Zlib,
}

impl MessageCompression {
    pub fn default() -> MessageCompression {
        MessageCompression::Gzip
    }

    pub fn compress(&self, message: &str) -> Result<Vec<u8>> {

        let mut cursor = io::Cursor::new(message);
        Ok(match *self {
            MessageCompression::None => message.to_owned().into_bytes(),
            MessageCompression::Gzip => {
                gzip::Encoder::new(Vec::new()).and_then(|mut encoder| {
                        io::copy(&mut cursor, &mut encoder)
                            .and_then(|_| encoder.finish().into_result())
                    })
                    .chain_err(|| ErrorKind::CompressMessageFailed("gzip"))?
            }
            MessageCompression::Zlib => {
                zlib::Encoder::new(Vec::new()).and_then(|mut encoder| {
                        io::copy(&mut cursor, &mut encoder)
                            .and_then(|_| encoder.finish().into_result())
                    })
                    .chain_err(|| ErrorKind::CompressMessageFailed("zlib"))?
            }
        })
    }
}
