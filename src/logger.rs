use std::collections::HashMap;

use hostname;
use log;
use log::set_boxed_logger;
use crate::{Backend, Error, Message, WireMessage};
use crate::errors::Result;

/// Logger for sending log-messages
///
/// A `Logger` instance can be either used as a standalone object to log directly
/// to a log-server or it can be installed as a `log`-crate log-handler (with `Logger::install`).
///
/// By default all encountered errors will be silently ignored. If you want the logger
/// to panic when an error occurs, you can change the behaviour with `Logger::enable_panic_on_error`.
pub struct Logger {
    hostname: String,
    backend: Box<dyn Backend>,
    default_metadata: HashMap<String, String>,
    panic_on_error: bool,
}

impl Logger {
    /// Construct a new `Logger` instance
    ///
    /// The backend needs to be boxed for usage as a logger with the `log`-crate.
    /// This constructor tries to determine the local hostname (required by GELF)
    /// with the help of the `hostname`-crate. If you want to set a custom hostname
    /// check out the `Logger::new_with_hostname` constructor.
    pub fn new(backend: Box<dyn Backend>) -> Result<Self> {
        hostname::get_hostname()
            .map(|hostname| Logger::new_with_hostname(backend, &hostname))
            .ok_or_else(|| format_err!("Failed to determine local hostname")
                    .context(Error::LoggerCreateFailed)
                    .into())
    }

    /// Construct a new `Logger` instance with predetermined hostname
    ///
    /// The backend needs to be boxed for usage as a logger with the `log`-crate. It
    /// uses the passed hostname for the GELF `host` field
    pub fn new_with_hostname(backend: Box<dyn Backend>, hostname: &str) -> Logger {
        Logger {
            hostname: String::from(hostname),
            backend: backend,
            default_metadata: HashMap::new(),
            panic_on_error: false,
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
    pub fn install<T: Into<log::LevelFilter>>(self, log_level: T) -> Result<()> {
        set_boxed_logger(Box::new(self))?;
        log::set_max_level(log_level.into());

        Ok(())
    }

    /// Log a message via the logger's transport to a GELF server.
    ///
    /// The logger will automatically add `default_metadata` fields to the message
    /// if missing in the passed `Message`.
    pub fn log_message(&self, msg: Message) {
        let result = self.backend.log_message(WireMessage::new(msg, &self));

        if result.is_err() && self.panic_on_error {
            panic!(result.unwrap_err());
        }
    }

    /// Return the hostname used for GELF's `host`-field
    pub fn hostname(&self) -> &String {
        &self.hostname
    }

    /// Set the hostname used for GELF's `host`-field
    pub fn set_hostname<S: Into<String>>(&mut self, hostname: S) -> &mut Self {
        self.hostname = hostname.into();
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
    /// logger.log_message(Message::new(String::from("This is important information")));
    /// // -> The message will contain an additional field "_facility" with the value "my_awesome_rust_service"
    /// ```
    pub fn set_default_metadata<S, T>(
        &mut self,
        key: S,
        value: T
    ) -> &mut Self
    where
        S: Into<String>,
        T: Into<String>
    {
        self.default_metadata.insert(key.into(), value.into());
        self
    }

    /// Return a flag whether the logger panics when it encounters an error
    pub fn panic_on_error(&self) -> bool {
        self.panic_on_error
    }

    /// Force the logger to panic when it encounters an error
    pub fn enable_panic_on_error(&mut self) -> &mut Self {
        self.panic_on_error = true;
        self
    }

    /// Force the logger to ignore an encountered error silently
    pub fn disable_panic_on_error(&mut self) -> &mut Self {
        self.panic_on_error = false;
        self
    }
}

impl log::Log for Logger {
    /// Determines if a log message with the specified metadata would be logged.
    ///
    /// See [docs](https://doc.rust-lang.org/log/log/trait.Log.html#tymethod.enabled)
    /// for more details
    fn enabled(&self, _: &log::Metadata) -> bool {
        // The logger does not dicard any log-level by itself, therefore it is
        // always enabled
        true
    }

    /// Logs the `LogRecord`.
    /// See [docs](https://doc.rust-lang.org/log/log/trait.Log.html#tymethod.log)
    /// for more details
    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            ()
        }

        self.log_message(From::from(record))
    }

    fn flush(&self) {}
}
