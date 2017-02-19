#[macro_use]
extern crate log;
extern crate gelf;

mod shared;

use gelf::*;
use log::LogLevelFilter;

/// Set the hostname of the graylog hsot to log to
///
/// In this example we also create debug listener on this interface (to print the
/// logged messages). Therefore this should be on the localhost (use [::1]:12201 for IPv6)
static GRAYLOG_HOST: &'static str = "127.0.0.1:12201";

/// Set a filter for log-messages. Messages below the defined level will be ignored
const LOG_FILTER: LogLevelFilter = LogLevelFilter::Trace;

/// Set the compression used for the messages. Compression is
/// disabled for readable examples
const MESSAGE_COMPRESSION: MessageCompression = MessageCompression::None;

/// Set the hostname which should be used in the GELF messages
static HOSTNAME: &'static str = "test.local";

fn main() {
    // Run debug graylog server (and make sure its started)
    let thread = ::std::thread::spawn(|| { shared::run_debug_server_tcp(GRAYLOG_HOST, 5); });

    // Create a UDP backend for given host and chunk_size
    let mut backend = TcpBackend::new(GRAYLOG_HOST).expect("Failed to create a TCP backend");

    // Configure compression (can be ommited, defaults to Gzip)
    backend.set_compression(MESSAGE_COMPRESSION);

    // Create the logger with the given backend
    let mut logger = Logger::new(Box::new(backend)).expect("Failed to create the logger");

    // Configure hostname (can be ommitted, defaults to a local hostname lookup)
    logger.set_hostname(String::from(HOSTNAME));

    // Add an example metadata field which is added to every message which does not contain
    // the key already
    logger.set_default_metadata(String::from("facility"),
                                String::from(::std::env::current_exe()
                                    .unwrap()
                                    .as_path()
                                    .to_string_lossy()));

    // Install the logger as a system logger
    logger.install(LOG_FILTER).expect("Failed to install the logger");

    // Log! Go!
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    // Wait for debug log server to shutdown
    thread.join().expect("Failed to shutdown debug graylog server");
}
