//! Landmarks along the sequence.
//!
//! The list below is the source of truth: an ordinal and its text. The value
//! and the time are *derived* — `examples/gen_milestones.rs` resolves each
//! ordinal by bisection and writes `milestones_gen.rs`, which is committed and
//! checked by [`tests::generated_table_is_current`]. Nothing is searched at
//! runtime.

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
    ("(0)(1,1)(2,1)(3,1)(4,0)", "", "SVO (Small Veblen Ordinal)."),
    ("(0)(1,1)(2,1)(3,1)(4,1)", "", "LVO (Large Veblen Ordinal)."),
    ("(0)(1,1)(2,2)", "", "BHO (Bachmann–Howard Ordinal) — where Ordinal Markup starts collapsing."),
    ("(0)(1,1)(2,2)(3,1)(4,2)", "", "The endgame of Ordinal Markup."),
    ("(0)(1,1)(2,2)(3,3)", "", "Fun fact: this one was reached while its author was on a plane back to Hong Kong."),
    ("(0)(1,1)(2,2)(3,3)(4,4)", "", "Why is this taking so long?"),
    ("(0)(1,1,1)", "", "BO — the start of the 3-row BMS series."),
    ("(0)(1,1,1)(2,1)", "", "BMS upgrades here; ω-Y and everything above 1-Y does not, yet."),
    ("(0)(1,1,1)(2,1)(1,1)(2,2,1)(3,2)", "", "The true upgrading point of Y sequences."),
    ("(0)(1,1,1)(2,1)(1,1,1)", "", "The result of the upgrade."),
    ("(0)(1,1,1)(2,1)(3,2)", "", "TFBO — the limit of Buchholz's OCF."),
    ("(0)(1,1,1)(2,1,1)", "", "Structurally close to ζ<sub>0</sub>, somehow."),
    ("(0)(1,1,1)(2,1,1)(3,1)", "", "BIO (Bird's Ordinal) — BAN's limit before extensions."),
    ("(0)(1,1,1)(2,1,1)(3,1)(2)", "", "EBO (Extended Buchholz Ordinal) — ψ(I) in BOCF; past here the uncountables are inaccessible ones."),
    ("(0)(1,1,1)(2,1,1)(3,1)(4,2)", "", "JO (Jäger's Ordinal)."),
    ("(0)(1,1,1)(2,1,1)(3,1)(4,2,1)", "", "Vacant separation point of TBOCF; non-upgrading OCFs are not intuitive."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)", "", "SIO (Small Inaccessible Ordinal) — the limit of Address Notation."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3)", "", "BMSO (Battery Management System Ordinal) — the limit of aSAN-1."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1)(4)", "", "Inaccessible SVO; Mahlo OCFs take over from here."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1)(4,2)", "", "SRO."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1,1)", "", "SMO (Small Mahlo Ordinal)."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(3,1,1)(2,1,1)(3,1,1)(3,1)(4,2)", "", ""),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(4)", "", "Where weakly compact collapsing functions begin."),
    ("(0)(1,1,1)(2,1,1)(3,1,1)(4,1)(5,2)", "", ""),
    ("(0)(1,1,1)(2,2)", "", "SSO (Small Stegert Ordinal) — the limit of pDAN."),
    ("(0)(1,1,1)(2,2,0)(2,2,0)", "", ""),
    ("(0)(1,1,1)(2,2,0)(3,3,0)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3,1)", "", "Filler."),
    ("(0)(1,1,1)(2,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(2,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3)", "", "SDO (Small Dropping Ordinal) — structurally like φ(ω,0); the limit of SAN, TON (projected) and more."),
    ("(0)(1,1,1)(2,2,1)(3,2,0)", "", "Limit of a certain exUNOCF extension. Eww."),
    ("(0)(1,1,1)(2,2,1)(3,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)", "", "Upper bound of the weakly compact OCFs."),
    ("(0)(1,1,1)(2,2,1)(3,3,0)(4,3,0)(5,3,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,0)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,0)(4,4,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,0)(4,4,1)(5,5,1)(6,5,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,0)(4,4,1)(5,5,1)(6,6,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,2,1)(4,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(3,3,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,0,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,3,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,0)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)", "", "Limit of degrees of reflection (optimal)."),
    ("(0)(1,1,1)(2,2,2)(3,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,0,0)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,1,0)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,1,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2)", "", "Limit of Weak Superstrong Binary Notation III (if it is even defined)."),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)(4,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,2,2)(4,2,2)(5,2,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,0)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,0,0)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,3,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,0)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)(5,5,5)", "", ""),
    ("(0)(1,1,1,1)", "", "TSSO (Trio Sequence System Ordinal) — 4-row matrices begin."),
    ("(0)(1,1,1,1)(1,1,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,1,0,0)(1,1,1,0)", "", "The first layer of upgrading."),
    ("(0)(1,1,1,1)(2,1,0,0)(1,1,1,1)", "", "Two layers of upgrading across two rows; the second is weaker."),
    ("(0)(1,1,1,1)(2,1,1,0)", "", ""),
    ("(0)(1,1,1,1)(2,1,1,0)(1,1,1,1)", "", "2,1,1 can be upgraded by 1,1,1,1 as well."),
    ("(0)(1,1,1,1)(2,1,1,0)(3,2,2,1)", "", ""),
    ("(0)(1,1,1,1)(2,1,1,1)", "", "ω<sup>2</sup> entries of TSS."),
    ("(0)(1,1,1,1)(2,1,1,1)(3,0,0,0)(2,0,0,0)", "", ""),
    ("(0)(1,1,1,1)(2,1,1,1)(3,1,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,0,0)", "", "QSS 0.5th Back Gear Ordinal (who named these?)."),
    ("(0)(1,1,1,1)(2,2,1,0)", "", ""),
    ("(0)(1,1,1,1)(2,2,1,0)(1,1,1,1)", "", "Drop rows 2–4 and another upgrade appears."),
    ("(0)(1,1,1,1)(2,2,1,1)", "", "No new kinds of upgrade — for now."),
    ("(0)(1,1,1,1)(2,2,1,1)(3,3,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,1,1)(3,3,1,1)(4,4,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,2,0)", "", "QSS 1st Back Gear Ordinal."),
    ("(0)(1,1,1,1)(2,2,2,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,2,2)", "", "Big Omega Back Ordinal."),
    ("(0)(1,1,1,1)(2,2,2,2)(3,3,3,3)", "", ""),
    ("(0)(1,1,1,1,1)", "", "QSSO (Quad Sequence System Ordinal)."),
    ("(0)(1,1,1,1,1)(0)", "", "hyp_cos muted a few people for 1 minute, 2 minutes, 4 minutes, and so on — one term of this ordinal's Y sequence per person."),
    ("(0)(1,1,1,1,1)(2,1,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,1,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,0)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,2)", "", ""),
    ("(0)(1,1,1,1,1,1)", "", "QiSSO (Quin Sequence System Ordinal)."),
    ("(0)(1,1,1,1,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1,1,1,1)", "", ""),
    ("(0)(1^(2))", "", "SHO (Small Hydra Ordinal). It may be the PTO of Z<sub>2</sub>, but that is unproven."),
    ("(0)(1^(2,2),1,1)", "", "1-Y(1,3,9)."),
    ("(0)(1^(2,2)(3,2)(4))", "", "MHO (Medium Hydra Ordinal) — the limit of ω-Y."),
    ("(0)(1^(2^(3)))", "", "One more nesting level: the next member of the chain whose supremum is LIMIT."),
    ("(0)(1^(2^(3^(4))))", "", "Another level."),
    ("(0)(1^(2^(3^(4^(5)))))", "", "Another level."),
    ("(0)(1^(2^(3^(4^(5^(6))))))", "", "Another level. The chain never ends; LIMIT is what it climbs toward."),
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


