use std::{env, fs::File};

use mkvedit::matroska::{MatroskaDocument, print_matroska_tree};

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = File::open(&args[1]).unwrap();
    let matroska_doc = MatroskaDocument::parse_from(file).unwrap();
    println!("{}", print_matroska_tree(&matroska_doc, true).unwrap());
    println!("{}", print_matroska_tree(&matroska_doc, false).unwrap());
}
