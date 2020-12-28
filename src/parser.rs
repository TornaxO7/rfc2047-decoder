use base64;
use charset;
use quoted_printable;
use std::{error, fmt, result, str};

use crate::lexer::Tokens;

#[derive(Debug)]
pub enum Error {
    Utf8DecodingError(str::Utf8Error),
    Base64DecodingError(base64::DecodeError),
    QuotedPrintableDecodingError(quoted_printable::QuotedPrintableError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Utf8DecodingError(err) => err.fmt(f),
            Error::Base64DecodingError(err) => err.fmt(f),
            Error::QuotedPrintableDecodingError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Utf8DecodingError(ref err) => Some(err),
            Error::Base64DecodingError(ref err) => Some(err),
            Error::QuotedPrintableDecodingError(ref err) => Some(err),
        }
    }
}

impl From<str::Utf8Error> for Error {
    fn from(err: str::Utf8Error) -> Error {
        Error::Utf8DecodingError(err)
    }
}

impl From<base64::DecodeError> for Error {
    fn from(err: base64::DecodeError) -> Error {
        Error::Base64DecodingError(err)
    }
}

impl From<quoted_printable::QuotedPrintableError> for Error {
    fn from(err: quoted_printable::QuotedPrintableError) -> Error {
        Error::QuotedPrintableDecodingError(err)
    }
}

type Result<T> = result::Result<T, Error>;

fn first_char_of(vec: &Vec<u8>) -> Result<char> {
    match str::from_utf8(&vec)?.chars().next() {
        Some(c) => Ok(c),
        None => Ok('Q'),
    }
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
    use charset::Charset;

    let decoded_str = if let Some(charset) = Charset::for_label(charset) {
        charset.decode(decoded_bytes).0
    } else {
        charset::decode_ascii(decoded_bytes)
    };

    Ok(decoded_str.into_owned())
}

pub fn run(tokens: &Tokens) -> Result<String> {
    let mut curr_charset: &Vec<u8> = &vec![];
    let mut curr_encoding: char = 'Q';
    let mut output = String::new();

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
                let decoded_bytes = decode_with_encoding(curr_encoding, encoded_bytes)?;
                let decoded_str = decode_with_charset(curr_charset, &decoded_bytes)?;
                output.push_str(&decoded_str);
            }
            RawText(decoded_bytes) => {
                output.push_str(str::from_utf8(&decoded_bytes).unwrap());
            }
        }
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use crate::lexer::{tests::*, Token};
    use crate::parser::*;

    fn assert_ok(s: &str, tokens: &[Token]) {
        assert_eq!(s.to_string(), run(&tokens.to_vec()).unwrap());
    }

    #[test]
    fn empty_tokens() {
        assert_ok("", &[])
    }

    #[test]
    fn decoded_text_only() {
        assert_ok("decoded-text", &[raw_text("decoded-text")]);
    }

    #[test]
    fn utf_8_q() {
        assert_ok(
            "decoded-text",
            &[
                charset("utf-8"),
                encoding("Q"),
                encoded_text("decoded-text"),
            ],
        );
    }

    #[test]
    fn utf_8_b() {
        assert_ok(
            "decoded-text",
            &[
                charset("utf-8"),
                encoding("B"),
                encoded_text("ZGVjb2RlZC10ZXh0"),
            ],
        );
    }

    #[test]
    fn iso_8858_1_q() {
        assert_ok(
            "decoded = text",
            &[
                charset("iso-8859-1"),
                encoding("q"),
                encoded_text("decoded_=3D_text"),
            ],
        );
    }
}
