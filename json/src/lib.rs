use std::{collections::HashMap, error::Error};

pub enum JsonValue {
    Error,
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<JsonValue>),
    Dict(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
enum Token {
    Eof,
    Null,
    Comma,
    Colon,
    True,
    False,
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Number(f64),
    String(String),
}

enum TokenizerStateValue {
    None,
    KeywordOrNumber(String),
    StringU(String, String),
    String(String),
}

enum TokenizerStateFunction {
    Error(String),
    Func(fn(TokenizerStateValue, char, &mut Vec<Token>) -> TokenizerState),
}

struct TokenizerState {
    next: TokenizerStateFunction,
    value: TokenizerStateValue,
}

fn tokenizer_state_default(
    _state: TokenizerStateValue,
    input: char,
    result: &mut Vec<Token>,
) -> TokenizerState {
    match input {
        ' ' | '\x0C' | '\n' | '\r' | '\t' | '\x0B' => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_default),
            value: TokenizerStateValue::None,
        },
        ',' | ':' | '{' | '}' | '[' | ']' => {
            result.push(match input {
                ',' => Token::Comma,
                ':' => Token::Colon,
                '{' => Token::BraceOpen,
                '}' => Token::BraceClose,
                '[' => Token::BracketOpen,
                ']' => Token::BracketClose,
                _ => panic!(),
            });
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_default),
                value: TokenizerStateValue::None,
            }
        }
        '"' => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_string_any),
            value: TokenizerStateValue::String("".to_string()),
        },
        _ => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_keyword_or_number),
            value: TokenizerStateValue::KeywordOrNumber(input.to_string()),
        },
    }
}

fn tokenizer_state_keyword_or_number(
    state: TokenizerStateValue,
    input: char,
    result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::KeywordOrNumber(str) = state {
        match input {
            ' ' | '\x0C' | '\n' | '\r' | '\t' | '\x0B' | ',' | ':' | '{' | '}' | '[' | ']'
            | '"' => match str.as_str() {
                "null" | "false" | "true" => {
                    result.push(match str.as_str() {
                        "null" => Token::Null,
                        "false" => Token::False,
                        "true" => Token::True,
                        _ => panic!(),
                    });
                    TokenizerState {
                        next: TokenizerStateFunction::Func(tokenizer_state_default),
                        value: TokenizerStateValue::None,
                    }
                }
                _ => {
                    if let Ok(value) = str.parse::<f64>() {
                        result.push(Token::Number(value));
                        tokenizer_state_default(TokenizerStateValue::None, input, result)
                    } else {
                        TokenizerState {
                            next: TokenizerStateFunction::Error(format!(
                                "Invalid keyword or number {}",
                                str
                            )),
                            value: TokenizerStateValue::None,
                        }
                    }
                }
            },
            _ => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_keyword_or_number),
                value: TokenizerStateValue::KeywordOrNumber(str + input.to_string().as_str()),
            },
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_any(
    state: TokenizerStateValue,
    input: char,
    result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::String(str) = state {
        match input {
            '"' => {
                result.push(Token::String(str));
                TokenizerState {
                    next: TokenizerStateFunction::Func(tokenizer_state_default),
                    value: TokenizerStateValue::None,
                }
            }
            '\\' => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_backslash),
                value: TokenizerStateValue::String(str),
            },
            _ => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_any),
                value: TokenizerStateValue::String(str + input.to_string().as_str()),
            },
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_backslash(
    state: TokenizerStateValue,
    input: char,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::String(str) = state {
        match input {
            '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_any),
                value: TokenizerStateValue::String(
                    str + (match input {
                        '"' => "\"",
                        '\\' => "\\",
                        '/' => "/",
                        'b' => "\x08",
                        'f' => "\x0C",
                        'n' => "\n",
                        'r' => "\r",
                        't' => "\t",
                        _ => panic!(),
                    }),
                ),
            },
            'u' => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_u0),
                value: TokenizerStateValue::StringU(str, "\\u".to_string()),
            },
            _ => TokenizerState {
                next: TokenizerStateFunction::Error(format!(
                    "Invalid escape sequence: \\{}",
                    input
                )),
                value: TokenizerStateValue::None,
            },
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_u0(
    state: TokenizerStateValue,
    input: char,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_string_u1),
            value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_u1(
    state: TokenizerStateValue,
    input: char,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_string_u2),
            value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_u2(
    state: TokenizerStateValue,
    input: char,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_string_u3),
            value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
        }
    } else {
        panic!()
    }
}

fn unicode_escape_to_original(escape_sequence: &str) -> Option<char> {
    if escape_sequence.len() == 6 && escape_sequence.starts_with("\\u") {
        let hex_digits = &escape_sequence[2..];
        if let Ok(code_point) = u32::from_str_radix(hex_digits, 16) {
            if let Some(original) = std::char::from_u32(code_point) {
                return Some(original);
            }
        }
    }
    None
}

fn tokenizer_state_string_u3(
    state: TokenizerStateValue,
    input: char,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        let escaped = escaped + input.to_string().as_str();
        if let Some(original) = unicode_escape_to_original(escaped.as_str()) {
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_any),
                value: TokenizerStateValue::String(str + original.to_string().as_str()),
            }
        } else {
            TokenizerState {
                next: TokenizerStateFunction::Error(format!(
                    "Invalid escape sequence: {}",
                    escaped
                )),
                value: TokenizerStateValue::None,
            }
        }
    } else {
        panic!()
    }
}

fn tokenize(input: &str) -> Result<Vec<Token>, Box<dyn Error>> {
    let mut state = TokenizerState {
        next: TokenizerStateFunction::Func(tokenizer_state_default),
        value: TokenizerStateValue::None,
    };
    let mut result = vec![];

    for c in input.chars() {
        state = match state.next {
            TokenizerStateFunction::Error(reason) => {
                return Err(reason.into());
            }
            TokenizerStateFunction::Func(func) => func(state.value, c, &mut result),
        };
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "{ \"test\": true }";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::BraceOpen,
                Token::String("test".to_string()),
                Token::Colon,
                Token::True,
                Token::BraceClose
            ]
        );
    }
}
