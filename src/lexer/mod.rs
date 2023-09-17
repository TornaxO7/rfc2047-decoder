pub mod encoded_word;

use chumsky::{prelude::Simple, text::whitespace, Parser};
use std::{collections::HashSet, fmt::Display, result};
use thiserror::Error;

use crate::{decoder::RecoverStrategy, Decoder};

use self::encoded_word::EncodedWord;

pub const QUESTION_MARK: u8 = b'?';
const SPACE: u8 = b' ';

/// A helper struct which implements [std::fmt::Display] for `Vec<String>` and
/// which contains the encoded words which are too long as a `String`.
///
/// # Example
/// ```
/// use rfc2047_decoder::{self, decode, RecoverStrategy, LexerError};
///
/// // the first string and the third string are more than 75 characters, hence
/// // they are actually invalid encoded words
/// let message = concat![
///     "=?utf-8?B?bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb==?=",
///     "among us",
///     "=?utf-8?B?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa==?=",
/// ];

/// let result = decode(message).unwrap_err();
/// if let rfc2047_decoder::Error::Lexer(LexerError::ParseEncodedWordTooLongError(invalid_encoded_words)) = result {
///     assert_eq!(invalid_encoded_words.0[0], "=?utf-8?B?bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb==?=");
///     assert_eq!(invalid_encoded_words.0[1], "=?utf-8?B?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa==?=");
/// } else {
///     assert!(false);
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TooLongEncodedWords(pub Vec<String>);

impl TooLongEncodedWords {
    pub fn new(encoded_words: Vec<String>) -> Self {
        Self(encoded_words)
    }
}

impl Display for TooLongEncodedWords {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut message = String::new();

        if !self.0.is_empty() {
            message = self.0[0].clone();

            for encoded_word in self.0.iter().skip(1) {
                message.push_str(&format!(", {}", encoded_word));
            }
        }

        f.write_str(&message)
    }
}

/// All errors which the lexer can throw.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("cannot parse bytes into tokens")]
    ParseBytesError(Vec<Simple<u8>>),
    #[error("Cannot parse the following encoded words, because they are too long: {0}")]
    ParseEncodedWordTooLongError(TooLongEncodedWords),
}

type Result<T> = result::Result<T, Error>;

pub type Tokens = Vec<Token>;

#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub enum Token {
    ClearText(Vec<u8>),
    EncodedWord(EncodedWord),
}

impl Token {
    /// Returns the amount of bytes which the token holds
    pub fn len(&self) -> usize {
        match self {
            Self::ClearText(clear_text) => clear_text.len(),
            Self::EncodedWord(encoded_word) => encoded_word.len(),
        }
    }
}

pub fn run(encoded_bytes: &[u8], decoder: Decoder) -> Result<Tokens> {
    let tokens = get_parser(&decoder)
        .parse(encoded_bytes)
        .map_err(Error::ParseBytesError)?;

    validate_tokens(tokens, &decoder)
}

fn get_parser(decoder: &Decoder) -> impl Parser<u8, Tokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let encoded_words_in_a_row = {
        let following_encoded_word =
            whitespace().ignore_then(encoded_word_parser(decoder).rewind());
        encoded_word_parser(decoder).then_ignore(following_encoded_word)
    };

    let single_encoded_word = encoded_word_parser(decoder);
    let single_clear_text = clear_text_parser(decoder);

    encoded_words_in_a_row
        .or(single_encoded_word)
        .or(single_clear_text)
        .repeated()
}

fn clear_text_parser(decoder: &Decoder) -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    const DEFAULT_EMPTY_INPUT_ERROR_MESSAGE: &str = "got empty input";

    take_until(encoded_word_parser(decoder).rewind().ignored().or(end())).try_map(
        |(chars, ()), span| {
            if chars.is_empty() {
                Err(Simple::custom(span, DEFAULT_EMPTY_INPUT_ERROR_MESSAGE))
            } else {
                Ok(Token::ClearText(chars))
            }
        },
    )
}

fn encoded_word_parser(decoder: &Decoder) -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let skip_encoded_word_length = decoder.too_long_encoded_word;

    let convert_to_token = move |encoded_word: EncodedWord| {
        if encoded_word.len() > encoded_word::MAX_LENGTH
            && skip_encoded_word_length == RecoverStrategy::Skip
        {
            Token::ClearText(encoded_word.get_bytes(true))
        } else {
            Token::EncodedWord(encoded_word)
        }
    };

    let is_especial = |c: u8| get_especials().contains(&c);

    let token = filter(move |&c: &u8| c != SPACE && !c.is_ascii_control() && !is_especial(c));
    let charset = token.repeated().at_least(1).collect::<Vec<u8>>();
    let encoding = token.repeated().at_least(1).collect::<Vec<u8>>();
    let encoded_text = filter(|&c: &u8| c != QUESTION_MARK && c != SPACE)
        .repeated()
        .collect::<Vec<u8>>();

    just(encoded_word::PREFIX)
        .ignore_then(charset)
        .then_ignore(just(QUESTION_MARK))
        .then(encoding)
        .then_ignore(just(QUESTION_MARK))
        .then(encoded_text)
        .then_ignore(just(encoded_word::SUFFIX))
        .map(EncodedWord::from_parser)
        .map(convert_to_token)
}

fn get_especials() -> HashSet<u8> {
    "()<>@,;:/[]?.=".bytes().collect()
}

fn validate_tokens(tokens: Tokens, decoder: &Decoder) -> Result<Tokens> {
    if let Some(too_long_encoded_words) = get_too_long_encoded_words(&tokens, decoder) {
        return Err(Error::ParseEncodedWordTooLongError(too_long_encoded_words));
    }

    Ok(tokens)
}

fn get_too_long_encoded_words(tokens: &Tokens, decoder: &Decoder) -> Option<TooLongEncodedWords> {
    let strategy = decoder.too_long_encoded_word;
    let mut too_long_encoded_words: Vec<String> = Vec::new();

    for token in tokens.iter() {
        if let Token::EncodedWord(encoded_word) = token {
            if token.len() > encoded_word::MAX_LENGTH && strategy == RecoverStrategy::Abort {
                too_long_encoded_words.push(encoded_word.to_string());
            }
        }
    }

    if too_long_encoded_words.is_empty() {
        None
    } else {
        Some(TooLongEncodedWords::new(too_long_encoded_words))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        lexer::{encoded_word::EncodedWord, run, Token},
        Decoder,
    };

    use super::{get_parser, Error, TooLongEncodedWords};
    use chumsky::Parser;

    #[test]
    fn encoded_word() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1?Q?Yeet?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::EncodedWord(EncodedWord {
                charset: "ISO-8859-1".as_bytes().to_vec(),
                encoding: "Q".as_bytes().to_vec(),
                encoded_text: "Yeet".as_bytes().to_vec(),
            })]
        );
    }

    #[test]
    fn clear_text() {
        let parser = get_parser(&Decoder::new());
        let message = "I use Arch by the way".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::ClearText(
                "I use Arch by the way".as_bytes().to_vec()
            )]
        );
    }

    // The following examples are from the encoded-form table in section 8:
    // https://datatracker.ietf.org/doc/html/rfc2047#section-8
    #[test]
    fn encoded_from_1() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1?Q?a?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![Token::EncodedWord(EncodedWord {
                charset: "ISO-8859-1".as_bytes().to_vec(),
                encoding: "Q".as_bytes().to_vec(),
                encoded_text: "a".as_bytes().to_vec()
            })]
        );
    }

    // see encoded_from_1
    #[test]
    fn encoded_from_2() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1?Q?a?= b".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                }),
                Token::ClearText(" b".as_bytes().to_vec()),
            ]
        );
    }

    // see encoded_from_1
    #[test]
    fn encoded_from_3() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                }),
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                })
            ]
        );
    }

    /// Test if parser can parse multiple encoded words in a row
    /// See: https://datatracker.ietf.org/doc/html/rfc2047#section-8
    #[test]
    fn multiple_encoded_words() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?= =?ISO-8859-1?Q?c?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                }),
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                }),
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "c".as_bytes().to_vec()
                })
            ]
        );
    }

    #[test]
    fn ignore_mutiple_spaces_between_encoded_words() {
        let parser = get_parser(&Decoder::new());
        let message =
            "=?ISO-8859-1?Q?a?=                               =?ISO-8859-1?Q?b?=".as_bytes();

        let parsed = parser.parse(message).unwrap();

        assert_eq!(
            parsed,
            vec![
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "a".as_bytes().to_vec(),
                }),
                Token::EncodedWord(EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "b".as_bytes().to_vec()
                })
            ]
        );
    }

    /// An encoded word with more then 75 chars should panic
    #[test]
    fn err_on_too_long_encoded_word() {
        // "=?" (2) + "ISO-8859-1" (10) + "?" (1) + "Q" (1) + "?" (1) + 'a' (60) + "?=" (2)
        // = 2 + 10 + 1 + 1 + 1 + 60 + 2
        // = 77 => too long
        let message =
            "=?ISO-8859-1?Q?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa?="
                .as_bytes();
        let parsed = run(message, Decoder::new());

        assert_eq!(
            parsed,
            Err(Error::ParseEncodedWordTooLongError(
                TooLongEncodedWords::new(vec![EncodedWord {
                    charset: "ISO-8859-1".as_bytes().to_vec(),
                    encoding: "Q".as_bytes().to_vec(),
                    encoded_text: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
                        .as_bytes()
                        .to_vec()
                }
                .to_string()])
            ))
        );
    }

    #[test]
    fn encoded_word_has_especials() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1(?Q?a?=".as_bytes();
        let parsed = parser.parse(message).unwrap();

        assert_eq!(parsed, vec![Token::ClearText(message.to_vec())]);
    }
}
