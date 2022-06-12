use chumsky::{Parser, prelude::Simple};

const QUESTION_MARK: char = '?';
const EQUAL_SYMBOL: char = '=';
const SPACE: char = ' ';
const ESPECIAL: &'static str = "{}<>@,@:/[]?.=";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Token(String),
    EncodedText(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedWord {
}

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

fn parser() -> impl Parser<char, Expr, Error = Simple<char>> {
    use chumsky::prelude::*;

    let token = filter(|c: &char| *c != SPACE
                       && !c.is_ascii_control()
                       && !is_especial(*c))
        .repeated().at_least(1)
        .collect::<String>()
        .map(|token| Expr::Token(token.to_string()));

    let encoded_text = filter(|c: &char| *c != QUESTION_MARK && *c != SPACE)
        .repeated().at_least(1)
        .collect::<String>()
        .map(|encoded_text| Expr::EncodedText(encoded_text.to_string()));

    let encoded_word = just(EncodedWord::PREFIX)
        .then(token)
        .then(just(QUESTION_MARK))
        .then(token)
        .then(just(QUESTION_MARK))
        .then(encoded_text)
        .then(just(EncodedWord::SUFFIX));

    encoded_word
}

fn is_especial(c: char) -> bool {
    ESPECIAL.contains(c)
}
