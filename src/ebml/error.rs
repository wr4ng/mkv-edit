use std::io;
use thiserror::Error;

// TODO: Maybe include offset information in errors?
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid VINT: no VINT marker found")]
    NoVINTMarker,

    #[error("invalid Element ID length: {0}")]
    InvalidElementIdLength(u8),

    // TODO: Better error messages
    #[error("unexpected EOF")]
    UnexpectedEOF,

    #[error("unexpected EOF when reading VINT")]
    UnexpectedEOFInVINT,

    #[error("unexpected EOF in element header")]
    UnexpectedEOFElementHeader,
}
