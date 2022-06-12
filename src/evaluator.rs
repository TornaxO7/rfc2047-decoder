use charset::{self, Charset};

// use crate::parser::{Ast, Node::*};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    DecodeUtf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    DecodeBase64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    DecodeQuotedPrintableError(#[from] quoted_printable::QuotedPrintableError),
}

fn decode_utf8(encoded_bytes: &Vec<u8>) -> Result<&str> {
    let decoded_bytes = std::str::from_utf8(&encoded_bytes)?;
    Ok(decoded_bytes)
}

fn decode_base64(encoded_bytes: &Vec<u8>) -> Result<Vec<u8>> {
    let decoded_bytes = base64::decode(&encoded_bytes)?;
    Ok(decoded_bytes)
}

fn decode_quoted_printable(encoded_bytes: &Vec<u8>) -> Result<Vec<u8>> {
    let parse_mode = quoted_printable::ParseMode::Robust;

    const SPACE: u8 = ' ' as u8;
    const UNDERSCORE: u8 = '_' as u8;

    let encoded_bytes = encoded_bytes
        .iter()
        .map(|b| if *b == UNDERSCORE { SPACE } else { *b })
        .collect::<Vec<_>>();

    let decoded_bytes = quoted_printable::decode(encoded_bytes, parse_mode)?;

    Ok(decoded_bytes)
}

pub fn decode_with_encoding(
    encoding: char,
    encoded_bytes: &Vec<u8>,
) -> Result<Vec<u8>> {
    match encoding.to_uppercase().next() {
        Some('B') => decode_base64(encoded_bytes),
        Some('Q') | _ => decode_quoted_printable(encoded_bytes),
    }
}

pub fn decode_with_charset(
    charset: &Vec<u8>,
    decoded_bytes: &Vec<u8>,
) -> Result<String> {
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
