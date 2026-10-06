//! Conversion between the layer representation and the ordinary one.

use std::cmp::Ordering;

use super::compare::{compare, height_compare, height_opt_compare};
use super::height::{from_height, to_height};
use super::types::{Column, Entry, Expr, Height};

/// A parsed layer-column entry.
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

fn vertical_increase(vert: &Vertical, h_diff: &Height) -> Vertical {
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

pub(crate) fn convert_to_layer(e: &Expr) -> Expr {
    let mut parsed_stack: Vec<LayerColumn> = Vec::new();
    convert_to_layer_inner(e, &mut parsed_stack)
}

fn convert_to_layer_inner(e: &Expr, parsed_stack: &mut Vec<LayerColumn>) -> Expr {
    let l_s = parsed_stack.len();
    let mut result: Expr = Vec::new();

    for i in 0..e.len() {
        let i_s = parsed_stack.len();
        let col = &e[i];
        let mut parsed_col: LayerColumn = Vec::new();
        let mut current_vertical: Vertical = Vec::new();

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
                parsed_col.push(LayerEntry {
                    value: 0,
                    parent,
                    height: Some(height.clone()),
                });
            } else {
                loop {
                    if kp == parsed_stack[parent as usize].len() {
                        parsed_col.push(LayerEntry {
                            value: 0,
                            parent,
                            height: Some(height.clone()),
                        });
                        break;
                    }
                    let ancestor = &parsed_stack[parent as usize][kp];
                    let ancestor_height = ancestor.height.as_ref().expect("layer height");
                    if height_compare(ancestor_height, &height) == Ordering::Less {
                        parsed_col.push(LayerEntry {
                            value: ancestor.value + 1,
                            parent,
                            height: Some(ancestor_height.clone()),
                        });
                    } else {
                        parsed_col.push(LayerEntry {
                            value: ancestor.value + 1,
                            parent,
                            height: Some(height.clone()),
                        });
                        break;
                    }
                    kp += 1;
                }
            }

            current_vertical = vertical_increase(&current_vertical, &height);
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
                height: convert_to_layer_inner(&height_expr, parsed_stack),
            });
        }
        result.push(result_col);
    }

    parsed_stack.truncate(l_s);
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
            current_vertical = vertical_increase(&current_vertical, &height);
        }

        result.push(result_col);
    }

    parsed_stack.truncate(l_s);
    result
}
