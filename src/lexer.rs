use std::fmt;

use charset::Charset;
use chumsky::{prelude::Simple, Parser};

use const_format::formatcp;

const QUESTION_MARK: u8 = '?' as u8;
const EQUAL_SYMBOL: u8 = '=' as u8;
const SPACE: u8 = ' ' as u8;
const ESPECIAL: &'static [u8] = "{}<>@,@:/[]?.=".as_bytes();

pub type Token = Vec<u8>;
pub type EncodedText = Vec<u8>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub struct Error(Vec<Simple<u8>>);

impl Error {
    pub fn new(errors: Vec<Simple<u8>>) -> Self {
        Self(errors)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for error in &self.0 {
            write!(f, "{}", error)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct EncodedWordTokens {
    pub charset: Token,
    pub encoding: Token,
    pub encoded_text: EncodedText,
}

impl EncodedWordTokens {
    pub const PREFIX: &'static [u8] = formatcp!("{}{}", EQUAL_SYMBOL, QUESTION_MARK).as_bytes();
    pub const SUFFIX: &'static [u8] = formatcp!("{}{}", QUESTION_MARK, EQUAL_SYMBOL).as_bytes();
    pub const MAX_CHARSET_LENGTH: usize = 75;
    pub const MIN_CHARSET_LENGTH: usize = 1;

    pub const MIN_ENCODING_LENGTH: usize = 1;

    pub const MIN_ENCODED_TEXT_LENGTH: usize = 1;
}

pub fn run(encoded_bytes: &[u8]) -> Result<EncodedWordTokens> {
    get_parser().parse(encoded_bytes).map_err(Error::new)
}

fn get_parser() -> impl Parser<u8, EncodedWordTokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let token = filter(|&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));

    let charset = token
        .repeated()
        .at_least(EncodedWordTokens::MIN_CHARSET_LENGTH)
        .at_most(EncodedWordTokens::MAX_CHARSET_LENGTH)
        .collect::<Vec<u8>>();
    let encoding = token
        .repeated()
        .at_least(EncodedWordTokens::MIN_ENCODING_LENGTH)
        .collect::<Vec<u8>>();

    let encoded_text = filter(|&c: &u8| c != QUESTION_MARK && c != SPACE)
        .repeated()
        .at_least(EncodedWordTokens::MIN_ENCODED_TEXT_LENGTH)
        .collect::<Vec<u8>>();

    let encoded_word = just(EncodedWordTokens::PREFIX)
        .ignore_then(charset)
        .then_ignore(just(QUESTION_MARK))
        .then(encoding)
        .then_ignore(just(QUESTION_MARK))
        .then(encoded_text)
        .then_ignore(just(EncodedWordTokens::SUFFIX))
        .map(|((charset, encoding), encoded_text)| EncodedWordTokens {
            charset,
            encoding,
            encoded_text,
        });

    encoded_word
}

fn is_especial(c: u8) -> bool {
    ESPECIAL.contains(&c)
}

#[cfg(test)]
mod tests {
    use super::get_parser;
    use chumsky::Parser;

    #[test]
    #[should_panic]
    fn invalid_prefix() {
        let parser = get_parser();
        let message = "?ISO-8859-1?Q?Yeet?=".as_bytes();

        parser.parse(message).unwrap();
    }

    /// missing a question mark
    #[test]
    #[should_panic]
    fn missing_question_mark() {
        let parser = get_parser();
        let message = "=?ISO-8859-1Q?Yeet?=".as_bytes();

        parser.parse(message).unwrap();
    }

    /// missing both question marks in the middle
    #[test]
    #[should_panic]
    fn missing_question_marks() {
        let parser = get_parser();
        let message = "=?ISO-8859-1QYeet?=".as_bytes();

        parser.parse(message).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_suffix() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?Yeet?".as_bytes();

        parser.parse(message).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_token_is_space() {
        let parser = get_parser();
        let message = "=?is space?Q?Yeet?=".as_bytes();

        parser.parse(message).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_token_is_ctl() {
        let parser = get_parser();
        let message = format!("=?{}?Q?Yeet?", 0 as char);
        let message = message.as_bytes();

        parser.parse(message).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_token_is_especial() {
        let parser = get_parser();
        let message = "=?)?Q?Yeet?".as_bytes();

        parser.parse(message).unwrap();
    }

    #[test]
    #[should_panic]
    fn invalid_encoded_text() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q? ?=".as_bytes();

        parser.parse(message).unwrap();
    }
}
