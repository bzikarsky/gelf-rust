use std::collections::HashMap;

use log;
use hostname;

use backends::Backend;
use message::{Message, WireMessage};
use errors::{Result, ErrorKind};

pub struct Logger {
    hostname: String,
    backend: Box<Backend>,
    default_metadata: HashMap<String, String>,
}

impl Logger {
    pub fn new(backend: Box<Backend>) -> Result<Self> {
        hostname::get_hostname()
            .map(|hostname| {
                Logger {
                    hostname: hostname,
                    backend: backend,
                    default_metadata: HashMap::new(),
                }
            })
            .ok_or(ErrorKind::LoggerCreateFailed("Failed to determine local hostname").into())
    }

    pub fn init<T: Into<log::LogLevelFilter>>(backend: Box<Backend>, log_level: T) -> Result<()> {
        Logger::new(backend)?.install(log_level)
    }

    pub fn install<T: Into<log::LogLevelFilter>>(self, log_level: T) -> Result<()> {
        log::set_logger(|max_level| {
            max_level.set(Into::into(log_level));
            Box::new(self)
        })?;

        Ok(())
    }

    pub fn log_message(&self, msg: Message) {
        self.backend.log(WireMessage::new(msg, &self));
    }

    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    pub fn set_hostname(&mut self, hostname: String) -> &mut Self {
        self.hostname = hostname;
        self
    }

    pub fn default_metadata(&self) -> &HashMap<String, String> {
        &self.default_metadata
    }

    pub fn add_default_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.default_metadata.insert(key, value);
        self
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::LogMetadata) -> bool {
        true
    }

    fn log(&self, record: &log::LogRecord) {
        if !self.enabled(record.metadata()) {
            ()
        }

        self.log_message(From::from(record))
    }
}
