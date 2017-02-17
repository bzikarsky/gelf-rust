use log::{LogLevel, LogLevelFilter};

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
    pub fn from_rust(level: &LogLevel) -> Level {
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

impl Into<LogLevel> for Level {
    fn into(self) -> LogLevel {
        self.to_rust()
    }
}

impl From<LogLevel> for Level {
    fn from(level: LogLevel) -> Level {
        Level::from_rust(&level)
    }
}

impl Into<LogLevelFilter> for Level {
    fn into(self) -> LogLevelFilter {
        self.to_rust().to_log_level_filter()
    }
}