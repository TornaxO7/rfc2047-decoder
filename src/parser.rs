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
    ClearText(Vec<u8>),
    EncodedWord {
        charset: Charset,
        encoding: Encoding,
        encoded_text: Vec<u8>,
    },
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
    let parsed_encoded_words = tokens
        .into_iter()
        .try_fold(ParsedEncodedWords::new(), convert_to_parsed_encoded_words)?;

    Ok(parsed_encoded_words)
}

fn convert_to_parsed_encoded_words(
    converted: ParsedEncodedWords,
    next: Token,
) -> Result<ParsedEncodedWords> {
    let previous_field = converted.pop();

    match (previous_field, next) {
        (Some(ParsedEncodedWord::ClearText(chars)), Token::ClearText(new_char)) => {
            append_byte(converted, chars, new_char)
        }
        (None | Some(ParsedEncodedWord::EncodedWord { .. }), Token::ClearText(new_char)) => {
            create_new_byte_vec(converted, new_char)
        }
        (
            _,
            Token::EncodedWord {
                charset,
                encoding,
                encoded_text,
            },
        ) => convert_encoded_word(converted, charset, encoding, encoded_text),
    }
}

fn append_byte(
    converted: ParsedEncodedWords,
    char_buffer: Vec<u8>,
    new_char: u8,
) -> Result<ParsedEncodedWords> {
    char_buffer.push(new_char);
    converted.push(ParsedEncodedWord::ClearText(char_buffer));
    Ok(converted)
}

fn create_new_byte_vec(
    converted: ParsedEncodedWords,
    new_char: u8,
) -> Result<ParsedEncodedWords> {
    let clear_text = ParsedEncodedWord::ClearText(vec![new_char]);
    converted.push(clear_text);
    Ok(converted)
}

fn convert_encoded_word(
    converted: ParsedEncodedWords,
    charset: Vec<u8>,
    encoding: Vec<u8>,
    encoded_text: Vec<u8>,
) -> Result<ParsedEncodedWords> {
    let encoding = Encoding::try_from(encoding)?;
    let charset = Charset::for_label(&charset)
        .ok_or_else(|| Error::UnknownCharset(format!("{:?}", charset)))?;

    let converted_encoded_word = ParsedEncodedWord::EncodedWord {
        charset,
        encoding,
        encoded_text,
    };

    converted.push(converted_encoded_word);
    Ok(converted)
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
            charset: Charset::for_label("US-ASCII".as_bytes()).unwrap(),
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
            charset: Charset::for_label("ISO-8859-1".as_bytes()).unwrap(),
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
            charset: Charset::for_label("ISO-8859-1".as_bytes()).unwrap(),
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
            charset: Charset::for_label("ISO-8859-1".as_bytes()).unwrap(),
            encoding: Encoding::B,
            encoded_text: "SWYgeW91IGNhbiByZWFkIHRoaXMgeW8=".as_bytes().to_vec(),
        }];

        assert_eq!(parsed, expected);
    }
}
