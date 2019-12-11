
use errors::{Error, Result};
use message::WireMessage;
use libdeflater::{CompressionLvl, Compressor};
use std::iter;
use std::collections::HashMap;
use std::cell::RefCell;

thread_local!(static COMPRESSORS: RefCell<DeflaterCompressor> = RefCell::new(DeflaterCompressor::new()));

/// MessageCompression represents all possible compression algorithms in GELF.
#[derive(PartialEq, Clone, Copy)]
pub enum MessageCompression {
    None,
    Gzip {
        level: i32
    },
    Zlib {
        level: i32
    },
}

impl MessageCompression {
    /// Return the default compression algorithm.
    pub fn default() -> MessageCompression {
        MessageCompression::Gzip {level: 1}
    }

    /// Compress a serialized message with the defined algorithm.
    pub fn compress(self, message: &WireMessage) -> Result<Vec<u8>> {
        let json = message.to_gelf()?;

        Ok(match self {
            MessageCompression::None => json.into_bytes(),
            MessageCompression::Gzip {level} => {
                COMPRESSORS.with(|compressor| {
                    compressor.borrow_mut().with(level, |compressor| {
                        let bound = compressor.gzip_compress_bound(json.as_bytes().len());

                        let mut buffer: Vec<u8> = iter::repeat(0 ).take(bound).collect();

                        compressor.gzip_compress(json.as_bytes(), buffer.as_mut_slice())
                            .map_err(|err| {
                                Error::CompressMessageFailed {
                                    compression_method: "gzip",
                                    compression_error: err.into()
                                }
                            })
                            .map(|size|buffer.truncate(size))
                            .map(move |_| buffer)
                    })
                })?
            }

            MessageCompression::Zlib {level} => {
                COMPRESSORS.with(|compressor| {
                    compressor.borrow_mut().with(level, |compressor| {
                        let bound = compressor.zlib_compress_bound(json.as_bytes().len());

                        let mut buffer: Vec<u8> = iter::repeat(0 ).take(bound).collect();

                        compressor.zlib_compress(json.as_bytes(), buffer.as_mut_slice())
                            .map_err(|err| {
                                Error::CompressMessageFailed {
                                    compression_method: "zlib",
                                    compression_error: err.into()
                                }
                            })
                            .map(|size|buffer.truncate(size))
                            .map(move |_| buffer)
                    })
                })?
            }
        })
    }
}

#[derive(Default)]
struct DeflaterCompressor {
    compressors: HashMap<i32, Compressor>
}

impl DeflaterCompressor {
    pub fn new() -> Self {
        Self::default()
    }

    fn with<F,R>(&mut self, level: i32, fa: F) -> R
        where F: Fn(&mut Compressor) -> R {

        let compressor = self.compressors.get_mut(&level);

        match compressor {
            None => {
                self.compressors.insert(level, Self::create_compression(level));

                fa(self.compressors.get_mut(&level).expect("Should be present"))
            },
            Some(c) => fa(c),
        }
    }

    fn create_compression(level: i32) -> Compressor {
        let compression_lvl = CompressionLvl::new(level).expect("Should be a valid level");

        Compressor::new(compression_lvl)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use ::{Message, Logger};
    use NullBackend;
    use serde_json::Value;

    #[test]
    fn test_compression_none() {
        let logger = Logger::new(Box::new(NullBackend::new())).expect("Should not be an error");

        let message = WireMessage::new(Message::new("Testing"), &logger);

        let compressor = MessageCompression::None;

        let actual = compressor.compress(&message).expect("Should success");

        let actual: Value = serde_json::from_slice(actual.as_slice()).expect("Should success to parse");
        let expected = serde_json::to_value(message).expect("Should success to encode");

        assert_eq!(actual, expected, "Should not compress any data and be equal");
    }

    #[test]
    fn test_compression_gzip() {
        let logger = Logger::new(Box::new(NullBackend::new())).expect("Should not be an error");

        let message = WireMessage::new(Message::new("Testing"), &logger);

        for level in 1..=12 {

            let compressor = MessageCompression::Gzip {level};

            let mut decompressor = libdeflater::Decompressor::new();

            let actual = compressor.compress(&message).expect("Should success");

            let mut buffer: Vec<u8> = iter::repeat(0).take(serde_json::to_vec(&message).unwrap().len()).collect();

            let decoded = decompressor.gzip_decompress(actual.as_slice(), buffer.as_mut_slice()).expect("Should not throw an error");

            buffer.truncate(decoded);

            let actual = buffer;

            let actual: Value = serde_json::from_slice(actual.as_slice()).expect("Should success to parse");

            let expected = serde_json::to_value(&message).expect("Should success to encode");

            assert_eq!(actual, expected, "Decoded data should be equal input");
        }
    }

    #[test]
    fn test_compression_zlib() {
        let logger = Logger::new(Box::new(NullBackend::new())).expect("Should not be an error");

        let message = WireMessage::new(Message::new("Testing"), &logger);

        for level in 1..=12 {

            let compressor = MessageCompression::Zlib {level};

            let mut decompressor = libdeflater::Decompressor::new();

            let actual = compressor.compress(&message).expect("Should success");

            let mut buffer: Vec<u8> = iter::repeat(0).take(serde_json::to_vec(&message).unwrap().len()).collect();

            let decoded = decompressor.zlib_decompress(actual.as_slice(), buffer.as_mut_slice()).expect("Should not throw an error");

            buffer.truncate(decoded);

            let actual = buffer;

            let actual: Value = serde_json::from_slice(actual.as_slice()).expect("Should success to parse");

            let expected = serde_json::to_value(&message).expect("Should success to encode");

            assert_eq!(actual, expected, "Decoded data should be equal input");
        }
    }

    #[test]
    fn test_concurrency_zlib() {

        for level in 1..=12 {

            let compressor = MessageCompression::Zlib {level};

            loom::model(move || {
                let logger = loom::sync::Arc::new(Logger::new(Box::new(NullBackend::new())).expect("Should not be an error"));

                for _ in 0..3 {
                    let logger = logger.clone();

                    loom::thread::spawn( move || {
                        let message = WireMessage::new(Message::new("Testing"), &logger);

                        let mut decompressor = libdeflater::Decompressor::new();

                        let actual = compressor.clone().compress(&message).expect("Should success");

                        let mut buffer: Vec<u8> = iter::repeat(0).take(serde_json::to_vec(&message).unwrap().len()).collect();

                        let decoded = decompressor.zlib_decompress(actual.as_slice(), buffer.as_mut_slice()).expect("Should not throw an error");

                        buffer.truncate(decoded);

                        let actual = buffer;

                        let actual: Value = serde_json::from_slice(actual.as_slice()).expect("Should success to parse");

                        let expected = serde_json::to_value(&message).expect("Should success to encode");

                        assert_eq!(actual, expected, "Decoded data should be equal input");
                    });
                }
            })

        }
    }

    #[test]
    fn test_concurrency_gzip() {

        for level in 1..=12 {
            let compressor = MessageCompression::Gzip { level };

            loom::model(move || {
                let logger = loom::sync::Arc::new(Logger::new(Box::new(NullBackend::new())).expect("Should not be an error"));

                for _ in 0..3 {
                    let logger = logger.clone();

                    loom::thread::spawn( move || {
                        let message = WireMessage::new(Message::new("Testing"), &logger);

                        let mut decompressor = libdeflater::Decompressor::new();

                        let actual = compressor.clone().compress(&message).expect("Should success");

                        let mut buffer: Vec<u8> = iter::repeat(0).take(serde_json::to_vec(&message).unwrap().len()).collect();

                        let decoded = decompressor.gzip_decompress(actual.as_slice(), buffer.as_mut_slice()).expect("Should not throw an error");

                        buffer.truncate(decoded);

                        let actual = buffer;

                        let actual: Value = serde_json::from_slice(actual.as_slice()).expect("Should success to parse");

                        let expected = serde_json::to_value(&message).expect("Should success to encode");

                        assert_eq!(actual, expected, "Decoded data should be equal input");
                    });
                }
            })

        }
    }

}