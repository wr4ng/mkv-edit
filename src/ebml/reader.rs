use core::fmt;
use std::io::{Read, Seek, SeekFrom};

use crate::ebml::error::EbmlError;
use crate::ebml::vint::VInt;

pub trait EbmlSchema {
    //TODO: Rename from master?
    fn is_master(id: u64) -> bool;
}

pub struct EbmlReader<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> EbmlReader<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    fn read_id(&mut self) -> Result<VInt, EbmlError> {
        VInt::read_id(&mut self.reader)
    }

    fn read_size(&mut self) -> Result<VInt, EbmlError> {
        VInt::read_size(&mut self.reader)
    }

    fn position(&mut self) -> Result<u64, EbmlError> {
        self.reader.stream_position().map_err(EbmlError::from)
    }

    fn seek(&mut self, pos: u64) -> Result<(), EbmlError> {
        self.reader
            .seek(SeekFrom::Start(pos))
            .map(|_| ())
            .map_err(EbmlError::from)
    }
}

#[derive(Debug)]
struct ByteRange {
    start: u64,
    length: u64,
}

pub struct Element {
    id: u64,
    header: ByteRange,
    data: ByteRange,
    children: Option<Vec<Element>>,
}

impl fmt::Debug for Element {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Element")
            .field("id", &format_args!("{:#x}", self.id))
            .field("header", &self.header)
            .field("data", &self.data)
            .field("children", &self.children)
            .finish()
    }
}

pub fn read_element<S: EbmlSchema, R: Read + Seek>(
    r: &mut EbmlReader<R>,
) -> Result<Element, EbmlError> {
    let header_start = r.position()?;
    let id_vint = r.read_id()?;
    let size_vint = r.read_size()?;
    let header_length = u64::from(id_vint.length + size_vint.length);

    let header = ByteRange {
        start: header_start,
        length: header_length,
    };

    let data_start = r.position()?;
    let data_length = size_vint.value;

    let data = ByteRange {
        start: data_start,
        length: data_length,
    };

    if S::is_master(id_vint.value) {
        // TODO: Placeholder for reading child elements
        let mut children = Vec::new();
        let end = data.start + data.length;

        while r.position()? < end {
            children.push(read_element::<S, R>(r)?);
        }
        return Ok(Element {
            id: id_vint.value,
            header,
            data,
            children: Some(children),
        });
    }
    r.seek(data.start + data.length)?;
    Ok(Element {
        id: id_vint.value,
        header,
        data,
        children: None,
    })
}
