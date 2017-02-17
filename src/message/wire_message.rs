use std::collections::HashMap;
use serde;
use serde::ser::SerializeMap;
use serde_json;

use super::{Message, ChunkSize, ChunkedMessage, MessageCompression};
use errors::{Result, ErrorKind, ResultExt};
use logger::Logger;

pub struct WireMessage<'a> {
    host: &'a str,
    message: Message<'a>,
}

impl<'a> WireMessage<'a> {
    pub fn new(mut msg: Message<'a>, logger: &'a Logger) -> Self {
        let additionals_from_default: HashMap<&String, &String> = logger.default_metadata()
            .iter()
            .filter(|&(key, _)| !msg.metadata.contains_key(key.as_str()))
            .collect();

        for (key, value) in additionals_from_default {
            msg.set_metadata(key.as_str(), value.clone()).ok();
        }

        WireMessage {
            host: logger.hostname(),
            message: msg,
        }
    }

    pub fn to_gelf(&self) -> Result<String> {
        serde_json::to_string(self).chain_err(|| ErrorKind::SerializeMessageFailed)
    }

    pub fn to_compressed_gelf(&self, compression: MessageCompression) -> Result<Vec<u8>> {
        let json_str = self.to_gelf()?;
        compression.compress(&json_str)
    }

    pub fn to_chunked_message(&self,
                              chunk_size: ChunkSize,
                              compression: MessageCompression)
                              -> Result<ChunkedMessage> {

        ChunkedMessage::new(chunk_size, self.to_compressed_gelf(compression)?)
    }
}

impl<'a> serde::Serialize for WireMessage<'a> {
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_key("version")?;
        map.serialize_value("1.1")?;

        map.serialize_key("host")?;
        map.serialize_value(self.host)?;

        map.serialize_key("short_message")?;
        map.serialize_value(self.message.short_message())?;

        if self.message.full_message().is_some() {
            map.serialize_key("full_message")?;
            map.serialize_value(&self.message.full_message())?;
        }

        if self.message.timestamp().is_some() {
            map.serialize_key("timestamp")?;
            map.serialize_value(&self.message.timestamp().unwrap().timestamp())?;
        }

        if self.message.level().is_some() {
            let level = self.message.level().unwrap() as i8;
            map.serialize_key("level")?;
            map.serialize_value(&level)?;
        }

        for (key, value) in self.message.all_metadata().iter() {
            let key = "_".to_string() + key;
            map.serialize_key(&key)?;
            map.serialize_value(value)?;
        }

        map.end()
    }
}