use super::Bencode;

pub(super) fn encode(value: &Bencode, out: &mut Vec<u8>) {
    match value {
        Bencode::Integer(n) => {
            out.push(b'i');
            out.extend(n.to_string().bytes());
            out.push(b'e');
        }
        Bencode::ByteString(s) => {
            out.extend(s.len().to_string().bytes());
            out.push(b':');
            out.extend_from_slice(s);
        }
        Bencode::List(items) => {
            out.push(b'l');
            for item in items {
                encode(item, out);
            }
            out.push(b'e');
        }
        Bencode::Dict(map) => {
            out.push(b'd');
            for (k, v) in map {
                encode(&Bencode::ByteString(k.clone()), out);
                encode(v, out);
            }
            out.push(b'e');
        }
    }
}