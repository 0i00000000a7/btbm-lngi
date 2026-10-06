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

mod compare;
mod display;
mod fs;
mod height;
mod layer;
mod lngi;
mod notation;
mod types;

#[cfg(target_arch = "wasm32")]
mod wasm;

pub use lngi::{family, get_time, get_time_inv, Lngi, LngiStep, START_MS};
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
