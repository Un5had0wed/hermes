use super::Bencode;
use std::fmt::{Display, Result, Formatter};

const MAX_INLINE_WIDTH: usize = 80;
const MAX_STRING_CHARS: usize = 100;
const MAX_BINARY_PREVIEW: usize = 8;

impl Display for Bencode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", pretty(self, 0))
    }
}

fn pretty(value: &Bencode, indent: usize) -> String {
    match value {
        Bencode::Integer(_) | Bencode::ByteString(_) => inline(value),
        Bencode::List(_) | Bencode::Dict(_) => {
            let candidate = inline(value);
            if indent * 2 + candidate.len() <= MAX_INLINE_WIDTH {
                candidate
            } else {
                multiline(value, indent)
            }
        }
    }
}

fn inline(value: &Bencode) -> String {
    match value {
        Bencode::Integer(n) => n.to_string(),
        Bencode::ByteString(s) => format_bytes(s),
        Bencode::List(items) => {
            let parts: Vec<String> = items.iter().map(inline).collect();
            format!("[{}]", parts.join(", "))
        }
        Bencode::Dict(map) => {
            let parts: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", format_bytes(k), inline(v)))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
    }
}

fn multiline(value: &Bencode, indent: usize) -> String {
    let pad = "  ".repeat(indent);
    let pad_inner = "  ".repeat(indent + 1);

    match value {
        Bencode::List(items) => {
            if items.is_empty() {
                return "[]".to_string();
            }
            let mut out = String::from("[\n");
            for (i, item) in items.iter().enumerate() {
                out.push_str(&pad_inner);
                out.push_str(&pretty(item, indent + 1));
                if i + 1 != items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push(']');
            out
        }
        Bencode::Dict(map) => {
            if map.is_empty() {
                return "{}".to_string();
            }
            let len = map.len();
            let mut out = String::from("{\n");
            for (i, (k, v)) in map.iter().enumerate() {
                out.push_str(&pad_inner);
                out.push_str(&format_bytes(k));
                out.push_str(": ");
                out.push_str(&pretty(v, indent + 1));
                if i + 1 != len {
                    out.push(',');
                }
                out.push('\n');
            }
            out.push_str(&pad);
            out.push('}');
            out
        }
        other => inline(other),
    }
}

fn format_bytes(bytes: &[u8]) -> String {
    if let Ok(s) = std::str::from_utf8(bytes) {
        if is_printable(s) {
            if s.chars().count() > MAX_STRING_CHARS {
                let truncated: String = s.chars().take(MAX_STRING_CHARS).collect();
                return format!("{truncated:?}..."); // {:?} gives quoting + escaping
            }
            return format!("{s:?}");
        }
    }
    format_binary(bytes)
}

fn is_printable(s: &str) -> bool {
    s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t')
}

fn format_binary(bytes: &[u8]) -> String {
    if bytes.len() <= MAX_BINARY_PREVIEW {
        format!("<{}>", hex(bytes))
    } else {
        format!("<{} bytes, {}...>", bytes.len(), hex(&bytes[..MAX_BINARY_PREVIEW]))
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(" ")
}