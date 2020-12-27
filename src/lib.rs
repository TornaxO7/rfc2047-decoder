mod lexer;
mod parser;

#[cfg(test)]
mod lexer_tests {
    use crate::lexer::*;

    #[test]
    fn empty_str() {
        assert_eq!(vec![] as Vec<Token>, run(""));
    }

    #[test]
    fn decoded_text_only() {
        assert_eq!(
            vec![Token::DecodedText("decoded string".as_bytes().to_vec())],
            run("decoded string")
        );
    }

    #[test]
    fn decoded_text_except() {
        assert_eq!(
            vec![
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded-text".as_bytes().to_vec()),
            ],
            run("=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn decoded_text_before() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".as_bytes().to_vec()),
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded-text".as_bytes().to_vec()),
            ],
            run("decoded-text=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn decoded_text_after() {
        assert_eq!(
            vec![
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded-text".as_bytes().to_vec()),
                Token::DecodedText("decoded-text".as_bytes().to_vec()),
            ],
            run("=?charset?encoding?encoded-text?=decoded-text")
        );
    }

    #[test]
    fn decoded_text_between() {
        assert_eq!(
            vec![
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded-text".as_bytes().to_vec()),
                Token::DecodedText("decoded-text".as_bytes().to_vec()),
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded-text".as_bytes().to_vec()),
            ],
            run("=?charset?encoding?encoded-text?=decoded-text=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn empty_encoded_text() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".as_bytes().to_vec()),
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
            ],
            run("decoded-text=?charset?encoding??=")
        );
    }

    #[test]
    fn encoded_text_with_question_mark() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".as_bytes().to_vec()),
                Token::Charset("charset".as_bytes().to_vec()),
                Token::Encoding("encoding".as_bytes().to_vec()),
                Token::EncodedText("encoded?text".as_bytes().to_vec()),
            ],
            run("decoded-text=?charset?encoding?encoded?text?=")
        );
    }
}

#[cfg(test)]
mod parser_tests {
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
            run(&vec![Token::DecodedText(
                "decoded-text".as_bytes().to_vec()
            )])
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

pub fn decode(encoded_str: &str) -> String {
    let tokens = crate::lexer::run(encoded_str);
    crate::parser::run(&tokens)
}

#[test]
fn decode_iso_8859_1_q() {
    assert_eq!(
        "decoded = text".to_string(),
        decode("=?iso-8859-1?Q?decoded_=3D_text?=")
    );
}
