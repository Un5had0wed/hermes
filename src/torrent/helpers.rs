use crate::bencode::Bencode;

use std::collections::BTreeMap;

pub(super) fn get<'a>(m: &'a BTreeMap<Vec<u8>, Bencode>, key: &[u8]) -> Option<&'a Bencode> {
    m.get(key)
}

pub(super) fn expect_string(b: &Bencode, field: &str) -> Result<String, String> {
    match b {
        Bencode::ByteString(s) => {
            String::from_utf8(s.clone()).map_err(|_| format!("'{field}' is not valid UTF-8"))
        }
        _ => Err(format!("'{field}' expected a string")),
    }
}

pub(super) fn expect_u64(b: &Bencode, field: &str) -> Result<u64, String> {
    match b {
        Bencode::Integer(n) => {
            u64::try_from(*n).map_err(|_| format!("'{field}' must not be negative"))
        }
        _ => Err(format!("'{field}' expected an integer")),
    }
}

pub(super) fn expect_bytes(b: &Bencode, field: &str) -> Result<Vec<u8>, String> {
    match b {
        Bencode::ByteString(s) => Ok(s.clone()),
        _ => Err(format!("'{field}' expected a byte string")),
    }
}

pub(super) fn expect_list<'a>(b: &'a Bencode, field: &str) -> Result<&'a Vec<Bencode>, String> {
    match b {
        Bencode::List(l) => Ok(l),
        _ => Err(format!("'{field}' expected a list")),
    }
}

pub(super) fn expect_dict<'a>(b: &'a Bencode, field: &str) -> Result<&'a BTreeMap<Vec<u8>, Bencode>, String> {
    match b {
        Bencode::Dict(d) => Ok(d),
        _ => Err(format!("'{field}' expected a dictionary")),
    }
}