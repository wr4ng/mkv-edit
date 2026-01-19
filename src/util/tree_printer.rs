use std::fmt;
use std::fmt::Write;

pub struct TreePrinter {
    ancestors: Vec<bool>,
}

impl TreePrinter {
    pub fn new() -> Self {
        Self {
            ancestors: Vec::new(),
        }
    }

    fn indent(&self, out: &mut String) {
        for &is_last in &self.ancestors {
            if is_last {
                out.push_str("    ");
            } else {
                out.push_str("│   ");
            }
        }
    }

    pub fn node(&self, out: &mut String, last: bool, label: impl fmt::Display) -> fmt::Result {
        self.indent(out);
        if last {
            out.push_str("└── ");
        } else {
            out.push_str("├── ");
        }
        writeln!(out, "{label}")
    }

    pub fn child_scope<F>(&mut self, last: bool, f: F) -> fmt::Result
    where
        F: FnOnce(&mut Self) -> fmt::Result,
    {
        self.ancestors.push(last);
        let res = f(self);
        self.ancestors.pop();
        res
    }
}

pub trait TreePrintable {
    fn print_tree(
        &self,
        out: &mut String,
        printer: &mut TreePrinter,
        last: bool,
        show_bytes: bool,
    ) -> fmt::Result;
}
