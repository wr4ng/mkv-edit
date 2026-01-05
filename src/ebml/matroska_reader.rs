use std::io::SeekFrom;
use std::io::{Read, Seek};

use crate::ebml_header::EbmlHeader;
use crate::ebml_header::parse_ebml_header;

use crate::ElementHeader;
use crate::ParseError;

#[derive(Debug)]
pub struct MatroskaDocument {
    pub ebml_header: EbmlHeader,
}

pub struct MatroskaReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }

    // TODO: Update struct?
    pub fn read_header(&mut self) -> Result<Option<(ElementHeader, u64)>, ParseError> {
        let header_start = self.reader.stream_position()?;
        ElementHeader::read_from(&mut self.reader)
            .map(|result| result.map(|header| (header, header_start)))
    }

    // Skip forward size bytes
    pub fn skip_payload(&mut self, size: u64) -> Result<(), ParseError> {
        //TODO: Need to handle seeking larger than 2^32 ahead?
        self.reader.seek(SeekFrom::Current(i64::try_from(size).unwrap()))?; // TODO: unwrap()
        Ok(())
    }

    pub fn position(&mut self) -> Result<u64, ParseError> {
        Ok(self.reader.stream_position()?)
    }

    pub fn parse_matroska_document(&mut self) -> Result<MatroskaDocument, ParseError> {
        let ebml_header = parse_ebml_header(self)?;
        Ok(MatroskaDocument { ebml_header })
    }

    pub fn read_unsigned_integer(&mut self, size: u64) -> Result<u64, ParseError> {
        crate::ebml::primitives::read_unsigned_integer(&mut self.reader, size)
    }

    pub fn read_string(&mut self, size: u64) -> Result<String, ParseError> {
        crate::ebml::primitives::read_string(&mut self.reader, size)
    }
}
