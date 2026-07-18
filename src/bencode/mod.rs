mod parser;
mod encoder;
mod display;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Bencode {
    Dict(BTreeMap<Vec<u8>, Bencode>),
    Integer(i64),
    List(Vec<Bencode>),
    ByteString(Vec<u8>),
}

impl Bencode {
    pub fn parse(bytes: &[u8]) -> Result<Bencode, String> {
        let (value, consumed) = parser::parse_value(bytes)?;
        if consumed != bytes.len() {
            return Err("Extra data after valid bencode".to_string());
        }
        Ok(value)
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        encoder::encode(self, &mut out);
        out
    }
}