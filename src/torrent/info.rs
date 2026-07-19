use crate::bencode::Bencode;

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

impl TryFrom<&Bencode> for Info {
    type Error = String;

    fn try_from(value: &Bencode) -> Result<Self, String> {
        let dict = value.as_dict("info")?;

        let name = dict
            .get(b"name".as_slice())
            .ok_or("info.name missing")?
            .as_string("info.name")?;

        let piece_length = dict
            .get(b"piece length".as_slice())
            .ok_or("info.piece length missing")?
            .as_u64("info.piece length")?;
        if piece_length == 0 {
            return Err("info.piece length must be positive".to_string());
        }

        let raw_pieces = dict
            .get(b"pieces".as_slice())
            .ok_or("info.pieces missing")?
            .as_bytes("info.pieces")?;
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

        let private = match dict.get(b"private".as_slice()) {
            Some(b) => b.as_u64("info.private")? != 0,
            None => false,
        };

        let length = dict
            .get(b"length".as_slice())
            .map(|b| b.as_u64("info.length"))
            .transpose()?;

        let md5sum = dict
            .get(b"md5sum".as_slice())
            .map(|b| b.as_string("info.md5sum"))
            .transpose()?;

        let files = match dict.get(b"files".as_slice()) {
            Some(b) => {
                let files_list = b.as_list("info.files")?;
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
        let dict = value.as_dict("files[]")?;

        let length = dict
            .get(b"length".as_slice())
            .ok_or("files[].length missing")?
            .as_u64("files[].length")?;

        let path_list = dict
            .get(b"path".as_slice())
            .ok_or("files[].path missing")?
            .as_list("files[].path")?;

        if path_list.is_empty() {
            return Err("files[].path must not be empty".to_string());
        }
        
        let path = path_list
            .iter()
            .map(|p| p.as_string("files[].path segment"))
            .collect::<Result<Vec<_>, _>>()?;

        let md5sum = dict
            .get(b"md5sum".as_slice())
            .map(|b| b.as_string("files[].md5sum"))
            .transpose()?;

        Ok(FileEntry { length, path, md5sum })
    }
}