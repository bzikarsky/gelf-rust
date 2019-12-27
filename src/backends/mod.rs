mod null;
mod tcp;
mod udp;

pub use self::null::NullBackend;
pub use self::tcp::TcpBackend;
pub use self::udp::UdpBackend;

use crate::{WireMessage, Result};

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
