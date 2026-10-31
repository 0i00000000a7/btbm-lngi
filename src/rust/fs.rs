//! The core expansion algorithm for the fundamental sequence.

use std::cmp::Ordering;

use super::compare::height_compare;
use super::height::to_height;
use super::types::{Column, Entry, Expr, Height};

/// Whether the expression is a limit (last column non-empty).
pub(crate) fn is_limit(expr: &Expr) -> bool {
    expr.last().map_or(false, |col| !col.is_empty())
}

fn has_next_layer(expr: &Expr) -> bool {
    expr.last().map_or(false, |col| !col.is_empty())
}

fn next_layer(expr: &Expr) -> &Expr {
    let col = expr.last().expect("next_layer: empty expression");
    &col.last().expect("next_layer: empty column").height
}

fn skip_layers(expr: &Expr, l: i64) -> &Expr {
    let mut current = expr;
    for _ in 0..l {
        current = next_layer(current);
    }
    current
}

fn tail_layer(expr: &Expr) -> i64 {
    let mut current = expr;
    let mut layer = -1;
    while has_next_layer(current) {
        current = next_layer(current);
        layer += 1;
    }
    layer
}

fn tail(expr: &Expr, t_layer: i64) -> i64 {
    let mut current_left = 0i64;
    let mut current = expr;
    for _ in 0..t_layer {
        current_left += current.len() as i64;
        current = next_layer(current);
    }
    current_left + current.len() as i64 - 1
}

fn root(expr: &Expr, t_layer: i64) -> i64 {
    let current = skip_layers(expr, t_layer);
    current.last().unwrap().last().unwrap().value
}

fn root_layer(expr: &Expr, r: i64) -> (i64, i64) {
    let mut current = expr;
    let mut current_left = 0i64;
    let mut current_layer = 0i64;
    loop {
        if r < current_left + current.len() as i64 {
            return (current_layer, r - current_left);
        }
        current_left += current.len() as i64;
        current = next_layer(current);
        current_layer += 1;
    }
}

fn without_last<T>(slice: &[T]) -> &[T] {
    &slice[..slice.len().saturating_sub(1)]
}

fn ascend_replace(
    expr: &[Column],
    r: i64,
    diff: i64,
    t_layer: Option<i64>,
    new_tail: &[Column],
) -> Expr {
    let mut result: Expr = Vec::with_capacity(expr.len());
    let last_i = expr.len().saturating_sub(1);
    for (i, col) in expr.iter().enumerate() {
        if t_layer == Some(0) && i == last_i {
            result.extend_from_slice(new_tail);
            continue;
        }
        let mut result_col: Column = Vec::with_capacity(col.len());
        let last_j = col.len().saturating_sub(1);
        for (j, entry) in col.iter().enumerate() {
            let new_t_layer = match t_layer {
                Some(t) if i == last_i && j == last_j => Some(t - 1),
                _ => None,
            };
            result_col.push(Entry {
                value: if entry.value >= r {
                    entry.value + diff
                } else {
                    entry.value
                },
                height: ascend_replace(&entry.height, r, diff, new_t_layer, new_tail),
            });
        }
        result.push(result_col);
    }
    result
}

/// [`ascend_replace`] for a single column, with no trailing-tail substitution
/// (the callers pass no `t_layer`). Avoids building a one-column `Expr` only to
/// clone it straight back out.
fn ascend_replace_col(col: &[Entry], r: i64, diff: i64) -> Column {
    col.iter()
        .map(|entry| Entry {
            value: if entry.value >= r {
                entry.value + diff
            } else {
                entry.value
            },
            height: ascend_replace(&entry.height, r, diff, None, &[]),
        })
        .collect()
}

fn is_special(expr: &Expr, t_layer: i64) -> bool {
    if t_layer == 0 {
        return false;
    }
    let mut current = expr;
    let mut current_left = 0i64;
    for _ in 0..t_layer {
        current_left += current.len() as i64;
        current = next_layer(current);
    }
    let last_col = match current.last() {
        Some(col) => col,
        None => return false,
    };
    if last_col.len() != 1 {
        return false;
    }
    let entry = &last_col[0];
    entry.height.is_empty() && entry.value == current_left - 1
}

fn expand_special(expr: &Expr, t_layer: i64, index: i64) -> Expr {
    let mut result: Expr = without_last(expr).to_vec();
    let col = expr.last().expect("expand_special: empty expression");
    let mut result_col: Column = without_last(col).to_vec();
    let entry = col.last().expect("expand_special: empty column");
    if t_layer > 1 {
        result_col.push(Entry {
            value: entry.value,
            height: expand_special(&entry.height, t_layer - 1, index),
        });
    } else {
        let new_entry = Entry {
            value: entry.value,
            height: without_last(&entry.height).to_vec(),
        };
        if result_col.is_empty() {
            result_col.push(new_entry.clone());
        }
        for _ in 0..index {
            result_col.push(new_entry.clone());
        }
    }
    result.push(result_col);
    result
}

fn root_appending_start(col_root: &Column, r: i64, col_tail: &Column, t: i64) -> i64 {
    let heights_root: Vec<Height> = col_root.iter().map(|e| to_height(&e.height, r)).collect();
    let heights_tail: Vec<Height> = without_last(col_tail)
        .iter()
        .map(|e| to_height(&e.height, t))
        .collect();
    let mut ir = 0usize;
    let mut it = 0usize;
    while ir != heights_root.len() && it != heights_tail.len() {
        let ord = height_compare(&heights_root[ir], &heights_tail[it]);
        if ord != Ordering::Less {
            it += 1;
        }
        if ord != Ordering::Greater {
            ir += 1;
        }
    }
    ir as i64
}

/// The notation's core expansion function (uncached).
pub(crate) fn expand(expr: &Expr, index: i64) -> Expr {
    if expr.is_empty() {
        return expr.clone();
    }
    let t_layer = tail_layer(expr);
    if t_layer < 0 {
        return without_last(expr).to_vec();
    }

    if is_special(expr, t_layer) {
        return expand_special(expr, t_layer, index);
    }

    let t = tail(expr, t_layer);
    let r = root(expr, t_layer);
    let (r_layer, ri) = root_layer(expr, r);

    let expr_root = skip_layers(expr, r_layer);
    let col_root = &expr_root[ri as usize];
    let expr_tail = skip_layers(expr_root, t_layer - r_layer);
    let col_tail = expr_tail.last().expect("expand: empty tail");

    let appending = root_appending_start(col_root, r, col_tail, t);
    let seed: &[Entry] = without_last(col_tail);

    let mut new_tail: Expr = Vec::new();
    let mut j = index;
    while j >= 1 {
        if ri != expr_root.len() as i64 - 1 {
            let new_tail_1 = ascend_replace(
                &expr_root[(ri + 1) as usize..],
                r,
                j * (t - r),
                Some(t_layer - r_layer),
                &new_tail,
            );
            let mut new_col = ascend_replace_col(seed, r, (j - 1) * (t - r));
            for k in appending..col_root.len() as i64 {
                let entry = &col_root[k as usize];
                new_col.push(Entry {
                    value: entry.value,
                    height: ascend_replace(&entry.height, r, j * (t - r), None, &[]),
                });
            }
            new_tail = std::iter::once(new_col).chain(new_tail_1).collect();
        } else {
            if appending == col_root.len() as i64 {
                panic!("Illegal state in BTBMS fundamental sequence");
            }
            let mut new_col = ascend_replace_col(seed, r, (j - 1) * (t - r));
            let root_len = col_root.len() as i64;
            for k in appending..root_len {
                let entry = &col_root[k as usize];
                let new_t_layer = if k == root_len - 1 {
                    Some(t_layer - r_layer - 1)
                } else {
                    None
                };
                new_col.push(Entry {
                    value: entry.value,
                    height: ascend_replace(&entry.height, r, j * (t - r), new_t_layer, &new_tail),
                });
            }
            new_tail = vec![new_col];
        }
        j -= 1;
    }

    ascend_replace(expr, 0, 0, Some(t_layer), &new_tail)
}
