use backends::Backend;
use message::WireMessage;
use errors::Result;

/// The `NullBackend` is a utility backend which discards all messages
pub struct NullBackend;

impl NullBackend {
    /// Construct a new NullBackend
    pub fn new() -> NullBackend {
        NullBackend {}
    }
}

impl Backend for NullBackend {
    fn panic_on_error(&self) -> bool {
        false
    }

    /// Log a message.
    ///
    /// Logging a message with NullBackend is a noop and will never fail.
    fn log_message(&self, _: WireMessage) -> Result<()> {
        Ok(())
    }
}