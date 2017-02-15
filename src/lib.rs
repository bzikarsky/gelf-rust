#![crate_type = "lib"]

extern crate log;
extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate serde;
extern crate serde_json;

use log::LogLevel;

pub use errors::CreateLoggerError;
pub use errors::InitLoggerError;
pub use errors::IllegalAdditionalNameError;
mod errors;

pub use backends::Backend;
mod backends;

pub use logger::Logger;
mod logger;

pub use message::Message;
mod message;

mod util;

#[derive(Debug, Clone, Copy)]
pub enum Level {
    Emergency = 0,
    Alert = 1,
    Critical = 2,
    Error = 3,
    Warning = 4,
    Notice = 5,
    Informational = 6,
    Debug = 7,
}

impl Level {
    pub fn from_rust(level: &log::LogLevel) -> Level {
        match *level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warning,
            LogLevel::Info => Level::Informational,
            LogLevel::Debug | LogLevel::Trace => Level::Debug,
        }
    }

    pub fn to_rust(&self) -> LogLevel {
        match *self {
            Level::Emergency | Level::Alert | Level::Critical | Level::Error => LogLevel::Error,
            Level::Warning => LogLevel::Warn,
            Level::Notice | Level::Informational => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
        }
    }
}



#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
