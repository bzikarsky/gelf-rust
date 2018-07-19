use std::collections::HashMap;
use std::borrow::Cow;
use chrono::{DateTime, Utc};
use log;

use errors::{Error, Result};
use level::Level;
use util;

pub use self::chunked_message::{ChunkSize, ChunkedMessage};
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
    short_message: Cow<'a, str>,
    full_message: Option<Cow<'a, str>>,
    timestamp: Option<DateTime<Utc>>,
    level: Level,
    metadata: HashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a > Message<'a> {
    /// Construct a new log message.
    ///
    /// All fields will use their defaults. This means usually Option::None.
    /// A notable exception is `level`. The GELF spec requires this field to
    /// default to Level::Alert.
    pub fn new<S>(
        short_message: S,
    ) -> Self
    where
        S: Into<Cow<'a, str>> + AsRef<str>
    {
        Self::new_with_level(short_message, Level::Alert)
    }

    /// Construct a new log message with a defined level
    ///
    /// All fields will use their defaults. This means usually Option::None.
    pub fn new_with_level<S>(
        short_message: S,
        level: Level
    ) -> Self
    where
        S: Into<Cow<'a, str>> + AsRef<str>
    {
        Message {
            short_message: short_message.into(),
            level,
            full_message: None,
            timestamp: None,
            metadata: HashMap::new(),
        }
    }

    /// Return the `short_message`
    pub fn short_message(&self) -> &Cow<'a, str> {
        &self.short_message
    }

    /// Set the `short_message`
    pub fn set_short_message<S>(
        &mut self,
        msg: S
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>> + AsRef<str>
    {
        self.short_message = msg.into();
        self
    }

    /// Return the `full_message`
    pub fn full_message(&self) -> &Option<Cow<'a, str>> {
        &self.full_message
    }

    /// Set the `full_message`
    pub fn set_full_message<S>(
        &mut self,
        msg: S
    ) -> &mut Self
    where
        S: Into<Cow<'a, str>> + AsRef<str>
    {
        self.full_message = Some(msg.into());
        self
    }

    // Clear the `full_message`
    pub fn clear_full_message(&mut self) -> &mut Self {
        self.full_message = None;
        self
    }

    /// Return the `timestamp`
    pub fn timestamp(&self) -> &Option<DateTime<Utc>> {
        &self.timestamp
    }

    /// Set the `timestamp`
    pub fn set_timestamp(&mut self, ts: DateTime<Utc>) -> &mut Self {
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
    pub fn metadata(&self, key: &'a str) -> Option<&Cow<'a, str>> {
        self.metadata.get(key)
    }

    /// Return all metadata
    pub fn all_metadata(&self) -> &HashMap<Cow<'a, str>, Cow<'a, str>> {
        &self.metadata
    }

    /// Set a metadata field with given key to value
    pub fn set_metadata<S, T>(
        &mut self,
        key: S,
        value: T
    ) -> Result<&mut Self>
    where
        S: Into<Cow<'a, str>> + AsRef<str>,
        T: Into<Cow<'a, str>> + AsRef<str>,
    {
        let key = key.into();

        if key == "id" {
            return Err(Error::IllegalNameForAdditional { name: key.into() }.into());
        }

        self.metadata.insert(key.into(), value.into());

        Ok(self)
    }
}

impl<'a> From<&'a log::Record<'a>> for Message<'a> {
    /// Create a `Message` from given `log::LogRecord` including all metadata
    fn from(record: &'a log::Record) -> Message<'a> {
        // Create message with given text and level
        let short_message = format!("{}", record.args());

        let mut msg = Message::new_with_level(
            short_message,
            record.level().into()
        );

        msg.set_timestamp(Utc::now());

        // Add default metadata, and ignore the results (`let _ = ...`) as all keys are valid
        // and set_metadata only fails on invalid keys
        let _ = msg.set_metadata("file", record.file().unwrap_or("(none)").to_string());
        let _ = msg.set_metadata("line", record.line().map(|v| v.to_string()).unwrap_or("(none)".into()));
        let _ = msg.set_metadata("module_path", record.module_path().unwrap_or("(none)").to_string());
        let _ = msg.set_metadata("process_id", util::pid().to_string());

        msg
    }
}
