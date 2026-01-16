use std::io::Cursor;

use mkvedit::ebml;
use mkvedit::ebml::EbmlReader;
use mkvedit::ebml::element::Element;

struct SimpleEbmlSchema;

impl ebml::EbmlSchema for SimpleEbmlSchema {
    fn is_master(id: u64) -> bool {
        id == 0x1A45DFA3 // EBML Header
    }
}

fn main() {
    let ebml_tree = Element::Master {
        id: 0x1A45DFA3, // EBML Header
        children: vec![
            Element::Raw {
                id: 0x4286,       // EBML Version
                data: vec![0x01], // Version 1
            },
            Element::Raw {
                id: 0x42F7,       // EBML Read Version
                data: vec![0x01], // Read version 1
            },
            Element::Raw {
                id: 0x42F2,       // EBML Max ID Length
                data: vec![0x04], // Max ID length 4 bytes
            },
        ],
    };

    let bytes = ebml_tree.to_bytes().unwrap();
    println!("Bytes = {:0X?}", &bytes);

    let cursor = Cursor::new(bytes);
    let mut ebml_reader = EbmlReader::new(cursor);
    let root = ebml::read_root::<SimpleEbmlSchema, _>(&mut ebml_reader).unwrap();
    dbg!(root);
}
