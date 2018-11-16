use failure;
use serde;
use serde::ser::SerializeMap;
use serde_json;
use std::collections::HashMap;

use errors::{Error, Result};
use logger::Logger;
use message::{ChunkSize, ChunkedMessage, Message, MessageCompression};

/// WireMessage is the representation of a fully assembled GELF message
///
/// A fully assembled requires information only present in the `Logger`:
/// Both the local hostname and possible missing metadata fields need to be
/// added to the message.
///
/// A WireMessage can be serialized to GELF/JSON (with and without compression)
/// and is the abstraction passed to the transportation backends.
pub struct WireMessage<'a> {
    host: &'a str,
    message: Message<'a>,
}

impl<'a> WireMessage<'a> {
    /// Construct a new wire message
    ///
    /// The logger is required for populating the `host`-field and metadata
    /// fields which were not added to the message.
    pub fn new(mut msg: Message<'a>, logger: &'a Logger) -> Self {
        // Filter all fields missing from the message
        let additionals_from_default: HashMap<&String, &String> = logger
            .default_metadata()
            .iter()
            .filter(|&(key, _)| !msg.metadata.contains_key(key.as_str()))
            .collect();

        // add the missing metadata
        for (key, value) in additionals_from_default {
            msg.set_metadata(key.as_str(), value.as_str()).ok();
        }

        WireMessage {
            host: logger.hostname(),
            message: msg,
        }
    }

    /// Return a GELF/JSON string of this message
    pub fn to_gelf(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| {
            failure::Error::from(e)
                .context(Error::SerializeMessageFailed)
                .into()
        })
    }

    /// Return a compressed GELF/JSON string of this message
    pub fn to_compressed_gelf(&self, compression: MessageCompression) -> Result<Vec<u8>> {
        compression.compress(&self)
    }

    /// Serialize the messages and prepare it for chunking
    pub fn to_chunked_message(
        &self,
        chunk_size: ChunkSize,
        compression: MessageCompression,
    ) -> Result<ChunkedMessage> {
        ChunkedMessage::new(chunk_size, self.to_compressed_gelf(compression)?)
    }
}

impl<'a> serde::Serialize for WireMessage<'a> {
    /// Serialize the message to a GELF/JSON string
    fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_key("version")?;
        map.serialize_value("1.1")?;

        map.serialize_key("host")?;
        map.serialize_value(self.host)?;

        map.serialize_key("short_message")?;
        map.serialize_value(&self.message.short_message())?;

        map.serialize_key("level")?;
        let level = self.message.level as u8;
        map.serialize_value(&level)?;

        if self.message.full_message().is_some() {
            map.serialize_key("full_message")?;
            map.serialize_value(&self.message.full_message())?;
        }

        if self.message.timestamp().is_some() {
            let datetime = &self.message.timestamp().unwrap();
            let value = format!("{}.{}", datetime.timestamp(), datetime.timestamp_subsec_millis());

            map.serialize_key("timestamp")?;
            map.serialize_value(&value)?;
        }

        for (key, value) in self.message.all_metadata().iter() {
            let key = "_".to_string() + key;
            map.serialize_key(&key)?;
            map.serialize_value(value)?;
        }

        map.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};
    use level::Level;

    #[test]
    fn wire_message_serialization() {
        let mut message = Message::new_with_level("short", Level::Alert);
        message.set_full_message("full");

        let datetime = Utc.ymd(2000, 1, 1).and_hms_micro(1, 2, 3, 123_456);
        message.set_timestamp(datetime);

        message.set_metadata("key1", "value1").unwrap();
        message.set_metadata("key2", "value2").unwrap();

        let wire_msg = WireMessage {
            host: "host_value",
            message,
        };

        let json = serde_json::to_value(wire_msg).expect("Failed to serialize WireMessage");

        assert_eq!(Some(json!("1.1")), json.get("version").cloned());
        assert_eq!(Some(json!("host_value")), json.get("host").cloned());
        assert_eq!(Some(json!("short")), json.get("short_message").cloned());
        assert_eq!(Some(json!("full")), json.get("full_message").cloned());
        assert_eq!(Some(json!(1)), json.get("level").cloned()); // Level::Alert = 1

        let timestamp_secs = datetime.timestamp();
        let expected_timestamp = format!("{}.{}", timestamp_secs, 123);
        assert_eq!(
            Some(json!(&expected_timestamp)),
            json.get("timestamp").cloned()
        );

        assert_eq!(Some(json!("value1")), json.get("_key1").cloned());
        assert_eq!(Some(json!("value2")), json.get("_key2").cloned());
    }
}
