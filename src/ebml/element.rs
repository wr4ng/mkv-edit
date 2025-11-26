use super::vint::{self, VINT};
use super::error;
use std::io::Read;

pub struct ElementHeader {
    pub id: VINT,
    pub data_size: VINT,
    pub header_size: u8,
}

pub fn read_element_header<R: Read>(reader: &mut R) -> Result<ElementHeader, error::ParseError> {
    let id_vint = vint::read_vint(reader)?;
    let size_vint = vint::read_vint(reader)?;

    Ok(ElementHeader {
        id: id_vint,
        data_size: size_vint,
        header_size: 0, //TODO: Calculate actual header size
    })
}