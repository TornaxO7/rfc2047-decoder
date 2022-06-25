#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.1.2")]

// mod evaluator;
mod lexer;
mod parser;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Lexer(#[from] lexer::Error),
    #[error(transparent)]
    Parser(#[from] parser::Error),
    // #[error(transparent)]
    // Evaluate(#[from] evaluator::Error),
}

/// Decode a RFC 2047 MIME Message Header.
///
/// ```rust
/// fn main() {
///     match rfc2047_decoder::decode("=?utf8?q?str_with_spaces?=".as_bytes()) {
///         Ok(s) => println!("{}", s),
///         Err(err) => panic!(err),
///     }
/// }
/// ```
///
/// # Errors
///
/// The function can return an error if the lexer,
/// the parser or the evaluator encounters an error.
pub fn decode(encoded_str: &[u8]) -> Result<String> {
    let encoded_word_tokens: lexer::EncodedWordTokens = lexer::run(&encoded_str)?;
    let encoded_word_parsed = parser::run(encoded_word_tokens)?;
    // let decoded_str = evaluator::run(&ats)?;

    // Ok(decoded_str)
    Ok("".to_string())
}

#[cfg(test)]
mod tests {
    use base64::decode;

    // fn assert_ok(decoded_str: &str, encoded_str: &str) {
    //     assert!(if let Ok(s) = decode(encoded_str.as_bytes()) {
    //         s == decoded_str
    //     } else {
    //         false
    //     });
    // }

    // #[test]
    // fn clear_empty() {
    //     assert_ok("", "");
    // }
    //
    // #[test]
    // fn clear_with_str() {
    //     assert_ok("str", "str");
    // }
    //
    // #[test]
    // fn clear_with_spaces() {
    //     assert_ok("str with spaces", "str with spaces");
    // }
    //
    // #[test]
    // fn utf8_qs_empty() {
    //     assert_ok("", "=?UTF-8?Q??=");
    // }
    //
    // #[test]
    // fn utf8_qs_with_str() {
    //     assert_ok("str", "=?UTF-8?Q?str?=");
    // }
    //
    // #[test]
    // fn utf8_qs_with_spaces() {
    //     assert_ok("str with spaces", "=?utf8?q?str_with_spaces?=");
    // }
    //
    // #[test]
    // fn utf8_qs_with_spec_chars() {
    //     assert_ok(
    //         "str with special çhàrß",
    //         "=?utf8?q?str_with_special_=C3=A7h=C3=A0r=C3=9F?=",
    //     );
    // }
    //
    // #[test]
    // fn utf8_qs_double() {
    //     assert_ok("strstr", "=?UTF-8?Q?str?=\r\n =?UTF-8?Q?str?=");
    //     assert_ok("strstr", "=?UTF-8?Q?str?=\n =?UTF-8?Q?str?=");
    //     assert_ok("strstr", "=?UTF-8?Q?str?= =?UTF-8?Q?str?=");
    //     assert_ok("strstr", "=?UTF-8?Q?str?==?UTF-8?Q?str?=");
    // }
    //
    // #[test]
    // fn utf8_b64_empty() {
    //     assert_ok("", "=?UTF-8?B??=");
    // }
    //
    // #[test]
    // fn utf8_b64_with_str() {
    //     assert_ok("str", "=?UTF-8?B?c3Ry?=");
    // }
    //
    // #[test]
    // fn utf8_b64_with_spaces() {
    //     assert_ok("str with spaces", "=?utf8?b?c3RyIHdpdGggc3BhY2Vz?=");
    // }
    //
    // #[test]
    // fn utf8_b64_with_spec_chars() {
    //     assert_ok(
    //         "str with special çhàrß",
    //         "=?utf8?b?c3RyIHdpdGggc3BlY2lhbCDDp2jDoHLDnw==?=",
    //     );
    // }
}
