//! A Rust port of the BTBMS (Bubby3's Transfinite BMS) notation.
//!
//! This module contains only the notation itself: fundamental sequences,
//! comparison, display/parse, and layer-representation conversion. It does not
//! include registration, `init`, or the special marker for the notation's
//! range limit (the JS `INFINITY`) -- that marker is not part of the notation.
//!
//! Attach it as a submodule of a crate (e.g. `mod rust;` in `src/lib.rs`), then:
//!
//! ```text
//! let notation = rust::Btbm::new();
//! let expr = notation.from_display("(0)(1)(2)").unwrap();
//! let seq = notation.fs(&expr, 2);
//! let text = notation.display(&seq, rust::DisplayType::Plain);
//! ```

mod bocf;
mod compare;
mod convert;
mod display;
mod fs;
pub mod milestones;
mod height;
mod layer;
mod lngi;
mod notation;
mod types;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use lngi::{family, get_time, get_time_inv, Lngi, LngiStep, START_MS};
pub use bocf::bocf_latex;
pub use milestones::Milestone;
pub use notation::Btbm;

pub use types::{Column, DisplayType, Entry, Expr, ParseError};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_round_trip() {
        let n = Btbm::new();
        let e = n.from_display("(0)(1)(2)").unwrap();
        assert_eq!(n.display(&e, DisplayType::Plain), "(0)(1)(2)");
    }

    #[test]
    fn display_with_height() {
        let n = Btbm::new();
        let e = n.from_display("(0)(1^(2))").unwrap();
        assert_eq!(n.display(&e, DisplayType::Plain), "(0)(1^(2))");
        assert_eq!(n.display(&e, DisplayType::Html), "(0)(1<sup>(2)</sup>)");
    }

    #[test]
    fn marked_display() {
        let n = Btbm::new();
        let e = n.from_display("(0)(1)").unwrap();
        assert_eq!(n.display_marked(&e, DisplayType::Plain, 1), "(:1)(1:2)");
    }

    #[test]
    fn fs_of_omega() {
        let n = Btbm::new();
        let e = n.from_display("(0)(1)").unwrap();
        assert!(n.is_limit(&e));
        assert_eq!(n.display(&n.fs(&e, 2), DisplayType::Plain), "(0)(0)(0)");
        assert_eq!(n.display(&n.fs(&e, 3), DisplayType::Plain), "(0)(0)(0)(0)");
    }

    #[test]
    fn layer_round_trip() {
        let n = Btbm::new();
        for src in ["(0)(1)", "(0)(1)(2)", "(0)(1)(2)(3)"] {
            let e = n.from_display(src).unwrap();
            let back = n.convert_from_layer(&n.convert_to_layer(&e));
            assert_eq!(back, e, "round trip failed for {src}");
        }
    }

    #[test]
    fn parse_error() {
        let n = Btbm::new();
        assert!(n.from_display("(").is_err());
        assert!(n.from_display("(0)(").is_err());
        assert!(n.from_display("(0)x").is_err());
    }
}

/// Randomised differential tests: the cached entry points must return exactly
/// what the uncached definition returns.
///
/// The notation's value is its well-ordering proof, so the caches here are not
/// allowed to be "close enough" — `fs` must be the same function as `expand`,
/// and a warm `Lngi` must produce the same ordinal as a fresh one. Sampling
/// cannot prove equality, but it does catch the failure mode that matters: a
/// cache that quietly answers with somebody else's fundamental sequence.
#[cfg(test)]
mod differential {
    use super::*;

    /// Deterministic xorshift64, so a failure is reproducible from the seed.
    struct Rng(u64);

    impl Rng {
        fn new(seed: u64) -> Self {
            Self(seed | 1)
        }

        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x >> 12;
            x ^= x << 25;
            x ^= x >> 27;
            self.0 = x;
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Samples per test. Enough to catch a cache that answers with the wrong
    /// entry while keeping the suite quick in both profiles.
    const SAMPLES: usize = 1_000;

    /// A pool of expressions that actually occur, built by walking the notation
    /// rather than by fabricating trees: `expand` only defines itself on
    /// reachable input.
    fn corpus(n: &Btbm, rng: &mut Rng) -> Vec<Expr> {
        let mut out: Vec<Expr> = (3..10).map(family).collect();
        for _ in 0..60 {
            let mut e = family(3 + rng.below(7) as i64);
            for _ in 0..3 {
                if !n.is_limit(&e) {
                    break;
                }
                e = n.fs(&e, rng.below(8) as i64);
            }
            out.push(e);
        }
        out
    }

    /// `fs` (memoized) must equal `expand` (the definition) everywhere.
    #[test]
    fn fs_equals_expand() {
        let n = Btbm::new();
        let mut rng = Rng::new(0x5EED_1234);
        let pool = corpus(&n, &mut rng);

        for _ in 0..SAMPLES {
            let e = &pool[rng.below(pool.len() as u64) as usize];
            let index = rng.below(8) as i64;
            let want = if n.is_limit(e) {
                crate::fs::expand(e, index)
            } else {
                crate::fs::expand(e, 0)
            };
            let got = n.fs(e, index);
            assert_eq!(
                got, want,
                "fs disagreed with expand for {} at index {index}",
                n.display(e, DisplayType::Plain)
            );
        }
    }

    /// The paired lookup has to be exactly two single lookups.
    #[test]
    fn fs_pair_equals_two_lookups() {
        let n = Btbm::new();
        let mut rng = Rng::new(0xC0FF_EE01);
        let pool = corpus(&n, &mut rng);

        for _ in 0..SAMPLES {
            let e = &pool[rng.below(pool.len() as u64) as usize];
            let (a, b) = (rng.below(8) as i64, rng.below(8) as i64);
            let got = n.fs_pair(e, a, b);
            let want = (n.fs(e, a), n.fs(e, b));
            assert_eq!(
                got,
                want,
                "fs_pair disagreed for {} at ({a}, {b})",
                n.display(e, DisplayType::Plain)
            );
        }
    }

    /// A long-lived `Lngi` (cache full of history) must give the same ordinal,
    /// and the same progress fraction, as a brand new one.
    #[test]
    fn warm_cache_equals_cold() {
        let mut warm = Lngi::new();
        let mut rng = Rng::new(0xBADC_0FFE);

        // Pollute the cache with a walk that visits a lot of expressions.
        let mut t = 0.0;
        for _ in 0..2_000 {
            t += 0.5;
            warm.step(t);
        }

        let d = Btbm::new();
        for _ in 0..SAMPLES {
            // Values from 2.0 up, biased towards the dense region below 6.
            let m = 2.0 + (rng.below(8 * 1024) as f64) / 1024.0;
            let hot = warm.ordinal_for_value(m);
            let cold = Lngi::new().ordinal_for_value(m);

            assert_eq!(
                d.display(&hot.expr, DisplayType::Plain),
                d.display(&cold.expr, DisplayType::Plain),
                "warm cache produced a different ordinal at value {m}"
            );
            assert_eq!(
                hot.frac.to_bits(),
                cold.frac.to_bits(),
                "progress fraction differed at value {m}"
            );
        }
    }

    /// Access order must not matter: the same value reached from above and from
    /// below has to agree (this is what `trim_below` could break).
    #[test]
    fn order_independent() {
        let d = Btbm::new();
        let mut rng = Rng::new(0x1234_ABCC);
        let mut values: Vec<f64> = (0..SAMPLES)
            .map(|_| 2.0 + (rng.below(8 * 1024) as f64) / 1024.0)
            .collect();

        let forward: Vec<String> = {
            let mut l = Lngi::new();
            values
                .iter()
                .map(|m| d.display(&l.ordinal_for_value(*m).expr, DisplayType::Plain))
                .collect()
        };

        values.reverse();
        let backward: Vec<String> = {
            let mut l = Lngi::new();
            values
                .iter()
                .map(|m| d.display(&l.ordinal_for_value(*m).expr, DisplayType::Plain))
                .collect()
        };
        backward.into_iter().rev().zip(forward).for_each(|(a, b)| {
            assert_eq!(a, b, "ordinal depended on the order of evaluation");
        });
    }
}
