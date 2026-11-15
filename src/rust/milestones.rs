//! Landmarks along the sequence.
//!
//! The list below is the source of truth: an ordinal and its text. The value
//! and the time are *derived* — `examples/gen_milestones.rs` resolves each
//! ordinal by bisection and writes `milestones_gen.rs`, which is committed and
//! checked by [`tests::generated_table_is_current`]. Nothing is searched at
//! runtime.

/// One landmark read in another notation.
///
/// Baked into the generated table: converting a long ordinal costs
/// milliseconds and the app would need one per row, so the build does them all
/// once. Only the notations that reach far enough are stored: BOCF, Veblen and
/// 1-Y.
pub struct Reading {
    /// Short notation name, as the chip shows it.
    pub label: &'static str,
    /// Whether the term is LaTeX, typeset by KaTeX, or plain text.
    pub latex: bool,
    /// The term itself.
    pub term: &'static str,
}

/// One landmark in the sequence.
pub struct Milestone {
    /// The ordinal, as the app writes it.
    pub ordinal: &'static str,
    pub title: &'static str,
    pub desc: &'static str,
    /// The value at which the ordinal first appears.
    pub value: f64,
    /// Milliseconds from the start point.
    pub at_ms: f64,
    /// The same ordinal in the weaker notations, strongest first.
    pub readings: &'static [Reading],
}

use crate::{Btbm, Expr};

/// The readings the converters produce for an ordinal, strongest notation
/// first: BOCF, then Veblen, then 1-Y.
///
/// Used by the milestone generator; the app reads the baked table instead of
/// converting anything at startup.
pub fn readings_for(n: &Btbm, e: &Expr) -> Vec<(&'static str, bool, String)> {
    let mut out = Vec::new();
    if let Some(term) = crate::bocf::bocf_latex(n, e) {
        out.push(("BOCF", true, term));
    }
    if let Some(term) = crate::bocf::veblen_latex(n, e) {
        out.push(("Veblen", true, term));
    }
    if let Some(term) = crate::convert::one_y(n, e) {
        out.push(("1-Y", false, term));
    }
    out
}

/// Hand-written readings for the landmarks the converters cannot reach.
///
/// Keyed by the spelling the generated table stores (the walk's own), which is
/// what the developer page knows. A row whose label a converter already
/// produced replaces it, so an approximation can be corrected here; any other
/// label is appended after the automatic ones.
///
/// Edited from the developer page, or here; either way the table has to be
/// regenerated.
pub const EXTRA_READINGS: &[(&str, &[Reading])] = &[
    // Written by the developer page (dev.html); edit there and press Rebuild.
    ("(0)(1^(2))(2,1,1)(1,1,1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,4,9,13,8,17,24,16" },
    ]),
    ("(0)(1^(2))(2,1,1)(1^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,4,9,13,8,17,24,17" },
    ]),
    ("(0)(1^(2))(2,1,1)(2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,4,9,13,8,17,24,18" },
    ]),
    ("(0)(1^(2))(2,1,1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,4,9,13,8,17,25" },
    ]),
    ("(0)(1^(2))(2,1,1)(3^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,4,9,13,8,17,24,37" },
    ]),
    ("(0)(1^(2))(2,1^(3))(1^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,5,5" },
    ]),
    ("(0)(1^(2))(2,1^(3))(2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,6" },
    ]),
    ("(0)(1^(2))(2,2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10" },
    ]),
    ("(0)(1^(2))(2,2)(1^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,18,9" },
    ]),
    ("(0)(1^(2))(2,2)(1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9" },
    ]),
    ("(0)(1^(2))(2,2)(1,1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,18,8" },
    ]),
    ("(0)(1^(2))(2,2)(3^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,18,26" },
    ]),
    ("(0)(1^(2))(2,2,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,19" },
    ]),
    ("(0)(1^(2))(2,2,1^(3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,19,9" },
    ]),
    ("(0)(1^(2))(2,2,1^(3))(2,1^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,19,13,9" },
    ]),
    ("(0)(1^(2))(2,2,1^(3))(2,2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,19,18" },
    ]),
    ("(0)(1^(2))(2,2,1^(3))(3,3,1^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,19,27,9" },
    ]),
    ("(0)(1^(2))(2,2,2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,4,9,13,20" },
    ]),
    ("(0)(1^(2))(2^(3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,5" },
    ]),
    ("(0)(1^(2))(2^(3))(3^(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,10,14,5" },
    ]),
    ("(0)(1^(2),1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11" },
    ]),
    ("(0)(1^(2),1)(2^(3),2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,18" },
    ]),
    ("(0)(1^(2),1)(1^(3),1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,11" },
    ]),
    ("(0)(1^(2),1)(2^(3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,16,5" },
    ]),
    ("(0)(1^(2),1)(2,2)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,16" },
    ]),
    ("(0)(1^(2),1)(2^(3),1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,17" },
    ]),
    ("(0)(1^(2),1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,19" },
    ]),
    ("(0)(1^(2),1,1,1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,11,19,35" },
    ]),
    ("(0)(1^(2),1^(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,12" },
    ]),
    ("(0)(1^(2),1^(2),1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,12,16,24" },
    ]),
    ("(0)(1^(2),1^(2),1^(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,7,12,16,25" },
    ]),
    ("(0)(1^(2)(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8" },
    ]),
    ("(0)(1^(2)(2),1)", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,7,11" },
    ]),
    ("(0)(1^(2)(2),1^(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,7,12" },
    ]),
    ("(0)(1^(2)(2),1^(2)(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,7,12,17" },
    ]),
    ("(0)(1^(2)(2)(2))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,8" },
    ]),
    ("(0)(1^(2)(3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9" },
    ]),
    ("(0)(1^(2)(3)(4))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9,10" },
    ]),
    ("(0)(1^(2)(3,3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9,11" },
    ]),
    ("(0)(1^(2)(3,3,3))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9,11,15" },
    ]),
    ("(0)(1^(2)(3^(4)))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9,12" },
    ]),
    ("(0)(1^(2)(3^(4)(5)))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,9,12,15,16,19" },
    ]),
    ("(0)(1^(2,1))", &[
        Reading { label: "1-Y", latex: false, term: "1,3,4,2,5,8,10" },
    ]),
];

/// The hand-written readings for a key, on their own.
pub fn extra_readings(key: &str) -> Vec<(&'static str, bool, String)> {
    merge_extra(Vec::new(), key)
}

/// The automatic readings with the hand-written ones folded in.
pub fn readings_with_extra(n: &Btbm, e: &Expr, key: &str) -> Vec<(&'static str, bool, String)> {
    merge_extra(readings_for(n, e), key)
}

fn merge_extra(
    mut rows: Vec<(&'static str, bool, String)>,
    key: &str,
) -> Vec<(&'static str, bool, String)> {
    let Some((_, extra)) = EXTRA_READINGS.iter().find(|(k, _)| *k == key) else {
        return rows;
    };
    for reading in extra.iter() {
        // An empty term on a label the table already has means "drop it": the
        // reading was wrong and there is nothing to put in its place.
        if reading.term.is_empty() {
            rows.retain(|(label, _, _)| *label != reading.label);
            continue;
        }
        let row = (reading.label, reading.latex, reading.term.to_string());
        match rows.iter_mut().find(|(label, _, _)| *label == reading.label) {
            Some(slot) => *slot = row,
            None => rows.push(row),
        }
    }
    rows
}

/// Hand-written part: `(ordinal, title, description)`. Edit this list, then
/// re-run `cargo run --release --example gen_milestones`.
///
/// Everything below SHO is imported from the ω-Y LNGI milestone table: each
/// landmark is the Y → BMS image of that table's ω-Y sequence, so the strings
/// are derived rather than copied. Above SHO the list is the nesting chain
/// `family(3)`, `family(4)`, … whose supremum is the notation's top element;
/// the final entry, `LIMIT`, is that top element itself — a real ordinal that
/// the walk never arrives at.
///
/// Ordinals are written in the level reading, which is how the Y → BMS
/// conversion writes them. The generator converts each to the walk's own
/// (parent) reading and stores that, so the table's strings are exactly what
/// the app shows at that moment.
pub const ENTRIES: &[(&str, &str, &str)] = &[
    // Written by the developer page (dev.html); edit there and press Rebuild.
    ("", "", "Zero — the empty matrix, where the walk begins."),
    ("(0)", "", "First successor ordinal — the journey starts here."),
    ("(0)(0)", "", "Second successor ordinal — and it already takes over 30 minutes to reach the next one."),
    ("(0)(0)(0)", "", "Fun fact: this site has changed a lot since the first release, but it still keeps its style."),
    ("(0)(1)", "", "First limit ordinal — still painfully slow."),
    ("(0)(1)(0)", "", "First successor of the first limit ordinal. Reaching a new ordinal now takes hours."),
    ("(0)(1)(1)", "", "Growth rate of Conway's Chained Array Notation."),
    ("(0)(1)(2)", "", "Growth rate of linear array notations."),
    ("(0)(1,1)", "", "SCO (Small Cantor Ordinal) — the proof-theoretic ordinal of Peano arithmetic, and the limit of PrSS."),
    ("(0)(1,1)(2,1)", "", "CO (Cantor Ordinal)."),
    ("(0)(1,1)(2,1)(2,1)", "", "LCO (Large Cantor Ordinal)."),
    ("(0)(1,1)(2,1)(2,1)(2,1)", "", "Getting closer and closer."),
    ("(0)(1,1)(2,1)(3)", "", "HCO (Hyper Cantor Ordinal) — the limit of LPrSS."),
    ("(0)(1,1)(2,1)(3,1)", "", "FSO (Feferman–Schütte Ordinal) — the limit of transfinite LPrSS."),
    ("(0)(1,1)(2,1)(3,1)(2,1)", "", "First Γ fixed point: Γ<sub>Γ<sub>Γ<sub>…</sub></sub></sub>."),
    ("(0)(1,1)(2,1)(3,1)(3,1)", "", "AO (Ackermann Ordinal) — the limit of WFSS."),
    ("(0)(1,1)(2,1)(3,1)(4)", "", "SVO (Small Veblen Ordinal)."),
    ("(0)(1,1)(2,1)(3,1)(4,1)", "", "LVO (Large Veblen Ordinal)."),
    ("(0)(1,1)(2,2)", "", "BHO (Bachmann–Howard Ordinal) — where Ordinal Markup starts collapsing."),
    ("(0)(1,1)(2,2)(3,1)(4,4)", "", "The endgame of Ordinal Markup."),
    ("(0)(1,1)(2,2)(3,3)", "", "Fun fact: this one was reached while its author was on a plane back to Hong Kong."),
    ("(0)(1,1)(2,2)(3,3)(4,4)", "", "Why is this taking so long?"),
    ("(0)(1,1,1)", "", "BO — the start of the 3-row BMS series."),
    ("(0)(1,1,1)(2,1)", "", "BMS upgrades here; ω-Y and everything above 1-Y does not, yet."),
    ("(0)(1,1,1)(2,1)(1,1)(4,4,4)(5,4)", "", "The true upgrading point of Y sequences."),
    ("(0)(1,1,1)(2,1)(1,1,1)", "", "The result of the upgrade."),
    ("(0)(1,1,1)(2,1)(3,3)", "", "TFBO — the limit of Buchholz's OCF."),
    ("(0)(1,1,1)(2,1,1)", "", "Structurally close to ζ<sub>0</sub>, somehow."),
    ("(0)(1,1,1)(2,1,1)(3,1)", "", "BIO (Bird's Ordinal) — BAN's limit before extensions."),
    ("(0)(1,1,1)(2,1,1)(3,1)(2)", "", "EBO (Extended Buchholz Ordinal) — ψ(I) in BOCF; past here the uncountables are inaccessible ones."),
    ("(0)(1,1,1)(2,1,1)(3,1)(4,4)", "", "JO (Jäger's Ordinal)."),
    ("(0)(1,1,1)(2,1,1)(3,1)(4,4,4)", "", "Vacant separation point of TBOCF; non-upgrading OCFs are not intuitive."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)", "", "SIO (Small Inaccessible Ordinal) — the limit of Address Notation."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3)", "", "BMSO (Battery Management System Ordinal) — the limit of aSAN-1."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1)(5)", "", "Inaccessible SVO; Mahlo OCFs take over from here."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1)(5,5)", "", "SRO."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1,1)", "", "SMO (Small Mahlo Ordinal)."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1,1)(2,1,1)(6,1,1)(6,1)(8,8)", "", ""),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(4)", "", "Where weakly compact collapsing functions begin."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(4,1)(5,5)", "", ""),
    ("(0)(1,1,1)(2,2)", "", "SSO (Small Stegert Ordinal) — the limit of pDAN."),
    ("(0)(1,1,1)(2,2)(2,2)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3,3)", "", "Filler."),
    ("(0)(1,1,1)(2,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(2,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3)", "", "SDO (Small Dropping Ordinal) — structurally like φ(ω,0); the limit of SAN, TON (projected) and more."),
    ("(0)(1,1,1)(2,2,1)(3,2)", "", "Limit of a certain exUNOCF extension. Eww."),
    ("(0)(1,1,1)(2,2,1)(3,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)", "", "Upper bound of the weakly compact OCFs."),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,3)(5,3)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,5)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,2,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,3)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)", "", "Limit of degrees of reflection (optimal)."),
    ("(0)(1,1,1)(2,2,2)(3,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,1,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2)", "", "Limit of Weak Superstrong Binary Notation III (if it is even defined)."),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)(4,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)(5,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,3,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)(5,5,5)", "", ""),
    ("(0)(1,1,1,1)", "", "TSSO (Trio Sequence System Ordinal) — 4-row matrices begin."),
    ("(0)(1,1,1,1)(1,1,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,1)(1,1,1)", "", "The first layer of upgrading."),
    ("(0)(1,1,1,1)(2,1)(1,1,1,1)", "", "Two layers of upgrading across two rows; the second is weaker."),
    ("(0)(1,1,1,1)(2,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,1,1)(1,1,1,1)", "", "2,1,1 can be upgraded by 1,1,1,1 as well."),
    ("(0)(1,1,1,1)(2,1,1)(3,3,3,3)", "", ""),
    ("(0)(1,1,1,1)(2,1,1,1)", "", "ω<sup>2</sup> entries of TSS."),
    ("(0)(1,1,1,1)(2,1,1,1)(3)(2)", "", ""),
    ("(0)(1,1,1,1)(2,1,1,1)(3,1,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2)", "", "QSS 0.5th Back Gear Ordinal (who named these?)."),
    ("(0)(1,1,1,1)(2,2,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,1)(1,1,1,1)", "", "Drop rows 2–4 and another upgrade appears."),
    ("(0)(1,1,1,1)(2,2,1,1)", "", "No new kinds of upgrade — for now."),
    ("(0)(1,1,1,1)(2,2,1,1)(3,3,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,1,1)(3,3,1,1)(4,4,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,2)", "", "QSS 1st Back Gear Ordinal."),
    ("(0)(1,1,1,1)(2,2,2,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,2,2)", "", "Big Omega Back Ordinal."),
    ("(0)(1,1,1,1)(2,2,2,2)(3,3,3,3)", "", ""),
    ("(0)(1,1,1,1,1)", "", "QSSO (Quad Sequence System Ordinal)."),
    ("(0)(1,1,1,1,1)(0)", "", "hyp_cos muted a few people for 1 minute, 2 minutes, 4 minutes, and so on — one term of this ordinal's Y sequence per person."),
    ("(0)(1,1,1,1,1)(2,1,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,2)", "", ""),
    ("(0)(1,1,1,1,1,1)", "", "QiSSO (Quin Sequence System Ordinal)."),
    ("(0)(1,1,1,1,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1,1,1,1)", "", ""),
    ("(0)(1^(2))", "", "SHO (Small Hydra Ordinal). It may be the PTO of Z<sub>2</sub>, but that is unproven."),
    ("(0)(1^(2))(0)", "", "SHO+1."),
    ("(0)(1^(2))(1)", "", ""),
    ("(0)(1^(2))(1)(3,3)", "", ""),
    ("(0)(1^(2))(1)(3^(4))", "", ""),
    ("(0)(1^(2))(1,1)", "", ""),
    ("(0)(1^(2))(1,1)(3,3)", "", ""),
    ("(0)(1^(2))(1,1)(3,3,3)", "", ""),
    ("(0)(1^(2))(1,1)(3^(4))", "", ""),
    ("(0)(1^(2))(1,1,1)", "", ""),
    ("(0)(1^(2))(1^(3))", "", ""),
    ("(0)(1^(2))(2)", "", ""),
    ("(0)(1^(2))(2)(1^(4))", "", ""),
    ("(0)(1^(2))(2,1)", "", ""),
    ("(0)(1^(2))(2,1)(1,1)", "", ""),
    ("(0)(1^(2))(2,1,1)", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)(4^(5))", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)(4^(5))(5,4,1)(6,6)", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)(4^(5))(5,4,1)(6^(7))", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)(4^(5))(5,4,1)(6^(7))(7,6,1)", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1)(4^(5))(5,4,4)", "", ""),
    ("(0)(1^(2))(2,1,1)(1,1,1,1)", "", ""),
    ("(0)(1^(2))(2,1,1)(1^(4))", "", ""),
    ("(0)(1^(2))(2,1,1)(2)", "", ""),
    ("(0)(1^(2))(2,1,1)(3^(4))", "", ""),
    ("(0)(1^(2))(2,1,1,1)", "", ""),
    ("(0)(1^(2))(2,1^(3))", "", ""),
    ("(0)(1^(2))(2,1^(3))(1^(4))", "", ""),
    ("(0)(1^(2))(2,1^(3))(2)", "", ""),
    ("(0)(1^(2))(2,2)", "", ""),
    ("(0)(1^(2))(2,2)(1,1)", "", ""),
    ("(0)(1^(2))(2,2)(1,1,1)", "", ""),
    ("(0)(1^(2))(2,2)(1^(4))", "", ""),
    ("(0)(1^(2))(2,2)(3^(4))", "", ""),
    ("(0)(1^(2))(2,2,1)", "", ""),
    ("(0)(1^(2))(2,2,1^(3))", "", ""),
    ("(0)(1^(2))(2,2,1^(3))(2,1^(4))", "", ""),
    ("(0)(1^(2))(2,2,1^(3))(2,2)", "", ""),
    ("(0)(1^(2))(2,2,1^(3))(3,3,1^(4))", "", ""),
    ("(0)(1^(2))(2,2,2)", "", ""),
    ("(0)(1^(2))(2^(3))", "", ""),
    ("(0)(1^(2))(2^(3))(3^(4))", "", ""),
    ("(0)(1^(2),1)", "", ""),
    ("(0)(1^(2),1)(1^(3),1)", "", ""),
    ("(0)(1^(2),1)(2,2)", "", ""),
    ("(0)(1^(2),1)(2^(3))", "", ""),
    ("(0)(1^(2),1)(2^(3),1)", "", ""),
    ("(0)(1^(2),1)(2^(3),2)", "", ""),
    ("(0)(1^(2),1,1)", "", ""),
    ("(0)(1^(2),1,1,1)", "", ""),
    ("(0)(1^(2),1^(2))", "", ""),
    ("(0)(1^(2),1^(2),1)", "", ""),
    ("(0)(1^(2),1^(2),1^(2))", "", ""),
    ("(0)(1^(2)(2))", "", ""),
    ("(0)(1^(2)(2),1)", "", ""),
    ("(0)(1^(2)(2),1^(2))", "", ""),
    ("(0)(1^(2)(2),1^(2)(2))", "", ""),
    ("(0)(1^(2)(2)(2))", "", ""),
    ("(0)(1^(2)(3))", "", ""),
    ("(0)(1^(2)(3)(4))", "", ""),
    ("(0)(1^(2)(3,3))", "", ""),
    ("(0)(1^(2)(3,3,3))", "", ""),
    ("(0)(1^(2)(3^(4)))", "", ""),
    ("(0)(1^(2)(3^(4)(5)))", "", ""),
    ("(0)(1^(2,1))", "", ""),
    ("(0)(1^(2,2),1,1)", "", "1-Y(1,3,9)."),
    ("(0)(1^(2,2)(3,2)(4))", "", "MHO (Medium Hydra Ordinal) — the limit of ω-Y."),
    ("(0)(1^(2^(3)))", "", "One more nesting level: the next member of the chain whose supremum is LIMIT."),
    ("(0)(1^(2^(3^(4))))", "", "Another level."),
    ("(0)(1^(2^(3^(4^(5)))))", "", "Another level."),
    ("(0)(1^(2^(3^(4^(5^(6))))))", "", "Another level. The chain never ends; LIMIT is what it climbs toward."),
    ("(0)(1^(2^(3^(4^(5^(6^(7)))))))", "", ""),
    ("(0)(1^(2^(3^(4^(5^(6^(7^(8))))))))", "", ""),
    ("LIMIT", "", ""),
];

include!("milestones_gen.rs");

/// The table as JSON, for the developer page.
///
/// JSON has no infinity, so a non-finite value is written as `null`; the page
/// restores it. Writing `inf` here would produce a document `JSON.parse` rejects.
pub fn json() -> String {
    fn esc(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    fn num(x: f64) -> String {
        if x.is_finite() {
            format!("{x}")
        } else {
            "null".to_string()
        }
    }

    let mut out = String::from("[");
    for (i, m) in MILESTONES.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!(
            "{{\"ordinal\":\"{}\",\"title\":\"{}\",\"desc\":\"{}\",\"value\":{},\"atMs\":{}}}",
            esc(m.ordinal),
            esc(m.title),
            esc(m.desc),
            num(m.value),
            num(m.at_ms)
        ));
        // The readings are baked in by the generator; splice them in before
        // the object's closing brace.
        out.pop();
        out.push_str(r#","readings":["#);
        for (k, reading) in m.readings.iter().enumerate() {
            if k > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                r#"{{"label":"{}","latex":{},"term":"{}"}}"#,
                esc(reading.label),
                reading.latex,
                esc(reading.term)
            ));
        }
        out.push_str("]}");
    }
    out.push(']');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{get_time_inv, Btbm, DisplayType, Lngi};

    #[test]
    fn generated_table_is_current() {
        let n = Btbm::new();
        let l = Lngi::new();
        assert_eq!(MILESTONES.len(), ENTRIES.len(), "regenerate the table");
        for (i, (ordinal, title, desc)) in ENTRIES.iter().enumerate() {
            if ordinal.is_empty() {
                // Zero sits just before the walk's first ordinal, `(0)`.
                assert_eq!(MILESTONES[i].value, 2.0);
                assert_eq!(MILESTONES[i].at_ms, 0.0);
                continue;
            }
            if *ordinal == "LIMIT" {
                assert!(MILESTONES[i].value.is_infinite(), "LIMIT must be unreachable");
                continue;
            }
            let parsed = n.from_display(ordinal).unwrap_or_else(|_| panic!("bad ordinal {ordinal}"));
            // The list is in the level reading; the walk is in the parent one.
            let e = n.convert_from_layer(&parsed);
            let value = l
                .reach_value(&e)
                .unwrap_or_else(|| panic!("{ordinal} lies past the end of the sequence"));
            let m = &MILESTONES[i];
            assert_eq!(m.ordinal, n.display(&e, DisplayType::Plain), "regenerate the table");
            assert_eq!(m.title, *title);
            assert_eq!(m.desc, *desc);
            // The readings are baked in by the generator; recompute them and
            // compare, so a change to a converter cannot leave the table stale.
            let want = readings_with_extra(&n, &e, m.ordinal);
            assert_eq!(m.readings.len(), want.len(), "regenerate the table");
            for (got, (label, latex, term)) in m.readings.iter().zip(want.iter()) {
                assert_eq!(got.label, *label);
                assert_eq!(got.latex, *latex);
                assert_eq!(got.term, term.as_str());
            }
            assert!(
                (m.value - value).abs() < 1e-9,
                "{ordinal}: value {} != {value}, regenerate",
                m.value
            );
            let at = get_time_inv(value);
            assert!(
                (m.at_ms - at).abs() < 1.0,
                "{ordinal}: time {} != {at}, regenerate",
                m.at_ms
            );
        }
    }

    /// The JSON must be parseable: every numeric field either parses or is
    /// `null`. (Descriptions may legitimately say "infinite", so a substring
    /// search for `inf` would be a false alarm.)
    #[test]
    fn json_is_well_formed() {
        let text = json();
        assert!(text.starts_with('[') && text.ends_with(']'));
        assert!(!text.contains(":NaN") && !text.contains(":inf"));
        for field in ["\"value\":", "\"atMs\":"] {
            for rest in text.split(field).skip(1) {
                let token = rest.split([',', '}']).next().unwrap();
                assert!(
                    token == "null" || token.parse::<f64>().is_ok(),
                    "bad {field} literal: {token}"
                );
            }
        }
        assert_eq!(text.matches("\"ordinal\":").count(), MILESTONES.len());
    }

    /// The table is ordered: later ordinals come later in the sequence.
    #[test]
    fn table_is_monotone() {
        for pair in MILESTONES.windows(2) {
            assert!(
                pair[0].value < pair[1].value,
                "{} >= {}",
                pair[0].ordinal,
                pair[1].ordinal
            );
        }
    }
}


