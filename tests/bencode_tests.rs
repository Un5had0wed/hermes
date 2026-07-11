use hermes::bencode::Bencode;

use std::collections::BTreeMap;

#[test]
fn parses_integer() {
    assert_eq!(Bencode::parse(b"i42e"), Ok(Bencode::Integer(42)));
    assert_eq!(Bencode::parse(b"i-42e"), Ok(Bencode::Integer(-42)));
    assert_eq!(Bencode::parse(b"i0e"), Ok(Bencode::Integer(0)));
    assert!(Bencode::parse(b"i03e").is_err());
    assert!(Bencode::parse(b"i-0e").is_err());
}

#[test]
fn parses_string() {
    assert_eq!(
        Bencode::parse(b"4:spam"),
        Ok(Bencode::ByteString(b"spam".to_vec()))
    );
    assert!(Bencode::parse(b"5:spam").is_err());
}

#[test]
fn parses_list() {
    assert_eq!(
        Bencode::parse(b"l4:spam4:eggse"),
        Ok(Bencode::List(vec![
            Bencode::ByteString(b"spam".to_vec()),
            Bencode::ByteString(b"eggs".to_vec()),
        ]))
    );
}

#[test]
fn parses_dict() {
    let mut expected = BTreeMap::new();
    expected.insert("cow".to_string(), Bencode::ByteString(b"moo".to_vec()));
    expected.insert("spam".to_string(), Bencode::ByteString(b"eggs".to_vec()));
    assert_eq!(
        Bencode::parse(b"d3:cow3:moo4:spam4:eggse"),
        Ok(Bencode::Dict(expected))
    );
}

#[test]
fn rejects_unsorted_dict_keys() {
    assert!(Bencode::parse(b"d4:spam4:eggs3:cow3:mooe").is_err());
}

#[test]
fn parses_nested() {
    assert_eq!(
        Bencode::parse(b"l4:spaml1:a1:bee"),
        Ok(Bencode::List(vec![
            Bencode::ByteString(b"spam".to_vec()),
            Bencode::List(vec![
                Bencode::ByteString(b"a".to_vec()),
                Bencode::ByteString(b"b".to_vec()),
            ]),
        ]))
    );
}
