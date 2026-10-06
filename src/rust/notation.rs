//! The notation object: cached FS plus a single entry point for the rest.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;

use super::compare::compare;
use super::display::{data_key, display, display_marked, from_display};
use super::fs::{expand, is_limit};
use super::layer::{convert_from_layer, convert_to_layer};
use super::types::{DisplayType, Expr, ParseError};

/// The BTBMS (Bubby3's Transfinite BMS) notation.
///
/// `fs` needs a memoization cache, so it lives on this struct; every other
/// operation is a pure function forwarded through methods.
#[derive(Default)]
pub struct Btbm {
    data: RefCell<HashMap<String, HashMap<i64, Expr>>>,
}

impl Btbm {
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the expression is a limit (last column non-empty).
    pub fn is_limit(&self, expr: &Expr) -> bool {
        is_limit(expr)
    }

    /// Compare two expressions.
    pub fn compare(&self, a: &Expr, b: &Expr) -> Ordering {
        compare(a, b)
    }

    /// The fundamental sequence `FS(expr, index)`.
    pub fn fs(&self, expr: &Expr, index: i64) -> Expr {
        if !is_limit(expr) {
            return expand(expr, 0);
        }
        let key = data_key(expr);
        if let Some(map) = self.data.borrow().get(&key) {
            if let Some(cached) = map.get(&index) {
                return cached.clone();
            }
        }
        let result = expand(expr, index);
        self.data
            .borrow_mut()
            .entry(key)
            .or_default()
            .insert(index, result.clone());
        result
    }

    /// Display the expression.
    pub fn display(&self, expr: &Expr, ty: DisplayType) -> String {
        display(expr, ty)
    }

    /// Display with column indices annotated.
    pub fn display_marked(&self, expr: &Expr, ty: DisplayType, start_index: i64) -> String {
        display_marked(expr, ty, start_index)
    }

    /// Parse an expression from its display string.
    pub fn from_display(&self, s: &str) -> Result<Expr, ParseError> {
        from_display(s)
    }

    /// Convert to the layer representation.
    pub fn convert_to_layer(&self, e: &Expr) -> Expr {
        convert_to_layer(e)
    }

    /// Restore from the layer representation.
    pub fn convert_from_layer(&self, e: &Expr) -> Expr {
        convert_from_layer(e)
    }
}
