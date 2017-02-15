#![crate_type = "lib"]

extern crate log;
extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate libflate;

mod util;
mod errors;
mod message;
mod backends;
mod logger;
mod level;

pub use errors::{CreateLoggerError, InitLoggerError, IllegalAdditionalNameError, CreateBackendError};
pub use backends::{Backend, UdpBackend, CHUNK_SIZE_WAN, CHUNK_SIZE_LAN};
pub use logger::Logger;
pub use message::Message;
pub use level::Level;