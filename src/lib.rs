mod lexer;

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
            vec![Token::DecodedText("decoded string".to_string())],
            run("decoded string")
        );
    }

    #[test]
    fn decoded_text_except() {
        assert_eq!(
            vec![
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded-text".to_string()),
            ],
            run("=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn decoded_text_before() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".to_string()),
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded-text".to_string()),
            ],
            run("decoded-text=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn decoded_text_after() {
        assert_eq!(
            vec![
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded-text".to_string()),
                Token::DecodedText("decoded-text".to_string()),
            ],
            run("=?charset?encoding?encoded-text?=decoded-text")
        );
    }

    #[test]
    fn decoded_text_between() {
        assert_eq!(
            vec![
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded-text".to_string()),
                Token::DecodedText("decoded-text".to_string()),
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded-text".to_string()),
            ],
            run("=?charset?encoding?encoded-text?=decoded-text=?charset?encoding?encoded-text?=")
        );
    }

    #[test]
    fn empty_encoded_text() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".to_string()),
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
            ],
            run("decoded-text=?charset?encoding??=")
        );
    }

    #[test]
    fn encoded_text_with_question_mark() {
        assert_eq!(
            vec![
                Token::DecodedText("decoded-text".to_string()),
                Token::Charset("charset".to_string()),
                Token::Encoding("encoding".to_string()),
                Token::EncodedText("encoded?text".to_string()),
            ],
            run("decoded-text=?charset?encoding?encoded?text?=")
        );
    }
}
