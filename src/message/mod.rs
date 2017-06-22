use std::collections::HashMap;
use chrono::{DateTime, UTC};
use log;

use level::Level;
use errors::{Result, ErrorKind};
use util;

pub use self::chunked_message::{ChunkedMessage, ChunkSize};
pub use self::compression::MessageCompression;
pub use self::wire_message::WireMessage;

mod chunked_message;
mod compression;
mod wire_message;

/// Message is thre representation of a GELF message.
///
/// `Message` provides a fluid setter and getter  interface to all of GELF's
/// features. Only the `host`-field is not available. It is managed the
/// `Logger`.
///
/// A `Message` can also be constructed from a `log::LogRecord`. All
/// available metadata is transferred over to the message object.
pub struct Message<'a> {
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<DateTime<UTC>>,
    level: Level,
    metadata: HashMap<&'a str, String>,
}

impl<'a> Message<'a> {
    /// Construct a new log message.
    ///
    /// All fields will use their defaults. This means usually Option::None.
    /// A notable exception is `level`. The GELF spec requires this field to
    /// default to Level::Alert.
    pub fn new(short_message: String) -> Self {
        Self::new_with_level(short_message, Level::Alert)
    }

    /// Construct a new log message with a defined level
    ///
    /// All fields will use their defaults. This means usually Option::None.
    pub fn new_with_level(short_message: String, level: Level) -> Self {
        Message {
            short_message: short_message,
            level: level,
            full_message: None,
            timestamp: None,
            metadata: HashMap::new(),
        }
    }

    /// Return the `short_message`
    pub fn short_message(&self) -> &String {
        &self.short_message
    }

    /// Set the `short_message`
    pub fn set_short_message(&mut self, msg: String) -> &mut Self {
        self.short_message = msg;
        self
    }

    /// Return the `full_message`
    pub fn full_message(&self) -> &Option<String> {
        &self.full_message
    }

    /// Set the `full_message`
    pub fn set_full_message(&mut self, msg: String) -> &mut Self {
        self.full_message = Some(msg);
        self
    }

    // Clear the `full_message`
    pub fn clear_full_message(&mut self) -> &mut Self {
        self.full_message = None;
        self
    }

    /// Return the `timestamp`
    pub fn timestamp(&self) -> &Option<DateTime<UTC>> {
        &self.timestamp
    }

    /// Set the `timestamp`
    pub fn set_timestamp(&mut self, ts: DateTime<UTC>) -> &mut Self {
        self.timestamp = Some(ts);
        self
    }

    /// Clear the `timestamp`
    pub fn clear_timestamp(&mut self) -> &mut Self {
        self.timestamp = None;
        self
    }

    /// Return the `level`
    pub fn level(&self) -> Level {
        self.level
    }

    /// Set the `level`
    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = level;
        self
    }

    /// Return a metadata field with given key
    pub fn metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Return all metadata
    pub fn all_metadata(&self) -> &HashMap<&'a str, String> {
        &self.metadata
    }

    /// Set a metadata field with given key to value
    pub fn set_metadata(&mut self, key: &'a str, value: String) -> Result<&mut Self> {
        if key == "id" {
            bail!(ErrorKind::IllegalNameForAdditional(String::from(key)));
        }

        self.metadata.insert(key, value);

        Ok(self)
    }
}

impl<'a> From<&'a log::LogRecord<'a>> for Message<'a> {
    /// Create a `Message` from given `log::LogRecord` including all metadata
    fn from(record: &'a log::LogRecord) -> Message<'a> {
        // Create message with given text and level
        let mut msg = Message::new_with_level(format!("{}", record.args()), record.level().into());

        msg.set_timestamp(UTC::now());

        // Add location meta-data
        msg.metadata.insert("file", record.location().file().to_owned());
        msg.metadata.insert("line", record.location().line().to_string());
        msg.metadata.insert("module_path", record.location().module_path().to_owned());

        // Add runtime meta-data
        msg.metadata.insert("process_id", util::pid().to_string());

        msg
    }
}
