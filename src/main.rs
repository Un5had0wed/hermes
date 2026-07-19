use std::env;
use std::process;
use std::fs;
use hermes::bencode::Bencode;

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: hermes <path-to-torrent-file>");
            process::exit(1);
        }
    };

    let bytes = fs::read(path).map_err(|e| format!("failed to read file: {e}"));
    if let Err(e) = bytes {
        eprintln!("{e}");
        process::exit(1);
    }

    let value = Bencode::parse(&bytes.unwrap());
    if let Err(e) = value {
        eprintln!("failed to parse bencode: {e}");
        process::exit(1);
    } 

    let value = value.unwrap();
    print!("{value}");
}