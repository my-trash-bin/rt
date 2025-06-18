use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub mod ffi {
    #![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use ffi::jsonc_value;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl From<json::JsonValue> for Value {
    fn from(v: json::JsonValue) -> Self {
        match v {
            json::JsonValue::Null => Value::Null,
            json::JsonValue::Boolean(b) => Value::Bool(b),
            json::JsonValue::Number(n) => Value::Number(n),
            json::JsonValue::String(s) => Value::String(s),
            json::JsonValue::List(list) => {
                Value::Array(list.into_iter().map(Value::from).collect())
            }
            json::JsonValue::Dict(map) => {
                Value::Object(map.into_iter().map(|(k, v)| (k, Value::from(v))).collect())
            }
        }
    }
}

impl From<Value> for json::JsonValue {
    fn from(v: Value) -> Self {
        match v {
            Value::Null => json::JsonValue::Null,
            Value::Bool(b) => json::JsonValue::Boolean(b),
            Value::Number(n) => json::JsonValue::Number(n),
            Value::String(s) => json::JsonValue::String(s),
            Value::Array(a) => {
                json::JsonValue::List(a.into_iter().map(json::JsonValue::from).collect())
            }
            Value::Object(o) => json::JsonValue::Dict(
                o.into_iter()
                    .map(|(k, v)| (k, json::JsonValue::from(v)))
                    .collect(),
            ),
        }
    }
}

pub fn parse_to_json(source: &str) -> Result<json::JsonValue, String> {
    let value = parse(source)?;
    Ok(json::JsonValue::from(value))
}

pub fn from_raw(val: &ffi::jsonc_value) -> Value {
    unsafe { convert_value(val) }
}

pub fn parse(source: &str) -> Result<Value, String> {
    let c_source = CString::new(source).map_err(|e| e.to_string())?;
    let mut out = std::mem::MaybeUninit::<ffi::jsonc_value>::uninit();
    let mut is_error: bool = false;
    let result = unsafe { ffi::jsonc_parse(c_source.as_ptr(), out.as_mut_ptr(), &mut is_error) };

    if result {
        return Err("allocation failure".to_string());
    }
    if is_error {
        return Err("parse error".to_string());
    }
    let value = unsafe { out.assume_init() };
    let rust_value = unsafe { convert_value(&value) };
    unsafe { ffi::jsonc_free(value) };
    Ok(rust_value)
}

unsafe fn convert_value(val: &ffi::jsonc_value) -> Value {
    match val.type_ {
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_NULL => Value::Null,
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_BOOLEAN => Value::Bool(val.value.boolean),
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_NUMBER => Value::Number(val.value.number),
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_STRING => {
            let cstr = CStr::from_ptr(val.value.string as *const c_char);
            Value::String(cstr.to_string_lossy().into_owned())
        }
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_ARRAY => {
            let slice =
                std::slice::from_raw_parts(val.value.array.values, val.value.array.count as usize);
            let mut vec = Vec::with_capacity(slice.len());
            for v in slice {
                vec.push(convert_value(v));
            }
            Value::Array(vec)
        }
        ffi::jsonc_value_type_JSONC_VALUE_TYPE_OBJECT => {
            let slice = std::slice::from_raw_parts(
                val.value.object.entries,
                val.value.object.count as usize,
            );
            let mut vec = Vec::with_capacity(slice.len());
            for entry in slice {
                let key = CStr::from_ptr(entry.key as *const c_char)
                    .to_string_lossy()
                    .into_owned();
                vec.push((key, convert_value(&entry.value)));
            }
            Value::Object(vec)
        }
        _ => Value::Null,
    }
}
