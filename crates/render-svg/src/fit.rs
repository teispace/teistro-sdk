//! Fitting a cell's labels into its outline without font metrics
//! (`03-design/render-svg.md` §6).
//!
//! Everything here is arithmetic on the unit square and a fixed number of
//! bisection steps, so the arrangement is the same on every platform.

use teistro_geometry::path::polygon_contains;
use teistro_geometry::{Path, Point};

/// How many times a box's scale is halved towards the largest that fits:
/// far past the hundredth of a user unit the output keeps.
const STEPS: u32 = 40;

/// How many points each edge of a candidate box is tested at, besides its
/// corners.
const EDGE_SAMPLES: u32 = 8;

/// The gap between two columns, in ems.
const COLUMN_GAP: f64 = 0.4;

/// One label, placed: the centre of its line in the unit square.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Line {
    pub(crate) at: Point,
}

/// A cell's labels arranged: where each line's centre is, and the font
/// size, in the unit square.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Stack {
    pub(crate) lines: Vec<Line>,
    pub(crate) font: f64,
}

/// A box the arrangement must not cover, such as the cell's own label: its
/// centre and half its width and height, in the unit square.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Clear {
    pub(crate) centre: Point,
    pub(crate) half_width: f64,
    pub(crate) half_height: f64,
}

/// What the arrangement needs from the style.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Metrics {
    /// The largest font, in the unit square.
    pub(crate) largest: f64,
    /// A character's estimated width, in ems.
    pub(crate) advance: f64,
    /// A line's height, in ems.
    pub(crate) line_height: f64,
}

/// Arranges `count` labels, the widest `widest` characters, about `anchor`
/// inside `outline` and off `clear`: the column count that allows the
/// largest font, fewer columns on a tie, filled top to bottom.
pub(crate) fn stack(
    outline: &Path,
    anchor: Point,
    clear: Option<Clear>,
    count: usize,
    widest: usize,
    metrics: Metrics,
) -> Stack {
    if count == 0 {
        return Stack {
            lines: Vec::new(),
            font: metrics.largest,
        };
    }
    let polygon = outline.flatten();
    let label = to_unit(widest.max(1)) * metrics.advance;
    let mut best: Option<(f64, usize, usize)> = None;
    for columns in 1..=count {
        let rows = count.div_ceil(columns);
        let width = to_unit(columns) * label + to_unit(columns - 1) * COLUMN_GAP;
        let height = to_unit(rows) * metrics.line_height;
        let font = largest_scale(&polygon, anchor, clear, width, height).min(metrics.largest);
        if best.is_none_or(|(so_far, ..)| font > so_far) {
            best = Some((font, columns, rows));
        }
    }
    let Some((font, columns, rows)) = best else {
        unreachable!("count is at least one, so one column count was tried")
    };
    let (width, height) = (
        to_unit(columns) * label + to_unit(columns - 1) * COLUMN_GAP,
        to_unit(rows) * metrics.line_height,
    );
    let lines = (0..count)
        .map(|index| {
            let (column, row) = (index / rows, index % rows);
            let x = -width / 2.0 + label / 2.0 + to_unit(column) * (label + COLUMN_GAP);
            let y = -height / 2.0 + metrics.line_height / 2.0 + to_unit(row) * metrics.line_height;
            Line {
                at: Point::new(anchor.x + x * font, anchor.y + y * font),
            }
        })
        .collect();
    Stack { lines, font }
}

/// The largest scale at which a `width` × `height` box, centred on
/// `anchor`, lies inside `polygon` and does not overlap `clear`.
fn largest_scale(
    polygon: &[Point],
    anchor: Point,
    clear: Option<Clear>,
    width: f64,
    height: f64,
) -> f64 {
    let fits = |scale: f64| {
        let (half_w, half_h) = (width * scale / 2.0, height * scale / 2.0);
        let overlaps = clear.is_some_and(|clear| {
            (anchor.x - clear.centre.x).abs() < half_w + clear.half_width
                && (anchor.y - clear.centre.y).abs() < half_h + clear.half_height
        });
        if overlaps {
            return false;
        }
        let corners = [
            Point::new(anchor.x - half_w, anchor.y - half_h),
            Point::new(anchor.x + half_w, anchor.y - half_h),
            Point::new(anchor.x + half_w, anchor.y + half_h),
            Point::new(anchor.x - half_w, anchor.y + half_h),
        ];
        corners
            .iter()
            .zip(corners.iter().cycle().skip(1))
            .all(|(&from, &to)| {
                (0..EDGE_SAMPLES).all(|step| {
                    let t = f64::from(step) / f64::from(EDGE_SAMPLES);
                    polygon_contains(
                        polygon,
                        Point::new(from.x + (to.x - from.x) * t, from.y + (to.y - from.y) * t),
                    )
                })
            })
    };
    let (mut low, mut high) = (0.0, 1.0 / width.max(height));
    if fits(high) {
        return high;
    }
    for _ in 0..STEPS {
        let middle = f64::midpoint(low, high);
        if fits(middle) {
            low = middle;
        } else {
            high = middle;
        }
    }
    low
}

#[expect(
    clippy::cast_precision_loss,
    reason = "a count of labels or characters is far below 2^52"
)]
pub(crate) const fn to_unit(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::float_cmp,
        reason = "tests fail by panicking, index their own fixtures and compare exact values"
    )]
    use super::*;

    const METRICS: Metrics = Metrics {
        largest: 0.04,
        advance: 0.6,
        line_height: 1.2,
    };

    fn square(side: f64) -> Path {
        let (low, high) = (0.5 - side / 2.0, 0.5 + side / 2.0);
        Path::polygon(&[
            Point::new(low, low),
            Point::new(high, low),
            Point::new(high, high),
            Point::new(low, high),
        ])
    }

    fn inside(stack: &Stack, path: &Path, widest: usize) -> bool {
        let half_w = to_unit(widest) * METRICS.advance * stack.font / 2.0;
        let half_h = METRICS.line_height * stack.font / 2.0;
        stack.lines.iter().all(|line| {
            [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)]
                .iter()
                .all(|(sx, sy)| {
                    path.contains(Point::new(line.at.x + sx * half_w, line.at.y + sy * half_h))
                })
        })
    }

    #[test]
    fn a_roomy_cell_draws_at_the_largest_size_in_one_column() {
        let cell = square(0.4);
        let stack = stack(&cell, Point::new(0.5, 0.5), None, 3, 2, METRICS);
        assert_eq!(stack.font, METRICS.largest);
        assert!(stack.lines.iter().all(|line| line.at.x == 0.5), "{stack:?}");
        assert!(
            (stack.lines[1].at.y - 0.5).abs() < 1e-12,
            "the middle line is on the anchor"
        );
        assert!(inside(&stack, &cell, 2));
    }

    #[test]
    fn a_crowded_cell_shrinks_and_spreads_into_columns() {
        let cell = square(0.12);
        let stack = stack(&cell, Point::new(0.5, 0.5), None, 10, 2, METRICS);
        assert!(stack.font < METRICS.largest);
        let columns = stack
            .lines
            .iter()
            .map(|line| line.at.x.to_bits())
            .collect::<std::collections::BTreeSet<_>>();
        assert!(columns.len() > 1, "{stack:?}");
        assert!(inside(&stack, &cell, 2), "{stack:?}");
    }

    #[test]
    fn a_triangle_keeps_its_labels_inside() {
        // A North Indian corner cell's shape: the box must shrink towards
        // the anchor rather than cross the hypotenuse.
        let cell = Path::polygon(&[
            Point::new(0.0, 0.0),
            Point::new(0.5, 0.0),
            Point::new(0.0, 0.5),
        ]);
        let anchor = Point::new(0.15, 0.12);
        let stack = stack(&cell, anchor, None, 4, 3, METRICS);
        assert!(inside(&stack, &cell, 3), "{stack:?}");
    }

    #[test]
    fn a_stack_keeps_off_the_cell_label() {
        let cell = square(0.3);
        let anchor = Point::new(0.5, 0.5);
        let label = Clear {
            centre: Point::new(0.45, 0.42),
            half_width: 0.03,
            half_height: 0.02,
        };
        let free = stack(&cell, anchor, None, 8, 2, METRICS);
        let kept = stack(&cell, anchor, Some(label), 8, 2, METRICS);
        // The label moves the stack, into more columns or a smaller font.
        assert_ne!(kept, free);
        let half_w = 2.0 * METRICS.advance * kept.font / 2.0;
        let half_h = METRICS.line_height * kept.font / 2.0;
        for line in &kept.lines {
            let apart = (line.at.x - label.centre.x).abs() >= half_w + label.half_width
                || (line.at.y - label.centre.y).abs() >= half_h + label.half_height;
            assert!(apart, "{line:?} covers the label");
        }
    }

    #[test]
    fn nothing_to_stack_is_nothing() {
        assert!(
            stack(&square(0.2), Point::new(0.5, 0.5), None, 0, 0, METRICS)
                .lines
                .is_empty()
        );
    }
}
