use core::fmt;
use std::io::{Read, Seek, SeekFrom};

use crate::ebml::element::{EbmlId, EbmlSize};
use crate::ebml::error::EbmlError;

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

    fn read_id(&mut self) -> Result<EbmlId, EbmlError> {
        EbmlId::read_from(&mut self.reader)
    }

    fn read_size(&mut self) -> Result<EbmlSize, EbmlError> {
        EbmlSize::read_from(&mut self.reader)
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

    fn at_eof(&mut self) -> Result<bool, EbmlError> {
        let pos = self.position()?;
        let mut buf = [0u8; 1];
        match self.reader.read(&mut buf) {
            Ok(0) => Ok(true),
            Ok(_) => {
                self.seek(pos)?;
                Ok(false)
            }
            Err(e) => Err(EbmlError::from(e)),
        }
    }

    pub fn read_range(&mut self, range: &ByteRange) -> Result<Vec<u8>, EbmlError> {
        //TODO: Maybe more appropriate error than InvalidVint
        let num_bytes = usize::try_from(range.length).map_err(|_| EbmlError::InvalidVint)?;
        let mut buf = vec![0u8; num_bytes];
        self.seek(range.start)?;
        self.reader.read_exact(&mut buf)?;
        Ok(buf)
    }
}

//TODO: Move to common?
#[derive(Debug, Clone)]
pub struct ByteRange {
    pub start: u64,
    pub length: u64,
}

#[derive(Clone)]
pub struct ParsedElement {
    pub id: u64,
    pub header: ByteRange,
    pub data: ByteRange,
    pub children: Option<Vec<ParsedElement>>,
}

impl fmt::Debug for ParsedElement {
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
) -> Result<ParsedElement, EbmlError> {
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
        let mut children = Vec::new();
        let end = data.start + data.length;

        while r.position()? < end {
            children.push(read_element::<S, R>(r)?);
        }
        return Ok(ParsedElement {
            id: id_vint.value,
            header,
            data,
            children: Some(children),
        });
    }
    r.seek(data.start + data.length)?;
    Ok(ParsedElement {
        id: id_vint.value,
        header,
        data,
        children: None,
    })
}

pub fn read_root<S: EbmlSchema, R: Read + Seek>(
    r: &mut EbmlReader<R>,
) -> Result<Vec<ParsedElement>, EbmlError> {
    let mut elements = Vec::new();
    while !r.at_eof()? {
        elements.push(read_element::<S, R>(r)?);
    }
    Ok(elements)
}
