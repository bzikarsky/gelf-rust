//! A simple UDP logging example
//!
//! By default the example tries to start a debug GELF server which logs the
//! GELF JSON messages and chunks to STDOUT. If you want to run your own debug
//! output (or log to a Graylog instance) you can disable the debug server by
//! passing the "--no-server" argument to the example run. E.g.:
//!
//! `cargo run --example simple_udp -- --no-server`
//!
//! It's also possible to specify a remote GELF host location with the flag
//! `--gelf-host <host>`. This allows you to log over IPv6 for example:
//!
//! `cargo run --example simple_udp -- --no-server --gelf-host [::1]:12201`

#[macro_use]
extern crate log;
extern crate gelf;

mod shared;

use gelf::*;
use log::LevelFilter as LogLevelFilter;
use shared::*;

/// Set a filter for log-messages. Messages below the defined level will be ignored
const LOG_FILTER: LogLevelFilter = LogLevelFilter::Trace;

/// Set the compression used for the messages. Compression is
/// disabled for readable examples
const MESSAGE_COMPRESSION: MessageCompression = MessageCompression::None;

/// Set the maximum size of a UDP packet
/// If you want to see chunking, set it to something small like ChunkSize::Custom(50)
const CHUNK_SIZE: ChunkSize = ChunkSize::LAN;

/// Set the hostname which should be used in the GELF messages
static HOSTNAME: &'static str = "test.local";

fn main() {
    // Default options:
    // - gelf_host: The UDP destination string (e.g. 127.0.0.1:12201 or [::1]:12201)
    // - run_debug_server: Whether the example should run its own debug server
    let mut options = Options {
        gelf_host: String::from("127.0.0.1:12201"),
        run_debug_server: true,
    };

    // Read command line options
    options.populate(::std::env::args());

    // Create a UDP backend for given host and chunk_size
    let mut backend = UdpBackend::new_with_chunksize(options.gelf_host.clone(), CHUNK_SIZE)
        .expect("Failed to create a UDP backend");

    // Configure compression (can be ommited, defaults to Gzip)
    backend.set_compression(MESSAGE_COMPRESSION);

    // Create the logger with the given backend
    let mut logger = Logger::new(Box::new(backend)).expect("Failed to create the logger");

    // Configure hostname (can be ommitted, defaults to a local hostname lookup)
    logger.set_hostname(String::from(HOSTNAME));

    // Add an example metadata field which is added to every message which does not contain
    // the key already
    logger.set_default_metadata(
        String::from("facility"),
        String::from(
            ::std::env::current_exe()
                .unwrap()
                .as_path()
                .to_string_lossy(),
        ),
    );

    // Install the logger as a system logger
    logger
        .install(LOG_FILTER)
        .expect("Failed to install the logger");

    // Run debug graylog server if required
    let thread = if options.run_debug_server {
        let host = options.gelf_host.clone();
        Some(::std::thread::spawn(|| {
            run_debug_server_udp(host, 1);
        }))
    } else {
        None
    };

    // Log! Go!
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    // Wait for a possible debug log server to shutdown
    if let Some(handle) = thread {
        handle
            .join()
            .expect("Failed to shutdown debug graylog server");
    }
}
