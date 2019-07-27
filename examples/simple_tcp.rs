//! A simple TCP logging example
//!
//! By default the example tries to start a debug GELF server which logs the
//! GELF JSON messages to STDOUT. If you want to run your own debug output (or
//! log to a Graylog instance) you can disable the debug server by passing the
//! "--no-server" argument to the example run. E.g.:
//!
//! `cargo run --example simple_tcp -- --no-server`
//!
//! It's also possible to specify a remote GELF host location with the flag
//! `--gelf-host <host>`. This allows you to log over IPv6 for example:
//!
//! `cargo run --example simple_tcp -- --no-server --gelf-host [::1]:12201`

#[macro_use]
extern crate log;
extern crate gelf;

mod shared;

use gelf::*;
use log::LevelFilter as LogLevelFilter;
use shared::*;

/// Set a filter for log-messages. Messages below the defined level will be ignored
const LOG_FILTER: LogLevelFilter = LogLevelFilter::Trace;

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

    // Run debug graylog server if required
    let thread = if options.run_debug_server {
        let host = options.gelf_host.clone();
        let handle = Some(::std::thread::spawn(|| {
            run_debug_server_tcp(host, 5);
        }));

        // Wait for the server to start
        ::std::thread::sleep(::std::time::Duration::new(1, 0));
        handle
    } else {
        None
    };

    // Create a UDP backend for given host and chunk_size
    let backend =
        TcpBackend::new(options.gelf_host.as_str()).expect("Failed to create a TCP backend");

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
