#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.2.2")]

mod decoder;
pub use decoder::{Decoder, Error, Result};

mod evaluator;
mod lexer;
mod parser;

/// Decodes the given RFC 2047 MIME Message Header encoded string
/// using a default decoder.
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> Result<String> {
    Decoder::new().decode(encoded_str)
}
