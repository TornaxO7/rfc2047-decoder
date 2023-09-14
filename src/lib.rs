#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.2.1")]

mod decoder;
pub use decoder::{Decoder, RecoverStrategy, Error};

mod evaluator;
mod lexer;
mod parser;

pub use lexer::{TooLongEncodedWords, LexerError};
pub use parser::ParserError;
pub use evaluator::EvaluatorError;

/// Decodes the given RFC 2047 MIME Message Header encoded string
/// using a default decoder.
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> Result<String, Error> {
    Decoder::new().decode(encoded_str)
}
