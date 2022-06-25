use charset::Charset;

use crate::lexer::{EncodedWordTokens, Token};

use std::convert::TryFrom;

pub type Result<T> = std::result::Result<T, Error>;

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

impl TryFrom<Token> for Encoding {
    type Error = Error;

    fn try_from(token: Token) -> Result<Self> {
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

#[derive(Debug, Clone)]
pub struct EncodedWordParsed {
    pub charset: Charset,
    pub encoding: Encoding,
    pub encoded_text: crate::lexer::EncodedText,
}

impl TryFrom<EncodedWordTokens> for EncodedWordParsed {
    type Error = Error;

    fn try_from(encoded_word_tokens: EncodedWordTokens) -> Result<Self> {
        let charset = Charset::for_label(&encoded_word_tokens.charset)
            .ok_or_else(|| Error::UnknownCharset(format!("{:?}", encoded_word_tokens.charset)))?;
        let encoding = Encoding::try_from(encoded_word_tokens.encoding)?;

        Ok(EncodedWordParsed {
            charset,
            encoding,
            encoded_text: encoded_word_tokens.encoded_text,
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

pub fn run(encoded_word: EncodedWordTokens) -> Result<EncodedWordParsed> {
    EncodedWordParsed::try_from(encoded_word)
}

#[cfg(test)]
mod tests {

    use crate::parser;
    use crate::lexer;
    use crate::parser::Encoding;

    /// Example taken from:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    ///
    /// `From` field
    #[test]
    fn test_parse1() {
        let message = "=?US-ASCII?Q?Keith_Moore?=".as_bytes();
        let tokens = lexer::run(&message).unwrap();
        let parsed = parser::run(tokens).unwrap();

        assert_eq!(parsed.encoding, Encoding::Q);
        assert_eq!(parsed.encoded_text, "Keith_Moore".as_bytes(), "{:#?}", parsed.encoded_text);
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

        assert_eq!(parsed.encoding, Encoding::Q);
        assert_eq!(parsed.encoded_text, "Keld_J=F8rn_Simonsen".as_bytes());
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

        assert_eq!(parsed.encoding, Encoding::Q);
        assert_eq!(parsed.encoded_text, "Andr=E9".as_bytes());
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

        assert_eq!(parsed.encoding, Encoding::B);
        assert_eq!(parsed.encoded_text, "SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=".as_bytes());
    }
}
