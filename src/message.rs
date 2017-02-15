use chrono::{DateTime, UTC};
use std::collections::HashMap;
use serde;
use serde::ser::SerializeMap;
use serde_json;

use errors::IllegalAdditionalNameError;
use log;
use util;
use Level;

pub struct Message<'a> {
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<DateTime<UTC>>,
    level: Option<Level>,
    additionals: HashMap<&'a str, String>,
}

impl<'a> Message<'a> {
    pub fn new(short_message: String, level: Option<Level>) -> Self {
        Message {
            short_message: short_message,
            level: level,
            full_message: None,
            timestamp: None,
            additionals: HashMap::new(),
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

    pub fn get_additional(&self, key: &str) -> Option<&String> {
        self.additionals.get(key)
    }

    pub fn set_additional(&mut self,
                          key: &'a str,
                          value: String)
                          -> Result<(), IllegalAdditionalNameError> {
        if key == "id" {
            return Err(IllegalAdditionalNameError(key));
        }

        self.additionals.insert(key, value);

        Ok(())
    }
}

impl<'a> From<&'a log::LogRecord<'a>> for Message<'a> {
    fn from(record: &'a log::LogRecord) -> Message<'a> {
        // Create message with given text and level
        let mut msg = Message::new(format!("{}", record.args()),
                                   Some(Level::from_rust(&record.level())));


        msg.set_timestamp(UTC::now());

        // Add location meta-data
        msg.additionals.insert("file", record.location().file().to_owned());
        msg.additionals.insert("line", record.location().line().to_string());
        msg.additionals.insert("module_path", record.location().module_path().to_owned());

        // Add runtime meta-data
        msg.additionals.insert("process_id", util::pid().to_string());

        msg
    }
}

pub struct WireMessage<'a> {
    host: &'a str,
    version: &'static str,
    short_message: String,
    full_message: Option<String>,
    timestamp: Option<DateTime<UTC>>,
    level: Option<Level>,
    additionals: HashMap<&'a str, String>,
}

impl<'a> WireMessage<'a> {
    pub fn new(msg: Message<'a>, host: &'a str) -> Self {
        WireMessage {
            version: "1.1",
            host: host,
            short_message: msg.short_message,
            full_message: msg.full_message,
            timestamp: msg.timestamp,
            level: msg.level,
            additionals: msg.additionals,
        }
    }

    pub fn to_json_string(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

impl<'a> serde::Serialize for WireMessage<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let mut map = serializer.serialize_map(None)?;

        map.serialize_key("version")?;
        map.serialize_value(self.version)?;

        map.serialize_key("host")?;
        map.serialize_value(self.host)?;

        map.serialize_key("short_message")?;
        map.serialize_value(&self.short_message)?;

        if self.full_message.is_some() {
            map.serialize_key("full_message")?;
            map.serialize_value(&self.full_message)?;
        }

        if self.timestamp.is_some() {
            map.serialize_key("timestamp")?;
            map.serialize_value(&self.timestamp.unwrap().timestamp())?;
        }

        if self.level.is_some() {
            let level = self.level.unwrap() as i8;
            map.serialize_key("level")?;
            map.serialize_value(&level)?;
        }

        for (key, value) in self.additionals.iter() {
            let key = "_".to_string() + key;
            map.serialize_key(&key)?;
            map.serialize_value(value)?;
        }

        map.end()
    }
}
