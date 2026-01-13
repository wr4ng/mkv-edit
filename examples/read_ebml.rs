use std::fs::File;

use mkvedit::ebml;

struct SimpleEbmlSchema;

impl ebml::EbmlSchema for SimpleEbmlSchema {
    // Only the EBML header is a master element in this simple schema
    fn is_master(id: u64) -> bool {
        id == 0x1A45DFA3
    }
}

fn main() {
    let file = File::open("sample.mkv").unwrap();
    let mut ebml_reader = ebml::EbmlReader::new(file);
    let root = ebml::read_root::<SimpleEbmlSchema, _>(&mut ebml_reader).unwrap();
    dbg!(root);
}
