use std::fs::File;

use mkv::ebml;

fn main() {
    let file: File = File::open("sample.mkv").expect("Failed to open file");
    // Try to read EBML header
    match ebml::read_element_header(&mut &file) {
        Ok(header) => {
            println!("EBML Element ID: {:X}, Size: {}", header.id.value, header.data_size.value);
        }
        Err(e) => {
            eprintln!("Error reading EBML header: {}", e);
        }
    }
}