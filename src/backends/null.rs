use backends::Backend;
use message::WireMessage;
use errors::Result;

pub struct NullBackend;

impl NullBackend {
    fn new() -> NullBackend {
        NullBackend {}
    }
}

impl Backend for NullBackend {
    fn panic_on_error(&self) -> bool {
        false
    }

    fn log_message(&self, _: WireMessage) -> Result<()> {
        Ok(())
    }
}