//! Conversion between expressions and "heights".

use super::types::{Entry, Expr, Height, HeightEntry};

/// Convert an expression into height form, shifting values by `r` and setting `mark`.
pub(crate) fn to_height(e: &Expr, r: i64) -> Height {
    e.iter()
        .map(|col| {
            col.iter()
                .map(|entry| {
                    let p = entry.value;
                    let (mark, value) = if p < r { (false, p) } else { (true, p - r) };
                    HeightEntry {
                        mark,
                        value,
                        height: to_height(&entry.height, r),
                    }
                })
                .collect()
        })
        .collect()
}

/// The inverse of [`to_height`].
pub(crate) fn from_height(h: &Height, r: i64) -> Expr {
    h.iter()
        .map(|col| {
            col.iter()
                .map(|entry| Entry {
                    value: entry.value + if entry.mark { r } else { 0 },
                    height: from_height(&entry.height, r),
                })
                .collect()
        })
        .collect()
}
