//! BTBMS LNGI: time -> value -> ordinal.
//!
//! Mirrors `get_time` / `ntl` / `num_to_lngi` from the local reference
//! `lngi.js`, but with the BTBMS notation in place of omega-Y, and without
//! omega-Y's `boost` handling.
//!
//! The time curve applies no acceleration or deceleration: `(0)(1^(2))`
//! (suspected PTO(Z_2)) is only the first term of the fundamental sequence of
//! the notation's top element `Limit`, not the limit of BTBMS, so it gets no
//! special dwell time.
//!
//! Internal representation note: `display` prints an entry's `value` plus one,
//! so the displayed `(0)` is an **empty column** and the displayed `(1)` is
//! `Entry { value: 0, .. }`.

use std::cmp::Ordering;

use super::notation::Btbm;
use super::types::{Entry, Expr};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Fixed start timestamp: 2026-10-06 18:00:00 UTC.
pub const START_MS: f64 = 1_791_309_600_000.0;

/// Time-curve scale (copied from the local reference `lngi.js:166`):
/// `x = log10(1 + t/864000)/2 + 2`.
const TIME_SCALE_MS: f64 = 864_000.0;

/// Termination threshold and depth cap for the recursive enumeration.
const LEVEL_EPS: f64 = 1e-12;
const MAX_DEPTH: usize = 100;
/// Search cap in `chain_at` for the least `i` with `beta[i] >= alpha`.
const MAX_SEARCH: i64 = 256;

// ---------------------------------------------------------------------------
// Time curve
// ---------------------------------------------------------------------------

/// The log value curve (local reference `lngi.js:166`).
pub fn get_time(t_ms: f64) -> f64 {
    (1.0 + t_ms / TIME_SCALE_MS).log10() / 2.0 + 2.0
}

/// Inverse of [`get_time`] (local reference `lngi.js:177`).
pub fn get_time_inv(x: f64) -> f64 {
    (10f64.powf((x - 2.0) * 2.0) - 1.0) * TIME_SCALE_MS
}

// ---------------------------------------------------------------------------
// The starting family
// ---------------------------------------------------------------------------

/// The family `family(k)`: the nested tower `(0)`, `(0)(1)`, `(0)(1^(2))`,
/// `(0)(1^(2^(3)))`, ...
///
/// `family(k)` has value exactly `k+1`:
///
/// | `k` | expression |
/// |---|---|
/// | 1 | `(0)` |
/// | 2 | `(0)(1)` = omega |
/// | 3 | `(0)(1^(2))` |
/// | 4 | `(0)(1^(2^(3)))` |
/// | 5 | `(0)(1^(2^(3^(4))))` |
///
/// This is not an arbitrary tower: it is exactly the fundamental sequence of
/// the notation's top element `Limit`, i.e. `family(k) = Limit[k-1]`
/// (the JS `infinity_FS(k-1)` in the local reference `BTBM.ts`). So walking
/// along `family` means walking up the cofinal sequence of `Limit` toward
/// `Limit` itself. `(0)(1^(2))` (suspected PTO(Z_2)) is only `Limit[1]`, the
/// first term of that chain -- **not** the limit of BTBMS.
pub fn family(k: i64) -> Expr {
    if k <= 1 {
        return vec![Vec::new()]; // (0)
    }
    vec![Vec::new(), vec![Entry { value: 0, height: nest(1, k) }]]
}

/// The superscript chain `(2^(3^(4^(...))))` inside `family(k)`: starting from
/// entry value `v`, nest until layer `k-2` (empty once `v > k-2`).
fn nest(v: i64, k: i64) -> Expr {
    if v > k - 2 {
        return Vec::new();
    }
    vec![vec![Entry { value: v, height: nest(v + 1, k) }]]
}

/// The least nonzero ordinal `(0)` (value 1).
pub fn min_ordinal() -> Expr {
    vec![Vec::new()]
}

// ---------------------------------------------------------------------------
// Value -> ordinal
// ---------------------------------------------------------------------------

/// The result of one lookup.
#[derive(Clone, Debug)]
pub struct LngiStep {
    /// The current ordinal.
    pub expr: Expr,
    /// How much is left until the next ordinal (the progress bar uses `1 - frac`).
    pub frac: f64,
}

/// LNGI state: holds the FS memoization cache.
#[derive(Default)]
pub struct Lngi {
    btbm: Btbm,
}

impl Lngi {
    pub fn new() -> Self {
        Self { btbm: Btbm::new() }
    }

    pub fn btbm(&self) -> &Btbm {
        &self.btbm
    }

    /// Elapsed time (milliseconds, relative to [`START_MS`]) -> ordinal.
    ///
    /// Negative time (before the start timestamp) is pinned to the least
    /// nonzero ordinal `(0)` (value 1).
    pub fn step(&mut self, elapsed_ms: f64) -> LngiStep {
        if elapsed_ms < 0.0 {
            let expr = min_ordinal();
            return LngiStep { expr, frac: 1.0 };
        }
        self.ordinal_for_value(get_time(elapsed_ms))
    }

    /// Value `m` -> ordinal.
    ///
    /// A value `m in [k, k+1)` corresponds to the interval
    /// `[family(k-1), family(k))` (`family(k)` has value exactly `k+1`). The
    /// ordinals inside the interval are enumerated by **recursive level
    /// decomposition**:
    ///
    /// Given `[alpha, beta)` (with `beta` a limit), split `x in [0,1)` into
    /// geometric levels `[1-2^-l, 1-2^-(l-1)) -> [gamma_{l-1}, gamma_l)`, where
    /// `gamma_0 = alpha` and `gamma_i = beta[i0+i-1]` (`i0` is the least index
    /// with `beta[i0] >= alpha`; if `beta[i0] = alpha` the whole chain shifts
    /// down by one). Each level then recurses on `[gamma_{l-1}, gamma_l)` until
    /// the interval collapses to a single point (`gamma_l = gamma_{l-1} + 1`).
    ///
    /// So `[omega, (0)(1^(2)))` walks through `omega -> omega+1 -> ... ->
    /// omega*2 -> ... -> omega^2 -> ... -> (0)(1,1) -> ... -> (0)(1,1,1) ->
    /// ... -> (0)(1^(2))`, i.e. the "complete recursive enumeration"; and every
    /// level is strictly monotone and continuous, so the whole curve is
    /// non-decreasing.
    pub fn ordinal_for_value(&self, m: f64) -> LngiStep {
        if m < 2.0 {
            let expr = min_ordinal();
            return LngiStep { expr, frac: 1.0 };
        }
        let k = m.floor() as i64;
        let x = m - k as f64;
        let alpha = family(k - 1);
        let beta = family(k);
        let (expr, y) = enum_below(&self.btbm, &alpha, &beta, x, 0);
        self.btbm.trim_below(&expr);
        LngiStep {
            expr,
            frac: 1.0 - y,
        }
    }

    /// Where in the sequence `target` first appears, as a value.
    ///
    /// The ordinal only grows with the value, so this is a plain bisection —
    /// no need for the fundamental-sequence walk the reference implementation
    /// uses. Returns `None` when the sequence never reaches `target` exactly
    /// (it is not a standard ordinal of this notation, or the value is beyond
    /// what the walk can reach).
    pub fn search_value(&self, target: &Expr) -> Option<f64> {
        let reach = |m: f64| self.ordinal_for_value(m).expr;
        let at = |m: f64| self.btbm.compare(&reach(m), target);

        // Bracket the answer: the value is at least 2, and the walk reaches
        // further with every doubling.
        let mut lo = 2.0;
        let mut hi = 4.0;
        while at(hi) == Ordering::Less {
            hi *= 2.0;
            if hi > 1e12 {
                return None;
            }
        }
        for _ in 0..200 {
            let mid = (lo + hi) / 2.0;
            if at(mid) == Ordering::Less {
                lo = mid;
            } else {
                hi = mid;
            }
            if hi - lo <= f64::EPSILON * hi {
                break;
            }
        }
        if at(hi) == Ordering::Equal {
            Some(hi)
        } else {
            None
        }
    }
}

/// Recursively enumerate inside `[alpha, beta)` by geometric level, returning
/// `(ordinal, position within the level)`.
fn enum_below(n: &Btbm, alpha: &Expr, beta: &Expr, x: f64, depth: usize) -> (Expr, f64) {
    if x <= LEVEL_EPS || depth >= MAX_DEPTH {
        return (alpha.clone(), x);
    }
    let t = 1.0 - x;
    // Level `l >= 1`: `t in (2^-l, 2^-(l-1)]`.
    let l = ((-t.log2()).floor() as i64 + 1).max(1);
    let y = 2.0 - t * 2f64.powi(l as i32);
    let (lo, hi) = match chain_at(n, alpha, beta, l) {
        Some(v) => v,
        None => return (alpha.clone(), x),
    };
    // The interval collapsed to a single point: `hi = lo + 1`.
    if n.compare(&hi, &append_zero(&lo)) == Ordering::Equal {
        return (lo, y);
    }
    enum_below(n, &lo, &hi, y, depth + 1)
}

/// The `l`-th level bounds `(gamma_{l-1}, gamma_l)` of `[alpha, beta)`.
fn chain_at(n: &Btbm, alpha: &Expr, beta: &Expr, l: i64) -> Option<(Expr, Expr)> {
    let (i0, beta_i0) = min_index_ge(n, beta, alpha)?;
    // If `beta[i0] == alpha` the levels start at `i0`; otherwise prepend a
    // `[alpha, beta[i0])` segment.
    let aligned = n.compare(&beta_i0, alpha) == Ordering::Equal;
    // `gamma(0)` is `alpha` itself, so only levels above the first need two
    // terms — and both come from the same limit, so they share one hash.
    if l == 1 {
        let next = if aligned { i0 + 1 } else { i0 };
        return Some((alpha.clone(), n.fs(beta, next)));
    }
    let index_of = |k: i64| if aligned { i0 + k } else { i0 + k - 1 };
    Some(n.fs_pair(beta, index_of(l - 1), index_of(l)))
}

/// The least `i` with `fs(beta, i) >= alpha`, together with that term; `None`
/// if not found within [`MAX_SEARCH`].
///
/// The term itself is returned because the caller needs it next: recomputing it
/// through the cache would repeat a lookup the scan has already done.
fn min_index_ge(n: &Btbm, beta: &Expr, alpha: &Expr) -> Option<(i64, Expr)> {
    for i in 0..MAX_SEARCH {
        let term = n.fs_probe(beta, i);
        if n.compare(&term, alpha) != Ordering::Less {
            return Some((i, term));
        }
    }
    None
}

fn append_zero(expr: &Expr) -> Expr {
    let mut out = expr.clone();
    out.push(Vec::new());
    out
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::compare::compare;
    use super::super::DisplayType;
    use super::*;

    fn n() -> Btbm {
        Btbm::new()
    }

    #[test]
    fn family_shapes() {
        let n = n();
        let d = |k| n.display(&family(k), DisplayType::Plain);
        assert_eq!(d(1), "(0)");
        assert_eq!(d(2), "(0)(1)");
        assert_eq!(d(3), "(0)(1^(2))");
        assert_eq!(d(4), "(0)(1^(2^(3)))");
        assert_eq!(d(5), "(0)(1^(2^(3^(4))))");
    }

    /// Principal points: `family(k)` is hit exactly at `m = k+1`.
    #[test]
    fn principal_points() {
        let l = Lngi::new();
        let n = n();
        let at = |m: f64| n.display(&l.ordinal_for_value(m).expr, DisplayType::Plain);
        assert_eq!(at(2.0), "(0)", "family(1)");
        assert_eq!(at(3.0), "(0)(1)", "omega = family(2)");
        assert_eq!(at(4.0), "(0)(1^(2))", "family(3)");
        assert_eq!(at(5.0), "(0)(1^(2^(3)))", "family(4)");
        assert_eq!(at(6.0), "(0)(1^(2^(3^(4))))", "family(5)");
    }

    /// Between two family members we really do the "complete recursive
    /// enumeration": `omega -> omega*2 -> omega^2 -> ... -> (0)(1,1) -> ...`.
    #[test]
    fn enumerates_between_families() {
        let l = Lngi::new();
        let n = n();
        let at = |m: f64| n.display(&l.ordinal_for_value(m).expr, DisplayType::Plain);
        assert_eq!(at(3.0), "(0)(1)", "omega");
        assert_eq!(at(3.0625), "(0)(1)(0)(3)", "omega*2");
        assert_eq!(at(3.125), "(0)(1)(1)", "omega^2");
        assert_eq!(at(3.5), "(0)(1,1)");
        assert_eq!(at(3.75), "(0)(1,1,1)");
        assert_eq!(at(3.875), "(0)(1,1,1,1)");
    }

    #[test]
    fn time_round_trip() {
        for t in [0.0, 1.0, 1e3, 8.64e9, 1.7e10] {
            let back = get_time_inv(get_time(t));
            assert!((back - t).abs() <= 1e-6 * (1.0 + t), "t={t} back={back}");
        }
    }

    /// Value -> ordinal is non-decreasing over the whole reachable range.
    #[test]
    fn ordinal_for_value_is_monotone() {
        let l = Lngi::new();
        let mut prev = l.ordinal_for_value(2.0).expr;
        let mut x = 2.0;
        while x < 7.0 {
            x += 1.0 / 2048.0;
            let cur = l.ordinal_for_value(x).expr;
            assert!(
                compare(&prev, &cur) != Ordering::Greater,
                "decreased at x={x}"
            );
            prev = cur;
        }
    }

    #[test]
    fn negative_time_is_the_minimum_ordinal() {
        let mut l = Lngi::new();
        let n = n();
        // Negative time -> least nonzero ordinal `(0)` (value 1); the "next"
        // ordinal is `(0)(0)` (value 2).
        let s = l.step(-1.0);
        assert_eq!(n.display(&s.expr, DisplayType::Plain), "(0)");
        assert!((1.0 - s.frac).abs() < 1e-12, "progress should be 0");

        // At the start point (t=0) the value is 2.0 (`get_time(0) = 2`), i.e.
        // `family(1) = (0)`.
        let s0 = l.step(0.0);
        assert_eq!(n.display(&s0.expr, DisplayType::Plain), "(0)");
    }

    /// The ordinal is non-decreasing in time.
    /// Every term the developer page offers as a child has to be a real,
    /// smaller ordinal that can be displayed and parsed back.
    #[test]
    fn fs_terms_are_usable_children() {
        let n = Btbm::new();
        for src in ["(0)(1)", "(0)(1)(1)", "(0)(1,1)", "(0)(1,1,1)", "(0)(1^(2))"] {
            let e = n.from_display(src).expect(src);
            for i in 0..4 {
                let term = n.fs(&e, i);
                assert!(
                    n.compare(&term, &e) == Ordering::Less,
                    "{src}[{i}] is not below its parent"
                );
                let text = n.display(&term, DisplayType::Plain);
                assert!(
                    n.from_display(&text).is_ok(),
                    "{src}[{i}] renders unparsable text: {text}"
                );
            }
        }
    }

    #[test]
    fn step_is_monotone_over_time() {
        let mut l = Lngi::new();
        // Sample uniformly in value (not in time): `get_time` is logarithmic,
        // so equal value steps spread across a huge time range while still
        // hitting the level boundaries that matter.
        let mut prev: Option<Expr> = None;
        let mut v = 2.0;
        while v < 6.0 {
            let cur = l.step(get_time_inv(v)).expr;
            if let Some(p) = &prev {
                assert!(
                    compare(p, &cur) != Ordering::Greater,
                    "ordinal went backwards at value {v}"
                );
            }
            prev = Some(cur);
            v += 1.0 / 512.0;
        }
    }
}
