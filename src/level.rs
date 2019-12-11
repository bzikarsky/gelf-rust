use log::{Level as LogLevel, LevelFilter as LogLevelFilter};

/// GELF's representation of an error level
///
/// GELF's error levels are equivalent to syslog's severity
/// information (specified in [RFC 5424](https://tools.ietf.org/html/rfc5424))
///
/// The levels dont match `log`s levels, but (lossy) conversion methods
/// are provided. These methods follow this conversion table:
///
/// | GELF / Syslog     | Rust      |
/// |-------------------|-----------|
/// | Emergency (0)     | Error (1) |
/// | Alert (1)         | Error (1) |
/// | Critical (2)      | Error (1) |
/// | Error (3)         | Error (1) |
/// | Warning (4)       | Warn (2)  |
/// | Notice (5)        | Info (3)  |
/// | Informational (6) | Info (3)  |
/// | Debug (7)         | Debug (4) |
/// | Debug (7)         | Trace (5) |
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
    /// Get the GELF error level from given Rust error level
    pub fn from_rust(level: LogLevel) -> Level {
        match level {
            LogLevel::Error => Level::Error,
            LogLevel::Warn => Level::Warning,
            LogLevel::Info => Level::Informational,
            LogLevel::Debug | LogLevel::Trace => Level::Debug,
        }
    }

    /// Get the Rust error level from this GELF error level
    pub fn to_rust(self) -> LogLevel {
        match self {
            Level::Emergency | Level::Alert | Level::Critical | Level::Error => LogLevel::Error,
            Level::Warning => LogLevel::Warn,
            Level::Notice | Level::Informational => LogLevel::Info,
            Level::Debug => LogLevel::Debug,
        }
    }
}

impl Into<LogLevel> for Level {
    /// Allow for Into conversion to Rust's LogLevel
    fn into(self) -> LogLevel {
        self.to_rust()
    }
}

impl From<LogLevel> for Level {
    /// Allow for Into conversion from Rust's LogLevel
    fn from(level: LogLevel) -> Level {
        Level::from_rust(level)
    }
}

impl Into<LogLevelFilter> for Level {
    /// Allow for Into conversion from Rust's LogLevelFilter
    fn into(self) -> LogLevelFilter {
        self.to_rust().to_level_filter()
    }
}
