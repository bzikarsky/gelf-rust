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

pub struct Message<'a> {
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<DateTime<UTC>>,
    level: Option<Level>,
    metadata: HashMap<&'a str, String>,
}

impl<'a> Message<'a> {
    pub fn new(short_message: String, level: Option<Level>) -> Self {
        Message {
            short_message: short_message,
            level: level,
            full_message: None,
            timestamp: None,
            metadata: HashMap::new(),
        }
    }

    pub fn short_message(&self) -> &String {
        &self.short_message
    }

    pub fn set_short_message(&mut self, msg: String) -> &mut Self {
        self.short_message = msg;
        self
    }

    pub fn full_message(&self) -> &Option<String> {
        &self.full_message
    }

    pub fn set_full_message(&mut self, msg: String) -> &mut Self {
        self.full_message = Some(msg);
        self
    }

    pub fn timestamp(&self) -> &Option<DateTime<UTC>> {
        &self.timestamp
    }

    pub fn set_timestamp(&mut self, ts: DateTime<UTC>) -> &mut Self {
        self.timestamp = Some(ts);
        self
    }

    pub fn level(&self) -> &Option<Level> {
        &self.level
    }

    pub fn set_level(&mut self, level: Level) -> &mut Self {
        self.level = Some(level);
        self
    }

    pub fn metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    pub fn all_metadata(&self) -> &HashMap<&'a str, String> {
        &self.metadata
    }

    pub fn set_metadata(&mut self, key: &'a str, value: String) -> Result<&mut Self> {
        if key == "id" {
            bail!(ErrorKind::IllegalNameForAdditional(String::from(key)));
        }

        self.metadata.insert(key, value);

        Ok(self)
    }
}

impl<'a> From<&'a log::LogRecord<'a>> for Message<'a> {
    fn from(record: &'a log::LogRecord) -> Message<'a> {
        // Create message with given text and level
        let mut msg = Message::new(format!("{}", record.args()),
                                   Some(Level::from_rust(&record.level())));

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
