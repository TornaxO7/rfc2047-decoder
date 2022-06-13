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
        let mut error_msg = String::new();

        (&self.0)
            .into_iter()
            .for_each(|error| error_msg.push_str(&format!("{}", error)));

        write!(f, "{}", error_msg)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedWord {
    pub charset: Token,
    pub encoding: Token,
    pub encoded_text: EncodedText,
}

impl EncodedWord {
    pub const PREFIX: &'static [u8] = formatcp!("{}{}", EQUAL_SYMBOL, QUESTION_MARK).as_bytes();
    pub const SUFFIX: &'static [u8] = formatcp!("{}{}", QUESTION_MARK, EQUAL_SYMBOL).as_bytes();
}

pub fn run(encoded_bytes: &[u8]) -> Result<EncodedWord> {
    parser().parse(encoded_bytes)
        .map_err(Error::new)
}

fn parser() -> impl Parser<u8, EncodedWord, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let token =
        filter(|&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));

    let charset = token
        .repeated()
        .at_least(1)
        .at_most(75)
        .collect::<Vec<u8>>();

    let encoding = token
        .repeated()
        .at_least(1)
        .collect::<Vec<u8>>();

    let encoded_text = filter(|&c: &u8| c != QUESTION_MARK && c != SPACE)
        .repeated()
        .at_least(1)
        .collect::<Vec<u8>>();

    let encoded_word = just(EncodedWord::PREFIX)
        .ignore_then(charset)
        .then_ignore(just(QUESTION_MARK))
        .then(encoding)
        .then_ignore(just(QUESTION_MARK))
        .then(encoded_text)
        .then_ignore(just(EncodedWord::SUFFIX))
        .map(|((charset, encoding), encoded_text)| EncodedWord {
            charset,
            encoding,
            encoded_text,
        });

    encoded_word
}

fn is_especial(c: u8) -> bool {
    ESPECIAL.contains(&c)
}
