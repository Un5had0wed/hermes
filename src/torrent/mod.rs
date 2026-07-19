mod helpers;
mod info;

use crate::bencode::Bencode;
use helpers::{get, expect_string, expect_list, expect_dict};
use info::{Info};
use sha1::{Digest, Sha1};
use std::path::Path;
use std::fs;

#[derive(Debug, Clone, PartialEq)]
pub struct Torrent {
    /* meta info */
    pub announce: Option<String>,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub info: Info,

    /* tracker info */
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
    pub port: u16,
}

impl TryFrom<&Bencode> for Torrent {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let dict = expect_dict(value, "torrent")?;

        let announce = get(dict, b"announce")
            .map(|b| expect_string(b, "announce"))
            .transpose()?;

        let announce_list = match get(dict, b"announce-list") {
            Some(b) => {
                let tiers = expect_list(b, "announce-list")?;
                let mut result = Vec::with_capacity(tiers.len());
                for (i, tier) in tiers.iter().enumerate() {
                    let urls = expect_list(tier, "announce-list tier")?
                        .iter()
                        .map(|u| expect_string(u, "announce-list url"))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(|e| format!("announce-list[{i}]: {e}"))?;
                    result.push(urls);
                }
                Some(result)
            }
            None => None,
        };

        let info_value = get(dict, b"info").ok_or("missing 'info' dict")?;
        let info = Info::try_from(info_value)?;

        let info_bytes = info_value.encode();
        let info_hash: [u8; 20] = Sha1::digest(&info_bytes).into();

        let peer_id = helpers::generate_peer_id();
        let port = helpers::get_port()?;

        Ok(Torrent { announce, announce_list, info, info_hash, peer_id, port })
    }
}

impl Torrent {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("failed to read file: {e}"))?;
        let value = Bencode::parse(&bytes)?;
        Torrent::try_from(&value)
    }
}