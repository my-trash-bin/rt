use std::collections::HashMap;

pub fn pack(input: HashMap<String, Vec<u8>>) -> Vec<u8> {
    let mut packed_data = Vec::new();

    for (key, value) in input {
        let key_len = key.len() as u32;
        let value_len = value.len() as u32;

        packed_data.extend(&key_len.to_le_bytes());
        packed_data.extend(key.as_bytes());
        packed_data.extend(&value_len.to_le_bytes());
        packed_data.extend(value);
    }

    packed_data
}

pub fn unpack(input: Vec<u8>) -> Option<HashMap<String, Vec<u8>>> {
    let mut unpacked_data = HashMap::new();
    let mut index = 0;

    while index < input.len() {
        if index + 8 > input.len() {
            return None; // Not enough data for key length and value length
        }

        let key_len_bytes: [u8; 4] = input[index..index + 4].try_into().unwrap();
        let key_len = u32::from_le_bytes(key_len_bytes);

        index += 4;

        if index + key_len as usize > input.len() {
            return None; // Not enough data for the key
        }

        let key_bytes = &input[index..index + key_len as usize];
        let key = String::from_utf8_lossy(key_bytes).into_owned();

        index += key_len as usize;

        if index + 4 > input.len() {
            return None; // Not enough data for value length
        }

        let value_len_bytes: [u8; 4] = input[index..index + 4].try_into().unwrap();
        let value_len = u32::from_le_bytes(value_len_bytes);

        index += 4;

        if index + value_len as usize > input.len() {
            return None; // Not enough data for the value
        }

        let value = input[index..index + value_len as usize].to_vec();

        index += value_len as usize;

        unpacked_data.insert(key, value);
    }

    Some(unpacked_data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack() {
        let mut input = HashMap::new();
        input.insert("input1".to_string(), (0..42).map(|i| i).collect());
        input.insert("input2".to_string(), (0..42).map(|i| i + 2).collect());
        input.insert("input3".to_string(), (0..42).map(|i| i * 2).collect());
        let output = unpack(pack(input.clone())).unwrap();

        assert_eq!(input, output);
    }
}
