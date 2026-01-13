mod error;
pub mod reader;
mod vint;

pub use reader::EbmlReader;
pub use reader::EbmlSchema;
pub use reader::{read_element, read_root};
