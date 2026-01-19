use std::io::{self, Read};

use crate::ebml::{error, vint::VariableInt};

fn leading_zeros_u64(value: u64) -> u8 {
    // leading_zeros on u64 always returns a value between 0 and 64
    u8::try_from(value.leading_zeros()).unwrap()
}

// Represents an EBML ID
// Wrapper around VINT with ID-specific semantics
// `EbmlId.value` includes the `VINT_MARKER` bit
pub struct EbmlId {
    pub value: u64,
    pub length: u8,
}

//TODO: Handle unknown sizes (all bits set to 1)
pub struct EbmlSize {
    pub value: u64,
    pub length: u8,
}

impl EbmlId {
    // Returns the number of bytes to represent `value` as an `EbmlId`
    // Assumes `value` is a valid EBML ID and includes the `VINT_MARKER` bit at correct position
    fn length_of(value: u64) -> u8 {
        (leading_zeros_u64(value) % 8) + 1
    }

    // Creates a new `EbmlId` from a u64 value
    // Assumes `value` is a valid EBML ID and includes the `VINT_MARKER` bit at correct position
    pub fn new(value: u64) -> Self {
        //TODO: Validate ID length
        let length = Self::length_of(value);
        EbmlId { value, length }
    }

    //TODO: Create zero-allocating version
    pub fn to_bytes(&self) -> Vec<u8> {
        self.value.to_be_bytes()[8 - usize::from(self.length)..].to_vec()
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
        //TODO: Validate ID length
        let vint = VariableInt::read_from(reader)?;
        Ok(EbmlId {
            value: vint.value,
            length: vint.length,
        })
    }
}

//TODO: Handle unknown sizes (all bits set to 1)
impl EbmlSize {
    // Returns the number of bytes to represent `value` as a VINT
    // VINT_MARKER for length n is at bit position 8n - n = 7n,
    // therefore (1 << 7n) - 1 is the maximum value representable with VINT of length n
    fn length_of(value: u64) -> u8 {
        for n in 1..=8 {
            // Calculate maximum value representable with VINT of length n
            // VINT_MARKER is at bit position 7n and we can represent all bits below that
            let max_value = (1 << (7 * n)) - 1;
            if value <= max_value {
                return n;
            }
        }
        //TODO: Determine proper error handling
        panic!("value too large to be represented as a VINT: {value}");
    }

    pub fn new(value: u64) -> Self {
        let length = Self::length_of(value);
        EbmlSize { value, length }
    }

    //TODO: Create zero-allocating version
    pub fn to_bytes(&self) -> Vec<u8> {
        let vint_value = self.value | (1 << (7 * self.length));
        vint_value.to_be_bytes()[8 - usize::from(self.length)..].to_vec()
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, error::EbmlError> {
        let vint = VariableInt::read_from(reader)?;
        // Clear VINT_MARKER bit
        // For length n, the VINT_MARKER is at bit position 8n - n = 7n
        let masked_value = vint.value & !(1 << (7 * vint.length));
        Ok(EbmlSize {
            value: masked_value,
            length: vint.length,
        })
    }
}

pub enum Element {
    Raw { id: u64, data: Vec<u8> },
    Master { id: u64, children: Vec<Element> },
    Root { children: Vec<Element> },
}

impl Element {
    //TODO: zero-alloc version
    pub fn to_bytes(&self) -> io::Result<Vec<u8>> {
        let mut buffer = Vec::new();

        match self {
            Element::Raw { id, data } => {
                let ebml_id = EbmlId::new(*id);
                let ebml_size = EbmlSize::new(data.len() as u64);

                buffer.extend(ebml_id.to_bytes());
                buffer.extend(ebml_size.to_bytes());
                buffer.extend(data);
            }
            Element::Master { id, children } => {
                let ebml_id = EbmlId::new(*id);
                let mut children_bytes = Vec::new();

                for child in children {
                    children_bytes.extend(child.to_bytes()?);
                }

                let ebml_size = EbmlSize::new(children_bytes.len() as u64);

                buffer.extend(ebml_id.to_bytes());
                buffer.extend(ebml_size.to_bytes());
                buffer.extend(children_bytes);
            }
            Element::Root { children } => {
                for child in children {
                    buffer.extend(child.to_bytes()?);
                }
            }
        }
        Ok(buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leading_zeros_u64() {
        for i in 0..64 {
            let value = 1u64 << i;
            assert_eq!(leading_zeros_u64(value), 63 - i);
        }
        assert_eq!(leading_zeros_u64(0), 64);
    }

    #[test]
    fn test_ebml_id_length_of() {
        let test_cases = vec![
            (0x1A45DFA3, 4),
            (0x82, 1),
            (0x4286, 2),
            (0x228681, 3),
            (0x12868101, 4),
        ];
        for (value, expected_length) in test_cases {
            let length = EbmlId::length_of(value);
            assert_eq!(length, expected_length);
        }
    }

    #[test]
    fn test_ebml_size_length_of() {
        let test_cases = vec![
            (0x7F, 1),
            (0x3FFF, 2),
            (0x1F_FFFF, 3),
            (0x0F_FF_FF_FF, 4),
            (0x07_FF_FF_FF_FF, 5),
            (0x03_FF_FF_FF_FF_FF, 6),
            (0x01_FF_FF_FF_FF_FF_FF, 7),
            (0x00_FF_FF_FF_FF_FF_FF_FF, 8),
        ];
        for (value, expected_length) in test_cases {
            let length = EbmlSize::length_of(value);
            assert_eq!(length, expected_length);
        }
    }

    #[test]
    fn test_ebml_id_to_bytes() {
        let id = EbmlId::new(0x1A45DFA3);
        let bytes = id.to_bytes();
        assert_eq!(bytes, vec![0x1A, 0x45, 0xDF, 0xA3]);
    }

    #[test]
    fn test_ebml_size_to_bytes() {
        let size = EbmlSize::new(0x3FFF);
        let bytes = size.to_bytes();
        assert_eq!(bytes, vec![0x7F, 0xFF]);
    }

    #[test]
    fn test_ebml_id_read_from() {
        let data = vec![0x1A, 0x45, 0xDF, 0xA3];
        let mut cursor = std::io::Cursor::new(data);
        let id = EbmlId::read_from(&mut cursor).unwrap();
        assert_eq!(id.value, 0x1A45DFA3);
        assert_eq!(id.length, 4);
    }

    #[test]
    fn test_ebml_size_read_from() {
        let data = vec![0x7F, 0xFF];
        let mut cursor = std::io::Cursor::new(data);
        let size = EbmlSize::read_from(&mut cursor).unwrap();
        assert_eq!(size.value, 0x3FFF);
        assert_eq!(size.length, 2);
    }
}
