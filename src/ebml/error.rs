use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EbmlError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid VINT encoding")]
    InvalidVint,

    #[error("unexpected EOF: {0}")]
    UnexpectedEof(&'static str),
}

pub fn map_eof_error(error: io::Error, field: &'static str) -> EbmlError {
    match error.kind() {
        io::ErrorKind::UnexpectedEof => EbmlError::UnexpectedEof(field),
        _ => EbmlError::Io(error),
    }
}
