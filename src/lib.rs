//! A GELF library for Rust.
//!
//! The library can be used either as a standalone logging library
//! or it can be used as a logging-subsystem with the
//! [`log`-crate](https://doc.rust-lang.org/log/log/index.html)
//!
//! # Use
//!
//! In general you should only use this crate in applications. Libraries should
//! only develop against [`log`](https://doc.rust-lang.org/log/log/index.html).
//! This then allows applications to use a custom logging-framework (like `gelf`)
//! to do the actual logging.
//!
//! ## Standalone
//!
//! Standalone usage is recommended if the lightweight `log`-crate's features
//! are not sufficient. In this case you need to inject/use an instance of
//! `gelf::Logger` directly. This allows for sending custom built `gelf::Message`-objects.
//! These messages can use all bells and whistles of GELF.
//!
//! ### Example
//!
//! ```
//! extern crate gelf;
//!
//! # use gelf::NullBackend;
//! use gelf::{Logger, UdpBackend, Message, Level};
//!
//! pub fn main() {
//!     // Set up logging
//!     let backend = UdpBackend::new("127.0.0.1:12201")
//!         .expect("Failed to create UDP backend");
//!     # let backend = NullBackend::new();
//!     let mut logger = Logger::new(Box::new(backend))
//!         .expect("Failed to determine hostname");
//!     logger.set_default_metadata(String::from("facility"),
//!          String::from("example-rust-app"));
//!
//!     // Create a (complex) message
//!     let mut message = Message::new(String::from("Custom message!"));
//!     message
//!         .set_full_message(String::from("The full message text is more descriptive"))
//!         .set_metadata("foo", String::from("bar")).unwrap()
//!         .set_metadata("baz", String::from("bat")).unwrap();
//!
//!     // Log it
//!     logger.log_message(message);
//! }
//! ```
//!
//! ## With `log`
//!
//! Usage with `log` allows to log easily with the help of its macros. There is no need to
//! inject or access the logger object anywhere in your application.
//!
//! All the context information (line, file, etc.) the `log`-crate provides is added as metadata
//! to the logged GELF message.
//!
//! ```
//! #[macro_use]
//! extern crate log;
//!
//! extern crate gelf;
//!
//! # use gelf::NullBackend;
//! use gelf::{Logger, UdpBackend, Message, Level};
//! use log::LevelFilter;
//!
//! pub fn main() {
//!     let backend = UdpBackend::new("127.0.0.1:12201")
//!         .expect("Failed to create UDP backend");
//!     # let backend = NullBackend::new();
//!
//!     // Init logging system
//!     let logger = Logger::new(Box::new(backend))
//!         .expect("Failed to determine hostname");
//!     logger.install(LevelFilter::Trace)
//!         .expect("Failed to install logger");
//!
//!     info!("Descend into our program!");
//!     somewhere()
//! }
//!
//! pub fn somewhere() {
//!     trace!("Trace something here!");
//!     over::the_rainbow();
//! }
//!
//! mod over {
//!     pub fn the_rainbow() {
//!         error!("Oh well...");
//!     }
//! }
//! ```
#![crate_type = "lib"]

extern crate chrono;
extern crate hostname;
extern crate libc;
extern crate libflate;
extern crate rand;
extern crate serde;

#[cfg_attr(test, macro_use)]
extern crate serde_json;

#[macro_use]
extern crate log;

#[macro_use]
extern crate failure;

mod backends;
mod errors;
mod level;
mod logger;
mod message;
mod util;

pub use backends::{Backend, NullBackend, TcpBackend, UdpBackend};
pub use errors::{Error, Result};
pub use level::Level;
pub use logger::Logger;
pub use message::{ChunkSize, Message, MessageCompression};
