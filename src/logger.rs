use Backend;
use CreateLoggerError;
use InitLoggerError;
use log;
use hostname;
use Message;
use message::WireMessage;

pub struct Logger {
    host: String,
    backend: Box<Backend>,
}

impl Logger {
    pub fn init(backend: Box<Backend>,
                log_level: log::LogLevelFilter,
                optional_host: Option<String>)
                -> Result<(), InitLoggerError> {

        let logger = try!(Logger::new(backend, optional_host));

        try!(log::set_logger(|max_level| {
            max_level.set(log_level);
            Box::new(logger)
        }));

        Ok(())
    }

    pub fn new(backend: Box<Backend>, host: Option<String>) -> Result<Self, CreateLoggerError> {
        host.or(hostname::get_hostname())
            .map(|host| {
                Logger {
                    host: host,
                    backend: backend,
                }
            })
            .ok_or(CreateLoggerError::NoHostname)
    }

    pub fn log_message(&self, msg: Message) {
        let wire_msg = WireMessage::new(msg, &self.host);
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
