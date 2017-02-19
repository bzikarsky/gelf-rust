use std::net;

use backends::Backend;
use message::{WireMessage, MessageCompression, ChunkSize};
use errors::{Result, ErrorKind, ResultExt};

/// UdpBackend is the default and standard GELF backend
///
/// It pushes messages to a GELF host via UDP. Messages are cut into chunks
/// of a certain chunk-size. This size is important since the chunk-size +
/// a stable overhead of 12 bytes needs to fit the transport layer's mtu.
///
/// If the message fits into a single chunk, no chunking is applied.
pub struct UdpBackend {
    socket: net::UdpSocket,
    destination: net::SocketAddr,
    chunk_size: ChunkSize,
    compression: MessageCompression,
}

impl UdpBackend {
    /// Construct a new UdpBackend with default chunk-size (ChunkSize::LAN)
    pub fn new<T: net::ToSocketAddrs>(destination: T) -> Result<UdpBackend> {
        Self::new_with_chunksize(destination, ChunkSize::LAN)
    }

    /// Construct an new UdpBackend with the given chunk-size
    pub fn new_with_chunksize<T: net::ToSocketAddrs>(destination: T,
                                                     chunk_size: ChunkSize)
                                                     -> Result<UdpBackend> {
        // Get a single net::SocketAddr form the destination-address type
        let destination_addr =
            destination.to_socket_addrs()
                .chain_err(|| {
                    ErrorKind::BackendCreationFailed("Failed to parse a destination address")
                })?
                .nth(0)
                .ok_or(ErrorKind::BackendCreationFailed("Invalid destination server address"))?;

        // Create an appropiate local socket for the given destination
        let local = match destination_addr {
            net::SocketAddr::V4(_) => "0.0.0.0:0",
            net::SocketAddr::V6(_) => "[::]:0",
        };

        let socket = net::UdpSocket::bind(local).chain_err(|| {
                ErrorKind::BackendCreationFailed("Failed to bind local socket")
            })?;

        Ok(UdpBackend {
            socket: socket,
            destination: destination_addr,
            chunk_size: chunk_size,
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

impl Backend for UdpBackend {
    /// Log a message via UDP.
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
}
