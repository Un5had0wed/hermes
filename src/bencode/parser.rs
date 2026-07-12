use std::collections::BTreeMap;
use super::Bencode;

pub(super) fn parse_value(bytes: &[u8]) -> Result<(Bencode, usize), String> {
    match bytes.first() {
        Some(b'd') => parse_dict(bytes),
        Some(b'i') => parse_integer(bytes),
        Some(b'l') => parse_list(bytes),
        Some(b'0'..=b'9') => parse_byte_string(bytes),
        Some(_) => Err("Invalid bencode format".to_string()),
        None => Err("Unexpected end of input".to_string()),
    }
}

fn parse_integer(bytes: &[u8]) -> Result<(Bencode, usize), String> {
    let mut idx = 0;
    if bytes.get(idx) != Some(&b'i') {
        return Err("Invalid integer format: expected 'i'".to_string());
    }
    idx += 1;

    let mut is_negative = false;
    if bytes.get(idx) == Some(&b'-') {
        is_negative = true;
        idx += 1;
    }

    let digits_start = idx;
    let mut num: i64 = 0;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        num = num
            .checked_mul(10)
            .and_then(|n| n.checked_add((bytes[idx] - b'0') as i64))
            .ok_or("Integer overflow")?;
        idx += 1;
    }

    if idx == digits_start {
        return Err("Invalid integer format: no digits".to_string());
    }

    if bytes[digits_start] == b'0' && idx - digits_start > 1 {
        return Err("Invalid integer format: leading zero".to_string());
    }
    if is_negative && bytes[digits_start] == b'0' {
        return Err("Invalid integer format: negative zero".to_string());
    }

    if bytes.get(idx) != Some(&b'e') {
        return Err("Invalid integer format: missing 'e' terminator".to_string());
    }
    idx += 1;

    if is_negative {
        num = -num;
    }

    Ok((Bencode::Integer(num), idx))
}

fn parse_byte_string(bytes: &[u8]) -> Result<(Bencode, usize), String> {
    let mut idx = 0;

    if bytes.is_empty() || !bytes[0].is_ascii_digit() {
        return Err("Invalid string format: expected length digit".to_string());
    }

    let mut length: usize = 0;
    while idx < bytes.len() && bytes[idx].is_ascii_digit() {
        length = length
            .checked_mul(10)
            .and_then(|n| n.checked_add((bytes[idx] - b'0') as usize))
            .ok_or("String length overflow")?;
        idx += 1;
    }

    if bytes[0] == b'0' && idx > 1 {
        return Err("Invalid string format: leading zero in length".to_string());
    }

    if bytes.get(idx) != Some(&b':') {
        return Err("Invalid string format: missing ':'".to_string());
    }
    idx += 1;

    if idx + length > bytes.len() {
        return Err("Invalid string format: length exceeds remaining bytes".to_string());
    }

    let string_bytes = &bytes[idx..idx + length];
    let bytes = string_bytes.to_vec();
    idx += length;

    Ok((Bencode::ByteString(bytes), idx))
}

fn parse_list(bytes: &[u8]) -> Result<(Bencode, usize), String> {
    let mut idx = 0;
    if bytes.get(idx) != Some(&b'l') {
        return Err("Invalid list format: expected 'l'".to_string());
    }
    idx += 1;

    let mut list = Vec::new();
    while idx < bytes.len() && bytes[idx] != b'e' {
        let (element, consumed) = parse_value(&bytes[idx..])?;
        list.push(element);
        idx += consumed;
    }

    if bytes.get(idx) != Some(&b'e') {
        return Err("Invalid list format: missing 'e' terminator".to_string());
    }
    idx += 1;

    Ok((Bencode::List(list), idx))
}

fn parse_dict(bytes: &[u8]) -> Result<(Bencode, usize), String> {
    let mut idx = 0;
    if bytes.get(idx) != Some(&b'd') {
        return Err("Invalid dict format: expected 'd'".to_string());
    }
    idx += 1;

    let mut map = BTreeMap::new();
    let mut last_key: Option<Vec<u8>> = None;

    while idx < bytes.len() && bytes[idx] != b'e' {
        let (key_val, key_consumed) = parse_byte_string(&bytes[idx..])?;
        idx += key_consumed;

        let key = match key_val {
            Bencode::ByteString(bytes) => bytes,
            _ => unreachable!("parse_byte_string always returns ByteString"),
        };

        // Keys must be in strictly increasing lexicographical byte order.
        if let Some(ref last) = last_key {
            if key <= *last {
                return Err("Invalid dict format: keys not sorted/unique".to_string());
            }
        }

        let (value, val_consumed) = parse_value(&bytes[idx..])?;
       idx += val_consumed;

        last_key = Some(key.clone());
        map.insert(key, value);
    }

    if bytes.get(idx) != Some(&b'e') {
        return Err("Invalid dict format: missing 'e' terminator".to_string());
    }
    idx += 1;

    Ok((Bencode::Dict(map), idx))
}
