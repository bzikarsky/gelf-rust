pub use self::udp::UdpBackend;
mod udp;

use message::WireMessage;

pub trait Backend: Sync + Send {
    fn log(&self, msg: WireMessage);
}