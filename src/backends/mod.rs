use message::WireMessage;
use errors::Result;

mod udp;
mod tcp;

pub use self::udp::UdpBackend;
pub use self::tcp::TcpBackend;

pub trait Backend: Sync + Send {
    fn panic_on_error(&self) -> bool;

    fn log_message(&self, msg: WireMessage) -> Result<()>;

    fn log(&self, msg: WireMessage) {
        let result = self.log_message(msg);

        if self.panic_on_error() && result.is_err() {
            panic!(result.unwrap_err());
        }
    }
}