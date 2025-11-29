use std::fmt;

pub struct NodeMeta {
    pub header_start: u64,
    pub data_start: u64,
    pub id: u32,
    pub data_size: u64,
}

// Custom Debug impl to format id as hex
impl fmt::Debug for NodeMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NodeMeta")
            .field("id", &format_args!("{:#X}", self.id))
            .field("header_start", &self.header_start)
            .field("data_start", &self.data_start)
            .field("data_size", &self.data_size)
            .finish()
    }
}

#[derive(Debug)]
pub enum ParsedElement {
    // EBML Header
    EBMLHeader(Vec<Element>),
    // Segment
    // Info
    // Tags?
    Unkown,
}

#[derive(Debug)]
pub struct Element {
    pub meta: NodeMeta,
    pub parsed: ParsedElement,
    // pub children: Vec<Node>,
}
