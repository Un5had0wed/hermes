mod info;

use crate::bencode::Bencode;
use info::{Info};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct Torrent {
    pub announce: Option<String>,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub info: Info,
}

impl TryFrom<&Bencode> for Torrent {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let dict = value.as_dict("torrent")?;

        let announce = dict
            .get(b"announce".as_slice())
            .map(|b| b.as_string("announce"))
            .transpose()?;

        let announce_list = match dict.get(b"announce-list".as_slice()) {
            Some(b) => {
                let tiers = b.as_list("announce-list")?;
                let mut result = Vec::with_capacity(tiers.len());
                for (i, tier) in tiers.iter().enumerate() {
                    let urls = tier
                        .as_list("announce-list tier")?
                        .iter()
                        .map(|u| u.as_string("announce-list url"))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| format!("announce-list[{i}]: {e}"))?;
                    result.push(urls);
                }
                Some(result)
            }
            None => None,
        };

        let info_value = dict.get(b"info".as_slice()).ok_or("missing 'info' dict")?;
        let info = Info::try_from(info_value)?;

        Ok(Torrent { announce, announce_list, info })
    }
}

impl Torrent {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("failed to read file: {e}"))?;
        let value = Bencode::parse(&bytes)?;
        Torrent::try_from(&value)
    }
}