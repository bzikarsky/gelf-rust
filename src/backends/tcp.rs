use failure;
use failure::Fail;
use std::io::Write;
use std::net;
use std::sync;

use backends::Backend;
use errors::{Error, Result};
use message::{MessageCompression, WireMessage};

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
        let socket = net::TcpStream::connect(destination).map_err(|e| {
            failure::Error::from(e)
                .context("Failed to establish TCP connection")
                .context(Error::BackendCreationFailed)
        })?;

        socket.set_nonblocking(true).map_err(|e| {
            e.context("Failed to set TcpStream to non-blocking mode")
                .context(Error::BackendCreationFailed)
        })?;

        Ok(TcpBackend {
            socket: sync::Arc::new(sync::Mutex::new(socket)),
            compression: MessageCompression::default(),
        })
    }

    /// Return the current set compression algorithm
    pub fn compression(&self) -> MessageCompression {
        self.compression
    }

    /// Set the compression algorithm
    pub fn set_compression(&mut self, compression: MessageCompression) -> &mut Self {
        self.compression = compression;
        self
    }
}

impl Backend for TcpBackend {
    /// Log a message over TCP.
    fn log_message(&self, msg: WireMessage) -> Result<()> {
        let mut msg = msg.to_compressed_gelf(self.compression)?;

        // raw messages need to be terminated with a 0-byte
        msg.push(0x00);

        let mut socket = self.socket.lock().unwrap();

        socket
            .write_all(&msg)
            .map_err(|e| e.context(Error::LogTransmitFailed))?;

        Ok(())
    }
}

impl Drop for TcpBackend {
    /// Try to close the connection gracefully when TcpBackend goes out of scope
    fn drop(&mut self) {
        // When drop() is called unwrap() should never fail
        let mut socket = self.socket.lock().unwrap();

        socket
            .flush()
            .and_then(|_| socket.shutdown(net::Shutdown::Both))
            .unwrap_or_else(|_| warn!("Failed to flush and shutdown tcp socket cleanly"));
    }
}
