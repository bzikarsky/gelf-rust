use std::net;

use backends::Backend;
use message::{WireMessage, MessageCompression, ChunkSize};
use errors::{Result, ErrorKind, ResultExt};

pub struct UdpBackend {
    socket: net::UdpSocket,
    destination: net::SocketAddr,
    chunk_size: ChunkSize,
    panic_on_error: bool,
    compression: MessageCompression,
}

impl UdpBackend {
    pub fn new<T: net::ToSocketAddrs>(destination: T) -> Result<UdpBackend> {
        Self::new_with_chunksize(destination, ChunkSize::LAN)
    }

    pub fn new_with_chunksize<T: net::ToSocketAddrs>(destination: T,
                                                     chunk_size: ChunkSize)
                                                     -> Result<UdpBackend> {
        let destination_addr =
            destination.to_socket_addrs()
                .chain_err(|| {
                    ErrorKind::BackendCreationFailed("Failed to parse a destination address")
                })?
                .nth(0)
                .ok_or(ErrorKind::BackendCreationFailed("Invalid destination server address"))?;

        let local = match destination_addr {
            net::SocketAddr::V4(_) => "127.0.0.1:0",
            net::SocketAddr::V6(_) => "[::1]:0",
        };

        let socket = net::UdpSocket::bind(local).chain_err(|| {
                ErrorKind::BackendCreationFailed("Failed to bind local socket")
            })?;


        Ok(UdpBackend {
            socket: socket,
            destination: destination_addr,
            chunk_size: chunk_size,
            panic_on_error: false,
            compression: MessageCompression::default(),
        })
    }

    pub fn set_panic_on_error(&mut self, mode: bool) -> &mut Self {
        self.panic_on_error = mode;
        self
    }

    pub fn compression(&self) -> MessageCompression {
        self.compression
    }

    pub fn set_compression(&mut self, compression: MessageCompression) -> &mut Self {
        self.compression = compression;
        self
    }
}

impl Backend for UdpBackend {
    fn log_message(&self, msg: WireMessage) -> Result<()> {
        let chunked_msg = msg.to_chunked_message(self.chunk_size, self.compression)?;
        let chunked_msg_size = chunked_msg.len();
        let sent_bytes = chunked_msg.iter()
            .map(|chunk| match self.socket.send_to(&chunk, self.destination) {
                Err(_) => 0,
                Ok(size) => size,
            })
            .fold(0_u64, |carry, size| carry + size as u64);

        if sent_bytes != chunked_msg_size {
            bail!(ErrorKind::LogTransmitFailed);
        }

        Ok(())
    }

    fn panic_on_error(&self) -> bool {
        self.panic_on_error
    }
}
