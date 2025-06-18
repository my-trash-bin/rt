use std::io::{self, Read};

use jsonc::{parse, Value};

fn serialize(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => {
            let mut escaped = String::new();

            for c in s.chars() {
                match c {
                    '"' => escaped.push_str(r#"\""#),    // "는 \"로 변환
                    '\\' => escaped.push_str(r#"\\"#),   // \는 \\로 변환
                    '\n' => escaped.push_str(r#"\n"#),   // \n은 \n으로 변환
                    '\r' => escaped.push_str(r#"\r"#),   // \r은 \r로 변환
                    '\t' => escaped.push_str(r#"\t"#),   // \t은 \t로 변환
                    '\x08' => escaped.push_str(r#"\b"#), // \x08 (백스페이스)은 \b로 변환
                    '\x0C' => escaped.push_str(r#"\f"#), // \x0C (폼 피드는 \f로 변환
                    _ => escaped.push(c),                // 나머지 문자는 그대로 추가
                }
            }

            format!("\"{}\"", escaped)
        }
        Value::Array(a) => format!(
            "[{}]",
            a.iter().map(serialize).collect::<Vec<_>>().join(",")
        ),
        Value::Object(o) => {
            let mut entries = o.iter().collect::<Vec<_>>();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            format!(
                "{{{}}}",
                entries
                    .iter()
                    .map(|(k, v)| format!(
                        "{}:{}",
                        serialize(&Value::String(k.to_string())),
                        serialize(v)
                    ))
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

fn main() {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .expect("Failed to read stdin");

    let json = parse(input.as_str());
    match json {
        Err(reason) => panic!("Failed to parse input: {}", reason),
        Ok(json) => println!("{}", serialize(&json)),
    }
}
