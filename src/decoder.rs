use std::result;
use thiserror::Error;

use crate::{evaluator, lexer, parser};

/// The possible errors which can occur while parsing the string.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Symbolises that an error occured in the lexer.
    #[error(transparent)]
    Lexer(#[from] lexer::Error),

    /// Symbolises that an error occured in the parser.
    #[error(transparent)]
    Parser(#[from] parser::Error),

    /// Symbolises that an error occured in the evaluator.
    #[error(transparent)]
    Evaluator(#[from] evaluator::Error),
}

/// Determines which strategy should be used if an encoded word isn't encoded as
/// described in the RFC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoverStrategy {
    /// Decode the encoded word although it's incorrectly encoded.
    ///
    /// # Example
    /// Take a look to [Decoder#RecoveryStrategy::Decode](Decoder#recoverstrategydecode).
    Decode,

    /// Skip the incorrectly encoded encoded word.
    ///
    /// # Example
    /// Take a look to [Decoder#RecoveryStrategy::Skip](Decoder#recoverstrategyskip).
    Skip,

    /// Abort the string-parsing and return an error.
    ///
    /// # Example
    /// Take a look to [Decoder#RecoveryStrategy::Abort](Decoder#recoverstrategyabort-default).
    Abort,
}

type Result<T> = result::Result<T, Error>;

/// Represents the decoder builder.
///
/// # Example
/// ```
/// use rfc2047_decoder::{Decoder, RecoverStrategy};
///
/// let decoder = Decoder::new()
///                 .too_long_encoded_word_strategy(RecoverStrategy::Skip);
/// let decoded_str = decoder.decode("=?UTF-8?B?c3Ry?=").unwrap();
///
/// assert_eq!(decoded_str, "str");
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Decoder {
    /// Determines which strategy should be used, if the parser encounters
    /// encoded words which are longer than allowed in the RFC (it's longer than 75 chars).
    pub too_long_encoded_word: RecoverStrategy,
}

impl Decoder {
    /// Equals [Decoder::default].
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the strategy if the decoder finds an encoded word which is too long.
    ///
    /// # Examples
    ///
    /// Each example uses the same encoded message:
    /// ```txt
    /// =?utf-8?B?TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gVXQgaW50ZXJkdW0gcXVhbSBldSBmYWNpbGlzaXMgb3JuYXJlLg==?=
    /// ```
    /// which exceeds the maximum length of 75 chars so it's actually invalid.
    ///
    /// ## RecoverStrategy::Skip
    /// Skips the invalid encoded word and parses it as clear text.
    ///
    /// ```rust
    /// use rfc2047_decoder::{Decoder, RecoverStrategy};
    ///
    /// let message = "=?utf-8?B?TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gVXQgaW50ZXJkdW0gcXVhbSBldSBmYWNpbGlzaXMgb3JuYXJlLg==?=";
    /// let decoder = Decoder::new()
    ///                 .too_long_encoded_word_strategy(RecoverStrategy::Skip);
    ///
    /// let parsed = decoder.decode(message).unwrap();
    ///
    /// // nothing changed!
    /// assert_eq!(parsed, message);
    /// ```
    ///
    /// ## RecoverStrategy::Decode
    /// Although the encoded word is invalid, keep decoding it.
    ///
    /// ```rust
    /// use rfc2047_decoder::{Decoder, RecoverStrategy};
    ///
    /// let message = "=?utf-8?B?TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gVXQgaW50ZXJkdW0gcXVhbSBldSBmYWNpbGlzaXMgb3JuYXJlLg==?=";
    /// let decoder = Decoder::new()
    ///                 .too_long_encoded_word_strategy(RecoverStrategy::Decode);
    ///
    /// let parsed = decoder.decode(message).unwrap();
    ///
    /// // could you decode it? ;)
    /// let expected_result = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Ut interdum quam eu facilisis ornare.";
    ///
    /// assert_eq!(parsed, expected_result);
    /// ```
    ///
    /// ## RecoverStrategy::Abort (default)
    /// The parser will return an `Err` and collects all encoded words which are
    /// too long. You can use them afterwards for error messages for example.
    ///
    /// ```rust
    /// use rfc2047_decoder::{Decoder, RecoverStrategy, Error::{self, Lexer}};
    /// use rfc2047_decoder::LexerError::ParseEncodedWordTooLongError;
    /// use rfc2047_decoder::TooLongEncodedWords;
    ///
    /// let message = "=?utf-8?B?TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gVXQgaW50ZXJkdW0gcXVhbSBldSBmYWNpbGlzaXMgb3JuYXJlLg==?=";
    /// // `RecoverStrategy::Abort` is the default strategy
    /// let decoder = Decoder::new();
    ///
    /// let parsed = decoder.decode(message);
    ///
    /// assert_eq!(parsed, Err(Lexer(ParseEncodedWordTooLongError(TooLongEncodedWords(vec!["=?utf-8?B?TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdC4gVXQgaW50ZXJkdW0gcXVhbSBldSBmYWNpbGlzaXMgb3JuYXJlLg==?=".to_string()])))));
    /// ```
    pub fn too_long_encoded_word_strategy(mut self, strategy: RecoverStrategy) -> Self {
        self.too_long_encoded_word = strategy;
        self
    }

    /// Decodes the given RFC 2047 MIME Message Header encoded string.
    pub fn decode<T: AsRef<[u8]>>(self, encoded_str: T) -> Result<String> {
        let text_tokens = lexer::run(encoded_str.as_ref(), self)?;
        let parsed_text = parser::run(text_tokens)?;
        let evaluated_string = evaluator::run(parsed_text)?;

        Ok(evaluated_string)
    }
}

impl Default for Decoder {
    /// Returns the decoder with the following default "settings":
    ///
    /// - `too_long_encoded_word`: [RecoverStrategy::Abort]
    fn default() -> Self {
        Self {
            too_long_encoded_word: RecoverStrategy::Abort,
        }
    }
}

#[cfg(test)]
mod tests {
    /// Here are the main-tests which are listed here:
    /// https://datatracker.ietf.org/doc/html/rfc2047#section-8
    /// Scroll down until you see the table.
    mod rfc_tests {
        use crate::decode;

        #[test]
        fn decode_encoded_word_single_char() {
            assert_eq!(decode("=?ISO-8859-1?Q?a?=").unwrap(), "a");
        }

        #[test]
        fn decode_encoded_word_separated_by_whitespace() {
            assert_eq!(decode("=?ISO-8859-1?Q?a?= b").unwrap(), "a b");
        }

        #[test]
        fn decode_two_encoded_chars() {
            assert_eq!(
                decode("=?ISO-8859-1?Q?a?= =?ISO-8859-1?Q?b?=").unwrap(),
                "ab"
            );
        }

        #[test]
        fn whitespace_between_two_encoded_words_should_be_ignored() {
            assert_eq!(
                decode("=?ISO-8859-1?Q?a?=  =?ISO-8859-1?Q?b?=").unwrap(),
                "ab"
            );
        }

        #[test]
        fn whitespace_chars_between_two_encoded_words_should_be_ignored() {
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
        fn whitespace_encoded_in_encoded_word() {
            assert_eq!(decode("=?ISO-8859-1?Q?a_b?=").unwrap(), "a b");
        }

        #[test]
        fn ignore_whitespace_between_two_encoded_words_but_not_the_encoded_whitespace() {
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
            assert_eq!(
                decode("=?utf-8?B?UG9ydGFsZSBIYWNraW5nVGVhbW==?=").unwrap(),
                "Portale HackingTeam",
            );
        }
    }
}
