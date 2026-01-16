use std::io::Read;

use crate::ebml::error;

fn leading_zeros_u8(byte: u8) -> u8 {
    // safe: leading_zeros on u8 always returns a value between 0 and 8
    u8::try_from(byte.leading_zeros()).unwrap()
}

pub struct VariableInt {
    pub value: u64,
    pub length: u8,
}

impl VariableInt {
    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
        let mut buf = [0u8; 1];
        reader
            .read_exact(&mut buf)
            .map_err(|error| error::map_eof_error(error, "reading VINT leading byte"))?;

        let leading_zeros = leading_zeros_u8(buf[0]);
        let length = leading_zeros + 1;

        if length > 8 || length == 0 {
            return Err(error::EbmlError::InvalidVint);
        }

        let mut value = u64::from(buf[0]);
        if length > 1 {
            let mut buffer = vec![0u8; usize::from(length - 1)];
            reader
                .read_exact(&mut buffer)
                .map_err(|error| error::map_eof_error(error, "reading VINT bytes"))?;
            for byte in buffer {
                value = (value << 8) | u64::from(byte);
            }
        }

        Ok(VariableInt { value, length })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebml::error::EbmlError;

    #[test]
    fn test_leading_zeros_u8() {
        for i in 0..8 {
            let value = 1u8 << i;
            assert_eq!(leading_zeros_u8(value), 7 - i);
        }
        assert_eq!(leading_zeros_u8(0), 8);
    }

    #[test]
    fn test_vint_read() {
        let data = vec![0b1000_1010];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VariableInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 1);
        assert_eq!(vint.value, 0b1000_1010);

        let data = vec![0b0100_0101, 0b1111_0000];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VariableInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 2);
        assert_eq!(vint.value, 0b0100_0101_1111_0000);

        let data = vec![0b0010_1100, 0b0000_1111, 0b1010_1010];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VariableInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 3);
        assert_eq!(vint.value, 0b0010_1100_0000_1111_1010_1010);
    }

    #[test]
    fn test_vint_eof() {
        let data = vec![0b0100_0101];
        let mut cursor = std::io::Cursor::new(data);
        let result = VariableInt::read_from(&mut cursor);
        assert!(matches!(result, Err(EbmlError::UnexpectedEof(_))));
    }

    #[test]
    fn test_vint_invalid() {
        let data = vec![0b0000_0000];
        let mut cursor = std::io::Cursor::new(data);
        let result = VariableInt::read_from(&mut cursor);
        assert!(matches!(result, Err(EbmlError::InvalidVint)));
    }
}
