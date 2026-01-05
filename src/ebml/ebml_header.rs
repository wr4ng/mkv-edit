use std::io::{Read, Seek};

use crate::{
    MatroskaReader, ParseError,
    ebml::{element::NodeMeta, matroska_ids},
};

#[derive(Debug)]
pub struct EbmlHeader {
    pub doctype: String,           // docType
    pub ebml_max_id_length: u64,   // EBMLMaxIDLength
    pub ebml_max_size_length: u64, // EBMLMaxSizeLength
}

pub fn parse_ebml_header<R: Read + Seek>(
    reader: &mut MatroskaReader<R>,
) -> Result<EbmlHeader, ParseError> {
    // Read EBML Header
    let Some((header, header_start)) = reader.read_header()? else {
        return Err(ParseError::UnexpectedEOF);
    };

    let data_start = header_start + u64::from(header.header_size);
    let meta = NodeMeta {
        id: header.id.value,
        header_start,
        data_start,
        data_size: header.data_size.value,
    };

    if meta.id != matroska_ids::EBML_HEADER_ID {
        //TODO: Better error
        return Err(ParseError::UnexpectedEOF);
    }

    // Read EBML Header children
    let mut doctype = String::new();
    let mut ebml_max_id_length = 4;
    let mut ebml_max_size_length = 8;

    while let Some((header, header_start)) = reader.read_header()? {
        let data_start = header_start + u64::from(header.header_size);
        let child_meta = NodeMeta {
            id: header.id.value,
            header_start,
            data_start,
            data_size: header.data_size.value,
        };

        match child_meta.id {
            matroska_ids::EBML_DOCTYPE_ID => {
                doctype = reader.read_string(child_meta.data_size)?;
            }
            matroska_ids::EBML_MAX_ID_LENGTH_ID => {
                ebml_max_id_length = reader.read_unsigned_integer(child_meta.data_size)?;
            }
            matroska_ids::EBML_MAX_SIZE_LENGTH_ID => {
                ebml_max_size_length = reader.read_unsigned_integer(child_meta.data_size)?;
            }
            _ => {
                //TODO: Handle unknown IDs
                println!("Unknown EBML Header child ID: {:#X}", child_meta.id);
                reader.skip_payload(child_meta.data_size)?;
            }
        }

        if reader.position()? >= meta.data_start + meta.data_size {
            if reader.position()? > meta.data_start + meta.data_size {
                println!(
                    "Warning: Read past EBML Header end: {} > {}",
                    reader.position()?,
                    meta.data_start + meta.data_size
                );
            }
            break;
        }
    }

    // Validate required fields
    if doctype != "matroska" {
        return Err(ParseError::InvalidMatroskaValue(format!(
            "docType={doctype}",
        )));
    }
    if ebml_max_id_length != 4 {
        return Err(ParseError::InvalidMatroskaValue(format!(
            "EBMLMaxIDLength={ebml_max_id_length}",
        )));
    }
    if ebml_max_size_length > 8 {
        return Err(ParseError::InvalidMatroskaValue(format!(
            "EBMLMaxSizeLength={ebml_max_size_length}",
        )));
    }

    Ok(EbmlHeader {
        doctype,
        ebml_max_id_length,
        ebml_max_size_length,
    })
}
