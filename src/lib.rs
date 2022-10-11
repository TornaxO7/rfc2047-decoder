#![doc(html_root_url = "https://docs.rs/rfc2047-decoder/0.1.3")]

mod evaluator;
mod lexer;
mod parser;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Lexer(#[from] lexer::Error),
    #[error(transparent)]
    Parser(#[from] parser::Error),
    #[error(transparent)]
    Evaluate(#[from] evaluator::Error),
}

/// Decode a RFC 2047 MIME Message Header.
///
/// ```
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
pub fn decode<T: AsRef<[u8]>>(encoded_str: T) -> Result<String> {
    let text_tokens: lexer::Tokens = lexer::run(encoded_str.as_ref())?;
    let parsed_text: parser::ParsedEncodedWords = parser::run(text_tokens)?;
    let evaluated_string: String = evaluator::run(parsed_text)?;

    Ok(evaluated_string)
}

#[cfg(test)]
mod tests {

    /// Here are the main-tests which are listed here:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    mod rfc_tests {
        use crate::decode;

        #[test]
        fn test_example_1() {
            assert_eq!(decode("=?ISO-8859-1?Q?a?=").unwrap(), "a");
        }

        #[test]
        fn test_example_2() {
            assert_eq!(decode("=?ISO-8859-1?Q?a?= b").unwrap(), "a b");
        }

        #[test]
        fn test_example_3() {
            assert_eq!(
                decode("=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=").unwrap(),
                "ab"
            );
        }

        #[test]
        fn test_example_4() {
            assert_eq!(
                decode("=?ISO-8859-1?Q?a?=  =?ISO-8859-1?Q?b?=").unwrap(),
                "ab"
            );
        }

        #[test]
        fn test_example_5() {
            assert_eq!(
                decode(
                    "=?ISO-8859-1?Q?a?=               
                          =?ISO-8859-1?Q?b?="
                )
                .unwrap(),
                "ab"
            );
        }

        #[test]
        fn test_example_6() {
            assert_eq!(decode("=?ISO-8859-1?Q?a_b?=").unwrap(), "a b");
        }

        #[test]
        fn test_example_7() {
            assert_eq!(
                decode("=?ISO-8859-1?Q?a?= =?ISO-8859-2?Q?_b?=").unwrap(),
                "a b"
            );
        }
    }

    /// Those are some custom tests
    mod custom_tests {
        use crate::decode;

        #[test]
        fn clear_empty() {
            assert_eq!(decode("").unwrap(), "");
        }

        #[test]
        fn clear_with_spaces() {
            assert_eq!(decode("str with spaces").unwrap(), "str with spaces");
        }

        #[test]
        fn utf8_qs_empty() {
            assert_eq!(decode("").unwrap(), "");
        }

        #[test]
        fn utf8_qs_with_str() {
            assert_eq!(decode("=?UTF-8?Q?str?=").unwrap(), "str");
        }

        #[test]
        fn utf8_qs_with_spaces() {
            assert_eq!(
                decode("=?utf8?q?str_with_spaces?=").unwrap(),
                "str with spaces"
            );
        }

        #[test]
        fn utf8_qs_with_spec_chars() {
            assert_eq!(
                decode("=?utf8?q?str_with_special_=C3=A7h=C3=A0r=C3=9F?=").unwrap(),
                "str with special çhàrß"
            );
        }

        #[test]
        fn utf8_qs_double() {
            assert_eq!(
                decode("=?UTF-8?Q?str?=\r\n =?UTF-8?Q?str?=").unwrap(),
                "strstr"
            );
            assert_eq!(
                decode("=?UTF-8?Q?str?=\n =?UTF-8?Q?str?=").unwrap(),
                "strstr"
            );
            assert_eq!(decode("=?UTF-8?Q?str?= =?UTF-8?Q?str?=").unwrap(), "strstr");
            assert_eq!(decode("=?UTF-8?Q?str?==?UTF-8?Q?str?=").unwrap(), "strstr");
        }

        #[test]
        fn utf8_b64_empty() {
            assert_eq!(decode("=?UTF-8?B??=").unwrap(), "");
        }

        #[test]
        fn utf8_b64_with_str() {
            assert_eq!(decode("=?UTF-8?B?c3Ry?=").unwrap(), "str");
        }

        #[test]
        fn utf8_b64_with_spaces() {
            assert_eq!(
                decode("=?utf8?b?c3RyIHdpdGggc3BhY2Vz?=").unwrap(),
                "str with spaces"
            );
        }

        #[test]
        fn utf8_b64_with_spec_chars() {
            assert_eq!(
                decode("=?utf8?b?c3RyIHdpdGggc3BlY2lhbCDDp2jDoHLDnw==?=").unwrap(),
                "str with special çhàrß"
            );
        }

        #[test]
        fn utf8_b64_trailing_bit() {
            assert_ok(
                "Portale HackingTeam",
                "=?utf-8?B?UG9ydGFsZSBIYWNraW5nVGVhbW==?=",
            );
        }
    }
}
