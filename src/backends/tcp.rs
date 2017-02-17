use std::net;
use std::io::Write;
use std::sync;

use backends::Backend;
use message::{WireMessage, MessageCompression};
use errors::{Result, ErrorKind, ResultExt};

pub struct TcpBackend {
    panic_on_error: bool,
    socket: sync::Arc<sync::Mutex<net::TcpStream>>,
    compression: MessageCompression,
}

impl TcpBackend {
    pub fn new<T: net::ToSocketAddrs>(destination: T) -> Result<TcpBackend> {
        let socket =
            net::TcpStream::connect(destination).chain_err(|| {
                    ErrorKind::BackendCreationFailed("Failed to establish TCP connection")
                })?;

        Ok(TcpBackend {
            panic_on_error: false,
            socket: sync::Arc::new(sync::Mutex::new(socket)),
            compression: MessageCompression::default(),
        })
    }
}

impl Backend for TcpBackend {
    fn panic_on_error(&self) -> bool {
        self.panic_on_error
    }

    fn log_message(&self, msg: WireMessage) -> Result<()> {
        let msg = msg.to_compressed_gelf(self.compression)?;
        let mut socket = self.socket.lock().unwrap();

        socket.write_all(&msg).chain_err(|| ErrorKind::LogTransmitFailed)
    }
}

impl Drop for TcpBackend {
    fn drop(&mut self) {
        // When drop() is called unwrap() should never fail
        let mut socket = self.socket.lock().unwrap();

        socket.flush()
            .and_then(|_| socket.shutdown(net::Shutdown::Both))
            .unwrap_or_else(|_| warn!("Failed to flush and shutdown tcp socket cleanly"));
    }
}