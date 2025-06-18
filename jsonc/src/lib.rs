use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

mod bindings;
pub use bindings::jsonc_value;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
}

pub fn parse(source: &str) -> Result<Value, String> {
    let c_source = CString::new(source).map_err(|e| e.to_string())?;
    let mut out = std::mem::MaybeUninit::<bindings::jsonc_value>::uninit();
    let mut is_error: bool = false;
    let result =
        unsafe { bindings::jsonc_parse(c_source.as_ptr(), out.as_mut_ptr(), &mut is_error) };

    if result {
        return Err("allocation failure".to_string());
    }
    if is_error {
        return Err("parse error".to_string());
    }
    let value = unsafe { out.assume_init() };
    let rust_value = unsafe { convert_value(&value) };
    unsafe { bindings::jsonc_free(value) };
    Ok(rust_value)
}

unsafe fn convert_value(val: &bindings::jsonc_value) -> Value {
    match val.type_ {
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_NULL => Value::Null,
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_BOOLEAN => Value::Bool(val.value.boolean),
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_NUMBER => Value::Number(val.value.number),
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_STRING => {
            let cstr = CStr::from_ptr(val.value.string as *const c_char);
            Value::String(cstr.to_string_lossy().into_owned())
        }
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_ARRAY => {
            let slice =
                std::slice::from_raw_parts(val.value.array.values, val.value.array.count as usize);
            let mut vec = Vec::with_capacity(slice.len());
            for v in slice {
                vec.push(convert_value(v));
            }
            Value::Array(vec)
        }
        bindings::jsonc_value_type_JSONC_VALUE_TYPE_OBJECT => {
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
            Value::Object(vec.into_iter().collect())
        }
        _ => Value::Null,
    }
}
