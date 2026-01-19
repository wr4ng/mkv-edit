use std::fmt;
use std::fmt::Write;

use crate::ebml::reader::ParsedElement;
use crate::matroska::{EbmlHeader, Field, Info, MatroskaDocument, Segment};
use crate::util::tree_printer::{TreePrintable, TreePrinter};

fn element_label(name: &str, raw: &ParsedElement, show_bytes: bool) -> String {
    let mut label = name.to_string();
    if show_bytes {
        write!(
            label,
            " [bytes {}..{}]",
            raw.header.start,
            raw.data.start + raw.data.length
        )
        .unwrap();
    }
    label
}

fn field_label<T: fmt::Debug>(name: &str, field: &Field<T>, show_bytes: bool) -> Option<String> {
    let raw = field.raw.as_ref()?;
    let mut label = format!("{name}: {:?}", field.value);
    if show_bytes {
        write!(
            label,
            " [bytes {}..{}]",
            raw.header.start,
            raw.data.start + raw.data.length
        )
        .unwrap();
    }
    Some(label)
}

impl TreePrintable for EbmlHeader {
    fn print_tree(
        &self,
        out: &mut String,
        printer: &mut TreePrinter,
        last: bool,
        show_bytes: bool,
    ) -> fmt::Result {
        printer.node(
            out,
            last,
            element_label("EBML Header", &self.raw, show_bytes),
        )?;
        printer.child_scope(last, |printer| {
            let labels = vec![
                field_label("docType", &self.doctype, show_bytes),
                field_label("docTypeVersion", &self.doctype_version, show_bytes),
                field_label("docTypeReadVersion", &self.doctype_read_version, show_bytes),
                field_label("maxIDLength", &self.max_id_length, show_bytes),
                field_label("maxSizeLength", &self.max_size_length, show_bytes),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

            for (i, label) in labels.iter().enumerate() {
                let is_last = i == labels.len() - 1;
                printer.node(out, is_last, label)?;
            }

            Ok(())
        })
    }
}

impl TreePrintable for Segment {
    fn print_tree(
        &self,
        out: &mut String,
        printer: &mut TreePrinter,
        last: bool,
        show_bytes: bool,
    ) -> fmt::Result {
        printer.node(out, last, element_label("Segment", &self.raw, show_bytes))?;
        printer.child_scope(last, |printer| {
            self.info.print_tree(out, printer, true, show_bytes)?;
            Ok(())
        })?;
        Ok(())
    }
}

impl TreePrintable for Info {
    fn print_tree(
        &self,
        out: &mut String,
        printer: &mut TreePrinter,
        last: bool,
        show_bytes: bool,
    ) -> fmt::Result {
        printer.node(out, last, element_label("Info", &self.raw, show_bytes))?;
        Ok(())
    }
}

pub fn print_matroska_tree(doc: &MatroskaDocument, show_bytes: bool) -> Result<String, fmt::Error> {
    let mut out = String::new();
    let mut printer = TreePrinter::new();

    out.push_str("MatroskaDocument\n");
    doc.ebml_header
        .print_tree(&mut out, &mut printer, false, show_bytes)?;
    doc.segment
        .print_tree(&mut out, &mut printer, true, show_bytes)?;

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ebml::element::Element, matroska};

    #[test]
    fn test_print_simple_tree() {
        let ebml_tree = Element::Root {
            children: vec![
                Element::Master {
                    id: matroska::EBML_HEADER_ID,
                    children: vec![
                        Element::Raw {
                            id: matroska::EBML_HEADER_DOCTYPE_ID,
                            data: b"matroska".to_vec(),
                        },
                        Element::Raw {
                            id: matroska::EBML_HEADER_MAX_ID_LENGTH_ID,
                            data: vec![0x04],
                        },
                    ],
                },
                Element::Master {
                    id: matroska::SEGMENT_ID,
                    children: vec![Element::Master {
                        id: matroska::INFO_ID,
                        children: vec![],
                    }],
                },
            ],
        };

        let bytes = ebml_tree.to_bytes().unwrap();
        let cursor = std::io::Cursor::new(bytes);
        let matroska_doc = MatroskaDocument::parse_from(cursor).unwrap();

        let tree_string = print_matroska_tree(&matroska_doc, false).unwrap();

        assert_eq!(
            "MatroskaDocument
├── EBML Header
│   ├── docType: \"matroska\"
│   └── maxIDLength: 4
└── Segment
    └── Info
",
            tree_string
        );
    }
}
