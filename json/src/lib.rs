use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<JsonValue>),
    Dict(HashMap<String, JsonValue>),
}

#[derive(Debug, PartialEq)]
enum Token {
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

#[derive(PartialEq)]
enum TokenizerStateFunction {
    Error(String),
    Func(fn(TokenizerStateValue, Option<char>, &mut Vec<Token>) -> TokenizerState),
}

struct TokenizerState {
    next: TokenizerStateFunction,
    value: TokenizerStateValue,
}

fn tokenizer_state_default(
    _state: TokenizerStateValue,
    input: Option<char>,
    result: &mut Vec<Token>,
) -> TokenizerState {
    match input {
        Some(input) if matches!(input, ' ' | '\x0C' | '\n' | '\r' | '\t' | '\x0B') => {
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_default),
                value: TokenizerStateValue::None,
            }
        }
        Some(input) if matches!(input, ',' | ':' | '{' | '}' | '[' | ']') => {
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
        Some(input) if matches!(input, '"') => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_string_any),
            value: TokenizerStateValue::String("".to_string()),
        },
        Some(input) => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_keyword_or_number),
            value: TokenizerStateValue::KeywordOrNumber(input.to_string()),
        },
        None => TokenizerState {
            next: TokenizerStateFunction::Func(tokenizer_state_default),
            value: TokenizerStateValue::None,
        },
    }
}

fn tokenizer_state_keyword_or_number(
    state: TokenizerStateValue,
    input: Option<char>,
    result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::KeywordOrNumber(str) = state {
        match input {
            Some(input)
                if matches!(
                    input,
                    ' ' | '\x0C'
                        | '\n'
                        | '\r'
                        | '\t'
                        | '\x0B'
                        | ','
                        | ':'
                        | '{'
                        | '}'
                        | '['
                        | ']'
                        | '"'
                ) =>
            {
                match str.as_str() {
                    "null" | "false" | "true" => {
                        result.push(match str.as_str() {
                            "null" => Token::Null,
                            "false" => Token::False,
                            "true" => Token::True,
                            _ => panic!(),
                        });
                        tokenizer_state_default(TokenizerStateValue::None, Some(input), result)
                    }
                    _ => {
                        if let Ok(value) = str.parse::<f64>() {
                            result.push(Token::Number(value));
                            tokenizer_state_default(TokenizerStateValue::None, Some(input), result)
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
                }
            }
            Some(input) => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_keyword_or_number),
                value: TokenizerStateValue::KeywordOrNumber(str + input.to_string().as_str()),
            },
            None => match str.as_str() {
                "null" | "false" | "true" => {
                    result.push(match str.as_str() {
                        "null" => Token::Null,
                        "false" => Token::False,
                        "true" => Token::True,
                        _ => panic!(),
                    });
                    tokenizer_state_default(TokenizerStateValue::None, None, result)
                }
                _ => {
                    if let Ok(value) = str.parse::<f64>() {
                        result.push(Token::Number(value));
                        tokenizer_state_default(TokenizerStateValue::None, None, result)
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
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_any(
    state: TokenizerStateValue,
    input: Option<char>,
    result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::String(str) = state {
        match input {
            Some(input) if matches!(input, '"') => {
                result.push(Token::String(str));
                TokenizerState {
                    next: TokenizerStateFunction::Func(tokenizer_state_default),
                    value: TokenizerStateValue::None,
                }
            }
            Some(input) if matches!(input, '\\') => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_backslash),
                value: TokenizerStateValue::String(str),
            },
            Some(input) => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_any),
                value: TokenizerStateValue::String(str + input.to_string().as_str()),
            },
            None => TokenizerState {
                next: TokenizerStateFunction::Error(format!("Invalid end of input: \"{}", str)),
                value: TokenizerStateValue::None,
            },
        }
    } else {
        panic!()
    }
}

fn tokenizer_state_string_backslash(
    state: TokenizerStateValue,
    input: Option<char>,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::String(str) = state {
        match input {
            Some(input) if matches!(input, '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't') => {
                TokenizerState {
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
                }
            }
            Some(input) if matches!(input, 'u') => TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_u0),
                value: TokenizerStateValue::StringU(str, "\\u".to_string()),
            },
            _ => TokenizerState {
                next: TokenizerStateFunction::Error(format!(
                    "Invalid escape sequence: \\{}",
                    match input {
                        Some(input) => input.to_string(),
                        None => "".to_string(),
                    },
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
    input: Option<char>,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        if let Some(input) = input {
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_u1),
                value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
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

fn tokenizer_state_string_u1(
    state: TokenizerStateValue,
    input: Option<char>,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        if let Some(input) = input {
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_u2),
                value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
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

fn tokenizer_state_string_u2(
    state: TokenizerStateValue,
    input: Option<char>,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        if let Some(input) = input {
            TokenizerState {
                next: TokenizerStateFunction::Func(tokenizer_state_string_u3),
                value: TokenizerStateValue::StringU(str, escaped + input.to_string().as_str()),
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
    input: Option<char>,
    _result: &mut Vec<Token>,
) -> TokenizerState {
    if let TokenizerStateValue::StringU(str, escaped) = state {
        if let Some(input) = input {
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

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut state = TokenizerState {
        next: TokenizerStateFunction::Func(tokenizer_state_default),
        value: TokenizerStateValue::None,
    };
    let mut result = vec![];

    for c in input.chars() {
        state = match state.next {
            TokenizerStateFunction::Error(reason) => {
                return Err(reason);
            }
            TokenizerStateFunction::Func(func) => func(state.value, Some(c), &mut result),
        };
    }

    state = match state.next {
        TokenizerStateFunction::Error(reason) => {
            return Err(reason);
        }
        TokenizerStateFunction::Func(func) => func(state.value, None, &mut result),
    };

    if state.next == TokenizerStateFunction::Func(tokenizer_state_default) {
        Ok(result)
    } else {
        Err("Invalid termination".to_string())
    }
}

fn parse_value(
    iter: &mut std::vec::IntoIter<Token>,
    position: String,
) -> Result<JsonValue, String> {
    match parse_list_entry(iter, position.clone())? {
        None => Err(format!("Expected value on {}", position)),
        Some(value) => Ok(value),
    }
}

fn parse_dict(
    iter: &mut std::vec::IntoIter<Token>,
    position: String,
) -> Result<HashMap<String, JsonValue>, String> {
    match iter.next() {
        Some(Token::String(key)) => {
            if !matches!(iter.next(), Some(Token::Colon)) {
                Err(format!("Expected ':' on {}[{}]", position, key))
            } else {
                match parse_value(iter, format!("{}[{}]", position, key)) {
                    Err(reason) => Err(reason),
                    Ok(value) => {
                        let mut result = HashMap::new();
                        result.insert(key, value);
                        let mut index = 1usize;
                        loop {
                            match iter.next() {
                                Some(Token::BraceClose) => return Ok(result),
                                Some(Token::Comma) => match iter.next() {
                                    Some(Token::String(key)) => {
                                        if !matches!(iter.next(), Some(Token::Colon)) {
                                            return Err(format!(
                                                "Expected ':' on {}[{}]",
                                                position, key
                                            ));
                                        } else {
                                            if result.contains_key(key.as_str()) {
                                                return Err(format!(
                                                    "Duplicate key {} on {}",
                                                    key, position
                                                ));
                                            }
                                            result.insert(
                                                key.clone(),
                                                parse_value(
                                                    iter,
                                                    format!("{}[{}]", position, key),
                                                )?,
                                            );
                                            index += 1;
                                        }
                                    }
                                    Some(Token::BraceClose) => return Ok(result),
                                    _ => {
                                        return Err(format!(
                                            "Expected key or '}}' on {}[{}]",
                                            position, index
                                        ))
                                    }
                                },
                                _ => {
                                    return Err(format!(
                                        "Expected ',' or '}}' on {}[{}]",
                                        position, index
                                    ))
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Token::BraceClose) => Ok(HashMap::new()),
        _ => Err(format!("Expected key or '}}' on {}", position)),
    }
}

fn parse_list_entry(
    iter: &mut std::vec::IntoIter<Token>,
    position: String,
) -> Result<Option<JsonValue>, String> {
    match iter.next() {
        Some(Token::BracketClose) => Ok(None),
        None | Some(Token::Comma | Token::Colon | Token::BraceClose) => {
            Err(format!("Expected value on {}", position))
        }
        Some(Token::Null) => Ok(Some(JsonValue::Null)),
        Some(Token::True) => Ok(Some(JsonValue::Boolean(true))),
        Some(Token::False) => Ok(Some(JsonValue::Boolean(false))),
        Some(Token::Number(value)) => Ok(Some(JsonValue::Number(value))),
        Some(Token::String(value)) => Ok(Some(JsonValue::String(value))),
        Some(Token::BraceOpen) => Ok(Some(JsonValue::Dict(parse_dict(iter, position)?))),
        Some(Token::BracketOpen) => Ok(Some(JsonValue::List(parse_list(iter, position)?))),
    }
}

fn parse_list(
    iter: &mut std::vec::IntoIter<Token>,
    position: String,
) -> Result<Vec<JsonValue>, String> {
    match parse_list_entry(iter, format!("{}[0]", position))? {
        None => Ok(vec![]),
        Some(value) => {
            let mut result = vec![value];
            let mut index = 1usize;
            loop {
                match iter.next() {
                    Some(Token::BracketClose) => return Ok(result),
                    Some(Token::Comma) => {
                        result.push(parse_value(iter, format!("{}[{}]", position, index))?);
                        index += 1;
                    }
                    _ => return Err(format!("Expected ',' or ']' on {}[{}]", position, index)),
                }
            }
        }
    }
}

fn string_to_literal(input: &str) -> String {
    let mut result = String::from("\"");

    for ch in input.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0C' => result.push_str("\\f"),
            _ if !ch.is_ascii_graphic() => {
                let utf16_code = ch as u32;
                result.push_str(&format!("\\u{:04X}", utf16_code));
            }
            _ => result.push(ch),
        }
    }

    result.push('"');
    result
}

impl JsonValue {
    pub fn new(source: &str) -> Result<JsonValue, String> {
        match tokenize(source) {
            Ok(tokens) => {
                let mut iter = tokens.into_iter();
                let result = parse_value(&mut iter, "root".to_string());
                if matches!(iter.next(), None) {
                    result
                } else {
                    Err("Extra token found".to_string())
                }
            }
            Err(reason) => Err(reason),
        }
    }

    pub fn serialize(&self) -> String {
        match self {
            JsonValue::Null => "null".to_string(),
            JsonValue::Boolean(true) => "true".to_string(),
            JsonValue::Boolean(false) => "false".to_string(),
            JsonValue::Number(number) => number.to_string(),
            JsonValue::String(string) => string_to_literal(string.as_str()),
            JsonValue::Dict(map) => format!(
                "{{{}}}",
                map.iter()
                    .map(|(key, value)| format!("{}:{}", string_to_literal(key), value.serialize()))
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            JsonValue::List(vec) => format!(
                "[{}]",
                vec.iter()
                    .map(|x| x.serialize())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let input = "{ \"t\\u0065st\": [true, 42e-1] }";
        let tokens = tokenize(input).unwrap();

        assert_eq!(
            tokens,
            vec![
                Token::BraceOpen,
                Token::String("test".to_string()),
                Token::Colon,
                Token::BracketOpen,
                Token::True,
                Token::Comma,
                Token::Number(4.2f64),
                Token::BracketClose,
                Token::BraceClose
            ]
        );
    }

    #[test]
    fn test_parse() {
        let input = "{ \"t\\u0065st\": [true, 42e-1] }";
        let result = JsonValue::new(input).unwrap();

        assert_eq!(
            result,
            JsonValue::Dict(
                vec![(
                    "t\u{0065}st".to_string(),
                    JsonValue::List(vec![JsonValue::Boolean(true), JsonValue::Number(4.2)])
                )]
                .into_iter()
                .collect()
            )
        );
    }

    #[test]
    fn test_serialize() {
        let input = JsonValue::Dict(
            vec![(
                "t\u{0065}st".to_string(),
                JsonValue::List(vec![JsonValue::Boolean(true), JsonValue::Number(4.2)]),
            )]
            .into_iter()
            .collect(),
        );
        let result = JsonValue::new(JsonValue::serialize(&input).as_str()).unwrap();

        assert_eq!(result, input);
    }
}
