use thiserror::Error;
use std::io;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid VINT length")]
    InvalidVINTLength,

    #[error("unexpected EOF in EBML VINT")]
    UnexpectedEOFInVINT,

    #[error("unexpected EOF in Element Data")]
    UnexpectedEOFInElementData,

    #[error("invalid Element ID")]
    InvalidElementId,
}

impl ParseError {
    pub fn map_io_vint(err: io::Error) -> ParseError {
        match err.kind() {
            io::ErrorKind::UnexpectedEof => ParseError::UnexpectedEOFInVINT,
            _ => ParseError::Io(err),
        }
    }
}
