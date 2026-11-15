//! Conversion between the layer representation and the ordinary one.

use std::cmp::Ordering;

use super::compare::{compare, height_compare, height_opt_compare, lex_compare};
use super::height::to_height;
use super::types::{Column, Entry, Expr, Height};

// ---------------------------------------------------------------------------
// To the layer representation
// ---------------------------------------------------------------------------
//
// The definition converts every entry's height into a tree (`to_height`), walks
// it, then converts the result back into an expression (`from_height`) for the
// recursive call. On a hundred-year ordinal that builds and throws away forty
// megabytes per frame, because the same subtrees are converted once per entry.
//
// A height is nothing but an expression plus a per-node `(mark, value)` pair,
// and both of those are a fold over a list of `Step`s, so the tree is carried
// as a view instead: `&Expr` plus a step list.
//
// That list is one stack shared by the whole recursion. A view only stores how
// deep into it to read (`depth`); the walk is a depth-first descent, so pushing
// on the way in and truncating on the way out keeps every live view's prefix
// intact. Nothing owns a copy of the list: `to_height` is one push,
// `from_height` rewrites one number, and comparisons read a slice.

/// One step of the transform that turns an expression into a height:
/// `if value >= base { value += delta }`.
///
/// `to_height` appends the step `(r, -r)`: it marks everything at or above `r`
/// and subtracts `r` from it. `from_height` appends nothing, it only shifts the
/// delta of the last step, which is what keeps the step list short.
#[derive(Clone, Copy, Debug)]
struct Step {
    base: i64,
    delta: i64,
}

/// An expression read as a height.
///
/// `depth` is how many steps of the shared stack apply, in order; the last step
/// is kept separately because `from_height` only rewrites it and because it is
/// the step that decides an entry's `mark`. With no last step nothing is
/// marked, which is what a plain expression looks like.
///
/// This is `Copy`: building a transform is O(1) and allocates nothing.
#[derive(Clone, Copy)]
struct HeightView<'a> {
    expr: &'a Expr,
    depth: usize,
    last: Option<Step>,
}

impl<'a> HeightView<'a> {
    fn plain(expr: &'a Expr) -> Self {
        Self {
            expr,
            depth: 0,
            last: None,
        }
    }

    /// `to_height` pushes the old `last` onto the shared stack. Every stepped
    /// view of one call uses the resulting depth, so this is done once per
    /// call, not once per column or entry.
    fn push_step(&self, steps: &mut Vec<Step>) -> usize {
        if let Some(previous) = self.last {
            steps.push(previous);
        }
        steps.len()
    }

    /// The same transform applied to a nested expression.
    fn on(&self, expr: &'a Expr) -> Self {
        Self { expr, ..*self }
    }

    /// `to_height(self, r)` for a column, given the depth from
    /// [`push_step`](Self::push_step).
    fn stepped(&self, depth: usize, r: i64) -> Self {
        Self {
            expr: self.expr,
            depth,
            last: Some(Step { base: r, delta: -r }),
        }
    }

    /// `from_height(self, r)`, which only shifts the last step's delta.
    fn from_height(&self, r: i64) -> Self {
        Self {
            expr: self.expr,
            depth: self.depth,
            last: self.last.map(|step| Step {
                base: step.base,
                delta: step.delta + r,
            }),
        }
    }
}

/// The value the steps before the last give to `value`.
fn view_prefix(steps: &[Step], value: i64) -> i64 {
    let mut v = value;
    for step in steps {
        if v >= step.base {
            v += step.delta;
        }
    }
    v
}

/// `(mark, value)` for an entry, in a single fold over the step list.
///
/// `mark` and the final value share the same condition on the prefix, so the
/// two halves that used to fold the list separately are computed together.
fn view_state(steps: &[Step], last: Option<Step>, value: i64) -> (bool, i64) {
    let prefix = view_prefix(steps, value);
    match last {
        Some(step) if prefix >= step.base => (true, prefix + step.delta),
        _ => (false, prefix),
    }
}

fn view_compare(steps: &[Step], a: &HeightView<'_>, b: &HeightView<'_>) -> Ordering {
    view_expr_compare(steps, a, a.expr, b, b.expr)
}

/// `None` is the not-yet-computed top height, greater than any real height.
fn view_opt_compare(
    steps: &[Step],
    a: Option<&HeightView<'_>>,
    b: Option<&HeightView<'_>>,
) -> Ordering {
    match (a, b) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(x), Some(y)) => view_compare(steps, x, y),
    }
}

fn view_expr_compare(
    steps: &[Step],
    a: &HeightView<'_>,
    ae: &Expr,
    b: &HeightView<'_>,
    be: &Expr,
) -> Ordering {
    lex_compare(
        ae,
        be,
        |x: &Column, y: &Column| view_column_compare(steps, a, x, b, y),
        true,
    )
}

fn view_column_compare(
    steps: &[Step],
    a: &HeightView<'_>,
    ac: &Column,
    b: &HeightView<'_>,
    bc: &Column,
) -> Ordering {
    lex_compare(
        ac,
        bc,
        |x: &Entry, y: &Entry| view_entry_compare(steps, a, x, b, y),
        true,
    )
}

fn view_entry_compare(
    steps: &[Step],
    a: &HeightView<'_>,
    ae: &Entry,
    b: &HeightView<'_>,
    be: &Entry,
) -> Ordering {
    let (am, av) = view_state(&steps[..a.depth], a.last, ae.value);
    let (bm, bv) = view_state(&steps[..b.depth], b.last, be.value);
    am.cmp(&bm)
        .then_with(|| av.cmp(&bv))
        .then_with(|| view_expr_compare(steps, a, &ae.height, b, &be.height))
}

/// A parsed layer-column entry.
#[derive(Clone, Copy)]
struct ToEntry<'a> {
    value: i64,
    height: Option<HeightView<'a>>,
}

fn vertical_increase<'a>(
    steps: &[Step],
    vert: &[HeightView<'a>],
    h_diff: &HeightView<'a>,
) -> Vec<HeightView<'a>> {
    let mut result = vert.to_vec();
    while result
        .last()
        .map_or(false, |last| view_compare(steps, last, h_diff) == Ordering::Less)
    {
        result.pop();
    }
    result.push(*h_diff);
    result
}

pub(crate) fn convert_to_layer(e: &Expr) -> Expr {
    let mut steps: Vec<Step> = Vec::new();
    let mut parsed_stack: Vec<Vec<ToEntry<'_>>> = Vec::new();
    convert_to_layer_inner(HeightView::plain(e), &mut steps, &mut parsed_stack)
}

fn convert_to_layer_inner<'a>(
    e: HeightView<'a>,
    steps: &mut Vec<Step>,
    parsed_stack: &mut Vec<Vec<ToEntry<'a>>>,
) -> Expr {
    let base = steps.len();
    let depth = e.push_step(steps);
    let l_s = parsed_stack.len();
    let mut result: Expr = Vec::new();

    for i in 0..e.expr.len() {
        let i_s = parsed_stack.len();
        let col = &e.expr[i];
        // Every entry of this column is compared at the same depth, so the
        // transform only has to be built once.
        let stepped = e.stepped(depth, i_s as i64);
        let mut parsed_col: Vec<ToEntry<'a>> = Vec::new();
        let mut current_vertical: Vec<HeightView<'a>> = Vec::new();

        for j in 0..col.len() {
            let entry = &col[j];
            // `e` is the result of a `from_height`, so the value that indexes
            // `parsed_stack` is the transformed one, not the raw one in `expr`.
            let parent = view_state(&steps[..e.depth], e.last, entry.value).1;
            let height = stepped.on(&entry.height);

            let mut kp = 0usize;
            let mut ki = 0usize;
            while kp != parsed_stack[parent as usize].len() && ki != current_vertical.len() {
                let ord = view_opt_compare(
                    &steps[..],
                    parsed_stack[parent as usize][kp].height.as_ref(),
                    Some(&current_vertical[ki]),
                );
                if ord != Ordering::Greater {
                    kp += 1;
                }
                if ord != Ordering::Less {
                    ki += 1;
                }
            }

            if kp == parsed_stack[parent as usize].len() {
                parsed_col.push(ToEntry {
                    value: 0,
                    height: Some(height),
                });
            } else {
                loop {
                    if kp == parsed_stack[parent as usize].len() {
                        parsed_col.push(ToEntry {
                            value: 0,
                            height: Some(height),
                        });
                        break;
                    }
                    let ancestor = &parsed_stack[parent as usize][kp];
                    let ancestor_height = ancestor.height.as_ref().expect("layer height");
                    if view_compare(&steps[..], ancestor_height, &height) == Ordering::Less {
                        parsed_col.push(ToEntry {
                            value: ancestor.value + 1,
                            height: Some(*ancestor_height),
                        });
                    } else {
                        parsed_col.push(ToEntry {
                            value: ancestor.value + 1,
                            height: Some(height),
                        });
                        break;
                    }
                    kp += 1;
                }
            }

            current_vertical = vertical_increase(&steps[..], &current_vertical, &height);
        }

        parsed_stack.push(parsed_col.clone());

        let mut result_col: Column = Vec::new();
        for parsed_entry in &parsed_col {
            let height = parsed_entry
                .height
                .as_ref()
                .expect("layer height")
                .from_height(i_s as i64);
            result_col.push(Entry {
                value: parsed_entry.value,
                height: convert_to_layer_inner(height, steps, parsed_stack),
            });
        }
        result.push(result_col);
    }

    steps.truncate(base);
    parsed_stack.truncate(l_s);
    result
}

// ---------------------------------------------------------------------------
// From the layer representation
// ---------------------------------------------------------------------------

/// A parsed layer-column entry, with its height materialized.
#[derive(Clone, Debug)]
struct LayerEntry {
    value: i64,
    parent: i64,
    /// `None` is the not-yet-computed top height (the original implementation
    /// used the infinity marker as a placeholder); greater than any real height.
    height: Option<Height>,
}

type LayerColumn = Vec<LayerEntry>;
type Vertical = Vec<Height>;

fn materialized_vertical_increase(vert: &Vertical, h_diff: &Height) -> Vertical {
    let mut result = vert.clone();
    while result
        .last()
        .map_or(false, |last| height_compare(last, h_diff) == Ordering::Less)
    {
        result.pop();
    }
    result.push(h_diff.clone());
    result
}

/// Compute the parent of the lower entry in `parsed`, and the value of the
/// higher one.
fn parent_info(parsed: &LayerColumn, vertical: &Vertical) -> (Option<i64>, i64) {
    let mut ip = 0usize;
    let mut iv = 0usize;
    let mut lower_parent: Option<i64> = None;
    while ip != parsed.len() && iv != vertical.len() {
        let ord = height_opt_compare(parsed[ip].height.as_ref(), Some(&vertical[iv]));
        if ord != Ordering::Less {
            lower_parent = Some(parsed[ip].parent);
            iv += 1;
        }
        if ord != Ordering::Greater {
            ip += 1;
        }
    }
    let higher_value = parsed.get(ip).map_or(-1, |entry| entry.value);
    (lower_parent, higher_value)
}

pub(crate) fn convert_from_layer(e: &Expr) -> Expr {
    let mut parsed_stack: Vec<LayerColumn> = Vec::new();
    convert_from_layer_inner(e, &mut parsed_stack)
}

fn convert_from_layer_inner(e: &Expr, parsed_stack: &mut Vec<LayerColumn>) -> Expr {
    let l_s = parsed_stack.len();
    let mut result: Expr = Vec::new();

    for i in 0..e.len() {
        let i_s = parsed_stack.len();
        let col = &e[i];
        parsed_stack.push(LayerColumn::new());

        let mut result_col: Column = Vec::new();
        let mut current_vertical: Vertical = Vec::new();

        for j in 0..col.len() {
            let entry = &col[j];

            let mut parent: i64 = if j == 0 {
                i_s as i64 - 1
            } else {
                parsed_stack[i_s][j - 1].parent
            };
            while parent >= 0 {
                let (lower_parent, higher_value) =
                    parent_info(&parsed_stack[parent as usize], &current_vertical);
                if higher_value < entry.value {
                    break;
                }
                parent = lower_parent.unwrap_or(parent - 1);
            }

            parsed_stack[i_s].push(LayerEntry {
                value: entry.value,
                parent,
                height: None,
            });

            let height_expr = convert_from_layer_inner(&entry.height, parsed_stack);
            let height = to_height(&height_expr, i_s as i64);
            let idx = parsed_stack[i_s].len() - 1;
            parsed_stack[i_s][idx].height = Some(height.clone());

            loop {
                let should_pop = match result_col.last() {
                    Some(top) => {
                        top.value == parent && compare(&top.height, &height_expr) == Ordering::Less
                    }
                    None => false,
                };
                if should_pop {
                    result_col.pop();
                } else {
                    break;
                }
            }

            result_col.push(Entry {
                value: parent,
                height: height_expr,
            });
            current_vertical = materialized_vertical_increase(&current_vertical, &height);
        }

        result.push(result_col);
    }

    parsed_stack.truncate(l_s);
    result
}

// ---------------------------------------------------------------------------
// Differential oracle
// ---------------------------------------------------------------------------

#[cfg(test)]
mod differential {
    //! The lazy conversion must produce exactly what the definition's direct
    //! transcription produces. That transcription materializes a `Height` for
    //! every entry; it is kept here, test-only, as an oracle, so a future
    //! change to the view arithmetic cannot quietly alter the ordinal.

    use super::*;
    use crate::height::from_height;
    use crate::{family, Btbm, DisplayType, Lngi};

    /// A parsed layer-column entry.
    #[derive(Clone, Debug)]
    struct RefEntry {
        value: i64,
        /// `None` is the not-yet-computed top height (the original implementation
        /// used the infinity marker as a placeholder); greater than any real height.
        height: Option<Height>,
    }

    type RefColumn = Vec<RefEntry>;
    type RefVertical = Vec<Height>;

    fn ref_vertical_increase(vert: &RefVertical, h_diff: &Height) -> RefVertical {
        let mut result = vert.clone();
        while result
            .last()
            .map_or(false, |last| height_compare(last, h_diff) == Ordering::Less)
        {
            result.pop();
        }
        result.push(h_diff.clone());
        result
    }

    fn reference_to_layer(e: &Expr) -> Expr {
        let mut parsed_stack: Vec<RefColumn> = Vec::new();
        ref_inner(e, &mut parsed_stack)
    }

    fn ref_inner(e: &Expr, parsed_stack: &mut Vec<RefColumn>) -> Expr {
        let l_s = parsed_stack.len();
        let mut result: Expr = Vec::new();

        for i in 0..e.len() {
            let i_s = parsed_stack.len();
            let col = &e[i];
            let mut parsed_col: RefColumn = Vec::new();
            let mut current_vertical: RefVertical = Vec::new();

            for j in 0..col.len() {
                let entry = &col[j];
                let parent = entry.value;
                let height = to_height(&entry.height, i_s as i64);

                let mut kp = 0usize;
                let mut ki = 0usize;
                while kp != parsed_stack[parent as usize].len() && ki != current_vertical.len() {
                    let ord = height_opt_compare(
                        parsed_stack[parent as usize][kp].height.as_ref(),
                        Some(&current_vertical[ki]),
                    );
                    if ord != Ordering::Greater {
                        kp += 1;
                    }
                    if ord != Ordering::Less {
                        ki += 1;
                    }
                }

                if kp == parsed_stack[parent as usize].len() {
                    parsed_col.push(RefEntry {
                        value: 0,
                        height: Some(height.clone()),
                    });
                } else {
                    loop {
                        if kp == parsed_stack[parent as usize].len() {
                            parsed_col.push(RefEntry {
                                value: 0,
                                height: Some(height.clone()),
                            });
                            break;
                        }
                        let ancestor = &parsed_stack[parent as usize][kp];
                        let ancestor_height = ancestor.height.as_ref().expect("layer height");
                        if height_compare(ancestor_height, &height) == Ordering::Less {
                            parsed_col.push(RefEntry {
                                value: ancestor.value + 1,
                                height: Some(ancestor_height.clone()),
                            });
                        } else {
                            parsed_col.push(RefEntry {
                                value: ancestor.value + 1,
                                height: Some(height.clone()),
                            });
                            break;
                        }
                        kp += 1;
                    }
                }

                current_vertical = ref_vertical_increase(&current_vertical, &height);
            }

            parsed_stack.push(parsed_col.clone());

            let mut result_col: Column = Vec::new();
            for parsed_entry in &parsed_col {
                let height_expr = from_height(
                    parsed_entry.height.as_ref().expect("layer height"),
                    i_s as i64,
                );
                result_col.push(Entry {
                    value: parsed_entry.value,
                    height: ref_inner(&height_expr, parsed_stack),
                });
            }
            result.push(result_col);
        }

        parsed_stack.truncate(l_s);
        result
    }

    /// A pool of expressions that actually occur, including deeply nested ones
    /// that force the ancestor branch and long step chains.
    fn corpus() -> Vec<Expr> {
        let n = Btbm::new();
        let mut out = Vec::new();
        for src in [
            "(0)",
            "(0)(1)",
            "(0)(1)(2)",
            "(0)(1)(2)(3)",
            "(0)(1)(2)(3)(4)",
            "(0)(1^(2))",
            "(0)(1^(2))(1)",
            "(0)(1,1)",
            "(0)(1^(2,2))",
            "(0)(1^(2))(2)",
            "(0)(1)(2,1)",
            "(0)(1^(2))(2^(3))",
        ] {
            if let Ok(e) = n.from_display(src) {
                out.push(e);
            }
        }
        for k in 3..9 {
            let e = family(k);
            for i in 0..6 {
                if !n.is_limit(&e) {
                    break;
                }
                out.push(n.fs(&e, i));
            }
            out.push(e);
        }
        // Real ordinals off the time curve, which is where the ancestor branch
        // dominates and the old implementation allocated tens of megabytes.
        let year_ms = 365.25 * 86_400_000.0;
        let mut l = Lngi::new();
        for years in [0.0, 1.0, 10.0, 100.0, 1000.0] {
            out.push(l.step(years * year_ms).expr);
        }
        out
    }

    #[test]
    fn lazy_conversion_matches_the_definition() {
        let n = Btbm::new();
        for e in corpus() {
            let want = reference_to_layer(&e);
            let got = convert_to_layer(&e);
            assert_eq!(
                got, want,
                "layer conversion disagreed for {}",
                n.display(&e, DisplayType::Plain)
            );
        }
    }
}
