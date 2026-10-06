//! Comparators.

use std::cmp::Ordering;

use super::types::{Column, Entry, Expr, Height, HeightColumn, HeightEntry};

/// Generic lexicographic comparison. When `longer_last` is true, longer is greater.
pub(crate) fn lex_compare<T>(
    a: &[T],
    b: &[T],
    cmp: impl Fn(&T, &T) -> Ordering,
    longer_last: bool,
) -> Ordering {
    let len = a.len().min(b.len());
    for i in 0..len {
        let ord = cmp(&a[i], &b[i]);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    let len_ord = a.len().cmp(&b.len());
    if longer_last {
        len_ord
    } else {
        len_ord.reverse()
    }
}

/// Compare two expressions.
pub(crate) fn compare(a: &Expr, b: &Expr) -> Ordering {
    lex_compare(a, b, column_compare, true)
}

fn column_compare(a: &Column, b: &Column) -> Ordering {
    lex_compare(a, b, entry_compare, true)
}

fn entry_compare(a: &Entry, b: &Entry) -> Ordering {
    a.value
        .cmp(&b.value)
        .then_with(|| compare(&a.height, &b.height))
}

/// Compare two heights.
pub(crate) fn height_compare(a: &Height, b: &Height) -> Ordering {
    lex_compare(a, b, height_column_compare, true)
}

fn height_column_compare(a: &HeightColumn, b: &HeightColumn) -> Ordering {
    lex_compare(a, b, height_entry_compare, true)
}

fn height_entry_compare(a: &HeightEntry, b: &HeightEntry) -> Ordering {
    a.mark
        .cmp(&b.mark)
        .then_with(|| a.value.cmp(&b.value))
        .then_with(|| height_compare(&a.height, &b.height))
}

/// Compare two optional heights: `None` is the "top" placeholder, greater than
/// any real height.
pub(crate) fn height_opt_compare(a: Option<&Height>, b: Option<&Height>) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => height_compare(x, y),
    }
}
