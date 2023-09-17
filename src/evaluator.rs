use base64::{
    alphabet,
    engine::{GeneralPurpose, GeneralPurposeConfig},
    Engine,
};
use charset::Charset;
use std::{result, string};
use thiserror::Error;

use crate::parser::{ClearText, Encoding, ParsedEncodedWord, ParsedEncodedWords};

/// All errors which the evaluator can throw.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    #[error(transparent)]
    DecodeUtf8Error(#[from] string::FromUtf8Error),
    #[error(transparent)]
    DecodeBase64Error(#[from] base64::DecodeError),
    #[error(transparent)]
    DecodeQuotedPrintableError(#[from] quoted_printable::QuotedPrintableError),
}

type Result<T> = result::Result<T, Error>;

fn decode_base64(encoded_bytes: Vec<u8>) -> Result<Vec<u8>> {
    let base64_decoder = {
        let config = GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true);
        GeneralPurpose::new(&alphabet::STANDARD, config)
    };

    let decoded_bytes = base64_decoder.decode(encoded_bytes)?;

    Ok(decoded_bytes)
}

fn decode_quoted_printable(encoded_bytes: Vec<u8>) -> Result<Vec<u8>> {
    let parse_mode = quoted_printable::ParseMode::Robust;

    const SPACE: u8 = b' ';
    const UNDERSCORE: u8 = b'_';

    let encoded_bytes = encoded_bytes
        .iter()
        .map(|b| if *b == UNDERSCORE { SPACE } else { *b })
        .collect::<Vec<_>>();

    let decoded_bytes = quoted_printable::decode(encoded_bytes, parse_mode)?;

    Ok(decoded_bytes)
}

fn decode_with_encoding(encoding: Encoding, encoded_bytes: Vec<u8>) -> Result<Vec<u8>> {
    match encoding {
        Encoding::B => decode_base64(encoded_bytes),
        Encoding::Q => decode_quoted_printable(encoded_bytes),
    }
}

fn decode_with_charset(charset: Option<Charset>, decoded_bytes: Vec<u8>) -> Result<String> {
    let decoded_str = match charset {
        Some(charset) => charset.decode(&decoded_bytes).0,
        None => charset::decode_ascii(&decoded_bytes),
    };

    Ok(decoded_str.into_owned())
}

fn decode_utf8_string(clear_text: ClearText) -> Result<String> {
    let decoded_bytes = String::from_utf8(clear_text)?;
    Ok(decoded_bytes)
}

fn decode_parsed_encoded_word(
    charset: Option<Charset>,
    encoding: Encoding,
    encoded_text: Vec<u8>,
) -> Result<String> {
    let decoded_bytes = decode_with_encoding(encoding, encoded_text)?;
    let decoded_str = decode_with_charset(charset, decoded_bytes)?;
    Ok(decoded_str)
}

pub fn run(parsed_encoded_words: ParsedEncodedWords) -> Result<String> {
    parsed_encoded_words
        .into_iter()
        .map(|parsed_encoded_word| match parsed_encoded_word {
            ParsedEncodedWord::ClearText(clear_text) => decode_utf8_string(clear_text),
            ParsedEncodedWord::EncodedWord {
                charset,
                encoding,
                encoded_text,
            } => decode_parsed_encoded_word(charset, encoding, encoded_text),
        })
        .collect()
}
