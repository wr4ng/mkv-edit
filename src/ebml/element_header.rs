use std::io::{Read, Seek};

use crate::ParseError;
use crate::ebml::vint::RawVint;

#[derive(Debug)]
pub struct ElementId {
    //TODO: Handle length > 4 for EBMLMaxIDLength > 4
    pub value: u32,
    pub length: u8, // 1–4 bytes
}

impl ElementId {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Option<Self>, ParseError> {
        let Some(raw_vint) = RawVint::read_from(reader)? else {
            return Ok(None);
        };
        if raw_vint.length > 4 {
            dbg!(&raw_vint);
            return Err(ParseError::InvalidElementIdLength(raw_vint.length));
        }
        let value =
            u32::try_from(raw_vint.value).expect("raw_vint value fits in u32 since length <= 4");

        Ok(Some(Self {
            value,
            length: raw_vint.length,
        }))
    }
}

#[derive(Debug)]
pub struct ElementSize {
    pub value: u64,
    pub length: u8, // 1–8 bytes
}

impl ElementSize {
    const fn from_raw_vint(raw_vint: &RawVint) -> Self {
        // Clear VINT_MARKER bit
        // VINT_MARKER is at position 8 * length - length = 7 * length (8 bytes forward, length bits back)
        let masked_value = raw_vint.value & !(1 << (7 * raw_vint.length));
        Self {
            value: masked_value,
            length: raw_vint.length,
        }
    }
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Option<Self>, ParseError> {
        let Some(raw_vint) = RawVint::read_from(reader)? else {
            return Ok(None);
        };
        Ok(Some(Self::from_raw_vint(&raw_vint)))
    }
}

#[derive(Debug)]
pub struct ElementHeader {
    pub id: ElementId,
    pub data_size: ElementSize,
    pub header_size: u8, // 2 VINT's -> 2-16 bytes (usually 2-12 bytes)
}

impl ElementHeader {
    pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Option<Self>, ParseError> {
        let header_start = reader.stream_position()?;

        let id = match ElementId::read_from(reader) {
            Ok(Some(id)) => id,
            Ok(None) => return Ok(None),
            Err(ParseError::UnexpectedEOFInVINT) => {
                return Err(ParseError::UnexpectedEOFElementHeader);
            }
            Err(error) => return Err(error),
        };

        let Some(data_size) = ElementSize::read_from(reader)? else {
            return Err(ParseError::UnexpectedEOFElementHeader);
        };

        let header_end = reader.stream_position()?;
        let header_size = u8::try_from(header_end - header_start)
            .expect("Element Header consists of 2 VINTS, at max 16 bytes");

        Ok(Some(Self {
            id,
            data_size,
            header_size,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_element_id() {
        // Element Id's values are taken directly from raw_vint
        let cases = vec![
            (vec![0b1001_1100], 1, 0b1001_1100),
            (vec![0b0101_0001, 0x00], 2, 0x5100),
            (vec![0b0011_1110, 0xAC, 0x12], 3, 0x3EAC12),
            (vec![0b0001_1111, 0xFF, 0xEE, 0xDD], 4, 0x1FFFEEDD),
        ];

        for (input, expected_length, expected_value) in cases {
            let mut cursor = Cursor::new(&input);
            let element_id = ElementId::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(
                element_id.length, expected_length,
                "input {:#02x?}, expected length {}, got {}",
                input, expected_length, element_id.length
            );
            assert_eq!(
                element_id.value, expected_value,
                "input {:#02x?}, expected value {:#x}, got {:#x}",
                input, expected_value, element_id.value
            );
        }
    }

    #[test]
    fn test_read_element_id_invalid_length() {
        let data = vec![0b0000_1111, 0xFF, 0xEE, 0xDD, 0xCC]; // Length 5, invalid
        let mut cursor = Cursor::new(data);
        let result = ElementId::read_from(&mut cursor);
        assert!(matches!(result, Err(ParseError::InvalidElementIdLength(5))));
    }

    #[test]
    fn test_read_element_size() {
        let cases = vec![
            (vec![0b1001_1100], 1, 0b0001_1100),
            (vec![0b0101_0001, 0x00], 2, 0x1100),
            (vec![0b0011_1110, 0xAC, 0x12], 3, 0x1EAC12),
            (vec![0b0001_1111, 0xFF, 0xEE, 0xDD], 4, 0x0FFFEEDD),
        ];

        for (input, expected_length, expected_value) in cases {
            let mut cursor = Cursor::new(&input);
            let element_size = ElementSize::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(
                element_size.length, expected_length,
                "input {:#02x?}, expected length {}, got {}",
                input, expected_length, element_size.length
            );
            assert_eq!(
                element_size.value, expected_value,
                "input {:#02x?}, expected value {:#x}, got {:#x}",
                input, expected_value, element_size.value
            );
        }
    }

    #[test]
    fn test_read_element_header() {
        let header_bytes = vec![
            // Element ID (EBML Header ID)
            0x1A,
            0x45,
            0xDF,
            0xA3,
            // Element Size
            (0b1000_0000 + 15), // Size: 15 bytes
        ];

        let mut cursor = Cursor::new(&header_bytes);
        let element_header = ElementHeader::read_from(&mut cursor).unwrap().unwrap();
        assert_eq!(element_header.id.length, 4);
        assert_eq!(element_header.id.value, 0x1A45DFA3);
        assert_eq!(element_header.data_size.length, 1);
        assert_eq!(element_header.data_size.value, 15);
        assert_eq!(element_header.header_size, 5);
    }

    #[test]
    fn test_read_element_header_eof() {
        let header_bytes = vec![
            // Element ID (incomplete)
            0x1A, 0x45,
        ];

        let mut cursor = Cursor::new(&header_bytes);
        let result = ElementHeader::read_from(&mut cursor);
        assert!(matches!(
            result,
            Err(ParseError::UnexpectedEOFElementHeader)
        ));
    }

    #[test]
    fn test_read_element_header_empty() {
        let header_bytes = vec![];

        let mut cursor = Cursor::new(&header_bytes);
        let result = ElementHeader::read_from(&mut cursor);
        assert!(matches!(result, Ok(None)));
    }
}
