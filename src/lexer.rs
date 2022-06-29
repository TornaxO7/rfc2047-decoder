use chumsky::{prelude::Simple, Parser};

const QUESTION_MARK: u8 = '?' as u8;
const SPACE: u8 = ' ' as u8;
const ESPECIAL: &'static [u8] = "{}<>@,@:/[]?=.".as_bytes();
const AMOUNT_DELIMITERS: usize = "=????=".len();

pub type Result<T> = std::result::Result<T, Error>;
pub type Tokens = Vec<Token>;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Couldn't get tokens from: {:?}", .0)]
    EncodingIssue(Vec<Simple<u8>>),

    #[error("The encoded word is too long: {:?}", .0)]
    EncodedWordTooLong(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Token {
    ClearText(Vec<u8>),
    EncodedWord {
        charset: Vec<u8>,
        encoding: Vec<u8>,
        encoded_text: Vec<u8>,
    },
}

impl Token {
    pub const ENCODED_WORD_PREFIX: &'static [u8] = "=?".as_bytes();
    pub const ENCODED_WORD_SUFFIX: &'static [u8] = "?=".as_bytes();
    pub const MAX_ENCODED_WORD_LENGTH: usize = 75;

    /// Returns the length of the encoded word including the delimiters
    pub fn len(&self) -> usize {
        match self {
            Token::ClearText(token) => token.len(),
            Token::EncodedWord {
                charset,
                encoding,
                encoded_text,
            } => charset.len() + encoding.len() + encoded_text.len() + AMOUNT_DELIMITERS,
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match self {
            Token::ClearText(token) => token.clone(),
            Token::EncodedWord {
                charset,
                encoding,
                encoded_text,
            } => {
                let mut bytes = Vec::new();
                bytes.extend(charset);
                bytes.extend(encoding);
                bytes.extend(encoded_text);
                bytes
            }
        }
    }

    pub fn get_encoded_word(((charset, encoding), encoded_text): ((Vec<u8>, Vec<u8>), Vec<u8>)) -> Self {
        Self::EncodedWord {
            charset,
            encoding,
            encoded_text
        }
    }
}

pub fn run(encoded_bytes: &[u8]) -> Result<Tokens> {
    let encoded_word = get_parser().parse(encoded_bytes);

    encoded_word.map_err(|err| Error::EncodingIssue(err))
}

fn is_especial(c: u8) -> bool {
    ESPECIAL.contains(&c)
}

fn get_parser() -> impl Parser<u8, Tokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let clear_text_parser = get_clear_text_parser();
    let encoded_word_parser = get_encoded_word_parser();

    encoded_word_parser
        .or(clear_text_parser)
        .repeated()
        .collect::<Tokens>()
}

fn get_clear_text_parser() -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    filter(|&c: &u8| {
        c.is_ascii_whitespace() || c.is_ascii_alphanumeric() || c.is_ascii_punctuation()
    })
    .repeated()
    .collect::<Vec<u8>>()
    .map(|clear_text| Token::ClearText(clear_text))
}

fn get_encoded_word_parser() -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let check_encoded_word_length = |token: Token, span| {
            if token.len() > Token::MAX_ENCODED_WORD_LENGTH {
                Err(Simple::custom(span, Error::EncodedWordTooLong(token.get_bytes())))
            } else {
                Ok(token)
            }
    };

    let token = filter(|&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));
    let charset = token.repeated().collect::<Vec<u8>>();
    let encoding = token.repeated().collect::<Vec<u8>>();
    let encoded_text = filter(|&c: &u8| c != QUESTION_MARK && c != SPACE)
        .repeated()
        .collect::<Vec<u8>>();

    just(Token::ENCODED_WORD_PREFIX)
        .ignore_then(charset)
        .then_ignore(just(QUESTION_MARK))
        .then(encoding)
        .then_ignore(just(QUESTION_MARK))
        .then(encoded_text)
        .then_ignore(just(Token::ENCODED_WORD_SUFFIX))
        .map(Token::get_encoded_word)
        .try_map(check_encoded_word_length)
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;

    use super::get_parser;
    use chumsky::Parser;

    #[test]
    fn test_encoded_word() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?Yeet?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::EncodedWord {
                charset: "ISO-8859-1".as_bytes().to_vec(),
                encoding: "Q".as_bytes().to_vec(),
                encoded_text: "Yeet".as_bytes().to_vec(),
            }]
        );
    }

    #[test]
    fn test_clear_text() {
        let parser = get_parser();
        let message = "I use Arch by the way".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::ClearText("I use Arch by the way".as_bytes().to_vec())]
        );
    }
}
