use chumsky::{prelude::Simple, Parser};

use std::fmt;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedWordTokens {
    pub charset: Token,
    pub encoding: Token,
    pub encoded_text: EncodedText,
}

impl EncodedWordTokens {
    pub const PREFIX: &'static [u8] = formatcp!("{}{}", EQUAL_SYMBOL, QUESTION_MARK).as_bytes();
    pub const SUFFIX: &'static [u8] = formatcp!("{}{}", QUESTION_MARK, EQUAL_SYMBOL).as_bytes();
    pub const MAX_CHARSET_LENGTH: u8 = 75;
    pub const MIN_CHARSET_LENGTH: u8 = 1;

    pub const MIN_ENCODING_LENGTH: u8 = 1;

    pub const MIN_ENCODED_TEXT_LENGTH: u8 = 1;
}

pub fn run(encoded_bytes: &[u8]) -> Result<EncodedWordTokens> {
    parser().parse(encoded_bytes)
        .map_err(Error::new)
}

fn parser() -> impl Parser<u8, EncodedWordTokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let token =
        filter(|&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));

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
