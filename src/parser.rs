use crate::lexer::{EncodedWordTokens, Token};

use core::slice::SlicePattern;
use std::convert::TryFrom;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Encoding {
    B,
    Q,
}

impl Encoding {
    pub const B_CHAR: char = 'b';
    pub const Q_CHAR: char = 'q';
}

impl TryFrom<u8> for Encoding {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let value = value as char;

        match value {
            Encoding::Q_CHAR => Ok(Self::Q),
            Encoding::B_CHAR => Ok(Self::B),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EncodedWordParsed {
    pub charset: crate::lexer::Token,
    pub encoding: Encoding,
    pub encoded_text: crate::lexer::EncodedText,
}

#[derive(Debug, Clone)]
pub struct EncodedBytes {
    pub charset: Vec<u8>,
    pub encoding: char,
    pub bytes: Vec<u8>,
}

pub type ClearBytes = Vec<u8>;

#[derive(Debug, Clone)]
pub enum Node {
    EncodedBytes(EncodedBytes),
    ClearBytes(ClearBytes),
}

pub type Ast = Vec<Node>;

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error(transparent)]
    DecodeUtf8Error(#[from] std::str::Utf8Error),
}

fn first_char_of(vec: &[u8]) -> Result<char> {
    match std::str::from_utf8(vec)?.to_uppercase().chars().next() {
        Some(c) => Ok(c),
        None => Ok('Q'),
    }
}

pub fn run(encoded_word: EncodedWordTokens) -> Result<Ast> {
    let mut curr_charset: Token = encoded_word.charset;
    let mut curr_encoding: char = first_char_of(encoded_word.encoding.as_slice());
    let mut ast: Ast = vec![];

    const CR: u8 = '\r' as u8;
    const LF: u8 = '\n' as u8;
    const SPACE: u8 = ' ' as u8;

    for token in tokens {
        use crate::lexer::Token::*;

        match token {
            Charset(charset) => {
                curr_charset = &charset;
            }
            Encoding(encoding) => {
                curr_encoding = first_char_of(&encoding)?;
            }
            EncodedText(encoded_bytes) => {
                ast.push(Node::EncodedBytes(EncodedBytes {
                    charset: curr_charset.clone(),
                    encoding: curr_encoding,
                    bytes: encoded_bytes.clone(),
                }));
            }
            ClearText(decoded_bytes) => match decoded_bytes[..] {
                [CR, LF, SPACE] => (),
                [LF, SPACE] => (),
                [SPACE] => (),
                _ => ast.push(Node::ClearBytes(decoded_bytes.clone())),
            },
        }
    }

    Ok(ast)
}

#[cfg(test)]
mod tests {
    use crate::parser;

    #[test]
    fn first_char_of() {
        assert_eq!('Q', parser::first_char_of(&vec![]).unwrap());
        assert_eq!('Q', parser::first_char_of(&"q".as_bytes()).unwrap());
        assert_eq!('Q', parser::first_char_of(&"Q".as_bytes()).unwrap());
        assert_eq!('B', parser::first_char_of(&"b".as_bytes()).unwrap());
        assert_eq!('B', parser::first_char_of(&"B".as_bytes()).unwrap());
        assert_eq!('B', parser::first_char_of(&"base64".as_bytes()).unwrap());
    }
}
