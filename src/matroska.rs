use std::io::{Read, Seek};

use thiserror::Error;

use crate::ebml::{
    self, EbmlReader, EbmlSchema,
    error::EbmlError,
    reader::{ByteRange, ParsedElement},
};

mod print_tree;
pub use print_tree::print_matroska_tree;

const EBML_HEADER_ID: u64 = 0x1A45_DFA3;
const EBML_HEADER_DOCTYPE_ID: u64 = 0x4282;
const EBML_HEADER_DOCTYPE_VERSION_ID: u64 = 0x4287;
const EBML_HEADER_DOCTYPE_READ_VERSION_ID: u64 = 0x4285;
const EBML_HEADER_MAX_ID_LENGTH_ID: u64 = 0x42F2;
const EBML_HEADER_MAX_SIZE_LENGTH_ID: u64 = 0x42F3;

pub struct MatroskaSchema;

impl EbmlSchema for MatroskaSchema {
    fn is_master(id: u64) -> bool {
        matches!(id, EBML_HEADER_ID)
    }
}

#[derive(Error, Debug)]
pub enum MatroskaParseError {
    #[error("EBML header missing")]
    MissingEbmlHeader,

    #[error("invalid EBML header: {0}")]
    InvalidEbmlHeader(&'static str),

    #[error("value error: {0}")]
    ValueError(#[from] ValueError),

    #[error("EBML error: {0}")]
    EbmlError(#[from] EbmlError),
}

pub trait MatroskaElement {
    const ID: u64;
    fn parse<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError>
    where
        Self: Sized;
}

pub struct MatroskaReader<R: Read + Seek> {
    ebml_reader: EbmlReader<R>,
}

impl<R: Read + Seek> MatroskaReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            ebml_reader: EbmlReader::new(reader),
        }
    }

    fn read_range(&mut self, range: &ByteRange) -> Result<Vec<u8>, MatroskaParseError> {
        self.ebml_reader
            .read_range(range)
            .map_err(MatroskaParseError::from)
    }
}

#[derive(Debug)]
pub struct Field<T> {
    pub raw: Option<ParsedElement>,
    pub value: T,
}

impl<T> Field<T> {
    pub fn new(value: T) -> Self {
        Self { raw: None, value }
    }
}

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("invalid UTF-8 string")]
    InvalidUTF8,

    #[error("invalid integer length: {0}")]
    InvalidLength(usize),
}

fn parse_string(bytes: Vec<u8>) -> Result<String, ValueError> {
    String::from_utf8(bytes).map_err(|_| ValueError::InvalidUTF8)
}

fn parse_u64(bytes: Vec<u8>) -> Result<u64, ValueError> {
    //TODO: Handle zero length
    if bytes.len() > 8 {
        return Err(ValueError::InvalidLength(bytes.len()));
    }
    let mut value: u64 = 0;
    for byte in bytes {
        value = (value << 8) | u64::from(byte);
    }
    Ok(value)
}

fn parse_field<T, R: Read + Seek>(
    reader: &mut MatroskaReader<R>,
    raw: &ParsedElement,
    parse_func: impl Fn(Vec<u8>) -> Result<T, ValueError>,
) -> Result<Field<T>, MatroskaParseError> {
    //TODO: Handle empty data (default value?)
    let bytes = reader.read_range(&raw.data)?;
    Ok(Field {
        raw: Some(raw.clone()),
        value: parse_func(bytes)?,
    })
}

#[derive(Debug)]
pub struct EbmlHeader {
    pub raw: ParsedElement,
    pub doctype: Field<String>,
    pub doctype_version: Field<u64>,
    pub doctype_read_version: Field<u64>,
    pub max_id_length: Field<u64>,
    pub max_size_length: Field<u64>,
}

impl MatroskaElement for EbmlHeader {
    const ID: u64 = EBML_HEADER_ID;

    fn parse<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError> {
        assert!(raw.id == Self::ID, "trying to parse invalid element");

        let mut doctype = None;
        let mut doctype_version = None;
        let mut doctype_read_version = None;
        let mut max_id_length = None;
        let mut max_size_length = None;

        for child in raw.children.as_deref().unwrap_or(&[]) {
            match child.id {
                EBML_HEADER_DOCTYPE_ID => {
                    doctype = Some(parse_field(reader, child, parse_string)?);
                }
                EBML_HEADER_DOCTYPE_VERSION_ID => {
                    doctype_version = Some(parse_field(reader, child, parse_u64)?);
                }
                EBML_HEADER_DOCTYPE_READ_VERSION_ID => {
                    doctype_read_version = Some(parse_field(reader, child, parse_u64)?);
                }
                EBML_HEADER_MAX_ID_LENGTH_ID => {
                    max_id_length = Some(parse_field(reader, child, parse_u64)?);
                }
                EBML_HEADER_MAX_SIZE_LENGTH_ID => {
                    max_size_length = Some(parse_field(reader, child, parse_u64)?);
                }
                _ => println!("Warning: unhandled EBML Header child ID {:X}", child.id),
            }
        }

        let doctype = doctype.ok_or(MatroskaParseError::InvalidEbmlHeader("missing docType"))?;
        let doctype_version = doctype_version.unwrap_or(Field::new(1));
        let doctype_read_version = doctype_read_version.unwrap_or(Field::new(1));
        let max_id_length = max_id_length.unwrap_or(Field::new(4));
        let max_size_length = max_size_length.unwrap_or(Field::new(8));

        // Validate Matroska EBML constraints
        if doctype.value != "matroska" {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "docType is not matroska",
            ));
        }
        if max_id_length.value != 4 {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "maxIDLength is not 4",
            ));
        }
        if max_size_length.value != 8 {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "maxSizeLength is not 8",
            ));
        }

        Ok(Self {
            raw: raw.clone(),
            doctype,
            doctype_version,
            doctype_read_version,
            max_id_length,
            max_size_length,
        })
    }
}

#[derive(Debug)]
pub struct MatroskaDocument {
    pub ebml_header: EbmlHeader,
}

impl MatroskaDocument {
    pub fn parse_from<R: Read + Seek>(reader: R) -> Result<Self, MatroskaParseError> {
        let mut matroska_reader = MatroskaReader::new(reader);
        let root = ebml::read_root::<MatroskaSchema, _>(&mut matroska_reader.ebml_reader)?;

        if root.is_empty() {
            return Err(MatroskaParseError::MissingEbmlHeader);
        }

        if root[0].id != EBML_HEADER_ID {
            return Err(MatroskaParseError::MissingEbmlHeader);
        }

        let ebml_header = EbmlHeader::parse(&mut matroska_reader, &root[0])?;
        Ok(Self { ebml_header })
    }
}
