use failure;
use failure::Fail;
use std::net;

use crate::{ChunkSize, MessageCompression, Result, Error, WireMessage, Backend};

/// UdpBackend is the default and standard GELF backend
///
/// It pushes messages to a GELF host via UDP. Messages are cut into chunks
/// of a certain chunk-size. This size is important since the chunk-size +
/// a stable overhead of 12 bytes needs to fit the transport layer's mtu.
///
/// If the message fits into a single chunk, no chunking is applied.
pub struct UdpBackend<T> {
    socket: net::UdpSocket,
    destination: T,
    chunk_size: ChunkSize,
    compression: MessageCompression,
}

impl<T: net::ToSocketAddrs + Send + Sync + Clone> UdpBackend<T> {
    /// Construct a new UdpBackend with default chunk-size (ChunkSize::LAN)
    pub fn new(destination: T) -> Result<UdpBackend<T>> {
        Self::new_with_chunksize(destination, ChunkSize::LAN)
    }

    /// Construct an new UdpBackend with the given chunk-size
    pub fn new_with_chunksize(
        destination: T,
        chunk_size: ChunkSize,
    ) -> Result<UdpBackend<T>> {
        // Get a single net::SocketAddr form the destination-address type
        let destination_addr = destination
            .to_socket_addrs()
            .map_err(|e| {
                failure::Error::from(e)
                    .context("Failed to parse a destination address")
                    .context(Error::BackendCreationFailed)
            })?
            .nth(0)
            .ok_or_else(|| format_err!("Invalid destination server address",)
                    .context(Error::BackendCreationFailed))?;

        // Create an appropiate local socket for the given destination
        let local = match destination_addr {
            net::SocketAddr::V4(_) => "0.0.0.0:0",
            net::SocketAddr::V6(_) => "[::]:0",
        };

        let socket = net::UdpSocket::bind(local).map_err(|e| {
            e.context("Failed to bind local socket")
                .context(Error::BackendCreationFailed)
        })?;

        socket.set_nonblocking(true).map_err(|e| {
            e.context("Failed to set UdpSocket to non-blocking mode")
                .context(Error::BackendCreationFailed)
        })?;

        Ok(UdpBackend {
            socket: socket,
            destination: destination,
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

impl<T: net::ToSocketAddrs + Send + Sync + Clone> Backend for UdpBackend<T> {
    /// Log a message via UDP.
    fn log_message(&self, msg: WireMessage) -> Result<()> {
        let chunked_msg = msg.to_chunked_message(self.chunk_size, self.compression)?;
        let chunked_msg_size = chunked_msg.len();
        let sent_bytes = chunked_msg
            .iter()
            .map(
                |chunk| match self.socket.send_to(&chunk, self.destination.clone()) {
                    Err(_) => 0,
                    Ok(size) => size,
                },
            )
            .fold(0_u64, |carry, size| carry + size as u64);

        if sent_bytes != chunked_msg_size {
            bail!(Error::LogTransmitFailed);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use UdpBackend;
    use std::net::UdpSocket;

    #[test]
    fn some() {
        let m = UdpSocket::bind("127.0.0.1:8080").unwrap();
        let res = m.send_to(&[0,0,0,0], "127.0.152.63:8081");

        let t = res.unwrap();
        let asd = 12;
    }

}
