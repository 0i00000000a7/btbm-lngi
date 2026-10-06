//! Display and parsing.

use super::types::{Column, DisplayType, Entry, Expr, ParseError};

/// Display in plain form; used as the cache key.
pub(crate) fn data_key(expr: &Expr) -> String {
    display(expr, DisplayType::Plain)
}

pub(crate) fn display(expr: &Expr, ty: DisplayType) -> String {
    let mut out = String::new();
    for col in expr {
        out.push_str(&display_column(col, ty));
    }
    out
}

fn display_column(col: &Column, ty: DisplayType) -> String {
    if col.is_empty() {
        return "(0)".to_string();
    }
    let inner: Vec<String> = col.iter().map(|entry| display_entry(entry, ty)).collect();
    format!("({})", inner.join(","))
}

fn display_entry(entry: &Entry, ty: DisplayType) -> String {
    let value = (entry.value + 1).to_string();
    if entry.height.is_empty() {
        return value;
    }
    let height = display(&entry.height, ty);
    match ty {
        DisplayType::Plain => format!("{value}^{height}"),
        DisplayType::Html => format!("{value}<sup>{height}</sup>"),
        DisplayType::Latex => format!("{value}^{{{height}}}"),
    }
}

pub(crate) fn display_marked(expr: &Expr, ty: DisplayType, start_index: i64) -> String {
    let mut out = String::new();
    let mut index = start_index;
    for col in expr {
        out.push_str(&display_column_marked(col, ty, index));
        index += 1;
    }
    out
}

fn display_column_marked(col: &Column, ty: DisplayType, index: i64) -> String {
    if col.is_empty() {
        return match ty {
            DisplayType::Plain => format!("(:{index})"),
            DisplayType::Html => {
                format!("(0)<sub><span style='color:#888'>{index}</span></sub>")
            }
            DisplayType::Latex => format!("(0)_{{\\color{{gray}}{index}}}"),
        };
    }
    let content: Vec<String> = col
        .iter()
        .map(|entry| display_entry_marked(entry, ty, index))
        .collect();
    let content = content.join(",");
    match ty {
        DisplayType::Plain => format!("({content}:{index})"),
        DisplayType::Html => {
            format!("({content})<sub><span style='color:#888'>{index}</span></sub>")
        }
        DisplayType::Latex => format!("({content})_{{\\color{{gray}}{index}}}"),
    }
}

fn display_entry_marked(entry: &Entry, ty: DisplayType, col_index: i64) -> String {
    let value = (entry.value + 1).to_string();
    if entry.height.is_empty() {
        return value;
    }
    let height = display_marked(&entry.height, ty, col_index + 1);
    match ty {
        DisplayType::Plain => format!("{value}^{height}"),
        DisplayType::Html => format!("{value}<sup>{height}</sup>"),
        DisplayType::Latex => format!("{value}^{{{height}}}"),
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
