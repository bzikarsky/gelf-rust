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

/// Set the maximum size of a UDP packet
/// If you want to see chunking, set it to something small like ChunkSize::Custom(50)
const CHUNK_SIZE: ChunkSize = ChunkSize::LAN;

/// Set the hostname which should be used in the GELF messages
static HOSTNAME: &'static str = "test.local";

fn main() {
    // Create a UDP backend for given host and chunk_size
    let mut backend = UdpBackend::new_with_chunksize(GRAYLOG_HOST, CHUNK_SIZE)
        .expect("Failed to create a UDP backend");

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

    // Run debug graylog server
    let thread = ::std::thread::spawn(|| { shared::run_debug_server_udp(GRAYLOG_HOST, 1); });

    // Log! Go!
    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    // Wait for debug log server to shutdown
    thread.join().expect("Failed to shutdown debug graylog server");
}
