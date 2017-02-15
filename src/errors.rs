use std::fmt;
use std::error;
use std::io;

use log;

#[derive(Debug)]
pub enum CreateLoggerError {
    NoHostname,
}

impl fmt::Display for CreateLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CreateLoggerError::NoHostname => {
                write!(fmt, "No hostname could be determined for the GELF logger")
            }
        }
    }
}

impl error::Error for CreateLoggerError {
    fn description(&self) -> &str {
        match *self {
            CreateLoggerError::NoHostname => "No hostname could be determined for the GELF logger",
        }
    }
}

#[derive(Debug)]
pub enum InitLoggerError {
    Create(CreateLoggerError),
    Install(log::SetLoggerError),
}

impl fmt::Display for InitLoggerError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InitLoggerError::Create(ref err) => write!(fmt, "Logger creation failed: {}", err),
            InitLoggerError::Install(ref err) => write!(fmt, "Logger instalation failed: {}", err),
        }
    }
}

impl error::Error for InitLoggerError {
    fn description(&self) -> &str {
        match *self {
            InitLoggerError::Create(_) => "Logger creation failed",
            InitLoggerError::Install(_) => "Logger instalation failed",
        }
    }
}

impl From<log::SetLoggerError> for InitLoggerError {
    fn from(err: log::SetLoggerError) -> InitLoggerError {
        InitLoggerError::Install(err)
    }
}

impl From<CreateLoggerError> for InitLoggerError {
    fn from(err: CreateLoggerError) -> InitLoggerError {
        InitLoggerError::Create(err)
    }
}

#[derive(Debug)]
pub struct IllegalAdditionalNameError<'a>(pub &'a str);

impl<'a> fmt::Display for IllegalAdditionalNameError<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt,
               "'{}' is not a legal name for an additional GELF field",
               self.0)
    }
}

impl<'a> error::Error for IllegalAdditionalNameError<'a> {
    fn description(&self) -> &str {
        "The specified name is not a legal name for an additional GELF field"
    }
}

#[derive(Debug)]
pub struct CreateBackendError(pub &'static str);

impl fmt::Display for CreateBackendError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Failed to create a GELF backennd: {}", self.0)
    }
}

impl error::Error for CreateBackendError {
    fn description(&self) -> &str {
        "Failed to create a GELF backend"
    }
}

impl From<io::Error> for CreateBackendError {
    fn from(err: io::Error) -> CreateBackendError {
        CreateBackendError("An unhandled IO error occured")
    }
}
