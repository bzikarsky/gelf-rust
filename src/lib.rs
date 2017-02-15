#![crate_type = "lib"]

extern crate log;
extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate serde;
extern crate serde_json;

mod util;
mod errors;
mod message;
mod backends;
mod logger;
mod level;

pub use errors::CreateLoggerError;
pub use errors::InitLoggerError;
pub use errors::IllegalAdditionalNameError;
pub use errors::CreateBackendError;
pub use backends::Backend;
pub use logger::Logger;
pub use message::Message;
pub use level::Level;