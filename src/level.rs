use log::{Level as LogLevel, LevelFilter as LogLevelFilter};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
#[derive(Debug, Clone, Copy, PartialEq)]
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

impl<'de> Deserialize<'de> for Level {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error> where
        D: Deserializer<'de> {
        serde_json::Value::deserialize(deserializer)?
            .as_i64()
            .map(Level::from)
            .ok_or_else(|| serde::de::Error::custom("Expected i64 for Log Level"))
    }
}

impl Serialize for Level {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer {
        serializer.serialize_i8(self.into())
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

impl Into<i8> for &Level {
    fn into(self) -> i8 {
        *self as i8
    }
}

impl From<i64> for Level {
    fn from(value: i64) -> Self {
        match value {
            0 => Level::Emergency,
            1 => Level::Alert,
            2 => Level::Critical,
            3 => Level::Error,
            4 => Level::Warning,
            5 => Level::Notice,
            6 => Level::Informational,
            7 => Level::Debug,
            _ => Level::Informational
        }
    }
}
