#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.2.1")]

mod decoder;
pub use decoder::{Decoder, Error, Result};

mod evaluator;
mod lexer;
mod parser;

/// Determines which strategy should be used if an encoded word isn't encoded as
/// described in the RFC.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RecoverStrategy {
    /// Decode the encoded word although it's incorrectly encoded.
    Decode,
    /// Skip the incorrectly encoded encoded word.
    Skip,
    /// Abort the string-parsing and return an error.
    Abort,
}

/// Decodes the given RFC 2047 MIME Message Header encoded string
/// using a default decoder.
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> Result<String> {
    Decoder::new().decode(encoded_str)
}
