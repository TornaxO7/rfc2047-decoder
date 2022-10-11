#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.1.3")]

pub mod decoder;
pub use decoder::Decoder;

mod evaluator;
mod lexer;
mod parser;

/// Decodes the given RFC 2047 MIME Message Header encoded string
/// using a default decoder.
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> decoder::Result<String> {
    Decoder::new().decode(encoded_str)
}
