use chumsky::{prelude::Simple, Parser};

const QUESTION_MARK: char = '?';
const EQUAL_SYMBOL: char = '=';
const SPACE: char = ' ';
const ESPECIAL: &'static str = "{}<>@,@:/[]?.=";

pub type Token = Vec<u8>;
pub type EncodedText = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Charset(Token),
    Encoding(Token),
    EncodedText(EncodedText),
    EncodedWord {
        charset: Box<Expr>,
        encoding: Box<Expr>,
        encoded_text: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedWord {}

impl EncodedWord {
    pub const PREFIX: String = format!("{}{}", EQUAL_SYMBOL, QUESTION_MARK);
    pub const SUFFIX: String = format!("{}{}", QUESTION_MARK, EQUAL_SYMBOL);
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("the charset section is invalid or not terminated")]
    ParseCharsetError,
    #[error("the encoding section is invalid or not terminated")]
    ParseEncodingError,
    #[error("the encoded text section is invalid or not terminated")]
    ParseEncodedTextError,
}

pub fn run(encoded_bytes: &[u8]) -> Result<EncodedWord> {
    Err(Error::ParseCharsetError)
}

fn parser() -> impl Parser<u8, Expr, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let token =
        filter(|&c: &u8| c != (SPACE as u8) && !c.is_ascii_control() && !is_especial(c as char));

    let charset = token
        .repeated()
        .at_least(1)
        .at_most(75)
        .collect::<Vec<u8>>()
        .map(|charset| Expr::Charset(charset));

    let encoding = token
        .repeated()
        .at_least(1)
        .collect::<Vec<u8>>()
        .map(|encoding| Expr::Encoding(encoding));

    let encoded_text = filter(|&c: &u8| c != (QUESTION_MARK as u8) && c != (SPACE as u8))
        .repeated()
        .at_least(1)
        .collect::<Vec<u8>>()
        .map(|encoded_text| Expr::EncodedText(encoded_text));

    let encoded_word = just(EncodedWord::PREFIX)
        .ignore_then(charset)
        .then_ignore(just(QUESTION_MARK))
        .then(encoding)
        .then_ignore(just(QUESTION_MARK))
        .then(encoded_text)
        .then_ignore(just(EncodedWord::SUFFIX))
        .map(|((charset, encoding), encoded_word)| Expr::EncodedWord {
            charset: Box::new(charset),
            encoding: Box::new(encoding),
            encoded_text: Box::new(encoded_word),
        });

    encoded_word
}

fn is_especial(c: char) -> bool {
    ESPECIAL.contains(c)
}
