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

fn append_char_to_bytes(bytes: &mut Vec<u8>, c: char) {
    let mut buff: [u8; 4] = [0; 4];
    c.encode_utf8(&mut buff);
    let mut char_as_vec = buff[0..c.len_utf8()].to_vec();
    bytes.append(&mut char_as_vec);
}

pub fn run(encoded_str: &str) -> Result<Tokens> {
    let mut encoded_chars = encoded_str.chars();
    let mut curr_char = encoded_chars.next();
    let mut tokens = vec![];
    let mut state = State::ClearText;
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
                None => return Err(Error::ParseCharsetError),
            },
            Encoding => match curr_char {
                Some('?') => {
                    state = EncodedText;
                    tokens.push(Token::Encoding(buffer.clone()));
                    buffer.clear();
                }
                Some(c) => append_char_to_bytes(&mut buffer, c),
                None => return Err(Error::ParseEncodingError),
            },
            EncodedText => match curr_char {
                Some('?') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('=') => {
                            state = ClearText;
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
                None => return Err(Error::ParseEncodedTextError),
            },
            ClearText => match curr_char {
                Some('=') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('?') => {
                            state = Charset;

                            if !buffer.is_empty() {
                                tokens.push(Token::ClearText(buffer.clone()));
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
                        tokens.push(Token::ClearText(buffer.clone()));
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
    use crate::lexer;

    #[test]
    fn append_char_to_bytes() {
        let mut buff: Vec<u8> = vec![];
        lexer::append_char_to_bytes(&mut buff, 'a');
        assert_eq!(vec![97], buff)
    }
}
