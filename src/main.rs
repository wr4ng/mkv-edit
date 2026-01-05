use std::fs::File;

fn main() {
    let file: File = File::open("sample.mkv").unwrap();
    let mut reader = mkv::MatroskaReader::new(file);
    let matroska_document = reader.parse_matroska_document().unwrap();
    dbg!(matroska_document);
}
