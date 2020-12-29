use base64;
use charset::{self, Charset};
use quoted_printable;
use std::{error, fmt, result, str};

use crate::parser::{Ast, Node::*};

#[derive(Debug)]
pub enum Error {
    DecodeUtf8Error(str::Utf8Error),
    DecodeBase64Error(base64::DecodeError),
    DecodeQuotedPrintableError(quoted_printable::QuotedPrintableError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::DecodeUtf8Error(err) => err.fmt(f),
            Error::DecodeBase64Error(err) => err.fmt(f),
            Error::DecodeQuotedPrintableError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::DecodeUtf8Error(ref err) => Some(err),
            Error::DecodeBase64Error(ref err) => Some(err),
            Error::DecodeQuotedPrintableError(ref err) => Some(err),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::DecodeUtf8Error(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::DecodeBase64Error(err)
    }
}

impl From<quoted_printable::QuotedPrintableError> for Error {
    fn from(err: quoted_printable::QuotedPrintableError) -> Error {
        Error::DecodeQuotedPrintableError(err)
    }
}

type Result<T> = result::Result<T, Error>;

fn decode_utf8(encoded_bytes: &Vec<u8>) -> Result<&str> {
    let decoded_bytes = str::from_utf8(&encoded_bytes)?;
    Ok(decoded_bytes)
}

fn decode_base64(encoded_bytes: &Vec<u8>) -> Result<Vec<u8>> {
    let decoded_bytes = base64::decode(&encoded_bytes)?;
    Ok(decoded_bytes)
}

fn decode_quoted_printable(encoded_bytes: &Vec<u8>) -> Result<Vec<u8>> {
    let parse_mode = quoted_printable::ParseMode::Robust;
    let encoded_bytes = encoded_bytes
        .iter()
        .map(|b| if *b == 95 { 32 } else { *b })
        .collect::<Vec<_>>();
    let decoded_bytes = quoted_printable::decode(encoded_bytes, parse_mode)?;

    Ok(decoded_bytes)
}

pub fn decode_with_encoding(encoding: char, encoded_bytes: &Vec<u8>) -> Result<Vec<u8>> {
    match encoding.to_uppercase().next() {
        Some('B') => decode_base64(encoded_bytes),
        Some('Q') | _ => decode_quoted_printable(encoded_bytes),
    }
}

pub fn decode_with_charset(charset: &Vec<u8>, decoded_bytes: &Vec<u8>) -> Result<String> {
    let decoded_str = match Charset::for_label(charset) {
        Some(charset) => charset.decode(decoded_bytes).0,
        None => charset::decode_ascii(decoded_bytes),
    };

    Ok(decoded_str.into_owned())
}

pub fn run(ast: &Ast) -> Result<String> {
    let mut output = String::new();

    for node in ast {
        match node {
            EncodedBytes(node) => {
                let decoded_bytes = decode_with_encoding(node.encoding, &node.bytes)?;
                let decoded_str = decode_with_charset(&node.charset, &decoded_bytes)?;
                output.push_str(&decoded_str);
            }
            ClearBytes(clear_bytes) => {
                let clear_str = decode_utf8(&clear_bytes)?;
                output.push_str(clear_str);
            }
        }
    }

    Ok(output)
}
