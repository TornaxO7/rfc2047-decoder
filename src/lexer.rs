use std::{fmt, result};

use crate::lexer::State::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Charset(Vec<u8>),
    Encoding(Vec<u8>),
    EncodedText(Vec<u8>),
    ClearText(Vec<u8>),
}

pub type Tokens = Vec<Token>;

enum State {
    Charset,
    Encoding,
    EncodedText,
    ClearText,
}

#[derive(Debug)]
pub enum Error {
    ParseCharsetError,
    ParseEncodingError,
    ParseEncodedTextError,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseCharsetError => {
                write!(f, "the charset section is invalid or not terminated")
            }
            Error::ParseEncodingError => {
                write!(f, "the encoding section is invalid or not terminated")
            }
            Error::ParseEncodedTextError => {
                write!(f, "the encoded text section is invalid or not terminated")
            }
        }
    }
}

type Result<T> = result::Result<T, Error>;

pub fn run(encoded_bytes: &[u8]) -> Result<Tokens> {
    let mut encoded_bytes_iter = encoded_bytes.iter();
    let mut curr_byte = encoded_bytes_iter.next();
    let mut tokens = vec![];
    let mut state = ClearText;
    let mut buffer: Vec<u8> = vec![];

    // 61 = Equal symbol '='
    // 63 = Question mark symbol '?'

    loop {
        match state {
            Charset => match curr_byte {
                Some(63) => {
                    state = Encoding;
                    tokens.push(Token::Charset(buffer.clone()));
                    buffer.clear();
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseCharsetError),
            },
            Encoding => match curr_byte {
                Some(63) => {
                    state = EncodedText;
                    tokens.push(Token::Encoding(buffer.clone()));
                    buffer.clear();
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseEncodingError),
            },
            EncodedText => match curr_byte {
                Some(63) => {
                    curr_byte = encoded_bytes_iter.next();

                    match curr_byte {
                        Some(61) => {
                            state = ClearText;
                            tokens.push(Token::EncodedText(buffer.clone()));
                            buffer.clear();
                        }
                        _ => {
                            buffer.push(63);
                            continue;
                        }
                    }
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseEncodedTextError),
            },
            ClearText => match curr_byte {
                Some(61) => {
                    curr_byte = encoded_bytes_iter.next();

                    match curr_byte {
                        Some(63) => {
                            state = Charset;

                            if !buffer.is_empty() {
                                tokens.push(Token::ClearText(buffer.clone()));
                                buffer.clear()
                            }
                        }
                        _ => {
                            buffer.push(61);
                            continue;
                        }
                    }
                }
                Some(b) => buffer.push(*b),
                None => {
                    if !buffer.is_empty() {
                        tokens.push(Token::ClearText(buffer.clone()));
                    }

                    break;
                }
            },
        }

        curr_byte = encoded_bytes_iter.next();
    }

    Ok(tokens)
}
