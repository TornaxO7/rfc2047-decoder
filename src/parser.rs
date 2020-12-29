use std::{error, fmt, result, str};

use crate::lexer::Tokens;

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

#[derive(Debug)]
pub enum Error {
    DecodeUtf8Error(str::Utf8Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeUtf8Error(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::DecodeUtf8Error(ref err) => Some(err),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::DecodeUtf8Error(err)
    }
}

type Result<T> = result::Result<T, Error>;

fn first_char_of(vec: &Vec<u8>) -> Result<char> {
    match str::from_utf8(&vec)?.to_uppercase().chars().next() {
        Some(c) => Ok(c),
        None => Ok('Q'),
    }
}

pub fn run(tokens: &Tokens) -> Result<Ast> {
    let mut curr_charset: &Vec<u8> = &vec![];
    let mut curr_encoding: char = 'Q';
    let mut ast: Ast = vec![];

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
                [13, 10, 32] => (),
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
    fn first_char_of() -> parser::Result<()> {
        assert_eq!('Q', parser::first_char_of(&vec![])?);
        assert_eq!('Q', parser::first_char_of(&"q".as_bytes().to_vec())?);
        assert_eq!('Q', parser::first_char_of(&"Q".as_bytes().to_vec())?);
        assert_eq!('B', parser::first_char_of(&"b".as_bytes().to_vec())?);
        assert_eq!('B', parser::first_char_of(&"B".as_bytes().to_vec())?);
        assert_eq!('B', parser::first_char_of(&"base64".as_bytes().to_vec())?);

        Ok(())
    }
}
