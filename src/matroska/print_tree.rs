use std::fmt::{self, Write};

use crate::matroska::{EbmlHeader, Field, MatroskaDocument};

fn indent(out: &mut String, depth: usize) {
    for _ in 1..depth {
        out.push_str("│   ");
    }
}

fn branch(out: &mut String, last: bool) {
    if last {
        out.push_str("└── ");
    } else {
        out.push_str("├── ");
    }
}

fn print_field<T: fmt::Debug>(
    out: &mut String,
    depth: usize,
    last: bool,
    name: &str,
    field: &Field<T>,
    show_raw: bool,
) {
    indent(out, depth);
    branch(out, last);

    write!(out, "{}: {:?}", name, field.value).unwrap();

    match &field.raw {
        Some(raw) => {
            if show_raw {
                write!(
                    out,
                    " [bytes {}..{}]",
                    raw.header.start,
                    raw.data.start + raw.data.length
                )
                .unwrap();
            }
        }
        None => {
            write!(out, " (default)").unwrap();
        }
    }

    out.push('\n');
}

fn print_ebml_header(
    out: &mut String,
    depth: usize,
    last: bool,
    ebml: &EbmlHeader,
    show_raw: bool,
) {
    indent(out, depth);
    branch(out, last);
    if show_raw {
        write!(
            out,
            "EBML Header [bytes {}..{}]\n",
            ebml.raw.header.start,
            ebml.raw.data.start + ebml.raw.data.length
        )
        .unwrap();
    } else {
        out.push_str("EBML Header\n");
    }

    print_field(out, depth + 1, false, "docType", &ebml.doctype, show_raw);
    print_field(
        out,
        depth + 1,
        false,
        "docTypeVersion",
        &ebml.doctype_version,
        show_raw,
    );
    print_field(
        out,
        depth + 1,
        false,
        "docTypeReadVersion",
        &ebml.doctype_read_version,
        show_raw,
    );
    print_field(
        out,
        depth + 1,
        false,
        "maxIDLength",
        &ebml.max_id_length,
        show_raw,
    );
    print_field(
        out,
        depth + 1,
        true,
        "maxSizeLength",
        &ebml.max_size_length,
        show_raw,
    );
}

pub fn print_matroska_tree(doc: &MatroskaDocument, show_bytes: bool) -> String {
    let mut out = String::new();

    out.push_str("MatroskaDocument\n");

    print_ebml_header(&mut out, 1, false, &doc.ebml_header, show_bytes);

    // Extend later:
    // print_segment(&mut out, 1, true, &doc.segment, show_raw);

    out
}
