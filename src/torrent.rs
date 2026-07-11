use crate::bencode::Bencode;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Torrent {
    pub announce: String,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub comment: Option<String>,
    pub created_by: Option<String>,
    pub creation_date: Option<i64>,
    pub encoding: Option<String>,
    pub info: Info,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    pub name: String,
    pub piece_length: i64,
    pub pieces: Vec<u8>,
    pub private: Option<bool>,
    pub mode: FileMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    Single {
        length: i64,
        md5sum: Option<String>
    },
    Multi {
        files: Vec<FileEntry>
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FileEntry {
    pub length: i64,
    pub path: Vec<String>,
    pub md5sum: Option<String>,
}

impl TryFrom<Bencode> for Torrent {
    type Error = String;

    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let mut dict = match value {
            Bencode::Dict(d) => d,
            _ => return Err("Torrent root must be a dict".to_string()),
        };

        let announce = take_string(&mut dict, "announce")?;
        let announce_list = match dict.remove("announce-list") {
            Some(Bencode::List(tiers)) => Some(
                tiers
                    .into_iter()
                    .map(|tier| match tier {
                        Bencode::List(urls) => urls.into_iter().map(|u| match u {
                            Bencode::String(s) => Ok(s),
                            _ => Err("announce-list entries must be strings".to_string()),
                        }).collect::<Result<Vec<_>, _>>(),
                        _ => Err("announce-list tiers must be lists".to_string()),
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            ),
            Some(_) => return Err("announce-list must be a list".to_string()),
            None => None,
        };
        let comment = take_optional_string(&mut dict, "comment")?;
        let created_by = take_optional_string(&mut dict, "created by")?;
        let creation_date = take_optional_int(&mut dict, "creation date")?;
        let encoding = take_optional_string(&mut dict, "encoding")?;
 
        let info_value = dict
            .remove("info")
            .ok_or("Missing required key: info")?;
        let info = Info::try_from(info_value)?;
 
        Ok(Torrent {
            announce,
            announce_list,
            comment,
            created_by,
            creation_date,
            encoding,
            info,
        })
    }
}

impl TryFrom<Bencode> for Info {
    type Error = String;
 
    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let mut dict = match value {
            Bencode::Dict(d) => d,
            _ => return Err("info must be a dict".to_string()),
        };
 
        let name = take_string(&mut dict, "name")?;
        let piece_length = take_int(&mut dict, "piece length")?;
        let pieces = match dict.remove("pieces") {
            Some(Bencode::String(s)) => s.into_bytes(),
            _ => return Err("Missing or invalid required key: pieces".to_string()),
        };
        let private = match dict.remove("private") {
            Some(Bencode::Integer(0)) => Some(false),
            Some(Bencode::Integer(1)) => Some(true),
            Some(_) => return Err("private must be 0 or 1".to_string()),
            None => None,
        };
 
        let mode = if let Some(files_val) = dict.remove("files") {
            let files = match files_val {
                Bencode::List(items) => items
                    .into_iter()
                    .map(FileEntry::try_from)
                    .collect::<Result<Vec<_>, _>>()?,
                _ => return Err("files must be a list".to_string()),
            };
            FileMode::Multi { files }
        } else {
            let length = take_int(&mut dict, "length")?;
            let md5sum = take_optional_string(&mut dict, "md5sum")?;
            FileMode::Single { length, md5sum }
        };
 
        Ok(Info {
            name,
            piece_length,
            pieces,
            private,
            mode,
        })
    }
}
 
impl TryFrom<Bencode> for FileEntry {
    type Error = String;
 
    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let mut dict = match value {
            Bencode::Dict(d) => d,
            _ => return Err("file entry must be a dict".to_string()),
        };
 
        let length = take_int(&mut dict, "length")?;
        let path = match dict.remove("path") {
            Some(Bencode::List(segments)) => segments
                .into_iter()
                .map(|seg| match seg {
                    Bencode::String(s) => Ok(s),
                    _ => Err("path segments must be strings".to_string()),
                })
                .collect::<Result<Vec<_>, _>>()?,
            _ => return Err("Missing or invalid required key: path".to_string()),
        };
        let md5sum = take_optional_string(&mut dict, "md5sum")?;
 
        Ok(FileEntry {
            length,
            path,
            md5sum,
        })
    }
}
 
// --- small helpers to pull typed, required/optional fields out of a dict ---

fn take_value<T>(
    dict: &mut BTreeMap<String, Bencode>,
    key: &str,
    expected_type: &str,
    extractor: impl FnOnce(Bencode) -> Option<T>,
) -> Result<Option<T>, String> {
    dict.remove(key)
        .map(|b| extractor(b).ok_or_else(|| format!("{key} must be a {expected_type}")))
        .transpose()
}

fn take_optional_string(dict: &mut BTreeMap<String, Bencode>, key: &str) -> Result<Option<String>, String> {
    take_value(dict, key, "string", |b| if let Bencode::String(s) = b { Some(s) } else { None })
}

fn take_string(dict: &mut BTreeMap<String, Bencode>, key: &str) -> Result<String, String> {
    take_optional_string(dict, key)?.ok_or_else(|| format!("Missing required key: {key}"))
}

fn take_optional_int(dict: &mut BTreeMap<String, Bencode>, key: &str) -> Result<Option<i64>, String> {
    take_value(dict, key, "integer", |b| if let Bencode::Integer(i) = b { Some(i) } else { None })
}

fn take_int(dict: &mut BTreeMap<String, Bencode>, key: &str) -> Result<i64, String> {
    take_optional_int(dict, key)?.ok_or_else(|| format!("Missing required key: {key}"))
}
