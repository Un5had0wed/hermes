mod metainfo;
mod helpers;

use crate::bencode::Bencode;

use std::path::Path;
use std::fs;

pub struct Torrent {
    pub metainfo: metainfo::MetaInfo,
}

impl TryFrom<&Bencode> for Torrent {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let metainfo = metainfo::MetaInfo::try_from(value)?;
        Ok(Torrent { metainfo })
    }
}

impl Torrent {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let bytes = fs::read(path).map_err(|e| format!("failed to read file: {e}"))?;
        let value = Bencode::parse(&bytes)?;
        Torrent::try_from(&value)
    }
}