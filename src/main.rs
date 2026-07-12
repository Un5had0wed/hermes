// 

use std::env;
use std::process;

use hermes::torrent::Torrent;

fn main() {
    let path = match env::args().nth(1) {
        Some(p) => p,
        None => {
            eprintln!("usage: hermes <path-to-torrent-file>");
            process::exit(1);
        }
    };

    match Torrent::load_file(&path) {
        Ok(torrent) => println!("{:#?}", torrent.metainfo),
        Err(e) => {
            eprintln!("failed to load torrent: {e}");
            process::exit(1);
        }
    }
}