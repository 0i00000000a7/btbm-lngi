//! Display and parsing.

use std::fmt::Write;

use super::types::{Column, DisplayType, Entry, Expr, ParseError};

// Everything writes into one buffer. The old version built a fresh `String`
// for every nested height and then copied it into the enclosing `format!`, so
// a deeply nested ordinal copied its own lower levels over and over: a 3.7 KB
// output allocated 5 MB. Writing in place keeps the work linear in the output.

pub(crate) fn display(expr: &Expr, ty: DisplayType) -> String {
    let mut out = String::new();
    write_expr(&mut out, expr, ty);
    out
}

/// Like [`display`], but returns one string per top-level column, so the caller
/// can lay the columns out as separate, wrappable pieces (a single KaTeX run is
/// one unbreakable box).
pub(crate) fn display_parts(expr: &Expr, ty: DisplayType) -> Vec<String> {
    expr.iter()
        .map(|col| {
            let mut out = String::new();
            write_column(&mut out, col, ty);
            out
        })
        .collect()
}

fn write_expr(out: &mut String, expr: &Expr, ty: DisplayType) {
    for col in expr {
        write_column(out, col, ty);
    }
}

fn write_column(out: &mut String, col: &Column, ty: DisplayType) {
    if col.is_empty() {
        out.push_str("(0)");
        return;
    }
    out.push('(');
    for (i, entry) in col.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_entry(out, entry, ty);
    }
    out.push(')');
}

fn write_entry(out: &mut String, entry: &Entry, ty: DisplayType) {
    write!(out, "{}", entry.value + 1).unwrap();
    if entry.height.is_empty() {
        return;
    }
    match ty {
        DisplayType::Plain => {
            out.push('^');
            write_expr(out, &entry.height, ty);
        }
        DisplayType::Html => {
            out.push_str("<sup>");
            write_expr(out, &entry.height, ty);
            out.push_str("</sup>");
        }
        DisplayType::Latex => {
            out.push_str("^{");
            write_expr(out, &entry.height, ty);
            out.push('}');
        }
    }
}

pub(crate) fn display_marked(expr: &Expr, ty: DisplayType, start_index: i64) -> String {
    let mut out = String::new();
    write_expr_marked(&mut out, expr, ty, start_index);
    out
}

/// Like [`display_marked`], but returns one string per top-level column.
pub(crate) fn display_marked_parts(
    expr: &Expr,
    ty: DisplayType,
    start_index: i64,
) -> Vec<String> {
    expr.iter()
        .enumerate()
        .map(|(i, col)| {
            let mut out = String::new();
            write_column_marked(&mut out, col, ty, start_index + i as i64);
            out
        })
        .collect()
}

fn write_expr_marked(out: &mut String, expr: &Expr, ty: DisplayType, start_index: i64) {
    for (i, col) in expr.iter().enumerate() {
        write_column_marked(out, col, ty, start_index + i as i64);
    }
}

fn write_column_marked(out: &mut String, col: &Column, ty: DisplayType, index: i64) {
    out.push('(');
    if col.is_empty() {
        // An empty column prints as `(0)` in the marked modes, and as nothing
        // before the `:index` in the plain one.
        if ty != DisplayType::Plain {
            out.push('0');
        }
    } else {
        for (i, entry) in col.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            write_entry_marked(out, entry, ty, index);
        }
    }
    match ty {
        DisplayType::Plain => {
            write!(out, ":{index})").unwrap();
        }
        DisplayType::Html => {
            out.push_str(")<sub><span style='color:#888'>");
            write!(out, "{index}").unwrap();
            out.push_str("</span></sub>");
        }
        DisplayType::Latex => {
            out.push_str(")_{\\textcolor{gray}{");
            write!(out, "{index}").unwrap();
            out.push_str("}}");
        }
    }
}

fn write_entry_marked(out: &mut String, entry: &Entry, ty: DisplayType, col_index: i64) {
    write!(out, "{}", entry.value + 1).unwrap();
    if entry.height.is_empty() {
        return;
    }
    match ty {
        DisplayType::Plain => {
            out.push('^');
            write_expr_marked(out, &entry.height, ty, col_index + 1);
        }
        DisplayType::Html => {
            out.push_str("<sup>");
            write_expr_marked(out, &entry.height, ty, col_index + 1);
            out.push_str("</sup>");
        }
        DisplayType::Latex => {
            out.push_str("^{");
            write_expr_marked(out, &entry.height, ty, col_index + 1);
            out.push('}');
        }
    }
}

pub(crate) fn from_display(s: &str) -> Result<Expr, ParseError> {
    let mut parser = Parser {
        bytes: s.as_bytes(),
        pos: 0,
        src: s,
    };
    parser.skip_spaces();
    let result = parser.parse_expr()?;
    parser.skip_spaces();
    if parser.pos != parser.bytes.len() {
        return Err(parser.error());
    }
    Ok(result)
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
    src: &'a str,
}

impl Parser<'_> {
    fn error(&self) -> ParseError {
        ParseError {
            input: self.src.to_string(),
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek() == Some(b' ') {
            self.pos += 1;
        }
    }

    fn skip_index(&mut self) {
        if self.peek() == Some(b':') {
            self.pos += 1;
            self.skip_spaces();
            while self.peek().map_or(false, |b| b.is_ascii_digit()) {
                self.pos += 1;
            }
        }
    }

    fn parse_number(&mut self) -> Result<i64, ParseError> {
        self.skip_spaces();
        let start = self.pos;
        while self.peek().map_or(false, |b| b.is_ascii_digit()) {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.error());
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).expect("ascii digits");
        text.parse::<i64>().map_err(|_| self.error())
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let mut result: Expr = Vec::new();
        self.skip_spaces();
        while self.peek() == Some(b'(') {
            result.push(self.parse_column()?);
            self.skip_spaces();
        }
        Ok(result)
    }

    fn parse_column(&mut self) -> Result<Column, ParseError> {
        self.skip_spaces();
        if self.peek() != Some(b'(') {
            return Err(self.error());
        }
        self.pos += 1;

        let mut entries: Column = Vec::new();
        self.skip_spaces();
        if !matches!(self.peek(), Some(b')') | Some(b':') | None) {
            entries.push(self.parse_entry()?);
            self.skip_spaces();
            while self.peek() == Some(b',') {
                self.pos += 1;
                self.skip_spaces();
                if self.peek() == Some(b')') {
                    break;
                }
                entries.push(self.parse_entry()?);
                self.skip_spaces();
            }
        }

        self.skip_spaces();
        self.skip_index();
        self.skip_spaces();
        if self.peek() != Some(b')') {
            return Err(self.error());
        }
        self.pos += 1;

        // Drop trailing 0 entries in the column (displayed 0 = internal -1).
        while entries.last().map_or(false, |entry| entry.value == -1) {
            entries.pop();
        }
        Ok(entries)
    }

    fn parse_entry(&mut self) -> Result<Entry, ParseError> {
        let value = self.parse_number()? - 1; // displayed 1-based, internal 0-based
        self.skip_spaces();
        if self.peek() == Some(b'^') {
            self.pos += 1;
            return Ok(Entry {
                value,
                height: self.parse_expr()?,
            });
        }
        Ok(Entry {
            value,
            height: Vec::new(),
        })
    }
}

// ---------------------------------------------------------------------------
// Differential oracle
// ---------------------------------------------------------------------------

#[cfg(test)]
mod differential {
    //! The buffered writers must print exactly what the original recursive
    //! `format!` version printed. That version is kept here, test-only, so a
    //! future edit to the LaTeX escaping or the marked indices cannot quietly
    //! change what the panels show.

    use super::*;
    use crate::types::DisplayType as T;
    use crate::{family, Btbm, Lngi};

    fn ref_display(expr: &Expr, ty: DisplayType) -> String {
        ref_display_parts(expr, ty).join("")
    }

    fn ref_display_parts(expr: &Expr, ty: DisplayType) -> Vec<String> {
        expr.iter().map(|col| ref_display_column(col, ty)).collect()
    }

    fn ref_display_column(col: &Column, ty: DisplayType) -> String {
        if col.is_empty() {
            return "(0)".to_string();
        }
        let inner: Vec<String> = col.iter().map(|e| ref_display_entry(e, ty)).collect();
        format!("({})", inner.join(","))
    }

    fn ref_display_entry(entry: &Entry, ty: DisplayType) -> String {
        let value = (entry.value + 1).to_string();
        if entry.height.is_empty() {
            return value;
        }
        let height = ref_display(&entry.height, ty);
        match ty {
            T::Plain => format!("{value}^{height}"),
            T::Html => format!("{value}<sup>{height}</sup>"),
            T::Latex => format!("{value}^{{{height}}}"),
        }
    }

    fn ref_display_marked(expr: &Expr, ty: DisplayType, start_index: i64) -> String {
        ref_display_marked_parts(expr, ty, start_index).join("")
    }

    fn ref_display_marked_parts(expr: &Expr, ty: DisplayType, start_index: i64) -> Vec<String> {
        expr.iter()
            .enumerate()
            .map(|(i, col)| ref_display_column_marked(col, ty, start_index + i as i64))
            .collect()
    }

    fn ref_display_column_marked(col: &Column, ty: DisplayType, index: i64) -> String {
        if col.is_empty() {
            return match ty {
                T::Plain => format!("(:{index})"),
                T::Html => format!("(0)<sub><span style='color:#888'>{index}</span></sub>"),
                T::Latex => format!("(0)_{{\\textcolor{{gray}}{{{index}}}}}"),
            };
        }
        let content: Vec<String> = col
            .iter()
            .map(|e| ref_display_entry_marked(e, ty, index))
            .collect();
        let content = content.join(",");
        match ty {
            T::Plain => format!("({content}:{index})"),
            T::Html => format!("({content})<sub><span style='color:#888'>{index}</span></sub>"),
            T::Latex => format!("({content})_{{\\textcolor{{gray}}{{{index}}}}}"),
        }
    }

    fn ref_display_entry_marked(entry: &Entry, ty: DisplayType, col_index: i64) -> String {
        let value = (entry.value + 1).to_string();
        if entry.height.is_empty() {
            return value;
        }
        let height = ref_display_marked(&entry.height, ty, col_index + 1);
        match ty {
            T::Plain => format!("{value}^{height}"),
            T::Html => format!("{value}<sup>{height}</sup>"),
            T::Latex => format!("{value}^{{{height}}}"),
        }
    }

    fn corpus() -> Vec<Expr> {
        let n = Btbm::new();
        let mut out = Vec::new();
        for src in [
            "(0)",
            "(0)(1)",
            "(0)(1)(2)",
            "(0)(1)(2)(3)",
            "(0)(1,1)",
            "(0)(1^(2))",
            "(0)(1^(2))(1)",
            "(0)(1^(2,2))",
            "(0)(1^(2))(2^(3))",
            "(0)(1^(2^(3^(4))))",
        ] {
            if let Ok(e) = n.from_display(src) {
                out.push(e);
            }
        }
        for k in 1..9 {
            out.push(family(k));
        }
        // Real ordinals off the curve: deep enough to exercise the nesting that
        // made the old version allocate megabytes.
        let year_ms = 365.25 * 86_400_000.0;
        let mut l = Lngi::new();
        for years in [0.0, 1.0, 10.0, 100.0, 1000.0] {
            out.push(l.step(years * year_ms).expr);
        }
        out
    }

    #[test]
    fn buffered_display_matches_the_recursive_version() {
        // The oracle is the old recursive `format!` version, whose unoptimized
        // debug frames are large. A real ordinal nests ~570 levels deep, past
        // the default test-thread stack, so run the comparison on a roomier
        // one. The buffered writers under test have no such requirement.
        std::thread::Builder::new()
            .stack_size(64 * 1024 * 1024)
            .spawn(compare_all)
            .expect("spawn comparison thread")
            .join()
            .expect("comparison thread");
    }

    fn compare_all() {
        for e in corpus() {
            for ty in [T::Plain, T::Html, T::Latex] {
                assert_eq!(display(&e, ty), ref_display(&e, ty), "display, {ty:?}");
                assert_eq!(
                    display_parts(&e, ty),
                    ref_display_parts(&e, ty),
                    "display_parts, {ty:?}"
                );
                assert_eq!(
                    display_marked(&e, ty, 1),
                    ref_display_marked(&e, ty, 1),
                    "display_marked, {ty:?}"
                );
                assert_eq!(
                    display_marked_parts(&e, ty, 1),
                    ref_display_marked_parts(&e, ty, 1),
                    "display_marked_parts, {ty:?}"
                );
            }
        }
    }
}
