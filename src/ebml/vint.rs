use std::io::Read;

use crate::ebml::error;

fn leading_zeros_u8(byte: u8) -> u8 {
    // leading_zeros on u8 always returns a value between 0 and 8
    u8::try_from(byte.leading_zeros()).unwrap()
}

#[derive(Debug)]
pub struct VInt {
    pub length: u8,
    pub value: u64,
}

impl VInt {
    fn read_from<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
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

        Ok(VInt { length, value })
    }

    pub fn read_id<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
        //TODO: Validate ID length
        Self::read_from(reader)
    }

    pub fn read_size<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
        let raw_vint = Self::read_from(reader)?;
        // Clear VINT_MARKER bit
        // VINT_MARKER is the first set bit in the most significant byte (the length'th bit in most significant byte)
        // For length n, the VINT_MARKER is at bit position 8n - n = 7n
        let masked_value = raw_vint.value & !(1 << (7 * raw_vint.length));
        Ok(VInt {
            length: raw_vint.length,
            value: masked_value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ebml::error::EbmlError;

    #[test]
    fn test_leading_zeros_u8() {
        assert_eq!(leading_zeros_u8(0b0000_0000), 8);
        assert_eq!(leading_zeros_u8(0b0000_0001), 7);
        assert_eq!(leading_zeros_u8(0b0000_0010), 6);
        assert_eq!(leading_zeros_u8(0b0000_0100), 5);
        assert_eq!(leading_zeros_u8(0b0000_1000), 4);
        assert_eq!(leading_zeros_u8(0b0001_0000), 3);
        assert_eq!(leading_zeros_u8(0b0001_0000), 3);
        assert_eq!(leading_zeros_u8(0b0010_0000), 2);
        assert_eq!(leading_zeros_u8(0b0100_0000), 1);
        assert_eq!(leading_zeros_u8(0b1000_0000), 0);
    }

    #[test]
    fn test_read_vint() {
        let data = vec![0b1000_1010];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 1);
        assert_eq!(vint.value, 0b1000_1010);

        let data = vec![0b0100_0101, 0b1111_0000];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 2);
        assert_eq!(vint.value, 0b0100_0101_1111_0000);

        let data = vec![0b0010_1100, 0b0000_1111, 0b1010_1010];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VInt::read_from(&mut cursor).unwrap();
        assert_eq!(vint.length, 3);
        assert_eq!(vint.value, 0b0010_1100_0000_1111_1010_1010);
    }

    #[test]
    fn test_eof_vint() {
        let data = vec![0b0100_0101];
        let mut cursor = std::io::Cursor::new(data);
        let result = VInt::read_from(&mut cursor);
        assert!(matches!(result, Err(EbmlError::UnexpectedEof(_))));
    }

    #[test]
    fn test_invalid_vint() {
        let data = vec![0b0000_0000];
        let mut cursor = std::io::Cursor::new(data);
        let result = VInt::read_from(&mut cursor);
        assert!(matches!(result, Err(EbmlError::InvalidVint)));
    }

    #[test]
    fn test_read_size() {
        let data = vec![0b1000_1010];
        let mut cursor = std::io::Cursor::new(data);
        let vint = VInt::read_size(&mut cursor).unwrap();
        assert_eq!(vint.length, 1);
        assert_eq!(vint.value, 0b0000_1010);
    }
}
