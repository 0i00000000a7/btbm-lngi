//! Hand-written BOCF readings for ordinals past the converter's reach.
//!
//! The conversion in [`super::bocf`] stops at EBO, but the notation does not:
//! above it the uncountables are inaccessible ones, and those ordinals have
//! settled BOCF readings. They are pinned in `bocf_points.txt` rather than
//! guessed by the converter — a wrong term is worse than no term.
//!
//! Only exact points are maintained. An ordinal that is not one of them is
//! printed between its neighbours — `previous < X < next` — so the intervals
//! are derived, not written down: adding a point in the middle splits the
//! interval around it and nothing else changes.

use super::notation::Btbm;
use super::types::Expr;

/// The table, embedded at compile time so a release build has no file to find.
///
/// One point per line: `<matrix> = <BOCF LaTeX> [= <another OCF's LaTeX>]`.
/// A line starting with `#` is a comment, blank lines are ignored, and only the
/// first reading is used.
const SOURCE: &str = include_str!("bocf_points.txt");

/// [`SOURCE`] parsed and sorted, built once.
///
/// Parsing is far more expensive than the comparison it feeds, and this runs on
/// every frame, so the table is parsed a single time.
fn points() -> &'static [(Expr, &'static str)] {
    use std::sync::OnceLock;
    static CACHED: OnceLock<Vec<(Expr, &'static str)>> = OnceLock::new();
    CACHED.get_or_init(|| {
        let n = Btbm::new();
        let mut out: Vec<(Expr, &'static str)> = SOURCE
            .lines()
            .filter_map(|raw| {
                let line = raw.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                let mut fields = line.split('=').map(str::trim);
                let matrix = fields.next()?;
                let latex = fields.next()?;
                Some((
                    n.from_display(matrix)
                        .unwrap_or_else(|_| panic!("bad matrix in bocf_points.txt: {matrix}")),
                    latex,
                ))
            })
            .collect();
        // The file is hand-edited, so its order is not trusted; sorting also
        // makes the lookups binary searches. A repeated matrix keeps its first
        // reading, which is why the sort has to stay stable.
        out.sort_by(|a, b| n.compare(&a.0, &b.0));
        out.dedup_by(|a, b| n.compare(&a.0, &b.0) == std::cmp::Ordering::Equal);
        out
    })
}

/// The pinned LaTeX for an ordinal the converter declines, if it is a point.
pub fn latex(n: &Btbm, expr: &Expr) -> Option<&'static str> {
    let points = points();
    let i = points.partition_point(|(at, _)| n.compare(at, expr) == std::cmp::Ordering::Less);
    match points.get(i) {
        Some((at, latex)) if n.compare(at, expr) == std::cmp::Ordering::Equal => Some(*latex),
        _ => None,
    }
}

/// The LaTeX bounds around an ordinal the converter declines, taken from the
/// two points it sits between; the caller prints `lower < X < upper`.
///
/// `None` at a point (that is [`latex`]'s job) and outside the table: below the
/// first point and above the last there is nothing to bracket it with, so the
/// caller falls back to the plain matrix form.
pub fn bounds(n: &Btbm, expr: &Expr) -> Option<(&'static str, &'static str)> {
    let points = points();
    let i = points.partition_point(|(at, _)| n.compare(at, expr) == std::cmp::Ordering::Less);
    if i == 0 || i == points.len() {
        return None;
    }
    if n.compare(&points[i].0, expr) == std::cmp::Ordering::Equal {
        return None;
    }
    Some((points[i - 1].1, points[i].1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every data line must parse and carry a reading: a typo in the file would
    /// otherwise surface as a panic when the app first asks for a conversion.
    #[test]
    fn data_file_is_valid() {
        let n = Btbm::new();
        let mut count = 0;
        for (i, raw) in SOURCE.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let fields: Vec<&str> = line.split('=').map(str::trim).collect();
            assert!(fields.len() >= 2, "line {}: no reading — {line}", i + 1);
            assert!(
                n.from_display(fields[0]).is_ok(),
                "line {}: unparseable matrix — {line}",
                i + 1
            );
            count += 1;
        }
        assert!(count > 100, "the table looks truncated: {count} points");
        assert!(points().len() > 100, "the parsed table is too small");
    }

    /// The edges: nothing brackets an ordinal outside the table, so the caller
    /// falls back to the plain matrix form there.
    #[test]
    fn bounds_stop_at_the_edges() {
        use super::super::types::DisplayType;

        let n = Btbm::new();
        let points = points();

        let below = n.from_display("(0)").expect("test input");
        assert_eq!(n.compare(&below, &points[0].0), std::cmp::Ordering::Less);
        assert_eq!(latex(&n, &below), None);
        assert_eq!(bounds(&n, &below), None);

        let last = &points[points.len() - 1].0;
        let plain = n.display(last, DisplayType::Plain);
        let above = n.from_display(&format!("{plain}(0)")).expect("test input");
        assert_eq!(n.compare(&above, last), std::cmp::Ordering::Greater);
        assert_eq!(latex(&n, &above), None);
        assert_eq!(bounds(&n, &above), None);
    }
}
