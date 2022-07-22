use chumsky::{prelude::Simple, text::whitespace, Parser};

const QUESTION_MARK: u8 = '?' as u8;
const SPACE: u8 = ' ' as u8;
const ESPECIALS: &'static [u8] = "{}<>@,@:/[]?=.".as_bytes();
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
    ClearText(u8),
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
            Token::ClearText(_) => 1,
            Token::EncodedWord {
                charset,
                encoding,
                encoded_text,
            } => charset.len() + encoding.len() + encoded_text.len() + AMOUNT_DELIMITERS,
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match self {
            Token::ClearText(token) => vec![*token],
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

    pub fn get_encoded_word(
        ((charset, encoding), encoded_text): ((Vec<u8>, Vec<u8>), Vec<u8>),
    ) -> Self {
        Self::EncodedWord {
            charset,
            encoding,
            encoded_text,
        }
    }
}

pub fn run(encoded_bytes: &[u8]) -> Result<Tokens> {
    let encoded_word = get_parser().parse(encoded_bytes);

    encoded_word.map_err(|err| Error::EncodingIssue(err))
}

fn get_parser() -> impl Parser<u8, Tokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let encoded_words_in_a_row = {
        let following_encoded_word = whitespace().ignore_then(encoded_word_parser().rewind());
        encoded_word_parser().then_ignore(following_encoded_word)
    };

    let single_encoded_word = encoded_word_parser();
    let single_clear_text = clear_text_parser();

    encoded_words_in_a_row
        .or(single_encoded_word)
        .or(single_clear_text)
        .repeated()
}

fn clear_text_parser() -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    any().map(|value: u8| Token::ClearText(value))
}

fn encoded_word_parser() -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let check_encoded_word_length = |token: Token, span| {
        if token.len() > Token::MAX_ENCODED_WORD_LENGTH {
            Err(Simple::custom(
                span,
                Error::EncodedWordTooLong(token.get_bytes()),
            ))
        } else {
            Ok(token)
        }
    };

    let is_especial = |c: u8| ESPECIALS.contains(&c);

    let token = filter(move |&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));
    let charset = token.repeated().at_least(1).collect::<Vec<u8>>();
    let encoding = token.repeated().at_least(1).collect::<Vec<u8>>();
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
            "I use Arch by the way"
                .chars()
                .into_iter()
                .map(|c: char| Token::ClearText(c as u8))
                .collect::<Vec<Token>>()
        );
    }

    // The following examples are from the encoded-form table in section 8:
    // https://datatracker.ietf.org/doc/html/rfc2047#section-8
    #[test]
    fn test_encoded_from_1() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?a?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::EncodedWord {
                charset: "ISO-8859-1".as_bytes().to_vec(),
                encoding: "Q".as_bytes().to_vec(),
                encoded_text: "a".as_bytes().to_vec()
            }]
        );
    }

    // see test_encoded_from_1
    #[test]
    fn test_encoded_from_2() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?a?= b".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                },
                Token::ClearText(b' '),
                Token::ClearText(b'b'),
            ]
        );
    }

    // see test_encoded_from_1
    #[test]
    fn test_encoded_from_3() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                },
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                }
            ]
        );
    }

    /// Test if parser can parse multiple encoded words in a row
    /// See: https://datatracker.ietf.org/doc/html/rfc2047#section-8
    #[test]
    fn test_multiple_encoded_words() {
        let parser = get_parser();
        let message = "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?= =?ISO-8859-1?Q?c?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                },
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                },
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "c".as_bytes().to_vec()
                }
            ]
        );
    }

    #[test]
    fn test_ignore_mutiple_spaces_between_encoded_words() {
        let parser = get_parser();
        let message =
            "=?ISO-8859-1?Q?a?=                               =?ISO-8859-1?Q?b?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                },
                Token::EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                }
            ]
        );
    }

    /// An encoded word with more then 75 chars should be parsed as a normal cleartext
    #[test]
    fn test_too_long_encoded_word() {
        let parser = get_parser();
        // "=?" (2) + "ISO-8859-1" (10) + "?" (1) + "Q" (1) + "?" (1) + 'a' (60) + "?=" (2)
        // = 2 + 10 + 1 + 1 + 60 + 2
        // = 76
        let message =
            "=?ISO-8859-1?Q?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa?="
                .as_bytes();
        let parsed = parser.parse(message).unwrap();

        let expected = {
            let mut expected = String::new();
            expected.push_str("=?ISO-8859-1?Q?");
            expected.push_str(&"a".repeat(60));
            expected.push_str("?=");
            expected
                .chars()
                .into_iter()
                .map(|c: char| Token::ClearText(c as u8))
                .collect::<Vec<Token>>()
        };

        assert_eq!(parsed, expected);
    }
}
