use std::io::{self, Read};

use crate::ebml::error::ParseError;

fn leading_zeros_u8(byte: u8) -> u8 {
    u8::try_from(byte.leading_zeros()).expect("leading 0's of u8 cannot exceed u8 (0-255)")
}

#[derive(Debug)]
pub struct RawVint {
    pub value: u64,
    pub length: u8, // 1–8 bytes
}

impl RawVint {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Option<Self>, ParseError> {
        let mut first_byte = [0u8; 1];
        let n = reader.read(&mut first_byte)?;
        if n == 0 {
            return Ok(None); // EOF reached
        }

        // Determine length by VINT_MARKER position
        let first_byte = first_byte[0];
        let length = leading_zeros_u8(first_byte) + 1;
        if length > 8 || length == 0 {
            return Err(ParseError::NoVINTMarker);
        }

        // Read VINT_DATA bytes
        let mut value = u64::from(first_byte);
        if length > 1 {
            let mut buffer = vec![0u8; (length - 1) as usize];
            reader.read_exact(&mut buffer).map_err(|e| match e.kind() {
                io::ErrorKind::UnexpectedEof => ParseError::UnexpectedEOFInVINT,
                _ => ParseError::Io(e),
            })?;
            for b in buffer {
                value = (value << 8) | u64::from(b);
            }
        }

        Ok(Some(Self { value, length }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_raw_vint() {
        let cases = vec![
            (vec![0x83], 1, 0x83),
            (vec![0x4F, 0xAB], 2, 0x4FAB),
            (vec![0x1A, 0x13, 0xBB, 0x00], 4, 0x1A13BB00),
            (
                vec![0x01, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0x00],
                8,
                0x0100FF00FF00FF00,
            ),
        ];

        for (input, expected_length, expected_value) in cases {
            let mut cursor = Cursor::new(&input);
            let raw_vint = RawVint::read_from(&mut cursor).unwrap().unwrap();
            assert_eq!(
                raw_vint.length, expected_length,
                "input {:#02x?}, expected length {}, got {}",
                input, expected_length, raw_vint.length
            );
            assert_eq!(
                raw_vint.value, expected_value,
                "input {:#02x?}, expected value {:#x}, got {:#x}",
                input, expected_value, raw_vint.value
            );
        }
    }

    #[test]
    fn test_rawint_eof() {
        let data = vec![];
        let mut cursor = Cursor::new(data);
        let result = RawVint::read_from(&mut cursor);
        assert!(matches!(result, Ok(None)));
    }

    #[test]
    fn test_no_vint_marker() {
        let data = vec![0x00]; // Invalid VINT, no VINT_MARKER in first byte
        let mut cursor = Cursor::new(data);
        let result = RawVint::read_from(&mut cursor);
        dbg!(&result);
        assert!(matches!(result, Err(ParseError::NoVINTMarker)));
    }

    #[test]
    fn test_unexpected_eof() {
        let cases = vec![
            vec![0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // Length = 8 but only 7 bytes provided
            vec![0x08, 0x00, 0x00],                         // Length = 5 but only 3 bytes provided
            vec![0x10, 0x00],                               // Length = 4 but only 2 bytes provided
            vec![0x20, 0x00],                               // Length = 3 but only 2 bytes provided
        ];

        for input in cases {
            let mut cursor = Cursor::new(&input);
            let result = RawVint::read_from(&mut cursor);
            assert!(
                matches!(result, Err(ParseError::UnexpectedEOFInVINT)),
                "input {:#02x?}, expected Err(ParseError::UnexpectedEOFInVINT) got {:?}",
                input,
                result
            );
        }
    }
}
