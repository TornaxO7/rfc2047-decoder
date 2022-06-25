use chumsky::{prelude::Simple, Parser};

const QUESTION_MARK: u8 = '?' as u8;
const SPACE: u8 = ' ' as u8;
const ESPECIAL: &'static [u8] = "{}<>@,@:/[]?=.".as_bytes();
const AMOUNT_DELIMITERS: usize = "=????=".len();

pub type Token = Vec<u8>;
pub type EncodedText = Vec<u8>;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("Couldn't get tokens from: {:?}", .0)]
    EncodingIssue(Vec<Simple<u8>>),

    #[error("The encoded word is too long: {:?}", .0)]
    EncodedWordTooLong(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct EncodedWordTokens {
    pub charset: Token,
    pub encoding: Token,
    pub encoded_text: EncodedText,
}

impl EncodedWordTokens {
    pub const PREFIX: &'static [u8] = "=?".as_bytes();
    pub const SUFFIX: &'static [u8] = "?=".as_bytes();
    pub const MAX_ENCODED_WORD_LENGTH: usize = 75;

    /// Returns the length of the encoded word including the delimiters
    pub fn len(&self) -> usize {
        self.charset.len() + self.encoding.len() + self.encoded_text.len() + AMOUNT_DELIMITERS
    }
}

pub fn run(encoded_bytes: &[u8]) -> Result<EncodedWordTokens> {
    let encoded_word = get_parser().parse(encoded_bytes);

    if let Ok(encoded_word) = &encoded_word {
        if encoded_word.len() > EncodedWordTokens::MAX_ENCODED_WORD_LENGTH {
            return Err(Error::EncodedWordTooLong(encoded_bytes.to_vec()));
        }
    }

    encoded_word.map_err(|err| Error::EncodingIssue(err))
}

fn get_parser() -> impl Parser<u8, EncodedWordTokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let token = filter(|&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));

    let charset = token.repeated().collect::<Vec<u8>>();
    let encoding = token.repeated().collect::<Vec<u8>>();

    let encoded_text = filter(|&c: &u8| c != QUESTION_MARK && c != SPACE)
        .repeated()
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
    fn test_valid_encoded_word() {
        
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?Yeet?=".as_bytes();

        parser.parse(message).unwrap();
    }

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
