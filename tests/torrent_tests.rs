use hermes::bencode::Bencode;
use hermes::torrent::{FileEntry, FileMode, Info, Torrent};
use std::collections::BTreeMap;

fn make_single_file_info() -> BTreeMap<String, Bencode> {
    let mut info = BTreeMap::new();
    info.insert("name".to_string(), Bencode::String("sample".to_string()));
    info.insert("piece length".to_string(), Bencode::Integer(65536));
    info.insert("pieces".to_string(), Bencode::String("abc".to_string()));
    info.insert("length".to_string(), Bencode::Integer(123));
    info
}

fn make_multi_file_info() -> BTreeMap<String, Bencode> {
    let mut info = make_single_file_info();
    let file_entry = Bencode::Dict(BTreeMap::from([
        ("length".to_string(), Bencode::Integer(10)),
        ("path".to_string(), Bencode::List(vec![
            Bencode::String("dir".to_string()),
            Bencode::String("file.txt".to_string()),
        ])),
    ]));
    info.insert("files".to_string(), Bencode::List(vec![file_entry]));
    info.remove("length");
    info
}

fn make_torrent_root(info: BTreeMap<String, Bencode>) -> Bencode {
    let mut root = BTreeMap::new();
    root.insert(
        "announce".to_string(),
        Bencode::String("http://tracker".to_string()),
    );
    root.insert("info".to_string(), Bencode::Dict(info));
    Bencode::Dict(root)
}

#[test]
fn parses_single_file_torrent() {
    let root = make_torrent_root(make_single_file_info());
    let torrent = Torrent::try_from(root).unwrap();

    assert_eq!(torrent.announce, "http://tracker");
    assert!(torrent.announce_list.is_none());
    assert_eq!(torrent.info.name, "sample");
    assert_eq!(torrent.info.piece_length, 65536);
    assert_eq!(torrent.info.pieces, b"abc".to_vec());

    match torrent.info.mode {
        FileMode::Single { length, md5sum } => {
            assert_eq!(length, 123);
            assert!(md5sum.is_none());
        }
        FileMode::Multi { .. } => panic!("expected single-file mode"),
    }
}

#[test]
fn parses_multi_file_torrent() {
    let root = make_torrent_root(make_multi_file_info());
    let torrent = Torrent::try_from(root).unwrap();

    match torrent.info.mode {
        FileMode::Multi { files } => {
            assert_eq!(files.len(), 1);
            assert_eq!(files[0].path, vec!["dir".to_string(), "file.txt".to_string()]);
            assert_eq!(files[0].length, 10);
        }
        FileMode::Single { .. } => panic!("expected multi-file mode"),
    }
}

#[test]
fn parses_optional_fields_and_announce_list() {
    let mut info = make_single_file_info();
    info.insert("md5sum".to_string(), Bencode::String("abc123".to_string()));
    info.insert("private".to_string(), Bencode::Integer(1));

    let mut root = BTreeMap::new();
    root.insert(
        "announce".to_string(),
        Bencode::String("http://tracker".to_string()),
    );
    root.insert(
        "announce-list".to_string(),
        Bencode::List(vec![
            Bencode::List(vec![
                Bencode::String("http://a".to_string()),
                Bencode::String("http://b".to_string()),
            ]),
            Bencode::List(vec![Bencode::String("http://c".to_string())]),
        ]),
    );
    root.insert("comment".to_string(), Bencode::String("hello".to_string()));
    root.insert("created by".to_string(), Bencode::String("me".to_string()));
    root.insert("creation date".to_string(), Bencode::Integer(42));
    root.insert("encoding".to_string(), Bencode::String("UTF-8".to_string()));
    root.insert("info".to_string(), Bencode::Dict(info));

    let torrent = Torrent::try_from(Bencode::Dict(root)).unwrap();

    assert_eq!(torrent.comment.as_deref(), Some("hello"));
    assert_eq!(torrent.created_by.as_deref(), Some("me"));
    assert_eq!(torrent.creation_date, Some(42));
    assert_eq!(torrent.encoding.as_deref(), Some("UTF-8"));
    assert_eq!(torrent.announce_list.as_ref().unwrap().len(), 2);
    assert_eq!(torrent.announce_list.as_ref().unwrap()[0][1], "http://b");
    assert_eq!(torrent.announce_list.as_ref().unwrap()[1][0], "http://c");

    match torrent.info.mode {
        FileMode::Single { length, md5sum } => {
            assert_eq!(length, 123);
            assert_eq!(md5sum.as_deref(), Some("abc123"));
        }
        FileMode::Multi { .. } => panic!("expected single-file mode"),
    }

    match torrent.info.private {
        Some(true) => {}
        _ => panic!("expected private flag to be true"),
    }
}

#[test]
fn rejects_invalid_top_level_shape() {
    let err = Torrent::try_from(Bencode::Integer(1)).unwrap_err();
    assert_eq!(err, "Torrent root must be a dict");
}

#[test]
fn rejects_invalid_announce_list_shape() {
    let mut root = BTreeMap::new();
    root.insert(
        "announce".to_string(),
        Bencode::String("http://tracker".to_string()),
    );
    root.insert("announce-list".to_string(), Bencode::String("bad".to_string()));
    root.insert(
        "info".to_string(),
        Bencode::Dict(make_single_file_info()),
    );

    let err = Torrent::try_from(Bencode::Dict(root)).unwrap_err();
    assert_eq!(err, "announce-list must be a list");
}

#[test]
fn rejects_invalid_announce_list_entry_shape() {
    let mut root = BTreeMap::new();
    root.insert(
        "announce".to_string(),
        Bencode::String("http://tracker".to_string()),
    );
    root.insert(
        "announce-list".to_string(),
        Bencode::List(vec![Bencode::String("not-a-tier".to_string())]),
    );
    root.insert(
        "info".to_string(),
        Bencode::Dict(make_single_file_info()),
    );

    let err = Torrent::try_from(Bencode::Dict(root)).unwrap_err();
    assert_eq!(err, "announce-list tiers must be lists");
}

#[test]
fn rejects_invalid_private_flag() {
    let mut info = make_single_file_info();
    info.insert("private".to_string(), Bencode::Integer(2));

    let err = Torrent::try_from(make_torrent_root(info)).unwrap_err();
    assert_eq!(err, "private must be 0 or 1");
}

#[test]
fn rejects_invalid_file_entry_path() {
    let mut info = make_single_file_info();
    let file_entry = Bencode::Dict(BTreeMap::from([
        ("length".to_string(), Bencode::Integer(10)),
        ("path".to_string(), Bencode::List(vec![Bencode::Integer(3)])),
    ]));
    info.insert("files".to_string(), Bencode::List(vec![file_entry]));
    info.remove("length");

    let err = Torrent::try_from(make_torrent_root(info)).unwrap_err();
    assert_eq!(err, "path segments must be strings");
}

#[test]
fn rejects_missing_required_info_fields() {
    let mut info = BTreeMap::new();
    info.insert("name".to_string(), Bencode::String("sample".to_string()));
    info.insert("pieces".to_string(), Bencode::String("abc".to_string()));

    let err = Torrent::try_from(make_torrent_root(info)).unwrap_err();
    assert_eq!(err, "Missing required key: piece length");
}

#[test]
fn can_convert_file_entry_directly() {
    let entry = Bencode::Dict(BTreeMap::from([
        ("length".to_string(), Bencode::Integer(7)),
        ("path".to_string(), Bencode::List(vec![
            Bencode::String("a".to_string()),
            Bencode::String("b".to_string()),
        ])),
    ]));

    let parsed = FileEntry::try_from(entry).unwrap();
    assert_eq!(parsed.length, 7);
    assert_eq!(parsed.path, vec!["a".to_string(), "b".to_string()]);
    assert!(parsed.md5sum.is_none());
}

#[test]
fn can_convert_info_directly() {
    let mut info = BTreeMap::new();
    info.insert("name".to_string(), Bencode::String("sample".to_string()));
    info.insert("piece length".to_string(), Bencode::Integer(64));
    info.insert("pieces".to_string(), Bencode::String("abc".to_string()));
    info.insert("length".to_string(), Bencode::Integer(11));

    let parsed = Info::try_from(Bencode::Dict(info)).unwrap();
    assert_eq!(parsed.name, "sample");
    assert_eq!(parsed.piece_length, 64);
    match parsed.mode {
        FileMode::Single { length, .. } => assert_eq!(length, 11),
        FileMode::Multi { .. } => panic!("expected single-file mode"),
    }
}
