#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.1.3")]

pub mod decoder;
pub use decoder::Decoder;

mod evaluator;
mod lexer;
mod parser;

/// Decodes a RFC 2047 MIME Message Header.
///
/// ```
/// fn main() {
///     match rfc2047_decoder::decode("=?utf8?q?str_with_spaces?=".as_bytes()) {
///         Ok(s) => println!("{}", s),
///         Err(err) => panic!(err),
///     }
/// }
/// ```
///
/// The function can return an error if the lexer, the parser or the
/// evaluator encounters an error.
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> decoder::Result<String> {
    Decoder::new().decode(encoded_str)
}
