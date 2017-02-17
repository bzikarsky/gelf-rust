error_chain!{ 
    foreign_links {
        LoggerInitFailed(::log::SetLoggerError);
    }

    errors {
        BackendCreationFailed(t: &'static str) {
            description("Failed to create a GELF backend")
            display("Failed to create the GELF backend: {}", t)
        }
        IllegalNameForAdditional(t: String) {
            description("The specified name is not a legal name for an additional GELF field")
            display("'{}' is not a legal name for an additional GELF field", t)
        }
        LoggerCreateFailed(t: &'static str) {
            description("Failed to create the GELF logger")
            display("Failed to create the GELF logger: {}", t)
        }
        LogTransmitFailed {
            description("Failed to create a GELF log message")
            display("Failed to create a GELF log message")
        }
        CompressMessageFailed(t: &'static str) {
            description("Failed to compress the message")
            display("Failed to compress the message with'{}'", t)
        }
        SerializeMessageFailed {
            description("Failed to serialize the message to GELF json")
            display("Failed to serialize the message to GELF json")
        }
        ChunkMessageFailed(t: &'static str) {
            description("Failed to chunk the message")
            display("Failed to chunk the message: {}", t)
        }
        IllegalChunkSize(t: u16) {
            description("Illegal chunk size")
            display("Illegal chunk size: {}", t)
        }
    }
}