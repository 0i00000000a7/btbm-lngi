//! WASM bindings: expose the notation operations and the LNGI state to the frontend.
//!
//! Only compiled for the `wasm32` target (mounted in `mod.rs` behind
//! `#[cfg(target_arch = "wasm32")]`), so `cargo test` (host target) does not
//! pull in wasm-bindgen.

use std::cell::RefCell;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::bocf;
use super::convert;
use super::display::{display_marked_parts, display_parts};
use super::lngi::{get_time, get_time_inv, Lngi, START_MS};
use super::notation::Btbm;
use super::types::{DisplayType, Expr};

thread_local! {
    /// Reuse a single [`Lngi`] (the FS memoization cache survives across ticks).
    static LNGI: RefCell<Lngi> = RefCell::new(Lngi::new());
}

/// The output format: LaTeX source when KaTeX math rendering is on, HTML otherwise.
fn format_of(latex: bool) -> DisplayType {
    if latex {
        DisplayType::Latex
    } else {
        DisplayType::Html
    }
}

/// Marker appended after the last kept column when the ordinal is truncated.
fn ellipsis(ty: DisplayType) -> &'static str {
    match ty {
        DisplayType::Latex => "\\ldots",
        _ => "\u{2026}",
    }
}

/// Render an expression: `mode` picks the structure, `ty` picks the format.
///
/// In `Html` form the superscript in `(0)(1^(2))` is a real superscript
/// (`(0)(1<sup>(2)</sup>)`); in `Latex` form it is `^{...}` for KaTeX.
///
/// `max_cols` caps how many columns are rendered (`0` = no cap). A truncated
/// ordinal keeps its first `max_cols` columns followed by an ellipsis. This is
/// a display-only cap: the ordinal is still computed in full.
fn render(n: &Btbm, e: &Expr, mode: &str, ty: DisplayType, max_cols: u32) -> String {
    let limit = max_cols as usize;
    let truncated = limit > 0 && e.len() > limit;

    let prefix: Expr;
    let expr: &Expr = if truncated {
        prefix = e[..limit].to_vec();
        &prefix
    } else {
        e
    };

    let mut parts = match mode {
        "layer" => display_parts(&n.convert_to_layer(expr), ty),
        "annotated" => display_marked_parts(expr, ty, 1),
        // "index" (and any unknown mode) -> the expression as-is.
        _ => display_parts(expr, ty),
    };
    if truncated {
        parts.push(ellipsis(ty).to_string());
    }

    // Columns are joined with "\n" in LaTeX form so the frontend can typeset
    // them separately and wrap between them — a single KaTeX run is one
    // unbreakable box. KaTeX treats "\n" as whitespace, so a caller that does
    // not split still renders correctly.
    if ty == DisplayType::Latex {
        parts.join("\n")
    } else {
        parts.join("")
    }
}

/// The result of the computation step.
///
/// `ordinal` is the canonical plain form, which is mode-independent; turning it
/// into a particular structure/format is a separate step ([`parse`]). When the
/// time is earlier than [`START_MS`] it is the least nonzero ordinal `(0)`.
#[derive(Serialize)]
struct StepState {
    /// Canonical plain form of the current ordinal.
    ordinal: String,
    /// The curve's value at this moment, `2` at the start. Shown in dev only.
    value: f64,
    /// Progress toward the next ordinal, `0..1` (the bar uses `percent * 100%`).
    percent: f64,
}

/// One level bar: a level of the walk's descent.
#[derive(Serialize)]
struct LevelBarRow {
    /// Canonical plain form of the term the level runs toward.
    ordinal: String,
    /// Progress through the level, `0..1`.
    percent: f64,
    /// Elapsed milliseconds at which the level completes.
    at_ms: f64,
}

/// Compute the ordinal at `elapsed_ms` (relative to [`START_MS`]).
///
/// This runs every frame, so it carries only what the hero needs: the walk's
/// own term and its progress. The nested levels cost a display and an inverse
/// curve evaluation each, so they live in [`lngi_levels`] instead.
#[wasm_bindgen]
pub fn lngi_step(elapsed_ms: f64) -> Result<JsValue, JsValue> {
    LNGI.with(|cell| {
        let mut l = cell.borrow_mut();
        // Before the start the curve is pinned to `(0)`, whose value is 2.
        let value = if elapsed_ms < 0.0 { 2.0 } else { get_time(elapsed_ms) };
        let s = l.step(elapsed_ms);
        let n = l.btbm();
        let state = StepState {
            ordinal: n.display(&s.expr, DisplayType::Plain),
            value,
            percent: 1.0 - s.frac,
        };
        serde_wasm_bindgen::to_value(&state).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// The nested levels the walk is inside at `elapsed_ms`, outermost first.
///
/// Kept apart from [`lngi_step`] because the stack is refreshed far less often
/// than the frame, and building it is the expensive half of the step.
#[wasm_bindgen]
pub fn lngi_levels(elapsed_ms: f64) -> Result<JsValue, JsValue> {
    LNGI.with(|cell| {
        let l = cell.borrow();
        let rows: Vec<LevelBarRow> = if elapsed_ms < 0.0 {
            Vec::new()
        } else {
            let (_, levels) = l.frame(get_time(elapsed_ms));
            let n = l.btbm();
            levels
                .iter()
                .map(|lvl| LevelBarRow {
                    ordinal: n.display(&lvl.ordinal, DisplayType::Plain),
                    percent: lvl.frac,
                    at_ms: get_time_inv(lvl.end_value),
                })
                .collect()
        };
        serde_wasm_bindgen::to_value(&rows).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Parse a plain-form ordinal and render it in `mode` (LaTeX when `latex`,
/// HTML otherwise), keeping at most `max_cols` columns (`0` = no cap).
/// Throws if the input does not parse.
#[wasm_bindgen]
pub fn parse(input: &str, mode: &str, latex: bool, max_cols: u32) -> Result<String, JsValue> {
    let n = Btbm::new();
    let e = n
        .from_display(input)
        .map_err(|e| JsValue::from_str(&format!("parse failed: {e}")))?;
    Ok(render(&n, &e, mode, format_of(latex), max_cols))
}

/// The ordinal as Buchholz OCF form, as LaTeX; `None` when it is past what the
/// notation reaches (or too wide to be worth converting) and no hand-written
/// reading covers it, in which case the caller shows the ordinary BTBMS form.
#[wasm_bindgen]
pub fn bocf_latex(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    bocf::bocf_latex(&n, &e)
}

/// The hand-written BOCF bounds around an ordinal the converter declines but
/// which sits between two pinned points, as `[lower, upper]` LaTeX; `null`
/// otherwise.
///
/// The caller prints `lower < term < upper`, with the term in BTBMS: past the
/// converter's reach the exact value is unknown, but its position is not.
#[wasm_bindgen]
pub fn bocf_bounds(plain: &str) -> Result<JsValue, JsValue> {
    let n = Btbm::new();
    let Ok(e) = n.from_display(plain) else {
        return Ok(JsValue::NULL);
    };
    match bocf::bocf_bounds(&n, &e) {
        Some((lower, upper)) => serde_wasm_bindgen::to_value(&[lower, upper])
            .map_err(|e| JsValue::from_str(&e.to_string())),
        None => Ok(JsValue::NULL),
    }
}

/// The ordinal as an extended Veblen expression, as LaTeX.
///
/// `None` from the Bachmann–Howard ordinal up, where no Veblen normal form
/// exists; the caller falls back to the BOCF form.
#[wasm_bindgen]
pub fn veblen_latex(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    bocf::veblen_latex(&n, &e)
}

/// The ordinal as a 0-Y sequence, e.g. `1,2,4,8`.
///
/// `None` once a column carries a nested height — the fragment 0-Y cannot
/// express — in which case the caller shows the ordinary BTBMS form.
#[wasm_bindgen]
pub fn zero_y(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    convert::zero_y(&n, &e)
}

/// The ordinal as triangular BMS ("DBMS"), e.g. `(0)(1,1)(2,2,2)`.
///
/// `None` under the same condition as [`zero_y`].
#[wasm_bindgen]
pub fn dbms(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    convert::triangular(&n, &e)
}

/// The ordinal as a 1-Y / ω-Y sequence, e.g. `1,2,4,8`.
///
/// `None` under the same condition as [`zero_y`] — from SHO up neither
/// notation can be reached from BTBMS.
#[wasm_bindgen]
pub fn one_y(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    convert::one_y(&n, &e)
}

/// The ordinal as a Vulcaniz sequence, e.g. `1,2,4,4`.
///
/// `None` outside the fragment the reference covers (below its
/// `1,2,4,…,512` tower), where the caller falls back to the BTBMS form.
#[wasm_bindgen]
pub fn vulcaniz(plain: &str) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    convert::vulcaniz(&n, &e)
}

/// One fundamental-sequence term of an ordinal, as plain text.
///
/// The developer page uses it to grow the expansion tree; `None` for a
/// successor (no fundamental sequence) or an unparseable input.
#[wasm_bindgen]
pub fn fs_term(plain: &str, index: i32) -> Option<String> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    if !n.is_limit(&e) {
        return None;
    }
    Some(n.display(&n.fs(&e, index as i64), DisplayType::Plain))
}

/// The fundamental sequence of the whole notation's limit: `family(1)`,
/// `family(2)`, … as plain strings. Developer tooling only.
#[wasm_bindgen]
pub fn families(count: u32) -> String {
    let n = Btbm::new();
    let mut out = String::from("[");
    for k in 1..=count as i64 {
        if k > 1 {
            out.push(',');
        }
        out.push('"');
        out.push_str(&n.display(&super::family(k), DisplayType::Plain));
        out.push('"');
    }
    out.push(']');
    out
}

/// The value at which an ordinal first appears. Developer tooling only.
#[wasm_bindgen]
pub fn ordinal_value(plain: &str) -> Option<f64> {
    let n = Btbm::new();
    let e = n.from_display(plain).ok()?;
    Lngi::new().search_value(&e)
}

/// The value at which a plain-form ordinal first appears, through the shared
/// memoized walk.
///
/// The search is a bisection over the value, so a warm cache is what makes it
/// cheap: this is the interactive counterpart of [`ordinal_value`].
///
/// `layer` says which of the two marked readings the text uses. Converted
/// forms (the Y → BMS output, and anything else written the way BMS writes
/// matrices) give each entry its column's level, while the walk itself gives
/// it the parent column, so the text has to be converted before it can be
/// compared.
#[wasm_bindgen]
pub fn search_ordinal(plain: &str, layer: bool) -> Option<f64> {
    let n = Btbm::new();
    let parsed = n.from_display(plain).ok()?;
    let e = if layer {
        n.convert_from_layer(&parsed)
    } else {
        parsed
    };
    LNGI.with(|cell| cell.borrow().search_value(&e))
}

/// The elapsed time (ms) at which the walk reaches `value`.
#[wasm_bindgen]
pub fn time_at_value(value: f64) -> f64 {
    super::lngi::time_at_value(value)
}

/// The roundest value inside `[t0, t1]` (elapsed ms); `None` when the window
/// holds no ordinal.
#[wasm_bindgen]
pub fn remarkable(t0_ms: f64, t1_ms: f64) -> Option<f64> {
    super::lngi::remarkable_value(t0_ms, t1_ms)
}

/// The ordinal at `value`, as plain text.
#[wasm_bindgen]
pub fn ordinal_at_value(value: f64) -> String {
    LNGI.with(|cell| cell.borrow().ordinal_text(value))
}

/// The milestone table as JSON. Used by the developer page only.
#[wasm_bindgen]
pub fn milestones() -> String {
    super::milestones::json()
}

/// The start timestamp (milliseconds, UTC).
#[wasm_bindgen]
pub fn start_ms() -> f64 {
    START_MS
}
