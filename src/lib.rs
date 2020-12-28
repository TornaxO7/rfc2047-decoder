use std::{error, fmt, result};

mod lexer;
mod parser;

#[derive(Debug)]
pub enum Error {
    LexerError(lexer::Error),
    ParserError(parser::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::LexerError(err) => err.fmt(f),
            Error::ParserError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}

impl From<lexer::Error> for Error {
    fn from(err: lexer::Error) -> Error {
        Error::LexerError(err)
    }
}

impl From<parser::Error> for Error {
    fn from(err: parser::Error) -> Error {
        Error::ParserError(err)
    }
}

pub type Result<T> = result::Result<T, Error>;

pub fn decode(encoded_str: &str) -> Result<String> {
    let tokens = crate::lexer::run(encoded_str)?;
    let decoded_str = crate::parser::run(&tokens)?;
    Ok(decoded_str)
}

#[cfg(test)]
mod tests {
    use crate::decode;

    #[test]
    fn decode_iso_8859_1_q() {
        assert_eq!(
            "decoded = text".to_string(),
            decode("=?iso-8859-1?Q?decoded_=3D_text?=").unwrap()
        );
    }
}
