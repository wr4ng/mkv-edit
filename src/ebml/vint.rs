use std::io::Read;

use crate::ebml::error::ParseError;

#[derive(Debug)]
pub struct VINT {
    pub value: u64,
    pub length: u8, // 1–8 bytes
}

pub fn read_vint<R: Read>(reader: &mut R) -> Result<VINT, ParseError> {
    // Read first byte to determine length
    let mut first = [0u8; 1];
    reader
        .read_exact(&mut first)
        .map_err(ParseError::map_io_vint)?;
    let first_byte = first[0];

    // Determine length by VINT_MARKER position
    let length = first_byte.leading_zeros() as u8 + 1;
    if length > 8 || length == 0 {
        return Err(ParseError::InvalidVINTLength);
    }

    // Strip length prefix up to VINT_MARKER position
    let mut value = match length {
        8 => 0x00,
        _ => (first_byte & (0xFF >> length)) as u64,
    };

    // Read remaining bytes
    for _ in 1..length {
        let mut b = [0u8; 1];
        reader.read_exact(&mut b).map_err(ParseError::map_io_vint)?;
        value = (value << 8) | (b[0] as u64);
    }

    Ok(VINT { value, length })
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    //TODO: Add more test-cases for different VINT lengths and edge cases
    #[test]
    fn test_read_vint() {
        let cases = vec![
            (vec![0b1001_1100], 1, 0b0001_1100),
            (vec![0b0001_0101, 0x00, 0x00, 0xFF], 4, 0x050000FF), // 0b0001_50000FF -> length=4, value=0x050000FF
            (vec![0b0111_1110, 0xAC], 2, 0x3EAC),
        ];

        for (input, expected_length, expected_value) in cases {
            let mut cursor = Cursor::new(&input);
            let vint = read_vint(&mut cursor).unwrap();
            assert_eq!(
                vint.length, expected_length,
                "input {:#02x?}, exptected length {} got {}",
                input, expected_length, vint.length
            );
            assert_eq!(
                vint.value, expected_value,
                "input {:#02x?}, expected {:#02x} got {:#02x}",
                input, expected_value, vint.value
            );
        }
    }

    #[test]
    fn test_invalid_vint_length() {
        // Invalid VINT (no leading 1)
        let data = vec![0x00];
        let mut cursor = Cursor::new(data);
        let result = read_vint(&mut cursor);
        assert!(matches!(result, Err(ParseError::InvalidVINTLength)));
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
            let result = read_vint(&mut cursor);
            assert!(
                matches!(result, Err(ParseError::UnexpectedEOFInVINT)),
                "input {:#02x?}, expected Err(ParseError::UnexpectedEOFInVINT) got {:?}",
                input,
                result
            );
        }
    }
}
