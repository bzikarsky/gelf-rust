use message::WireMessage;
use errors::Result;

mod udp;
mod tcp;
mod null;

pub use self::udp::UdpBackend;
pub use self::tcp::TcpBackend;
pub use self::null::NullBackend;

/// A trait for a GELF backend
///
/// A backend is responsible for transporting a `WireMessage` to a
/// Gelf host. It is responsible for creating required sockets and chosing
/// proper serialization and encoding options (e.g. chunking with
/// `ChunkedMessage` or compression with `MessageCompression`)
pub trait Backend: Sync + Send {
    /// Log a message.
    fn log_message(&self, msg: WireMessage) -> Result<()>;
}