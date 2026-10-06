//! WASM bindings: expose the notation operations and the LNGI state to the frontend.
//!
//! Only compiled for the `wasm32` target (mounted in `mod.rs` behind
//! `#[cfg(target_arch = "wasm32")]`), so `cargo test` (host target) does not
//! pull in wasm-bindgen.

use std::cell::RefCell;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use super::display::display;
use super::lngi::{Lngi, START_MS};
use super::notation::Btbm;
use super::types::{DisplayType, Expr};

thread_local! {
    /// Reuse a single [`Lngi`] (the FS memoization cache survives across ticks).
    static LNGI: RefCell<Lngi> = RefCell::new(Lngi::new());
}

/// Render an expression according to `mode`.
///
/// Except for `latex` (which emits LaTeX source), every mode renders HTML, so
/// the superscript in `(0)(1^(2))` is a real superscript
/// (`(0)(1<sup>(2)</sup>)`).
fn render(n: &Btbm, e: &Expr, mode: &str) -> String {
    match mode {
        "layer" => display(&n.convert_to_layer(e), DisplayType::Html),
        "annotated" => n.display_marked(e, DisplayType::Html, 1),
        "latex" => n.display(e, DisplayType::Latex),
        // "index" (and any unknown mode) -> the ordinal's HTML form as-is.
        _ => n.display(e, DisplayType::Html),
    }
}

/// The LNGI state (the result of one tick).
///
/// When the time is earlier than [`START_MS`], the ordinal is pinned to the
/// least nonzero ordinal `(0)` (value 1) and the progress is 0.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LngiState {
    /// The current ordinal (rendered per `mode`).
    ordinal: String,
    /// The next ordinal.
    next_ordinal: String,
    /// Progress toward the next ordinal, `0..1` (the bar width uses `percent * 100%`).
    percent: f64,
}

/// Compute the LNGI state at `elapsed_ms` (relative to [`START_MS`]).
#[wasm_bindgen]
pub fn lngi_state(elapsed_ms: f64, mode: &str) -> Result<JsValue, JsValue> {
    LNGI.with(|cell| {
        let mut l = cell.borrow_mut();
        let s = l.step(elapsed_ms);
        let n = l.btbm();
        let state = LngiState {
            ordinal: render(n, &s.expr, mode),
            next_ordinal: render(n, &s.next, mode),
            percent: 1.0 - s.frac,
        };
        serde_wasm_bindgen::to_value(&state).map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

/// Parse a display string and re-render it in its `mode` (validation + normalization).
#[wasm_bindgen]
pub fn parse(input: &str, mode: &str) -> Result<String, JsValue> {
    let n = Btbm::new();
    let e = n
        .from_display(input)
        .map_err(|e| JsValue::from_str(&format!("parse failed: {e}")))?;
    Ok(render(&n, &e, mode))
}

/// The start timestamp (milliseconds, UTC).
#[wasm_bindgen]
pub fn start_ms() -> f64 {
    START_MS
}
