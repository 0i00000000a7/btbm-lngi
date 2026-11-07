//! Conversions from the layered BMS matrix to other matrix notations.
//!
//! 0-Y is a sequence of integers; triangular BMS ("DBMS") is a matrix whose
//! columns are conceptually ragged. Both are plain integer matrices with no
//! nested heights, so they only reach the fragment that has none — the strong
//! fragment keeps its structure in the heights, and neither notation can carry
//! it. Past that the callers fall back to the BTBMS display.

use std::cmp::Ordering;

use super::notation::Btbm;
use super::types::{Column, Expr};

/// Columns of equal length, `i64` because the 0-Y counts grow past `i32`.
type Matrix = Vec<Vec<i64>>;

/// The layered expression as a plain matrix, or `None` when a column carries a
/// nested height.
fn matrix(layered: &[Column]) -> Option<Matrix> {
    let mut out: Matrix = Vec::with_capacity(layered.len());
    let mut rows = 0;
    for col in layered {
        let mut c = Vec::with_capacity(col.len());
        for entry in col {
            if !entry.height.is_empty() {
                return None;
            }
            c.push(entry.value + 1);
        }
        rows = rows.max(c.len());
        out.push(c);
    }
    // A BMS matrix always has at least the bottom row, even when every column
    // is the zero column — `(0)` is one empty column here.
    rows = rows.max(1);
    for col in &mut out {
        col.resize(rows, 0);
    }
    Some(out)
}

// ---------------------------------------------------------------------------
// BMS → 0-Y
// ---------------------------------------------------------------------------

/// `1,2,4,…`: one count per column, accumulated up the rows.
fn bms_to_0y_values(m: &Matrix) -> Vec<i64> {
    let cols = m.len();
    if cols == 0 {
        return Vec::new();
    }
    let rows = m[0].len();

    let mut result = vec![1i64; cols];
    for row in (0..rows).rev() {
        let mut stack: Vec<usize> = Vec::new();
        for col in 0..cols {
            while let Some(&back) = stack.last() {
                if m[back][row] >= m[col][row] {
                    stack.pop();
                } else {
                    break;
                }
            }
            if let Some(&back) = stack.last() {
                result[col] += result[back];
            }
            stack.push(col);
        }
    }

    result
}

fn join_values(values: &[i64]) -> String {
    values
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn bms_to_0y(m: &Matrix) -> String {
    join_values(&bms_to_0y_values(m))
}

// ---------------------------------------------------------------------------
// BMS → triangular BMS
// ---------------------------------------------------------------------------

fn col_less(a: &[i64], b: &[i64]) -> bool {
    let n = a.len().min(b.len());
    for i in 0..n {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
    }
    a.len() < b.len()
}

/// Lexicographic comparison of two column sequences.
fn seq_cmp(a: &Matrix, b: &Matrix) -> i32 {
    let n = a.len().min(b.len());
    for i in 0..n {
        if a[i] == b[i] {
            continue;
        }
        return if col_less(&a[i], &b[i]) { -1 } else { 1 };
    }
    match a.len().cmp(&b.len()) {
        Ordering::Less => -1,
        Ordering::Equal => 0,
        Ordering::Greater => 1,
    }
}

/// Largest 1-based row whose entry is positive; 0 if the column is all zero.
fn last_positive_row(col: &[i64]) -> i64 {
    for i in (0..col.len()).rev() {
        if col[i] > 0 {
            return i as i64 + 1;
        }
    }
    0
}

fn increment_prefix(col: &[i64], count: i64) -> Vec<i64> {
    let mut r = col.to_vec();
    for i in 0..(count as usize).min(r.len()) {
        r[i] += 1;
    }
    r
}

fn increment_row(col: &[i64], row: i64) -> Vec<i64> {
    let mut r = col.to_vec();
    if row >= 1 && (row as usize) <= r.len() {
        r[(row - 1) as usize] += 1;
    }
    r
}

fn first_row_column(value: i64, n: usize) -> Vec<i64> {
    let mut r = vec![0i64; n];
    if n > 0 {
        r[0] = value;
    }
    r
}

/// For every row and column, which columns to the left are ancestors.
///
/// Stored as bit sets rather than sets of indices: the ancestor set of a cell is
/// a copy of its parent's plus the parent, so a plain set costs O(columns) per
/// cell to copy, which is what made wide matrices take tens of milliseconds.
struct AncestorIndex {
    n: usize,
    columns: usize,
    words: usize,
    parents: Vec<Vec<i32>>,
    ancestors: Vec<Vec<Vec<u64>>>,
}

impl AncestorIndex {
    fn new(cols: &Matrix) -> Self {
        if cols.is_empty() {
            return AncestorIndex {
                n: 0,
                columns: 0,
                words: 0,
                parents: Vec::new(),
                ancestors: Vec::new(),
            };
        }
        let columns = cols.len();
        let n = cols[0].len();
        let words = (columns + 63) / 64;

        let mut parents = vec![vec![-1i32; columns]; n + 1];
        let mut ancestors: Vec<Vec<Vec<u64>>> = vec![vec![vec![0u64; words]; columns]; n + 1];

        // Row 0 is virtual: every column to the left is an ancestor.
        for c in 1..columns {
            parents[0][c] = c as i32 - 1;
            for a in 0..c {
                ancestors[0][c][a / 64] |= 1 << (a % 64);
            }
        }

        for row in 1..=n {
            let vi = row - 1;
            for c in 0..columns {
                let mut parent: i32 = -1;
                // Right to left: the nearest column whose entry is smaller.
                for a in (0..c).rev() {
                    if ancestors[row - 1][c][a / 64] & (1 << (a % 64)) == 0 {
                        continue;
                    }
                    if cols[a][vi] < cols[c][vi] {
                        parent = a as i32;
                        break;
                    }
                }
                parents[row][c] = parent;
                if parent >= 0 {
                    let pu = parent as usize;
                    let src = ancestors[row][pu].clone();
                    let dst = &mut ancestors[row][c];
                    for (d, v) in dst.iter_mut().zip(src.iter()) {
                        *d |= *v;
                    }
                    dst[pu / 64] |= 1 << (pu % 64);
                }
            }
        }

        AncestorIndex {
            n,
            columns,
            words,
            parents,
            ancestors,
        }
    }

    fn has_ancestor_column(&self, element: usize, row: usize, ancestor: usize) -> bool {
        if row > self.n || element >= self.columns || ancestor >= self.columns {
            return false;
        }
        let bits = &self.ancestors[row][element];
        bits[ancestor / 64] & (1 << (ancestor % 64)) != 0
    }

    fn parent_is_column(&self, element: usize, row: usize, parent: usize) -> bool {
        if row > self.n || element >= self.columns {
            return false;
        }
        self.parents[row][element] == parent as i32
    }
}

/// `None` when the matrix has no triangular form (the reference gives up too).
fn bms_to_triangular(m: &Matrix) -> Option<Matrix> {
    let mut cols = m.to_vec();
    if cols.is_empty() {
        return Some(Vec::new());
    }
    let n = cols[0].len();
    if n < 2 {
        return Some(cols);
    }

    let mut idx = 0usize;
    let mut steps = 0i64;
    const STEP_LIMIT: i64 = 100_000;

    while idx < cols.len() {
        steps += 1;
        if steps > STEP_LIMIT {
            return None;
        }

        let x = cols[idx].clone();
        let k = last_positive_row(&x);
        if k >= n as i64 - 1 {
            idx += 1;
            continue;
        }

        let y = increment_prefix(&x, k + 1);
        let z = increment_row(&y, k + 2);

        let x_start = idx + 1;
        if x_start >= cols.len() || col_less(&cols[x_start], &z) {
            idx += 1;
            continue;
        }

        let mut x_end = x_start;
        while x_end < cols.len() && !col_less(&cols[x_end], &z) {
            x_end += 1;
        }

        let ancestor = AncestorIndex::new(&cols);
        let mut x_prime: Matrix = Vec::new();

        for cursor in x_start..x_end {
            let t = cols[cursor].clone();

            let mut l: i64 = -1;
            for row in 0..=(k as usize + 1) {
                if ancestor.has_ancestor_column(cursor, row, idx) {
                    l = row as i64;
                }
            }
            if l < 0 {
                return None;
            }

            let is_last = cursor == x_end - 1;
            if is_last {
                if (l as usize) >= n {
                    return None;
                }
                if ancestor.parent_is_column(cursor, l as usize, idx) && t[l as usize] == 0 {
                    l -= 1;
                }
            }
            if l < 0 {
                return None;
            }

            x_prime.push(increment_prefix(&t, l));
        }

        let mut insertion: Matrix = Vec::new();
        insertion.push(y.clone());
        insertion.extend(x_prime.iter().cloned());

        let mut comparison = insertion.clone();
        comparison.push(first_row_column(y[0] + 1, n));

        let remainder: Matrix = cols[x_end..].to_vec();

        // X is always deleted; the replacement only goes in when it compares
        // greater than what follows it.
        cols.drain(x_start..x_end);
        if seq_cmp(&comparison, &remainder) > 0 {
            let mut next: Matrix = Vec::with_capacity(cols.len() + insertion.len());
            next.extend_from_slice(&cols[..x_start]);
            next.extend(insertion);
            next.extend_from_slice(&cols[x_start..]);
            cols = next;
        }

        idx += 1;
    }

    Some(cols)
}

// ---------------------------------------------------------------------------
// Entry points
// ---------------------------------------------------------------------------

/// The trimmed matrix: `(0)(1,1)(2,2,2)`. Trailing zeros are padding, so they
/// are dropped; the ragged shape is the notation.
fn render(m: &Matrix) -> String {
    let mut out = String::new();
    for col in m {
        let mut len = col.len();
        while len > 0 && col[len - 1] == 0 {
            len -= 1;
        }
        if len == 0 {
            out.push_str("(0)");
            continue;
        }
        out.push('(');
        for (i, v) in col[..len].iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            out.push_str(&v.to_string());
        }
        out.push(')');
    }
    out
}

/// The ordinal as a 0-Y sequence, e.g. `1,2,4,8`.
///
/// No column cap: both passes are linear in the matrix and cost microseconds,
/// unlike the OCF rewrites, which is what the cap exists for.
pub fn zero_y(n: &Btbm, expr: &Expr) -> Option<String> {
    let m = matrix(&n.convert_to_layer(expr))?;
    Some(bms_to_0y(&m))
}

/// The ordinal as triangular BMS, e.g. `(0)(1,1)(2,2,2)`.
pub fn triangular(n: &Btbm, expr: &Expr) -> Option<String> {
    let m = matrix(&n.convert_to_layer(expr))?;
    if m.is_empty() {
        return Some(String::new());
    }
    let out = bms_to_triangular(&m)?;
    Some(render(&out))
}

/// The ordinal as a 1-Y / ω-Y sequence, e.g. `1,2,4,8`.
///
/// The DBMS matrix is read back as an ordinary BMS matrix — ragged columns
/// padded with zeros — and the 0-Y conversion runs on that. In this fragment
/// 1-Y and ω-Y give the same sequence; they only diverge past SHO, where this
/// notation stops anyway.
pub fn one_y(n: &Btbm, expr: &Expr) -> Option<String> {
    let m = matrix(&n.convert_to_layer(expr))?;
    Some(join_values(&one_y_values(&m)?))
}

/// The ω-Y sequence behind [`one_y`], as numbers (the Vulcaniz bound needs to
/// compare them).
fn one_y_values(m: &Matrix) -> Option<Vec<i64>> {
    if m.is_empty() {
        return Some(Vec::new());
    }
    let mut tri = bms_to_triangular(m)?;
    let rows = tri.iter().map(Vec::len).max().unwrap_or(0);
    for col in &mut tri {
        col.resize(rows, 0);
    }
    Some(bms_to_0y_values(&tri))
}

// ---------------------------------------------------------------------------
// BMS → Vulcaniz
// ---------------------------------------------------------------------------

/// The reference stops Vulcaniz at the `1,2,4,…,512` tower: the correspondence
/// below it was checked, above it is unproven.
const VULCANIZ_LIMIT: [i64; 10] = [1, 2, 4, 8, 16, 32, 64, 128, 256, 512];

/// Lexicographic comparison of two integer sequences; a strict prefix is less.
fn sequence_less(a: &[i64], b: &[i64]) -> bool {
    for i in 0..a.len().min(b.len()) {
        if a[i] != b[i] {
            return a[i] < b[i];
        }
    }
    a.len() < b.len()
}

/// BMS → PMS: each entry becomes the distance to the last column whose entry
/// one row lower was the previous depth (the "marked distance" reading).
fn bms_to_pms(m: &Matrix) -> Option<Matrix> {
    let cols = m.len();
    if cols == 0 {
        return Some(Vec::new());
    }
    let rows = m[0].len();
    let max = m.iter().flat_map(|c| c.iter()).copied().max().unwrap_or(0) as usize;
    // Per row, the last 1-based column index seen at each depth.
    let mut last = vec![vec![0i64; max + 1]; rows];
    let mut out = vec![vec![0i64; rows]; cols];
    for i in 0..cols {
        for j in 0..rows {
            let depth = m[i][j] as usize;
            if depth > 0 {
                let parent = last[j][depth - 1];
                if parent == 0 {
                    return None;
                }
                out[i][j] = i as i64 + 1 - parent;
            }
            last[j][depth] = i as i64 + 1;
        }
    }
    Some(out)
}

/// PMS → Vulcaniz: for each column, the height of every entry's parent chain,
/// flattened into one integer sequence.
fn pms_to_vz(m: &Matrix) -> String {
    let cols = m.len();
    let rows = m.first().map(Vec::len).unwrap_or(0);
    let mut sequence: Vec<i64> = Vec::new();
    for i in 0..cols {
        let mut heights: Vec<i64> = Vec::with_capacity(rows);
        for j in 0..rows {
            let mut height = -1i64;
            let mut index = i as i64 + 1;
            while index > 0 {
                height += 1;
                let dist = m[index as usize - 1][j];
                index -= if dist != 0 { dist } else { index };
            }
            heights.push(height);
        }
        while heights.len() > 1 && *heights.last().unwrap() == 0 {
            heights.pop();
        }
        let v = heights[0] + 1;
        sequence.push(v);
        for h in &heights[1..] {
            sequence.push(v + h + 1);
        }
    }
    join_values(&sequence)
}

/// The ordinal as a Vulcaniz sequence, e.g. `1,2,4,4`.
///
/// `None` outside the fragment the reference covers (the `1,2,4,…,512` tower
/// and beyond), where the caller falls back to the BTBMS form.
pub fn vulcaniz(n: &Btbm, expr: &Expr) -> Option<String> {
    let m = matrix(&n.convert_to_layer(expr))?;
    if !sequence_less(&one_y_values(&m)?, &VULCANIZ_LIMIT) {
        return None;
    }
    let pms = bms_to_pms(&m)?;
    Some(pms_to_vz(&pms))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn term(n: &Btbm, s: &str) -> Expr {
        n.from_display(s).expect("test input")
    }

    /// One count per column, accumulated up the rows.
    #[test]
    fn zero_y_sequence() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "1"),
            ("(0)(0)", "1,1"),
            ("(0)(1)", "1,2"),
            ("(0)(1)(1)", "1,2,2"),
            ("(0)(1)(2)", "1,2,3"),
            ("(0)(1,1)", "1,3"),
            ("(0)(1,1)(2,1)", "1,3,5"),
            ("(0)(1,1)(2,1)(3,1)", "1,3,5,7"),
            ("(0)(1,1,1)", "1,4"),
            ("(0)(1,1,1)(2,2)", "1,4,8"),
            ("(0)(1,1,1,1)", "1,5"),
            ("(0)(1,1)(2,2)", "1,3,6"),
        ];
        for (src, want) in cases {
            assert_eq!(zero_y(&n, &term(&n, src)).as_deref(), Some(want), "{src}");
        }
    }

    /// The ragged matrix, trailing padding trimmed.
    #[test]
    fn triangular_form() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "(0)"),
            ("(0)(1)", "(0)(1)"),
            ("(0)(1,1)", "(0)(1)(2,1)"),
            ("(0)(1,1)(2,1)", "(0)(1)(2,1)(3,1)"),
            ("(0)(1,1,1)", "(0)(1)(2,1)(3,2,1)"),
            ("(0)(1,1,1)(2,2)", "(0)(1)(2,1)(3,2,1)(4,3)"),
            ("(0)(1,1,1,1)", "(0)(1)(2,1)(3,2,1)(4,3,2,1)"),
        ];
        for (src, want) in cases {
            assert_eq!(triangular(&n, &term(&n, src)).as_deref(), Some(want), "{src}");
        }
    }

    /// 1-Y: the DBMS matrix read back as BMS, then the 0-Y pass.
    #[test]
    fn one_y_sequence() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "1"),
            ("(0)(0)", "1,1"),
            ("(0)(1)", "1,2"),
            ("(0)(1,1)", "1,2,4"),
            ("(0)(1,1)(2,1)", "1,2,4,6"),
            ("(0)(1,1,1)", "1,2,4,8"),
            ("(0)(1,1,1)(2,2)", "1,2,4,8,13"),
            ("(0)(1,1,1,1)", "1,2,4,8,16"),
        ];
        for (src, want) in cases {
            assert_eq!(one_y(&n, &term(&n, src)).as_deref(), Some(want), "{src}");
        }
    }

    /// Vulcaniz: the reference's PMS pass, then the parent-chain heights.
    #[test]
    fn vulcaniz_sequence() {
        let n = Btbm::new();
        let cases = [
            ("(0)", "1"),
            ("(0)(1)", "1,2"),
            ("(0)(1,1)", "1,2,4"),
            ("(0)(1,1)(2,1)", "1,2,4,3,5"),
            ("(0)(1,1,1)", "1,2,4,4"),
            ("(0)(1,1,1)(2,2)", "1,2,4,4,3,6"),
            ("(0)(1,1,1,1)", "1,2,4,4,4"),
            ("(0)(1,1,1,1,1)", "1,2,4,4,4,4"),
        ];
        for (src, want) in cases {
            assert_eq!(vulcaniz(&n, &term(&n, src)).as_deref(), Some(want), "{src}");
        }

        // The reference stops at the `1,2,4,…,512` tower: the tower itself and
        // everything past it are outside the checked fragment.
        assert_eq!(
            one_y(&n, &term(&n, "(0)(1,1,1,1,1,1,1,1,1)")).as_deref(),
            Some("1,2,4,8,16,32,64,128,256,512")
        );
        assert_eq!(vulcaniz(&n, &term(&n, "(0)(1,1,1,1,1,1,1,1,1)")), None);
        assert_eq!(vulcaniz(&n, &term(&n, "(0)(1^(2))")), None);
    }

    /// All three run per frame, so they have to stay well under a frame budget.
    #[test]
    fn conversions_stay_cheap() {
        use crate::Lngi;
        use std::time::Instant;

        let l = Lngi::new();
        let n = Btbm::new();
        let mut worst = 0.0f64;
        let mut longest = 0usize;
        for i in 0..400 {
            let v = 2.0 + 2.0 * (i as f64) / 400.0;
            let e = l.ordinal_for_value(v).expr;
            // Best of three: a single sample can be stretched by another test
            // thread, which says nothing about the conversion's own cost.
            let mut best = f64::INFINITY;
            let (mut a, mut b, mut c) = (None, None, None);
            for _ in 0..3 {
                let start = Instant::now();
                a = zero_y(&n, &e);
                b = triangular(&n, &e);
                c = one_y(&n, &e);
                best = best.min(start.elapsed().as_secs_f64());
            }
            worst = worst.max(best * 1e3);
            longest = longest
                .max(a.map_or(0, |s| s.len()))
                .max(b.map_or(0, |s| s.len()))
                .max(c.map_or(0, |s| s.len()));
        }
        // All three are linear in the matrix; a frame is 16 ms.
        let budget = if cfg!(debug_assertions) { 250.0 } else { 8.0 };
        assert!(worst < budget, "worst conversion was {worst:.1} ms");
        assert!(longest > 0);
    }

    /// Nested heights are the strong fragment: neither notation can carry them,
    /// so the caller has to fall back rather than print something wrong.
    #[test]
    fn declines_when_a_column_carries_a_height() {
        let n = Btbm::new();
        for src in ["(0)(1^(2))", "(0)(1^(2))(1)"] {
            let e = term(&n, src);
            assert_eq!(zero_y(&n, &e), None, "{src}");
            assert_eq!(triangular(&n, &e), None, "{src}");
            assert_eq!(one_y(&n, &e), None, "{src}");
        }
    }
}
