# BTBMS LNGI

A real-time visualization of the LNGI walk over **BTBMS**, with conversions to
weaker notations.

LNGI turns wall-clock time into an ordinal: a fixed curve maps elapsed
milliseconds to a value, and the value names the ordinal the walk is currently
standing on. Here the notation is BTBMS — a layered Bashicu-like matrix — so the
walk climbs through `(0)`, `(0)(1)`, `(0)(1)(1)`, … and the display keeps up at
60 fps.

## What you see

- **Current ordinal** — the term at the current moment, rendered in whichever
  structure you pick from the display-mode menu: BTBMS itself, or one of the
  weaker notations it converts into. Converted forms have a ceiling; past it the
  BTBMS form is shown instead of a wrong answer.
- **Milestones** — 127 landmarks from zero up to `LIMIT`, each with its arrival
  time, a description, and a progress bar. Switch between the upcoming-landmark
  bars and the full list. Everything below SHO is imported from the ω-Y LNGI
  table as the Y → BMS image of each ω-Y sequence.
- **Search** — type an ordinal and get the moment it is reached; or pick a day,
  month or year and see the roundest ordinal inside it.
- **Rewind** — drag the odometer-style sliders to move the view back, or type a
  date (BC included). A signed speed slider freezes time at 0×, runs it
  backwards below that, and speeds it up to 10⁸×.
- **Panels** — any number of extra views of the same ordinal, resizable in both
  directions.
- English and Chinese, light and dark, and every setting is kept in
  `localStorage`.

## How it works

The curve is

```
value(t) = log10(1 + t / 864000) / 2 + 2        (up to SHO, value 4)
value(t) = 4 + (raw - 4) / 0.5                  (above SHO)
```

with `t` in milliseconds since the fixed start. The second line is the
speed-up: SHO keeps its moment, and everything past it runs at twice the
reference pace. A value `m ∈ [k, k+1)` falls
inside the interval `[family(k-1), family(k))`, and the ordinals inside that
interval are enumerated by recursive level decomposition. `family(k)` is exactly
`Limit[k-1]`, the `k`-th term of the fundamental sequence of the notation's top
element, so walking along `family` means climbing toward `LIMIT` itself.

The milestone table is generated, not hand-timed: edit `ENTRIES` in
`src/rust/milestones.rs`, then

```sh
cargo run --release --example gen_milestones
```

which bisects the curve for each entry and writes `src/rust/milestones_gen.rs`.
A test keeps the committed table in sync with the source list.

## Development

Requirements: Rust with the `wasm32-unknown-unknown` target, `wasm-pack`, and
pnpm.

```sh
pnpm install
pnpm wasm     # build the Rust core into src/wasm
pnpm dev      # dev server
pnpm build    # type-check + production build
pnpm preview  # serve the build
```

Tests run on the host target (the wasm bindings are behind `cfg(target_arch =
"wasm32")`, so they stay out of the way):

```sh
cargo test --release --lib
```

The Rust core lives in `src/rust/`; the frontend is Vue 3 with the Vapor
renderer.

## Credits

Inspired by [ω-Y LNGI](https://mrredsharkfan.github.io/w-Y-global-lngi/main.html)
by Termiunsfinity — a much larger project built on a different notation, and the
reason this one exists.

## License

MIT. See [LICENSE](LICENSE).
