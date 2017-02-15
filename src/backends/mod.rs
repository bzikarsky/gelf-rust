use message::WireMessage;

mod udp;

pub use self::udp::UdpBackend;
pub use self::udp::CHUNK_SIZE_LAN;
pub use self::udp::CHUNK_SIZE_WAN;

pub trait Backend: Sync + Send {
    fn log(&self, msg: WireMessage);
}