use std::collections::HashMap;

use log;
use hostname;

use backends::Backend;
use message::{Message, WireMessage};
use errors::{Result, ErrorKind};

/// Logger for sending log-messages
///
/// A `Logger` instance can be either used as a standalone object to log directly
/// to a log-server or it can be installed as a `log`-crate log-handler (with `Logger::install`).
pub struct Logger {
    hostname: String,
    backend: Box<Backend>,
    default_metadata: HashMap<String, String>,
}

impl Logger {
    /// Construct a new `Logger` instance
    ///
    /// The backend needs to be boxed for usage as a logger with the `log`-crate.
    /// This constructor tries to determine the local hostname (required by GELF)
    /// with the help of the `hostname`-crate. If you want to set a custom hostname
    /// check out the `Logger::new_with_hostname` constructor
    pub fn new(backend: Box<Backend>) -> Result<Self> {
        hostname::get_hostname()
            .map(|hostname| Logger::new_with_hostname(backend, &hostname))
            .ok_or(ErrorKind::LoggerCreateFailed("Failed to determine local hostname").into())
    }

    /// Construct a new `Logger` instance with predetermined hostname
    ///
    /// The backend needs to be boxed for usage as a logger with the `log`-crate. It
    /// uses the passed hostname for the GELF `host` field
    pub fn new_with_hostname(backend: Box<Backend>, hostname: &str) -> Logger {
        Logger {
            hostname: String::from(hostname),
            backend: backend,
            default_metadata: HashMap::new(),
        }
    }

    /// Install a logger instance as a `log`-Logger
    ///
    /// This method wraps `log::set_logger` as a convenience function as required
    /// by the `log`-crate. `log_level` defines the maximum log-level the logger
    /// should log.
    ///
    /// Note that installing the logger consumes it. To uninstall you need to call
    /// `log::shutdown_logger` which returns the boxed, original `Logger` instance.
    pub fn install<T: Into<log::LogLevelFilter>>(self, log_level: T) -> Result<()> {
        log::set_logger(|max_level| {
            max_level.set(Into::into(log_level));
            Box::new(self)
        })?;

        Ok(())
    }

    /// Log a message via the logger's transport to a GELF server.
    ///
    /// The logger will automatically all `default_metadata` fields to the message
    /// which are missing in the passed `Message`.
    pub fn log_message(&self, msg: Message) {
        self.backend.log(WireMessage::new(msg, &self));
    }

    /// Return the hostname used for GELF's `host`-field
    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    /// Set the hostname used for GELF's `host`-field
    pub fn set_hostname(&mut self, hostname: String) -> &mut Self {
        self.hostname = hostname;
        self
    }

    /// Return all default metadata
    pub fn default_metadata(&self) -> &HashMap<String, String> {
        &self.default_metadata
    }

    /// Set a default metadata field
    ///
    /// Every logged `Message` is checked for every default_metadata field.
    /// If it contains an entry with the key, the default is ignored. But if
    /// there is no additional information present, the default is added to the message.
    ///
    /// This can be used for example to add a `facility` to every message:
    ///
    /// ```
    /// # use gelf::{Logger, NullBackend, Message};
    /// # let backend = NullBackend::new();
    /// # let mut logger = Logger::new(Box::new(backend)).unwrap();
    /// logger.set_default_metadata(String::from("facility"), String::from("my_awesome_rust_service"));
    ///
    /// logger.log_message(Message::new(String::from("This is important information"), None));
    /// // -> The message will contain an additional field "_facility" with the value "my_awesome_rust_service"
    /// ```
    pub fn set_default_metadata(&mut self, key: String, value: String) -> &mut Self {
        self.default_metadata.insert(key, value);
        self
    }
}

impl log::Log for Logger {
    /// Determines if a log message with the specified metadata would be logged.
    ///
    /// See [docs](https://doc.rust-lang.org/log/log/trait.Log.html#tymethod.enabled)
    /// for more details
    fn enabled(&self, _: &log::LogMetadata) -> bool {
        // The logger does not dicard any log-level by itself, therefore it is
        // always enabled
        true
    }

    /// Logs the `LogRecord`.
    /// See [docs](https://doc.rust-lang.org/log/log/trait.Log.html#tymethod.log)
    /// for more details
    fn log(&self, record: &log::LogRecord) {
        if !self.enabled(record.metadata()) {
            ()
        }

        self.log_message(From::from(record))
    }
}
