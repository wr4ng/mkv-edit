use std::io::{Read, Seek};

use thiserror::Error;

use crate::ebml::{
    self, EbmlReader, EbmlSchema,
    error::EbmlError,
    primitives::{ValueError, parse_string, parse_u64},
    reader::{ByteRange, ParsedElement},
};

mod printer;
pub use printer::print_matroska_tree;

pub const EBML_HEADER_ID: u64 = 0x1A45_DFA3;
pub const EBML_HEADER_DOCTYPE_ID: u64 = 0x4282;
pub const EBML_HEADER_DOCTYPE_VERSION_ID: u64 = 0x4287;
pub const EBML_HEADER_DOCTYPE_READ_VERSION_ID: u64 = 0x4285;
pub const EBML_HEADER_MAX_ID_LENGTH_ID: u64 = 0x42F2;
pub const EBML_HEADER_MAX_SIZE_LENGTH_ID: u64 = 0x42F3;

pub const SEGMENT_ID: u64 = 0x1853_8067;
pub const INFO_ID: u64 = 0x1549_A966;

pub struct MatroskaSchema;

impl EbmlSchema for MatroskaSchema {
    fn is_master(id: u64) -> bool {
        matches!(id, EBML_HEADER_ID | SEGMENT_ID)
    }
}

#[derive(Error, Debug)]
pub enum MatroskaParseError {
    #[error("EBML header missing")]
    MissingEbmlHeader,

    #[error("invalid EBML header: {0}")]
    InvalidEbmlHeader(&'static str),

    #[error("missing required element: {0}")]
    MissingElement(&'static str),

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
    pub raw: ParsedElement,
    pub value: T,
}

impl<T> Field<T> {
    fn parse<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
        parse_func: impl Fn(Vec<u8>) -> Result<T, ValueError>,
    ) -> Result<Self, MatroskaParseError> {
        //TODO: Handle raw.data.lenth == 0 (default value)
        let bytes = reader.read_range(&raw.data)?;
        Ok(Self {
            raw: raw.clone(),
            value: parse_func(bytes)?,
        })
    }
}

impl Field<String> {
    pub fn parse_string<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError> {
        Self::parse(reader, raw, parse_string)
    }
}

impl Field<u64> {
    pub fn parse_u64<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError> {
        Self::parse(reader, raw, parse_u64)
    }
}

#[derive(Debug)]
pub enum OptionalField<T> {
    Present(Field<T>),
    Default(T),
}

impl<T: Copy> OptionalField<T> {
    pub fn value(&self) -> T {
        match self {
            OptionalField::Present(field) => field.value,
            OptionalField::Default(value) => *value,
        }
    }

    pub fn new_default(value: T) -> Self {
        OptionalField::Default(value)
    }

    pub fn new_or_default(field: Option<Field<T>>, default: T) -> Self {
        match field {
            Some(f) => OptionalField::Present(f),
            None => OptionalField::Default(default),
        }
    }
}

#[derive(Debug)]
pub struct EbmlHeader {
    pub raw: ParsedElement,
    pub doctype: Field<String>,
    pub doctype_version: OptionalField<u64>,
    pub doctype_read_version: OptionalField<u64>,
    pub max_id_length: OptionalField<u64>,
    pub max_size_length: OptionalField<u64>,
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
                    doctype = Some(Field::parse_string(reader, child)?);
                }
                EBML_HEADER_DOCTYPE_VERSION_ID => {
                    doctype_version = Some(Field::parse_u64(reader, child)?);
                }
                EBML_HEADER_DOCTYPE_READ_VERSION_ID => {
                    doctype_read_version = Some(Field::parse_u64(reader, child)?);
                }
                EBML_HEADER_MAX_ID_LENGTH_ID => {
                    max_id_length = Some(Field::parse_u64(reader, child)?);
                }
                EBML_HEADER_MAX_SIZE_LENGTH_ID => {
                    max_size_length = Some(Field::parse_u64(reader, child)?);
                }
                _ => println!("Warning: unhandled EBML Header child ID {:X}", child.id),
            }
        }

        let doctype = doctype.ok_or(MatroskaParseError::InvalidEbmlHeader("missing docType"))?;
        let doctype_version = OptionalField::new_or_default(doctype_version, 1);
        let doctype_read_version = OptionalField::new_or_default(doctype_read_version, 1);
        let max_id_length = OptionalField::new_or_default(max_id_length, 4);
        let max_size_length = OptionalField::new_or_default(max_size_length, 8);

        // Validate Matroska EBML constraints
        if doctype.value != "matroska" {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "docType is not matroska",
            ));
        }
        if max_id_length.value() != 4 {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "maxIDLength is not 4",
            ));
        }
        if max_size_length.value() != 8 {
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
pub struct Segment {
    pub raw: ParsedElement,
    pub info: Info,
}

impl MatroskaElement for Segment {
    const ID: u64 = SEGMENT_ID;

    fn parse<R: Read + Seek>(
        reader: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError> {
        assert!(raw.id == Self::ID, "trying to parse invalid element");

        let mut info = None;

        for child in raw.children.as_deref().unwrap_or(&[]) {
            match child.id {
                INFO_ID => {
                    info = Some(Info::parse(reader, child)?);
                }
                _ => println!("Warning: unhandled Segment child ID {:X}", child.id),
            }
        }

        let info = info.ok_or(MatroskaParseError::MissingElement("Info"))?;

        Ok(Self {
            raw: raw.clone(),
            info,
        })
    }
}

#[derive(Debug)]
pub struct Info {
    pub raw: ParsedElement,
}

impl MatroskaElement for Info {
    const ID: u64 = INFO_ID;

    fn parse<R: Read + Seek>(
        _: &mut MatroskaReader<R>,
        raw: &ParsedElement,
    ) -> Result<Self, MatroskaParseError> {
        assert!(raw.id == Self::ID, "trying to parse invalid element");

        //TODO: Parse Info children
        Ok(Self { raw: raw.clone() })
    }
}

#[derive(Debug)]
pub struct MatroskaDocument {
    pub ebml_header: EbmlHeader,
    pub segment: Segment,
}

impl MatroskaDocument {
    pub fn parse_from<R: Read + Seek>(reader: R) -> Result<Self, MatroskaParseError> {
        let mut matroska_reader = MatroskaReader::new(reader);
        let root = ebml::read_root::<MatroskaSchema, _>(&mut matroska_reader.ebml_reader)?;

        if root.is_empty() {
            return Err(MatroskaParseError::MissingEbmlHeader);
        }

        if root.len() < 2 {
            return Err(MatroskaParseError::InvalidEbmlHeader(
                "missing Segment element",
            ));
        }

        if root[0].id != EBML_HEADER_ID {
            return Err(MatroskaParseError::MissingEbmlHeader);
        }

        let ebml_header = EbmlHeader::parse(&mut matroska_reader, &root[0])?;
        let segment = Segment::parse(&mut matroska_reader, &root[1])?;

        Ok(Self {
            ebml_header,
            segment,
        })
    }
}
