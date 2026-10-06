//! Core data types.

/// A full expression: a sequence of columns.
pub type Expr = Vec<Column>;

/// A column: a sequence of entries.
pub type Column = Vec<Entry>;

/// One entry in the matrix: a value plus an (possibly empty) nested expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub value: i64,
    pub height: Expr,
}

/// The output of `to_height` / the input of `from_height`.
pub(crate) type Height = Vec<HeightColumn>;
pub(crate) type HeightColumn = Vec<HeightEntry>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeightEntry {
    pub mark: bool,
    pub value: i64,
    pub height: Height,
}

/// Display format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayType {
    Plain,
    Html,
    Latex,
}

/// A `from_display` parse error.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub input: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Illegal input string: {}", self.input)
    }
}

impl std::error::Error for ParseError {}
