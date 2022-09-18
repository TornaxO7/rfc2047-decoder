use charset::Charset;

use crate::lexer::{Token, Tokens};

use std::convert::TryFrom;

pub type Result<T> = std::result::Result<T, Error>;
pub type ClearText = Vec<u8>;
pub type ParsedEncodedWords = Vec<ParsedEncodedWord>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding {
    B,
    Q,
}

impl Encoding {
    pub const B_CHAR: char = 'b';
    pub const Q_CHAR: char = 'q';
    pub const ENCODING_LENGTH: usize = 1;
}

impl TryFrom<Vec<u8>> for Encoding {
    type Error = Error;

    fn try_from(token: Vec<u8>) -> Result<Self> {
        if token.len() > Self::ENCODING_LENGTH {
            return Err(Error::EncodedWordTooBig);
        }

        let encoding = token.first().ok_or(Error::EmptyEncoding)?;
        let encoding = *encoding as char;
        let encoding = encoding.to_ascii_lowercase();

        match encoding {
            Encoding::Q_CHAR => Ok(Self::Q),
            Encoding::B_CHAR => Ok(Self::B),
            _ => Err(Error::UnknownEncoding(encoding)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ParsedEncodedWord {
    ClearText(ClearText),
    EncodedWord {
        charset: Option<Charset>,
        encoding: Encoding,
        encoded_text: Vec<u8>,
    },
}

impl ParsedEncodedWord {
    pub fn convert_encoded_word(
        charset: Vec<u8>,
        encoding: Vec<u8>,
        encoded_text: Vec<u8>,
    ) -> Result<Self> {
        let encoding = Encoding::try_from(encoding)?;
        let charset = Charset::for_label(&charset);

        Ok(Self::EncodedWord {
            charset,
            encoding,
            encoded_text,
        })
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("Unknown charset: {}", .0)]
    UnknownCharset(String),

    #[error("Unknown encoding: {}. Encoding can be only either 'Q' or 'B'.", .0)]
    UnknownEncoding(char),

    #[error("The encoded word is too big")]
    EncodedWordTooBig,

    #[error("Encoding is empty")]
    EmptyEncoding,
}

pub fn run(tokens: Tokens) -> Result<ParsedEncodedWords> {
    let parsed_encoded_words = convert_tokens_to_encoded_words(tokens)?;
    Ok(parsed_encoded_words)
}

fn convert_tokens_to_encoded_words(tokens: Tokens) -> Result<ParsedEncodedWords> {
    tokens.into_iter().map(|token: Token| match token {
        Token::ClearText(clear_text) => Ok(ParsedEncodedWord::ClearText(clear_text)),
        Token::EncodedWord {
            charset,
            encoding,
            encoded_text,
        } => ParsedEncodedWord::convert_encoded_word(charset, encoding, encoded_text),
    }).collect()
}

#[cfg(test)]
mod tests {

    use charset::Charset;

    use crate::lexer;
    use crate::parser;
    use crate::parser::{Encoding, ParsedEncodedWord};

    /// Example taken from:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    ///
    /// `From` field
    #[test]
    fn test_parse1() {
        let message = "=?US-ASCII?Q?Keith_Moore?=".as_bytes();
        let tokens = lexer::run(&message).unwrap();
        let parsed = parser::run(tokens).unwrap();

        let expected = vec![ParsedEncodedWord::EncodedWord {
            charset: Charset::for_label("US-ASCII".as_bytes()),
            encoding: Encoding::Q,
            encoded_text: "Keith_Moore".as_bytes().to_vec(),
        }];

        assert_eq!(parsed, expected);
    }

    /// Example taken from:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    ///
    /// `To` field
    #[test]
    fn test_parse2() {
        let message = "=?ISO-8859-1?Q?Keld_J=F8rn_Simonsen?=".as_bytes();
        let tokens = lexer::run(&message).unwrap();
        let parsed = parser::run(tokens).unwrap();

        let expected = vec![ParsedEncodedWord::EncodedWord {
            charset: Charset::for_label("ISO-8859-1".as_bytes()),
            encoding: Encoding::Q,
            encoded_text: "Keld_J=F8rn_Simonsen".as_bytes().to_vec(),
        }];

        assert_eq!(parsed, expected);
    }

    /// Example taken from:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    ///
    /// `CC` field
    #[test]
    fn test_parse3() {
        let message = "=?ISO-8859-1?Q?Andr=E9?=".as_bytes();
        let tokens = lexer::run(&message).unwrap();
        let parsed = parser::run(tokens).unwrap();

        let expected = vec![ParsedEncodedWord::EncodedWord {
            charset: Charset::for_label("ISO-8859-1".as_bytes()),
            encoding: Encoding::Q,
            encoded_text: "Andr=E9".as_bytes().to_vec(),
        }];

        assert_eq!(parsed, expected);
    }

    /// Example taken from:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    ///
    /// `Subject` field
    #[test]
    fn test_parse4() {
        let message = "=?ISO-8859-1?B?SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=?=".as_bytes();
        let tokens = lexer::run(&message).unwrap();
        let parsed = parser::run(tokens).unwrap();

        let expected = vec![ParsedEncodedWord::EncodedWord {
            charset: Charset::for_label("ISO-8859-1".as_bytes()),
            encoding: Encoding::B,
            encoded_text: "SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=".as_bytes().to_vec(),
        }];

        assert_eq!(parsed, expected);
    }
}
