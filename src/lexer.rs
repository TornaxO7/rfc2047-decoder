#[derive(Debug, PartialEq)]
pub enum Token {
    Charset(String),
    Encoding(String),
    EncodedText(String),
    DecodedText(String),
}

#[derive(Debug, PartialEq)]
pub enum State {
    Charset,
    Encoding,
    EncodedText,
    DecodedText,
}

pub fn run(encoded_str: &str) -> Vec<Token> {
    let mut encoded_chars = encoded_str.chars();
    let mut tokens = vec![];
    let mut curr_char = encoded_chars.next();
    let mut state = State::DecodedText;
    let mut buffer = String::new();

    loop {
        match state {
            State::Charset => match curr_char {
                Some('?') => {
                    state = State::Encoding;
                        tokens.push(Token::Charset(buffer.clone()));
                        buffer.clear();
                }
                Some(c) => buffer.push(c),
                None => panic!("Charset section not terminated"),
            },
            State::Encoding => match curr_char {
                Some('?') => {
                    state = State::EncodedText;
                        tokens.push(Token::Encoding(buffer.clone()));
                        buffer.clear();
                }
                Some(c) => buffer.push(c),
                None => panic!("Encoding section not terminated"),
            },
            State::EncodedText => match curr_char {
                Some('?') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('=') => {
                            state = State::DecodedText;

                            if buffer.len() > 0 {
                                tokens.push(Token::EncodedText(buffer.clone()));
                                buffer.clear();
                            }
                        }
                        _ => {
                            buffer.push('?');
                            continue;
                        }
                    }
                }
                Some(c) => buffer.push(c),
                None => panic!("Encoded text section not terminated"),
            },
            State::DecodedText => match curr_char {
                Some('=') => {
                    curr_char = encoded_chars.next();

                    match curr_char {
                        Some('?') => {
                            state = State::Charset;

                            if buffer.len() > 0 {
                                tokens.push(Token::DecodedText(buffer.clone()));
                                buffer.clear()
                            }
                        }
                        _ => {
                            buffer.push('=');
                            continue;
                        }
                    }
                }
                Some(c) => buffer.push(c),
                None => {
                    if buffer.len() > 0 {
                        tokens.push(Token::DecodedText(buffer.clone()));
                    }

                    break;
                }
            },
        }

        curr_char = encoded_chars.next();
    }

    tokens
}
