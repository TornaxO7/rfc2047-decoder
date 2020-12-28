use base64;
use charset::{decode_ascii, Charset};
use quoted_printable;

use crate::lexer::Token;

pub fn apply_encoding(encoding: char, encoded_bytes: &Vec<u8>) -> Vec<u8> {
    match encoding {
        'b' | 'B' => base64::decode(&encoded_bytes).unwrap(),
        'q' | 'Q' | _ => {
            let parse_mode = quoted_printable::ParseMode::Robust;
            let encoded_bytes = encoded_bytes
                .iter()
                .map(|b| if *b == 95 { 32 } else { *b })
                .collect::<Vec<_>>();
            quoted_printable::decode(encoded_bytes, parse_mode).unwrap()
        }
    }
}

pub fn run(tokens: &Vec<Token>) -> String {
    let mut charset_buff: Vec<u8> = vec![];
    let mut encoding_buff: Vec<u8> = vec![];
    let mut decoded_text = String::new();

    for token in tokens {
        match token {
            Token::Charset(charset) => {
                charset_buff = charset.clone();
            }
            Token::Encoding(encoding) => {
                encoding_buff = encoding.clone();
            }
            Token::EncodedText(encoded_bytes) => {
                let encoding = std::str::from_utf8(&encoding_buff)
                    .unwrap()
                    .chars()
                    .next()
                    .unwrap();
                let decoded_bytes = apply_encoding(encoding, encoded_bytes);
                let cow = if let Some(charset) = Charset::for_label(&charset_buff) {
                    charset.decode(&decoded_bytes).0
                } else {
                    decode_ascii(&decoded_bytes)
                };

                decoded_text.push_str(&cow.into_owned());
            }
            Token::RawText(decoded_bytes) => {
                decoded_text.push_str(std::str::from_utf8(&decoded_bytes).unwrap());
            }
        }
    }

    decoded_text
}

#[cfg(test)]
mod tests {
    use crate::lexer::Token;
    use crate::parser::*;

    #[test]
    fn empty_tokens() {
        assert_eq!("".to_string(), run(&vec![] as &Vec<Token>));
    }

    #[test]
    fn decoded_text_only() {
        assert_eq!(
            "decoded-text".to_string(),
            run(&vec![Token::RawText("decoded-text".as_bytes().to_vec())])
        );
    }

    #[test]
    fn utf_8_q() {
        assert_eq!(
            "decoded-text".to_string(),
            run(&vec![
                Token::Charset("utf-8".as_bytes().to_vec()),
                Token::Encoding("Q".as_bytes().to_vec()),
                Token::EncodedText("decoded-text".as_bytes().to_vec())
            ])
        );
    }

    #[test]
    fn utf_8_b() {
        assert_eq!(
            "decoded-text".to_string(),
            run(&vec![
                Token::Charset("utf-8".as_bytes().to_vec()),
                Token::Encoding("B".as_bytes().to_vec()),
                Token::EncodedText("ZGVjb2RlZC10ZXh0".as_bytes().to_vec())
            ])
        );
    }

    #[test]
    fn iso_8858_1_q() {
        assert_eq!(
            "decoded = text".to_string(),
            run(&vec![
                Token::Charset("iso-8859-1".as_bytes().to_vec()),
                Token::Encoding("q".as_bytes().to_vec()),
                Token::EncodedText("decoded_=3D_text".as_bytes().to_vec())
            ])
        );
    }
}
