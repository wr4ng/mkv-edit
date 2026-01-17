use std::{env, fs::File};

use mkvedit::matroska::MatroskaDocument;

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = File::open(&args[1]).unwrap();
    let matroska_doc = MatroskaDocument::parse_from(file).unwrap();
    dbg!(matroska_doc);
}
