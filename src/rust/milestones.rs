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
/// The list is the approximation chain of the notation's own limit — one
/// landmark per level, `family(1)`, `family(2)`, … — rather than assorted
/// ordinals picked by hand. The final entry, `LIMIT`, is that limit itself:
/// a real ordinal that the walk never arrives at.
pub const ENTRIES: &[(&str, &str, &str)] = &[
    // Written by the developer page (dev.html); edit there and press Rebuild.
    ("", "", "Zero — the empty matrix, where the walk begins."),
    ("(0)", "", "First successor of zero — the first landmark after the start."),
    ("(0)(0)", "", "An early successor. Landmarks this close to the start are days apart."),
    ("(0)(1)", "", "First limit ordinal. Its fundamental sequence is 0, 1, 2, ..."),
    ("(0)(1)(0)(3)", "", "First successor of the first limit ordinal."),
    ("(0)(1)(0)(3)(0)(5)", "", "Growth rate of Conway's Chained Array Notation."),
    ("(0)(1)(1)", "", "Growth rate of Extended Conway's Chained Array Notation."),
    ("(0)(1)(2)", "", "The growth rate of Linear Array Notations."),
    ("(0)(1,1)", "", "SCO (Small Cantor Ordinal) — the proof-theoretic ordinal (PTO) of Peano arithmetic, and the limit of PrSS."),
    ("(0)(1,1)(2,1)", "", "CO (Cantor Ordinal)"),
    ("(0)(1,1)(2,1)(3)(4,4)", "", "LCO (Large Cantor Ordinal)"),
    ("(0)(1,1)(2,1)(3)(4,4)(5)(6,6)", "", "A stepping stone between LCO and HCO."),
    ("(0)(1,1)(2,1)(3)(4,4)(5,4)", "", "HCO (Hyper Cantor Ordinal), also the limit of LPrSS"),
    ("(0)(1,1)(2,1)(3,1)", "", "FSO (Feferman–Schütte Ordinal), also the limit of Transfinite LPrSS"),
    ("(0)(1,1)(2,1)(3,1)(4)", "", "First &Gamma; fixed point, i.e. &Gamma;<sub>&Gamma;<sub>&Gamma;<sub>...</sub></sub></sub>"),
    ("(0)(1,1)(2,1)(3,1)(4)(5,5)(6,5)", "", "AO (Ackermann Ordinal), also the limit of WFSS (weak fundamental sequences system)"),
    ("(0)(1,1)(2,1)(3,1)(4)(5,5)(6,5)(7,5)", "", "SVO (Small Veblen Ordinal)"),
    ("(0)(1,1)(2,1)(3,1)(4,1)", "", "LVO (Large Veblen Ordinal)"),
    ("(0)(1,1)(2,2)", "", "BHO (Bachmann-Howard Ordinal). Collapsing functions become necessary from here on."),
    ("(0)(1,1)(2,2)(3,1)(4,4)(5,1)(6,6)", "", "Endgame of Ordinal Markup."),
    ("(0)(1,1)(2,2)(3,3)", "", "A filler landmark on the climb toward 3-row columns."),
    ("(0)(1,1)(2,2)(3,3)(4,4)", "", "Another filler landmark, just below the first 3-row column."),
    ("(0)(1,1,1)", "", "The start of the 3-row BMS series."),
    ("(0)(1,1,1)(2,1)(3,3,3)", "", "Where BMS itself begins upgrading; ω-Y and anything above 1-Y do not reach this point yet."),
    ("(0)(1,1,1)(2,1,1)", "", "The true upgrading point of Y sequence."),
    ("(0)(1,1,1)(2,1,1)(2,1)(4,4,4)(5,4,4)", "", "The result of that upgrade."),
    ("(0)(1,1,1)(2,1,1)(3,1)(4,4,4)(5,4,4)", "", "TFBO, Limit of Buchholz's OCF"),
    ("(0)(1,1,1)(2,2)", "", "Structurally similar to &zeta;<sub>0</sub>, though the correspondence is not obvious."),
    ("(0)(1,1,1)(2,2)(3,1,1)(4,4)", "", "Bird's Ordinal (BIO), BAN's limit before extensions."),
    ("(0)(1,1,1)(2,2)(3,2)(4)", "", "Extended Buchholz Ordinal (EBO). The uncountables here go beyond inaccessibles."),
    ("(0)(1,1,1)(2,2)(3,2)(4,1,1)(5,5)(6,5)", "", "Jager's Ordinal (JO)"),
    ("(0)(1,1,1)(2,2)(3,2)(4,2)", "", "Vacant separation point of TBOCF. The non-upgrading OCF is not intuitive to learn."),
    ("(0)(1,1,1)(2,2)(3,2)(4,2)(5,2)(6,2)(7)(8,8,8)(9,9)(10,9)(11,9)(12,9)(13,9)", "", "Second fixed point of &Pi;<sub>2</sub> (second order) arithmetic. Third order is far harder: only Arai's OCF, cOCF, Stegert's OCF and the weakly compact Rathjen OCF reach it."),
    ("(0)(1,1,1)(2,2)(3,2)(4,2)(5,2)(6,2)(7)(8,8,8)(9,9)(10,9)(11,9)(12,9)(13,9)(13,8,8)(15,15)(16,8,8)(17,17)(18,8,8)(19,19)(20,8,8)(21,21)", "", "An important point in Rathjen's OCF: the first ordinal that needs the stability function &Xi; to avoid infinite descending chains."),
    ("(0)(1,1,1)(2,2)(3,2)(4,2)(5,2)(6,2)(7,2)(8,2)(9,2)(10,1)(11,11,11)(12,12)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3)", "", "Small Inaccessible Ordinal (SIO), and the limit of Address Notation."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1)(5,5,5)(6,6)(7,5,5)(8,8)(9,1)", "", "IFP, and Limit of Caffy's bot"),
    ("(0)(1,1,1)(2,2)(3,3)(4,1)(5,5,5)(6,6)(7,7)", "", "Battery Management System Ordinal (BMSO), and limit of aSAN-1."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,1)(7,7,7)(8,8)(9,9)(10,1)(11,11,11)", "", "An inaccessible analogue of ACO."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,1)(7,7,7)(8,8)(9,9)(10,7,7)", "", "Inaccessible SVO. Mahlo OCFs take over from here."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,1,1)(7,7)", "", "SRO."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,6)", "", "Small Mahlo Ordinal (SMO)"),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,6)(7,1)(8,8,8)(9,9)(10,10)(11,8,8)(12,12)(13,8,8)(14,14)(15,1)(16,16,16)(17,17)(18,18)(19,16,16)(20,20)(21,16,16)", "", "Mahlo OCFs disagree about where this sits: some put it early, some late."),
    ("(0)(1,1,1)(2,2)(3,3)(4,1,1)(5,5)(6,6)(7,1)(8,8,8)(9,9)(10,10)(11,8,8)(12,12)(13,12)(14,8,8)(15,15)(16,15)(17,8,8)(18,18)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3)(4,2)", "", "Start of Weakly Compact OCN."),
    ("(0)(1,1,1)(2,2)(3,3)(4,3)(5,2)(6,6)(7,6)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3,3)", "", "Small Stergert Ordinal (SSO), and limit of pDAN."),
    ("(0)(1,1,1)(2,2)(3,3,3)(4,1,1)(5,5)(6,6,6)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3,3)(4,4)", "", ""),
    ("(0)(1,1,1)(2,2)(3,3,3)(4,4)(5,5,5)", "", "Filler between SSO and the next named ordinal."),
    ("(0)(1,1,1)(2,2)(3,3,3)(4,4)(5,5,5)(6,6)(7,6)(8,3)(9,9,9)(10,10)", "", ""),
    ("(0)(1,1,1)(2,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,1,1)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,2)", "", "Small Dropping Ordinal (SDO). Structurally similar to &phi;(&omega;,0); the limit of SAN, TON (projected) and several others."),
    ("(0)(1,1,1)(2,2,1)(3,3)", "", "Limit of a certain exUNOCF extension."),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)", "", "Upper bound of Weakly Compact OCFs."),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7,7)(8,8,7)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7,7)(8,8,7)(9,9)(10,10,10)(11,11,10)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7,7)(8,8,7)(9,9)(10,10,10)(11,11,10)(12,12)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7,7)(8,8,7)(9,9)(10,10,10)(11,11,10)(12,12)(13,13,13)(14,10)(15,15,15)(16,16,15)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3)(4,4,4)(5,5,4)(6,6)(7,7,7)(8,8,7)(9,9)(10,10,10)(11,11,10)(12,12)(13,13,13)(14,14,13)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,1,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,1,1)(5,5)(6,6,6)(7,7,6)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,1,1)(5,5)(6,6,6)(7,7,6)(8,8,6)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,1,1)(5,5)(6,6,6)(7,7,6)(8,8,6)(9,6,6)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,2)(5,5,5)(6,6,5)(7,7,5)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,2)(5,5,5)(6,6,5)(7,7,5)(8,6)(9,9,9)(10,10,9)(11,11,9)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,2,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,2,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,2,1)(5,5,1)(6,2,1)(7,7,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,3)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4)(5,5,5)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4)(5,5,5)(6,6,5)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4)(5,5,5)(6,6,5)(7,7,5)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,1)(3,3,1)(4,4,1)(5,5,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)", "", "Limit of Degrees of Reflection (Optimal)."),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,5)(7,7,7)(8,8,8)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,5,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,5,4)(7,7,7)(8,4)(9,9,9)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,5,5)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)", "", "Limit of Weak Superstrong Binary Notation III."),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)(7,3)(8,8,8)(9,9,9)(10,10)", "", "Limit of Aarex's Superstrong Binary Notation."),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)(7,7,7)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)(7,7,7)(8,8,8)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)(7,7,7)(8,8,8)(9,8)(10,10,10)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3)(4,4,4)(5,5,5)(6,6)(7,7,7)(8,8,8)(9,9)(10,10,10)(11,11,11)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,1)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,3)(5,5,5)(6,6,6)(7,7,6)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,1)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,1)(5,5,5)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,1)(5,5,5)(6,6,5)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,2)(4,4,2)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1)(2,2,2)(3,3,3)(4,4,4)(5,5,5)", "", ""),
    ("(0)(1,1,1,1)", "", "Trio Sequence System Ordinal (TSSO). The start of 4-row sequences, where landmarks begin arriving faster."),
    ("(0)(1,1,1,1)(2)", "", ""),
    ("(0)(1,1,1,1)(2,1)(3,3,3,3)(3)(5,5,5,5)(6,5)", "", "The first layer of upgrading."),
    ("(0)(1,1,1,1)(2,1,1)(3,3,3,3)(4,1)(5,5,5,5)", "", "Two layers of upgrading across two different rows in QSS; the second is weaker."),
    ("(0)(1,1,1,1)(2,1,1,1)", "", ""),
    ("(0)(1,1,1,1)(2,2)(2,1,1)", "", "Note that 2,1,1 can be upgraded by 1,1,1,1 as well."),
    ("(0)(1,1,1,1)(2,2)(3,3,3)", "", ""),
    ("(0)(1,1,1,1)(2,2)(3,3,3,3)", "", "&omega;<sup>2</sup> entries of TSS."),
    ("(0)(1,1,1,1)(2,2)(3,3,3,3)(4,2)(5,5,5,5)(6,1)", "", ""),
    ("(0)(1,1,1,1)(2,2)(3,3,3,3)(4,4)(5,5,5,5)", "", ""),
    ("(0)(1,1,1,1)(2,2,1)", "", "QSS 0.5th Back Gear Ordinal."),
    ("(0)(1,1,1,1)(2,2,1)(3,3,3,3)", "", ""),
    ("(0)(1,1,1,1)(2,2,1,1)(2,2,1)(4,4,4,4)", "", "Drop the 2nd, 3rd and 4th rows and another upgrade appears."),
    ("(0)(1,1,1,1)(2,2,2)", "", "No new kinds of upgrade — for now."),
    ("(0)(1,1,1,1)(2,2,2)(3,3,3)", "", ""),
    ("(0)(1,1,1,1)(2,2,2)(3,3,3)(4,4,4)", "", ""),
    ("(0)(1,1,1,1)(2,2,2)(3,3,3,3)", "", "QSS 1st Back Gear Ordinal"),
    ("(0)(1,1,1,1)(2,2,2,1)", "", ""),
    ("(0)(1,1,1,1)(2,2,2,2)", "", "Big Omega Back Ordinal"),
    ("(0)(1,1,1,1)(2,2,2,2)(3,3,3,3)", "", ""),
    ("(0)(1,1,1,1,1)", "", "Quad Sequence System Ordinal (QSSO)"),
    ("(0)(1,1,1,1,1)(0)", "", "hyp_cos muted a few people for 1 minute, 2 minutes, 4 minutes, and so on — one term of this ordinal's Y sequence per person."),
    ("(0)(1,1,1,1,1)(2,2)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2)(3,3,3,3,3)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2)(3,3,3,3,3)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,1)", "", ""),
    ("(0)(1,1,1,1,1)(2,2,2,2,2)", "", ""),
    ("(0)(1,1,1,1,1,1)", "", "Quin Sequence System Ordinal (QiSSO)"),
    ("(0)(1,1,1,1,1,1,1)", "", ""),
    ("(0)(1,1,1,1,1,1,1,1)", "", ""),
    ("(0)(1^(2))", "", "Small Hydra Ordinal (SHO). Even the weak version desyncs from ordinary DBMS. It may be the PTO of Z<sub>2</sub>, but that is unproven."),
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
    use crate::{get_time_inv, Btbm, Lngi};

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
            let e = n.from_display(ordinal).unwrap_or_else(|_| panic!("bad ordinal {ordinal}"));
            let value = l
                .search_value(&e)
                .unwrap_or_else(|| panic!("{ordinal} is not reachable"));
            let m = &MILESTONES[i];
            assert_eq!(m.ordinal, *ordinal);
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


