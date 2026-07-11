#[derive(Debug, Clone, PartialEq)]
pub struct Torrent {
    announce: String,
    announce_list: Option<Vec<Vec<String>>>,
    comment: Option<String>,
    created_by: Option<String>,
    creation_date: Option<u64>,
    encoding: Option<String>,
    info: Info
}

#[derive(Debug, Clone, PartialEq)]
pub struct Info {
    name: String,
    piece_length: u64,
    pieces: Vec<u8>,
    private: bool,
    mode: FileMode
}

#[derive(Debug, Clone, PartialEq)]
pub enum FileMode {
    Single {
        length: u64,
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
