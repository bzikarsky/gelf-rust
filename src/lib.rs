#![crate_type = "lib"]

extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate serde;
extern crate serde_json;
extern crate rand;
extern crate libflate;

#[macro_use]
extern crate log;

#[macro_use]
extern crate error_chain;

mod util;
mod errors;
mod message;
mod backends;
mod logger;
mod level;

pub use errors::{Error, ErrorKind, Result};
pub use backends::{UdpBackend, TcpBackend, NullBackend};
pub use logger::Logger;
pub use message::{Message, MessageCompression, ChunkSize};
pub use level::Level;