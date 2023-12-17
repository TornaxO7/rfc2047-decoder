//! # Introduction
//! This crate provides a [Decoder] and the function [decode], in order to decode
//! encoded words as described in the [RFC 2047].
//!
//! [RFC 2047]: https://datatracker.ietf.org/doc/html/rfc2047
//!
//! # Where to start looking
//! You will likely want to start looking into [Decoder] and/or the [decode]
//! to use this crate.

mod decoder;
pub use decoder::{Decoder, Error, RecoverStrategy};

mod evaluator;
mod lexer;
mod parser;

pub use evaluator::Error as EvaluatorError;
pub use lexer::{Error as LexerError, TooLongEncodedWords};
pub use parser::Error as ParserError;

/// Decodes the given RFC 2047 MIME Message Header encoded string
/// using a default decoder.
///
/// This function equals doing `Decoder::new().decode`.
///
/// # Example
/// ```
/// use rfc2047_decoder::{decode, Decoder};
///
/// let encoded_message = "=?ISO-8859-1?Q?hello_there?=".as_bytes();
/// let decoded_message = "hello there";
///
/// // This ...
/// assert_eq!(decode(encoded_message).unwrap(), decoded_message);
///
/// // ... equals this:
/// assert_eq!(Decoder::new().decode(encoded_message).unwrap(), decoded_message);
/// ```
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> Result<String, Error> {
    Decoder::new().decode(encoded_str)
}
