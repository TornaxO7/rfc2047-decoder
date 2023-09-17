use std::fmt::Display;

use super::QUESTION_MARK;

pub const PREFIX: &[u8] = "=?".as_bytes();
pub const SUFFIX: &[u8] = "?=".as_bytes();
pub const MAX_LENGTH: usize = 75;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EncodedWord {
    pub charset: Vec<u8>,
    pub encoding: Vec<u8>,
    pub encoded_text: Vec<u8>,
}

impl EncodedWord {
    pub fn new(charset: Vec<u8>, encoding: Vec<u8>, encoded_text: Vec<u8>) -> Self {
        Self {
            charset,
            encoding,
            encoded_text,
        }
    }

    pub fn from_parser(((charset, encoding), encoded_text): ((Vec<u8>, Vec<u8>), Vec<u8>)) -> Self {
        Self::new(charset, encoding, encoded_text)
    }

    /// Returns the amount of `char`s for this encoded word
    pub fn len(&self) -> usize {
        self.get_bytes(true).len()
    }

    pub fn get_bytes(&self, with_delimiters: bool) -> Vec<u8> {
        let mut bytes = Vec::new();

        if with_delimiters {
            bytes.extend(PREFIX);
            bytes.extend(&self.charset);
            bytes.extend(&[QUESTION_MARK]);
            bytes.extend(&self.encoding);
            bytes.extend(&[QUESTION_MARK]);
            bytes.extend(&self.encoded_text);
            bytes.extend(SUFFIX);
        } else {
            bytes.extend(&self.charset);
            bytes.extend(&self.encoding);
            bytes.extend(&self.encoded_text);
        }

        bytes
    }
}

impl Display for EncodedWord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let charset = String::from_utf8(self.charset.clone()).unwrap();
        let encoding = String::from_utf8(self.encoding.clone()).unwrap();
        let encoded_text = String::from_utf8(self.encoded_text.clone()).unwrap();

        write!(f, "=?{}?{}?{}?=", charset, encoding, encoded_text)
    }
}
