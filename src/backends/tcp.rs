use std::net;
use std::io::Write;
use std::sync;

use backends::Backend;
use message::{WireMessage, MessageCompression};
use errors::{Result, ErrorKind, ResultExt};

/// TcpBackend is a simple GELF over TCP backend.
///
/// WireMessages are simply serialized and optionally compressed and pushed to
/// a Gelf host over TCP. TCP's stream-based nature requires no chunking.
pub struct TcpBackend {
    socket: sync::Arc<sync::Mutex<net::TcpStream>>,
    compression: MessageCompression,
}

impl TcpBackend {
    /// Construct a new TcpBackend.
    pub fn new<T: net::ToSocketAddrs>(destination: T) -> Result<TcpBackend> {
        let socket =
            net::TcpStream::connect(destination).chain_err(|| {
                    ErrorKind::BackendCreationFailed("Failed to establish TCP connection")
                })?;

        Ok(TcpBackend {
            socket: sync::Arc::new(sync::Mutex::new(socket)),
            compression: MessageCompression::default(),
        })
    }
}

impl Backend for TcpBackend {
    /// Log a message over TCP.
    fn log_message(&self, msg: WireMessage) -> Result<()> {
        let msg = msg.to_compressed_gelf(self.compression)?;
        let mut socket = self.socket.lock().unwrap();

        socket.write_all(&msg).chain_err(|| ErrorKind::LogTransmitFailed)
    }
}

impl Drop for TcpBackend {
    /// Try to close the connection gracefully when TcpBackend goes out of scope
    fn drop(&mut self) {
        // When drop() is called unwrap() should never fail
        let mut socket = self.socket.lock().unwrap();

        socket.flush()
            .and_then(|_| socket.shutdown(net::Shutdown::Both))
            .unwrap_or_else(|_| warn!("Failed to flush and shutdown tcp socket cleanly"));
    }
}