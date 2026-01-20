use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("invalid UTF-8 string")]
    InvalidUTF8,

    #[error("invalid integer length: {0}")]
    InvalidLength(usize),
}

pub fn parse_string(bytes: Vec<u8>) -> Result<String, ValueError> {
    String::from_utf8(bytes).map_err(|_| ValueError::InvalidUTF8)
}

pub fn parse_u64(bytes: Vec<u8>) -> Result<u64, ValueError> {
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
