use crate::bencode::Bencode;
use super::helpers::{get, expect_u64, expect_string, expect_list, expect_dict, expect_bytes};

use sha1::{Digest, Sha1};

#[derive(Debug, Clone, PartialEq)]
pub struct MetaInfo {
    pub announce: Option<String>,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub info: Info,
    pub info_hash: [u8; 20],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub name: String,
    pub piece_length: u64,
    pub pieces: Vec<[u8; 20]>,
    pub private: bool,

    /* Either single or multi-file entry */

    /* Single file entry */
    pub length: Option<u64>,
    pub md5sum: Option<String>,

    /* Multi-file entry */
    pub files: Option<Vec<FileEntry>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub length: u64,
    pub path: Vec<String>,
    pub md5sum: Option<String>,
}

impl TryFrom<&Bencode> for MetaInfo {
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

        Ok(MetaInfo { announce, announce_list, info, info_hash })
    }
}

impl TryFrom<&Bencode> for Info {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let dict = expect_dict(value, "info")?;

        let name = expect_string(
            get(dict, b"name").ok_or("info.name missing")?,
            "info.name",
        )?;

        let piece_length = expect_u64(
            get(dict, b"piece length").ok_or("info.piece length missing")?,
            "info.piece length",
        )?;
        if piece_length == 0 {
            return Err("info.piece length must be positive".to_string());
        }

        let raw_pieces = expect_bytes(
            get(dict, b"pieces").ok_or("info.pieces missing")?,
            "info.pieces",
        )?;
        if raw_pieces.len() % 20 != 0 {
            return Err("info.pieces length is not a multiple of 20".to_string());
        }
        let pieces = raw_pieces
            .chunks_exact(20)
            .map(|c| {
                let mut hash = [0u8; 20];
                hash.copy_from_slice(c);
                hash
            })
            .collect();

        let private = match get(dict, b"private") {
            Some(b) => expect_u64(b, "info.private")? != 0,
            None => false,
        };

        let length = get(dict, b"length")
            .map(|b| expect_u64(b, "info.length"))
            .transpose()?;

        let md5sum = get(dict, b"md5sum")
            .map(|b| expect_string(b, "info.md5sum"))
            .transpose()?;

        let files = match get(dict, b"files") {
            Some(b) => {
                let files_list = expect_list(b, "info.files")?;
                let parsed = files_list
                    .iter()
                    .enumerate()
                    .map(|(i, entry)| {
                        FileEntry::try_from(entry).map_err(|e| format!("info.files[{i}]: {e}"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Some(parsed)
            }
            None => None,
        };

        // Enforce mutual exclusivity here, since the flat struct can't.
        match (&length, &files) {
            (Some(_), Some(_)) => return Err("info has both 'length' and 'files'".to_string()),
            (None, None) => return Err("info has neither 'length' nor 'files'".to_string()),
            _ => {}
        }

        Ok(Info { name, piece_length, pieces, private, length, md5sum, files })
    }
}

impl TryFrom<&Bencode> for FileEntry {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let dict = expect_dict(value, "files[]")?;

        let length = expect_u64(
            get(dict, b"length").ok_or("files[].length missing")?,
            "files[].length",
        )?;

        let path_list = expect_list(
            get(dict, b"path").ok_or("files[].path missing")?,
            "files[].path",
        )?;
        if path_list.is_empty() {
            return Err("files[].path must not be empty".to_string());
        }
        let path = path_list
            .iter()
            .map(|p| expect_string(p, "files[].path segment"))
            .collect::<Result<Vec<_>, _>>()?;

        let md5sum = get(dict, b"md5sum")
            .map(|b| expect_string(b, "files[].md5sum"))
            .transpose()?;

        Ok(FileEntry { length, path, md5sum })
    }
}
