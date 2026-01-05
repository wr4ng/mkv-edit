use std::io::{Read, Seek};

use crate::ParseError;

pub fn read_unsigned_integer<R: Read + Seek>(reader: &mut R, size: u64) -> Result<u64, ParseError> {
    let mut buf = vec![0u8; usize::try_from(size).unwrap()]; // TODO: unwrap()
    reader.read_exact(&mut buf)?;
    let mut value = 0u64;
    for b in buf {
        value = (value << 8) | u64::from(b);
    }
    Ok(value)
}

pub fn read_string<R: Read + Seek>(reader: &mut R, size: u64) -> Result<String, ParseError> {
    let mut buf = vec![0u8; usize::try_from(size).unwrap()]; // TODO: unwrap()
    reader.read_exact(&mut buf)?;
    let s = String::from_utf8(buf).map_err(|_| ParseError::UnexpectedEOF)?; // TODO: Better error
    if !s.is_ascii() {
        return Err(ParseError::UnexpectedEOF); // TODO: Better error
    }
    Ok(s)
}
