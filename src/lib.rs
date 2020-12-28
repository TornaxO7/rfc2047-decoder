mod lexer;
mod parser;

type Error = lexer::Error;

pub fn decode(encoded_str: &str) -> Result<String, Error> {
    let tokens = crate::lexer::run(encoded_str)?;
    Ok(crate::parser::run(&tokens))
}

#[cfg(test)]
mod tests {
    use crate::decode;

    #[test]
    fn decode_iso_8859_1_q() {
        assert_eq!(
            Ok("decoded = text".to_string()),
            decode("=?iso-8859-1?Q?decoded_=3D_text?=")
        );
    }
}
