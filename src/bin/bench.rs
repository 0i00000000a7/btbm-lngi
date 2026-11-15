//! Timings for the walk and the render at a given point on the curve.
//!
//! ```text
//! cargo run --release --bin bench            # 1, 10, 100, 1k, 10k years
//! cargo run --release --bin bench 1e3 1e4    # or the years you care about
//! ```
//!
//! Each sample walks a fresh `Lngi` forward a frame at a time, so the numbers
//! include whatever the memoization caches do in a running app: the first
//! calls pay for the cache and the rest ride on it.
//!
//! Set `DUMP=1` to also print the layered form of each ordinal, which is what
//! the conversion in `layer.rs` is checked against.

use std::time::Instant;

use btbm_core::{Btbm, DisplayType, Lngi};

const YEAR_MS: f64 = 365.25 * 86_400_000.0;
/// One frame at 60 Hz, which is the interval the app actually steps by.
const FRAME_MS: f64 = 1000.0 / 60.0;
const WARMUP: usize = 16;
const SAMPLES: usize = 64;

fn main() {
    let years: Vec<f64> = std::env::args()
        .skip(1)
        .filter_map(|arg| arg.parse().ok())
        .collect();
    let years = if years.is_empty() {
        vec![1.0, 10.0, 100.0, 1000.0, 10000.0]
    } else {
        years
    };

    println!(
        "{:>8}  {:>7}  {:>7}  {:>7}  {:>7}  {:>8}  {:>8}",
        "years", "plain", "step", "parse", "layer", "display", "total"
    );
    for years in years {
        measure(years);
    }
}

fn measure(years: f64) {
    let mut lngi = Lngi::new();
    let start = years * YEAR_MS;

    // Warm the caches the way a running app has them.
    let mut expr = lngi.step(start).expr;
    for i in 1..WARMUP {
        expr = lngi.step(start + i as f64 * FRAME_MS).expr;
    }

    let step = time(SAMPLES, |i| {
        lngi.step(start + (WARMUP + i) as f64 * FRAME_MS);
    });

    let btbm = Btbm::new();
    let plain = btbm.display(&expr, DisplayType::Plain);

    let parsed = time(SAMPLES, |_| {
        btbm.from_display(&plain).expect("the plain form parses");
    });
    let layered = time(SAMPLES, |_| {
        btbm.convert_to_layer(&expr);
    });
    let displayed = time(SAMPLES, |_| {
        btbm.display(&expr, DisplayType::Html);
    });

    println!(
        "{:>8.0}  {:>7}  {:>6.2}ms  {:>6.2}ms  {:>6.2}ms  {:>7.2}ms  {:>7.2}ms",
        years,
        plain.len(),
        step,
        parsed,
        layered,
        displayed,
        step + parsed + layered + displayed
    );

    if std::env::var_os("DUMP").is_some() {
        let layer = btbm.convert_to_layer(&expr);
        println!("[{}] {}", years, btbm.display(&layer, DisplayType::Plain));
    }
}

/// Mean milliseconds per call over `samples` calls.
fn time(samples: usize, mut call: impl FnMut(usize)) -> f64 {
    let start = Instant::now();
    for i in 0..samples {
        call(i);
    }
    start.elapsed().as_secs_f64() * 1000.0 / samples as f64
}
