use chumsky::{prelude::Simple, text::whitespace, Parser};
use std::{collections::HashSet, result, fmt::Display};
use thiserror::Error;

use crate::{decoder::RecoverStrategy, Decoder};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TooLongEncodedWords(Vec<String>);

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

#[derive(Error, Debug, Clone, PartialEq)]
pub enum Error {
    #[error("cannot parse bytes into tokens")]
    ParseBytesError(Vec<Simple<u8>>),
    #[error("Cannot parse the following encoded words, because they are too long: {0}")]
    ParseEncodedWordTooLongError(TooLongEncodedWords),
}

type Result<T> = result::Result<T, Error>;

const QUESTION_MARK: u8 = b'?';
const SPACE: u8 = b' ';
const AMOUNT_DELIMITERS: usize = "=????=".len();

pub type Tokens = Vec<Token>;

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
            Self::ClearText(_) => 1,
            Self::EncodedWord {
                charset,
                encoding,
                encoded_text,
            } => charset.len() + encoding.len() + encoded_text.len() + AMOUNT_DELIMITERS,
        }
    }

    pub fn is_encoded_word(&self) -> bool {
        match self {
            Self::EncodedWord {..} => true,
            _ => false,
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        match self {
            Token::ClearText(token) => (*token).clone(),
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

pub fn run(encoded_bytes: &[u8], decoder: Decoder) -> Result<Tokens> {
    let tokens = get_parser(&decoder)
        .parse(encoded_bytes)
        .map_err(|err| Error::ParseBytesError(err))?;

    validate_tokens(tokens, &decoder)
}

fn get_parser(decoder: &Decoder) -> impl Parser<u8, Tokens, Error = Simple<u8>> {
    use chumsky::prelude::*;

    let encoded_words_in_a_row = {
        let following_encoded_word =
            whitespace().ignore_then(encoded_word_parser(&decoder).rewind());
        encoded_word_parser(&decoder).then_ignore(following_encoded_word)
    };

    let single_encoded_word = encoded_word_parser(&decoder);
    let single_clear_text = clear_text_parser(&decoder);

    encoded_words_in_a_row
        .or(single_encoded_word)
        .or(single_clear_text)
        .repeated()
}

fn clear_text_parser(decoder: &Decoder) -> impl Parser<u8, Token, Error = Simple<u8>> {
    use chumsky::prelude::*;

    const DEFAULT_EMPTY_INPUT_ERROR_MESSAGE: &str = "got empty input";

    take_until(encoded_word_parser(&decoder).rewind().ignored().or(end())).try_map(
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

    let check_encoded_word_length = move |token: Token, _span| {
        if token.len() > Token::MAX_ENCODED_WORD_LENGTH {
            match skip_encoded_word_length {
                RecoverStrategy::Skip => Ok(Token::ClearText(token.get_bytes())),
                // Hint: The lexer 
                RecoverStrategy::Decode | RecoverStrategy::Abort => Ok(token),
            }
        } else {
            Ok(token)
        }
    };

    let is_especial = |c: u8| get_especials().contains(&c);

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

    for token in tokens.into_iter() {
        if token.is_encoded_word() && token.len() > Token::MAX_ENCODED_WORD_LENGTH && strategy == RecoverStrategy::Abort {
            let encoded_word = String::from_utf8(token.get_bytes()).unwrap();
            too_long_encoded_words.push(encoded_word);
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
    use crate::{lexer::Token, Decoder};

    use super::get_parser;
    use chumsky::Parser;

    #[test]
    fn test_encoded_word() {
        let parser = get_parser(&Decoder::new());
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
    fn test_encoded_from_1() {
        let parser = get_parser(&Decoder::new());
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
        let parser = get_parser(&Decoder::new());
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
                Token::ClearText(" b".as_bytes().to_vec()),
            ]
        );
    }

    // see test_encoded_from_1
    #[test]
    fn test_encoded_from_3() {
        let parser = get_parser(&Decoder::new());
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
        let parser = get_parser(&Decoder::new());
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
        let parser = get_parser(&Decoder::new());
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
        let parser = get_parser(&Decoder::new());
        // "=?" (2) + "ISO-8859-1" (10) + "?" (1) + "Q" (1) + "?" (1) + 'a' (60) + "?=" (2)
        // = 2 + 10 + 1 + 1 + 1 + 60 + 2
        // = 77 => too long
        let message =
            "=?ISO-8859-1?Q?aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa?="
                .as_bytes();
        let parsed = parser.parse(message).unwrap();

        assert_eq!(parsed, vec![Token::ClearText(message.to_vec())]);
    }

    #[test]
    fn test_encoded_word_has_especials() {
        let parser = get_parser(&Decoder::new());
        let message = "=?ISO-8859-1(?Q?a?=".as_bytes();
        let parsed = parser.parse(message).unwrap();

        assert_eq!(parsed, vec![Token::ClearText(message.to_vec())]);
    }
}
