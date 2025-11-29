#![warn(clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_panics_doc, clippy::missing_errors_doc)]

//TODO: Rename module and/or split
mod ebml;

use ebml::element::Element;
use ebml::element::NodeMeta;
use ebml::element_header::ElementHeader;

pub use ebml::error::ParseError;
pub use ebml::matroska_reader::MatroskaReader;
