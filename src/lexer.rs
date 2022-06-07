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

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("the charset section is invalid or not terminated")]
    ParseCharsetError,
    #[error("the encoding section is invalid or not terminated")]
    ParseEncodingError,
    #[error("the encoded text section is invalid or not terminated")]
    ParseEncodedTextError,
}

pub fn run(encoded_bytes: &[u8]) -> Result<Tokens, Error> {
    let mut encoded_bytes_iter = encoded_bytes.iter();
    let mut curr_byte = encoded_bytes_iter.next();
    let mut tokens = vec![];
    let mut state = ClearText;
    let mut buffer: Vec<u8> = vec![];

    const EQUAL_SYMBOL: u8 = '=' as u8;
    const QUESTION_MARK_SYMBOL: u8 = '?' as u8;

    loop {
        match state {
            Charset => match curr_byte {
                Some(&QUESTION_MARK_SYMBOL) => {
                    state = Encoding;
                    tokens.push(Token::Charset(buffer.clone()));
                    buffer.clear();
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseCharsetError),
            },
            Encoding => match curr_byte {
                Some(&QUESTION_MARK_SYMBOL) => {
                    state = EncodedText;
                    tokens.push(Token::Encoding(buffer.clone()));
                    buffer.clear();
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseEncodingError),
            },
            EncodedText => match curr_byte {
                Some(&QUESTION_MARK_SYMBOL) => {
                    curr_byte = encoded_bytes_iter.next();

                    match curr_byte {
                        Some(&EQUAL_SYMBOL) => {
                            state = ClearText;
                            tokens.push(Token::EncodedText(buffer.clone()));
                            buffer.clear();
                        }
                        _ => {
                            buffer.push(QUESTION_MARK_SYMBOL);
                            continue;
                        }
                    }
                }
                Some(b) => buffer.push(*b),
                None => return Err(Error::ParseEncodedTextError),
            },
            ClearText => match curr_byte {
                Some(&EQUAL_SYMBOL) => {
                    curr_byte = encoded_bytes_iter.next();

                    match curr_byte {
                        Some(&QUESTION_MARK_SYMBOL) => {
                            state = Charset;

                            if !buffer.is_empty() {
                                tokens.push(Token::ClearText(buffer.clone()));
                                buffer.clear()
                            }
                        }
                        _ => {
                            buffer.push(EQUAL_SYMBOL);
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
