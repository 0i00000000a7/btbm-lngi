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
//!
//! Each point also carries the curve value at which the walk reaches it. That
//! value is what the lookup compares: a binary search over `f64`s instead of a
//! dozen comparisons of thousand-character terms, which is the difference
//! between a fifth of a millisecond and nothing at all on every frame.

use super::notation::Btbm;
use super::types::Expr;

/// The table, embedded at compile time so a release build has no file to find.
///
/// One point per line:
///
/// ```text
/// <matrix> = <BOCF LaTeX> [= <another OCF's LaTeX>] @ <value> <R|->
/// ```
///
/// `R` marks a point the walk stops on exactly; `-` marks one it steps over,
/// which means no ordinal is ever equal to it. A line starting with `#` is a
/// comment, blank lines are ignored, and only the first reading is used.
///
/// The values are generated, not written: see `regenerate_values` below.
const SOURCE: &str = include_str!("bocf_points.txt");

/// One pinned reading and where the walk meets it.
struct Point {
    /// The ordinal the reading is for.
    at: Expr,
    /// The curve value at which the walk first reaches or passes [`Point::at`];
    /// infinite when the walk never gets that far.
    value: f64,
    /// Whether the walk stops exactly on [`Point::at`]. When it steps over the
    /// point, the interval above it is never empty, so no ordinal can equal it
    /// and the exactness check can be skipped.
    reached: bool,
    latex: &'static str,
}

fn parse(raw: &'static str, n: &Btbm) -> Option<Point> {
    let line = raw.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let (body, meta) = line
        .rsplit_once('@')
        .unwrap_or_else(|| panic!("bocf_points.txt: no precomputed value — {line}"));

    let mut fields = body.split('=').map(str::trim);
    let matrix = fields.next().unwrap_or_default();
    let latex = fields
        .next()
        .unwrap_or_else(|| panic!("bocf_points.txt: no reading — {line}"));

    let mut meta = meta.split_whitespace();
    let value: f64 = meta
        .next()
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("bocf_points.txt: unreadable value — {line}"));

    Some(Point {
        at: n
            .from_display(matrix)
            .unwrap_or_else(|_| panic!("bad matrix in bocf_points.txt: {matrix}")),
        value,
        reached: meta.next() == Some("R"),
        latex,
    })
}

/// [`SOURCE`] parsed and sorted, built once.
///
/// Parsing is far more expensive than the lookup it feeds, and this runs on
/// every frame, so the table is parsed a single time.
fn points() -> &'static [Point] {
    use std::sync::OnceLock;
    static CACHED: OnceLock<Vec<Point>> = OnceLock::new();
    CACHED.get_or_init(|| {
        let n = Btbm::new();
        let mut out: Vec<Point> = SOURCE.lines().filter_map(|raw| parse(raw, &n)).collect();
        // The file is hand-edited, so its order is not trusted; sorting by the
        // ordinal also makes the values non-decreasing. A repeated matrix keeps
        // its first reading, which is why the sort has to stay stable.
        out.sort_by(|a, b| n.compare(&a.at, &b.at));
        out.dedup_by(|a, b| n.compare(&a.at, &b.at) == std::cmp::Ordering::Equal);
        out
    })
}

/// The pinned LaTeX for an ordinal the converter declines, if it is a point.
pub fn latex(n: &Btbm, expr: &Expr) -> Option<&'static str> {
    let points = points();
    let i = points.partition_point(|p| n.compare(&p.at, expr) == std::cmp::Ordering::Less);
    match points.get(i) {
        Some(p) if n.compare(&p.at, expr) == std::cmp::Ordering::Equal => Some(p.latex),
        _ => None,
    }
}

/// The LaTeX bounds around the ordinal that sits at `value`; the caller prints
/// `lower < X < upper`.
///
/// `None` at a point (that is [`latex`]'s job) and outside the table: below the
/// first point and above the last there is nothing to bracket it with, so the
/// caller falls back to the plain matrix form.
///
/// The two points bracketing `value` bracket the ordinal as well, because the
/// walk is monotone in the value. A point the walk steps over can never be the
/// ordinal itself, so only points it stops on need the exactness check.
pub fn bounds(value: f64, n: &Btbm, expr: &Expr) -> Option<(&'static str, &'static str)> {
    let points = points();
    let i = points.partition_point(|p| p.value <= value);
    if i == 0 || i == points.len() {
        return None;
    }
    let lower = &points[i - 1];
    if lower.reached && n.compare(&lower.at, expr) == std::cmp::Ordering::Equal {
        return None;
    }
    Some((lower.latex, points[i].latex))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The lookup the value index replaces, kept as the reference the tests
    /// compare against.
    fn bounds_by_ordinal(n: &Btbm, expr: &Expr) -> Option<(&'static str, &'static str)> {
        let points = points();
        let i = points.partition_point(|p| n.compare(&p.at, expr) == std::cmp::Ordering::Less);
        if i == 0 || i == points.len() {
            return None;
        }
        if n.compare(&points[i].at, expr) == std::cmp::Ordering::Equal {
            return None;
        }
        Some((points[i - 1].latex, points[i].latex))
    }

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
            let point =
                parse(line, &n).unwrap_or_else(|| panic!("line {}: unreadable — {line}", i + 1));
            assert!(!point.latex.is_empty(), "line {}: empty reading", i + 1);
            count += 1;
        }
        assert!(count > 100, "the table looks truncated: {count} points");
        assert!(points().len() > 100, "the parsed table is too small");
    }

    /// The values are the whole point of the index: if they drift out of step
    /// with the ordinals, lookups silently land in the wrong interval.
    #[test]
    fn values_are_non_decreasing() {
        for pair in points().windows(2) {
            assert!(
                pair[0].value <= pair[1].value,
                "values out of order: {} then {}",
                pair[0].value,
                pair[1].value
            );
        }
    }

    /// The index must answer exactly what the ordinal comparison answered, at
    /// every value the walk passes through.
    #[test]
    fn value_index_matches_the_order() {
        use super::super::lngi::Lngi;

        let n = Btbm::new();
        let points = points();
        let first = points[0].value;
        let last = points
            .iter()
            .rev()
            .find(|p| p.value.is_finite())
            .map_or(first, |p| p.value);
        assert!(first.is_finite() && last > first, "no usable value range");

        let mut checked = 0;
        let mut bounded = 0;
        let steps = 4000;
        for i in 0..=steps {
            let value = first - 0.01 + (last - first + 0.02) * (i as f64 / steps as f64);
            // A fresh walk per sample: `ordinal_for_value` memoizes whole
            // terms, and one instance would grow without bound over the sweep.
            let lngi = Lngi::new();
            let expr = lngi.ordinal_for_value(value).expr;
            let by_value = bounds(value, &n, &expr).map(|b| (b.0.to_string(), b.1.to_string()));
            let by_order = bounds_by_ordinal(&n, &expr).map(|b| (b.0.to_string(), b.1.to_string()));
            assert_eq!(by_value, by_order, "mismatch at value {value}");
            checked += 1;
            if by_value.is_some() {
                bounded += 1;
            }
        }
        assert!(checked > 3000);
        assert!(bounded > 100, "the sweep never landed inside the table");
    }

    /// The edges: nothing brackets an ordinal outside the table, so the caller
    /// falls back to the plain matrix form there.
    #[test]
    fn bounds_stop_at_the_edges() {
        let n = Btbm::new();
        let points = points();

        let below = n.from_display("(0)").expect("test input");
        assert_eq!(n.compare(&below, &points[0].at), std::cmp::Ordering::Less);
        assert_eq!(latex(&n, &below), None);
        assert_eq!(bounds(f64::NEG_INFINITY, &n, &below), None);
        assert_eq!(bounds(f64::INFINITY, &n, &below), None);
    }

    /// Rewrites `bocf_points.txt` with the value and reach flag of every point.
    ///
    /// ```text
    /// cargo test --lib regenerate_values -- --ignored
    /// ```
    ///
    /// The file is a source, so this is deliberately not part of the normal
    /// test run: it needs a fresh compile afterwards for the new table to be
    /// the one under test.
    #[test]
    #[ignore = "rewrites bocf_points.txt"]
    fn regenerate_values() {
        use super::super::lngi::Lngi;

        let n = Btbm::new();
        let mut out = String::new();
        for raw in SOURCE.lines() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                out.push_str(raw);
                out.push('\n');
                continue;
            }
            let body = line.rsplit_once('@').map_or(line, |(body, _)| body.trim());
            let matrix = body.split('=').next().unwrap_or_default().trim();
            let expr = n
                .from_display(matrix)
                .unwrap_or_else(|_| panic!("bad matrix: {matrix}"));

            // A fresh walk per point: `reach_value` memoizes ordinals, and a
            // shared instance would hold every term of a 200-step bisection.
            let lngi = Lngi::new();
            let reach = lngi.reach_value(&expr);
            let reached = reach.is_some() && lngi.search_value(&expr).is_some();
            let value = reach.unwrap_or(f64::INFINITY);
            out.push_str(&format!(
                "{body} @ {value:?} {}\n",
                if reached { 'R' } else { '-' }
            ));
        }
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/src/rust/bocf_points.txt"),
            out,
        )
        .expect("write bocf_points.txt");
    }
}
