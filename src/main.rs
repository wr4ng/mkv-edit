use std::fs::File;

use mkv;

fn main() {
    let file: File = File::open("sample.mkv").unwrap();
    let mut reader = mkv::MatroskaReader::new(file);
    let root = reader.parse_elements();
    println!("{:#?}", root);
}
