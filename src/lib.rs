#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.1.0")]

use std::{error, fmt, result};

mod lexer;
mod parser;

/// Errors can come from the lexer or the parser.
/// Lexer errors are related to invalid syntaxes.
/// Parser errors are related to decoding issues.
#[derive(Debug)]
pub enum Error {
    RunLexerError(lexer::Error),
    RunParserError(parser::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::RunLexerError(err) => err.fmt(f),
            Error::RunParserError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<lexer::Error> for Error {
    fn from(err: lexer::Error) -> Error {
        Error::RunLexerError(err)
    }
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Error {
        Error::RunParserError(err)
    }
}

/// Wrapper around `std::result::Result`.
pub type Result<T> = result::Result<T, Error>;

/// Decode a RFC 2047 MIME Message Header.
///
/// ```rust
/// use rfc2047_decoder;
///
/// fn main() -> rfc2047_decoder::Result<()> {
///     let encoded_str = "=?UTF-8?Q?encoded_str_with_symbol_=E2=82=AC?=";
///     let decoded_str = "encoded str with symbol €";
///
///     assert_eq!(rfc2047_decoder::decode(encoded_str)?, decoded_str);
///     Ok(())
/// }
/// ```
///
/// # Errors
///
/// The function can return an error if the lexer or
/// the parser encounters an error.
pub fn decode(encoded_str: &str) -> Result<String> {
    let tokens = crate::lexer::run(encoded_str)?;
    let decoded_str = crate::parser::run(&tokens)?;
    Ok(decoded_str)
}

#[cfg(test)]
mod tests {
    use crate::{decode, Result};

    fn assert_decode_ok(decoded_str: &str, encoded_str: &str) -> Result<()> {
        assert_eq!(decode(encoded_str)?, decoded_str);
        Ok(())
    }

    #[test]
    fn utf8_q() -> Result<()> {
        assert_decode_ok(
            "encoded str with symbol €",
            "=?UTF-8?Q?encoded_str_with_symbol_=E2=82=AC?=",
        )
    }

    #[test]
    fn utf8_b() -> Result<()> {
        assert_decode_ok(
            "encoded str with symbol €",
            "=?UTF-8?B?ZW5jb2RlZCBzdHIgd2l0aCBzeW1ib2wg4oKs?=",
        )
    }
}
