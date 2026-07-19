use crate::bencode::Bencode;

use std::collections::BTreeMap;
use rand::{random_range};
use std::net::TcpListener;

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

pub(super) fn generate_peer_id() -> [u8; 20] {
    let mut peer_id = [0u8; 20];
    
    // 1. Establish the custom prefix for Hermes
    let prefix = b"-HM0001-";
    peer_id[..8].copy_from_slice(prefix);
    
    // 2. Define the character pool
    let charset = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    
    // 3. Fill the remaining 12 bytes using rand
    for i in 8..20 {
        let idx = random_range(0..charset.len());
        peer_id[i] = charset[idx];
    }
    
    peer_id
}

pub(super) fn get_port() -> Result<u16, String> {
    // 6881-6889 based on BitTorrent protocol specification 
    let mut port = 6881;

    while port <= 6889 && is_port_available(port) == false {
        port += 1;
    }

    if port > 6889 {
        return Err("No available port found in the range 6881-6889".to_string());
    }

    Ok(port)
}

fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}