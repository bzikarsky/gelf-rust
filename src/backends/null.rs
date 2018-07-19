use backends::Backend;
use errors::Result;
use message::WireMessage;

/// The `NullBackend` is a utility backend which discards all messages
pub struct NullBackend;

impl NullBackend {
    /// Construct a new NullBackend
    pub fn new() -> NullBackend {
        NullBackend {}
    }
}

impl Backend for NullBackend {
    /// Log a message.
    ///
    /// Logging a message with NullBackend is a noop and will never fail.
    fn log_message(&self, _: WireMessage) -> Result<()> {
        Ok(())
    }
}
