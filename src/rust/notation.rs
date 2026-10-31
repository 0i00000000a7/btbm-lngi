//! The notation object: cached FS plus a single entry point for the rest.

use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use super::compare::compare;
use super::display::{display, display_marked, from_display};
use super::fs::{expand, is_limit};
use super::layer::{convert_from_layer, convert_to_layer};
use super::types::{DisplayType, Expr, ParseError};

/// Only trim the cache once it holds more than this many results; trimming on
/// every step costs more than the memory it frees.
const CACHE_SOFT_LIMIT: usize = 512;

/// The structural hash of an expression, used as the cache's first-level key.
///
/// Hashing is the expensive part of a lookup (it walks the whole tree), so the
/// key is the hash itself rather than the expression: one caller that needs
/// several terms of the same limit can then pay for it once.
fn hash_expr(expr: &Expr) -> u64 {
    let mut hasher = DefaultHasher::new();
    expr.hash(&mut hasher);
    hasher.finish()
}

/// The BTBMS (Bubby3's Transfinite BMS) notation.
///
/// `fs` needs a memoization cache, so it lives on this struct; every other
/// operation is a pure function forwarded through methods.
#[derive(Default)]
pub struct Btbm {
    data: RefCell<HashMap<u64, Vec<(Expr, HashMap<i64, Expr>)>>>,
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

    /// The fundamental sequence `FS(expr, index)`, memoized.
    pub fn fs(&self, expr: &Expr, index: i64) -> Expr {
        if !is_limit(expr) {
            return expand(expr, 0);
        }
        let mut data = self.data.borrow_mut();
        let slot = Self::slot(&mut data, expr);
        Self::term(slot, expr, index)
    }

    /// `FS(expr, a)` and `FS(expr, b)`, resolved with a single hash of `expr`.
    ///
    /// The caller (the level decomposition) always needs two neighbouring terms
    /// of the same limit, and hashing that limit twice cost as much as the
    /// expansion it was saving.
    pub fn fs_pair(&self, expr: &Expr, a: i64, b: i64) -> (Expr, Expr) {
        if !is_limit(expr) {
            return (expand(expr, 0), expand(expr, 0));
        }
        let mut data = self.data.borrow_mut();
        let slot = Self::slot(&mut data, expr);
        let first = Self::term(slot, expr, a);
        let second = Self::term(slot, expr, b);
        (first, second)
    }

    /// The term map for `expr`, created if missing. Hashes `expr` once.
    fn slot<'a>(
        data: &'a mut HashMap<u64, Vec<(Expr, HashMap<i64, Expr>)>>,
        expr: &Expr,
    ) -> &'a mut HashMap<i64, Expr> {
        let bucket = data.entry(hash_expr(expr)).or_default();
        let idx = match bucket.iter().position(|(key, _)| key == expr) {
            Some(i) => i,
            None => {
                bucket.push((expr.clone(), HashMap::new()));
                bucket.len() - 1
            }
        };
        &mut bucket[idx].1
    }

    /// One term, computed and stored if missing.
    fn term(map: &mut HashMap<i64, Expr>, expr: &Expr, index: i64) -> Expr {
        if let Some(cached) = map.get(&index) {
            return cached.clone();
        }
        let result = expand(expr, index);
        map.insert(index, result.clone());
        result
    }

    /// Drop every memoized result below `floor`, once the cache has grown past
    /// [`CACHE_SOFT_LIMIT`].
    ///
    /// Time only moves forward, so an ordinal below the current one can never
    /// be asked for again — keeping it would only grow the map. Trimming on
    /// every step costs more than the memory it frees, hence the threshold.
    pub fn trim_below(&self, floor: &Expr) {
        let mut data = self.data.borrow_mut();
        let total: usize = data
            .values()
            .flat_map(|bucket| bucket.iter())
            .map(|(_, map)| map.len())
            .sum();
        if total <= CACHE_SOFT_LIMIT {
            return;
        }
        data.retain(|_, bucket| {
            bucket.retain_mut(|(_, map)| {
                map.retain(|_, e| compare(e, floor) != Ordering::Less);
                !map.is_empty()
            });
            !bucket.is_empty()
        });
    }

    /// The fundamental sequence *without* touching the cache.
    ///
    /// Used for the probes in `min_index_ge`: a probe that fails is below the
    /// current ordinal, and since time only moves forward that value can never
    /// be asked for again, so memoizing it would only waste work and memory.
    pub fn fs_probe(&self, expr: &Expr, index: i64) -> Expr {
        if !is_limit(expr) {
            return expand(expr, 0);
        }
        expand(expr, index)
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
