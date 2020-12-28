use std::{fmt, result};

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Charset(Vec<u8>),
    Encoding(Vec<u8>),
    EncodedText(Vec<u8>),
    RawText(Vec<u8>),
}

pub type Tokens = Vec<Token>;

enum State {
    Charset,
    Encoding,
    EncodedText,
    RawText,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    CharsetStructureError,
    EncodingStructureError,
    EncodedTextStructureError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::CharsetStructureError => {
                write!(f, "the charset section is invalid or not terminated")
            }
            Error::EncodingStructureError => {
                write!(f, "the encoding section is invalid or not terminated")
            }
            Error::EncodedTextStructureError => {
                write!(f, "the encoded text section is invalid or not terminated")
            }
        }
    }
}

type Result<T> = result::Result<T, Error>;

fn append_char_to_bytes(bytes: &mut Vec<u8>, c: char) {
    let mut buff: [u8; 4] = [0; 4];
    c.encode_utf8(&mut buff);
    let mut char_as_vec = buff[0..c.len_utf8()].to_vec();
    bytes.append(&mut char_as_vec);
}

pub fn run(encoded_str: &str) -> Result<Tokens> {
    use crate::lexer::State::*;

    let mut encoded_chars = encoded_str.chars();
    let mut curr_char = encoded_chars.next();
    let mut tokens = vec![];
    let mut state = State::RawText;
    let mut buffer: Vec<u8> = vec![];

    loop {
        match state {
            Charset => match curr_char {
                Some('?') => {
                    state = Encoding;
                    tokens.push(Token::Charset(buffer.clone()));
                    buffer.clear();
                }
                Some(c) => append_char_to_bytes(&mut buffer, c),
                None => return Err(Error::CharsetStructureError),
            },
            Encoding => match curr_char {
                Some('?') => {
                    state = EncodedText;
                    tokens.push(Token::Encoding(buffer.clone()));
                    buffer.clear();
                }
                Some(c) => append_char_to_bytes(&mut buffer, c),
                None => return Err(Error::EncodingStructureError),
            },
            EncodedText => match curr_char {
                Some('?') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('=') => {
                            state = RawText;
                            tokens.push(Token::EncodedText(buffer.clone()));
                            buffer.clear();
                        }
                        _ => {
                            append_char_to_bytes(&mut buffer, '?');
                            continue;
                        }
                    }
                }
                Some(c) => append_char_to_bytes(&mut buffer, c),
                None => return Err(Error::EncodedTextStructureError),
            },
            RawText => match curr_char {
                Some('=') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('?') => {
                            state = Charset;

                            if !buffer.is_empty() {
                                tokens.push(Token::RawText(buffer.clone()));
                                buffer.clear()
                            }
                        }
                        _ => {
                            append_char_to_bytes(&mut buffer, '=');
                            continue;
                        }
                    }
                }
                Some(c) => append_char_to_bytes(&mut buffer, c),
                None => {
                    if !buffer.is_empty() {
                        tokens.push(Token::RawText(buffer.clone()));
                    }

                    break;
                }
            },
        }

        curr_char = encoded_chars.next();
    }

    Ok(tokens)
}

#[cfg(test)]
pub mod tests {
    use crate::lexer::*;

    // Charset token test utilities
    pub fn charset(s: &str) -> Token {
        Token::Charset(s.as_bytes().to_vec())
    }

    // Encoding token test utilities
    pub fn encoding(s: &str) -> Token {
        Token::Encoding(s.as_bytes().to_vec())
    }

    // Encoded text token test utilities
    pub fn encoded_text(s: &str) -> Token {
        Token::EncodedText(s.as_bytes().to_vec())
    }

    // Raw text token test utilities
    pub fn raw_text(s: &str) -> Token {
        Token::RawText(s.as_bytes().to_vec())
    }

    fn assert_ok(tokens: &[Token], s: &str) {
        assert_eq!(run(s).unwrap(), tokens.to_vec())
    }

    #[test]
    fn empty_str() {
        assert_ok(&[], "")
    }

    #[test]
    fn decoded_text_only() {
        assert_ok(&[raw_text("decoded string")], "decoded string")
    }

    #[test]
    fn decoded_text_except() {
        assert_ok(
            &[
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded-text"),
            ],
            "=?charset?encoding?encoded-text?=",
        )
    }

    #[test]
    fn decoded_text_before() {
        assert_ok(
            &[
                raw_text("decoded-text"),
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded-text"),
            ],
            "decoded-text=?charset?encoding?encoded-text?=",
        )
    }

    #[test]
    fn decoded_text_after() {
        assert_ok(
            &[
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded-text"),
                raw_text("decoded-text"),
            ],
            "=?charset?encoding?encoded-text?=decoded-text",
        )
    }

    #[test]
    fn decoded_text_between() {
        assert_ok(
            &[
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded-text"),
                raw_text("decoded-text"),
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded-text"),
            ],
            "=?charset?encoding?encoded-text?=decoded-text=?charset?encoding?encoded-text?=",
        )
    }

    #[test]
    fn empty_encoded_text() {
        assert_ok(
            &[
                raw_text("decoded-text"),
                charset("charset"),
                encoding("encoding"),
                encoded_text(""),
            ],
            "decoded-text=?charset?encoding??=",
        )
    }

    #[test]
    fn encoded_text_with_question_mark() {
        assert_ok(
            &[
                raw_text("decoded-text"),
                charset("charset"),
                encoding("encoding"),
                encoded_text("encoded?text"),
            ],
            "decoded-text=?charset?encoding?encoded?text?=",
        )
    }

    #[test]
    fn invalid_charset_structure() {
        assert_eq!(Err(Error::CharsetStructureError), run("=?charset"));
    }

    #[test]
    fn invalid_encoding_structure() {
        assert_eq!(
            Err(Error::EncodingStructureError),
            run("=?charset?encoding")
        );
    }

    #[test]
    fn invalid_encoded_text_structure() {
        assert_eq!(
            Err(Error::EncodedTextStructureError),
            run("=?charset?encoding?encoded-text")
        );
    }
}
