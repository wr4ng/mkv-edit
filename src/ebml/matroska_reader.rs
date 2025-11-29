use std::io::SeekFrom;
use std::io::{Read, Seek};

use crate::ebml::element::ParsedElement;
use crate::ebml::matroska_ids;
use crate::Element;
use crate::ElementHeader;
use crate::NodeMeta;
use crate::ParseError;

pub struct MatroskaReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub fn new(reader: R) -> Self {
        MatroskaReader { reader }
    }

    // TODO: Update struct?
    pub fn read_header(&mut self) -> Result<Option<(ElementHeader, u64)>, ParseError> {
        let header_start = self.reader.stream_position()?;
        ElementHeader::read_from(&mut self.reader)
            .map(|result| result.map(|header| (header, header_start)))
    }

    /// Skip forward size bytes
    pub fn skip_payload(&mut self, size: u64) -> Result<(), ParseError> {
        //TODO: Need to handle seeking larger than 2^32 ahead?
        self.reader.seek(SeekFrom::Current(size as i64))?;
        Ok(())
    }

    pub fn parse_elements(&mut self) -> Result<Vec<Element>, ParseError> {
        let mut found = Vec::new();

        while let Some((header, header_start)) = self.read_header()? {
            let data_start = header_start + u64::from(header.header_size);
            let meta = NodeMeta {
                id: header.id.value,
                header_start,
                data_start,
                data_size: header.data_size.value,
            };

            let parsed = match meta.id {
                matroska_ids::EBML_HEADER_ID => {
                    ParsedElement::EBMLHeader(parse_ebml_header_children(self, &meta)?)
                }
                // matroska_ids::SEGMENT_ID => ParsedElement::Unkown,
                _ => {
                    self.skip_payload(meta.data_size)?;
                    ParsedElement::Unkown
                }
            };

            let element = Element { meta, parsed };
            found.push(element);
        }

        return Ok(found);
    }
}

pub fn parse_ebml_header_children<R: Read + Seek>(
    reader: &mut MatroskaReader<R>,
    meta: &NodeMeta,
) -> Result<Vec<Element>, ParseError> {
    // reader.skip_payload(meta.data_size)?;

    // If there's no data, return empty vec
    if reader.reader.stream_position()? >= meta.data_start + meta.data_size {
        return Ok(Vec::new());
    }

    let mut children = Vec::new();

    while let Some((header, header_start)) = reader.read_header()? {
        let data_start = header_start + u64::from(header.header_size);
        let child_meta = NodeMeta {
            id: header.id.value,
            header_start,
            data_start,
            data_size: header.data_size.value,
        };

        let child_parsed = match child_meta.id {
            _ => {
                // For now, just skip payloads of children
                reader.skip_payload(child_meta.data_size)?;
                ParsedElement::Unkown
            }
        };

        let child_element = Element {
            meta: child_meta,
            parsed: child_parsed,
        };
        children.push(child_element);

        // Check if we've reached the end of the EBML Header data
        if reader.reader.stream_position()? >= meta.data_start + meta.data_size {
            break;
        }
    }

    Ok(children)
}
