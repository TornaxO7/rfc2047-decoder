#[derive(Debug, PartialEq)]
pub enum Token {
    Charset(Vec<u8>),
    Encoding(Vec<u8>),
    EncodedText(Vec<u8>),
    DecodedText(Vec<u8>),
}

#[derive(Debug, PartialEq)]
pub enum State {
    Charset,
    Encoding,
    EncodedText,
    DecodedText,
}

pub fn encode_utf8_char(c: char) -> Vec<u8> {
    let mut buff: [u8;4] = [0;4];
    c.encode_utf8(&mut buff);
    buff[0..c.len_utf8()].to_vec()
}

pub fn run(encoded_str: &str) -> Vec<Token> {
    let mut encoded_chars = encoded_str.chars();
    let mut tokens = vec![];
    let mut curr_char = encoded_chars.next();
    let mut state = State::DecodedText;
    let mut buffer: Vec<u8> = vec![];

    loop {
        match state {
            State::Charset => match curr_char {
                Some('?') => {
                    state = State::Encoding;
                        tokens.push(Token::Charset(buffer.clone()));
                        buffer.clear();
                }
                Some(c) => buffer.append(encode_utf8_char(c).as_mut()),
                None => panic!("Charset section not terminated"),
            },
            State::Encoding => match curr_char {
                Some('?') => {
                    state = State::EncodedText;
                        tokens.push(Token::Encoding(buffer.clone()));
                        buffer.clear();
                }
                Some(c) => buffer.append(encode_utf8_char(c).as_mut()),
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
                            buffer.append(encode_utf8_char('?').as_mut());
                            continue;
                        }
                    }
                }
                Some(c) => buffer.append(encode_utf8_char(c).as_mut()),
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
                            buffer.append(encode_utf8_char('=').as_mut());
                            continue;
                        }
                    }
                }
                Some(c) => buffer.append(encode_utf8_char(c).as_mut()),
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
