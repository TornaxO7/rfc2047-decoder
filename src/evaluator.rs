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
    const SOFT_LINE_BREAK: u8 = b'=';

    // Appended after the encoded text so that the quoted-printable decoder
    // cannot treat a significant trailing space as the end of the line. It is
    // not a hex digit, so it can never complete a malformed trailing `=X`
    // escape either.
    const SENTINEL: u8 = b'X';

    let mut encoded_bytes = encoded_bytes
        .iter()
        .map(|b| if *b == UNDERSCORE { SPACE } else { *b })
        .collect::<Vec<_>>();

    // `quoted_printable::decode` strips trailing whitespace, which is correct
    // for quoted-printable bodies but not for RFC 2047 Q-encoded words: a
    // space at the end of the encoded word (`_` or `=20`) is significant
    // (RFC 2047 §4.2). Adjacent encoded words are concatenated without the
    // whitespace that separates them, so a stripped space silently joins two
    // words together.
    if encoded_bytes.last() == Some(&SOFT_LINE_BREAK) {
        // A lone trailing `=` is a soft line break, not a significant space.
        // Appending the sentinel would turn it into a malformed escape.
        return Ok(quoted_printable::decode(encoded_bytes, parse_mode)?);
    }

    encoded_bytes.push(SENTINEL);

    let mut decoded_bytes = quoted_printable::decode(encoded_bytes, parse_mode)?;
    decoded_bytes.pop();

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
