use std::collections::BTreeMap;
use super::Bencode;

impl Bencode {
    pub fn as_dict(&self, field: &str) -> Result<&BTreeMap<Vec<u8>, Bencode>, String> {
        match self {
            Bencode::Dict(d) => Ok(d),
            _ => Err(format!("'{field}' expected a dictionary")),
        }
    }

    pub fn as_list(&self, field: &str) -> Result<&Vec<Bencode>, String> {
        match self {
            Bencode::List(l) => Ok(l),
            _ => Err(format!("'{field}' expected a list")),
        }
    }

    pub fn as_u64(&self, field: &str) -> Result<u64, String> {
        match self {
            Bencode::Integer(n) => u64::try_from(*n).map_err(|_| format!("'{field}' must not be negative")),
            _ => Err(format!("'{field}' expected an integer")),
        }
    }

    pub fn as_string(&self, field: &str) -> Result<String, String> {
        match self {
            Bencode::ByteString(s) => {
                String::from_utf8(s.clone()).map_err(|_| format!("'{field}' is not valid UTF-8"))
            }
            _ => Err(format!("'{field}' expected a string")),
        }
    }

    pub fn as_bytes(&self, field: &str) -> Result<&[u8], String> {
        match self {
            Bencode::ByteString(s) => Ok(s),
            _ => Err(format!("'{field}' expected a byte string")),
        }
    }

    /// Convenience for dict lookup + presence check in one call.
    pub fn get_field<'a>(&'a self, key: &[u8], field: &str) -> Result<&'a Bencode, String> {
        self.as_dict(field)?
            .get(key)
            .ok_or_else(|| format!("'{field}' missing required key"))
    }
}