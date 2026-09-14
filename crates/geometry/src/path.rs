//! Points and closed outlines in the unit square, and the plane geometry a
//! layout's validation needs.
//!
//! The square is `[0, 1] × [0, 1]` with **y downwards**, the convention of
//! every drawing surface a renderer targets, so no consumer flips an axis.
//!
//! An outline is a path of line and quadratic segments, not a polygon,
//! because the Nepali lotus's petals are curves (`03-design/chart-geometry.md`
//! §2). The checks that need a polygon flatten it to a fixed number of
//! segments, so they are deterministic and the same on every platform.

use serde::{Deserialize, Serialize};

/// A point in the unit square, y downwards.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Point {
    /// From the left edge, 0 to 1.
    pub x: f64,
    /// From the top edge, 0 to 1.
    pub y: f64,
}

impl Point {
    /// A point.
    #[must_use]
    pub const fn new(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    /// Whether the point lies in the closed unit square.
    #[must_use]
    pub fn in_unit_square(self) -> bool {
        (0.0..=1.0).contains(&self.x) && (0.0..=1.0).contains(&self.y)
    }

    /// The point `t` of the way from `self` to `to`.
    #[must_use]
    fn lerp(self, to: Point, t: f64) -> Point {
        Point::new(self.x + (to.x - self.x) * t, self.y + (to.y - self.y) * t)
    }
}

/// One step of an outline, from wherever the previous step ended.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Segment {
    /// A straight line to a point.
    Line {
        /// Where the line ends.
        to: Point,
    },
    /// A quadratic Bézier curve to a point, pulled towards its control.
    Quad {
        /// The control point.
        control: Point,
        /// Where the curve ends.
        to: Point,
    },
    /// A circular arc about a centre, from where the previous step ended
    /// to a point the same distance from it, the short way or the long way
    /// as `clockwise` says. What a radial layout's sectors are bounded by.
    Arc {
        /// The circle's centre.
        centre: Point,
        /// Which way the arc runs, as a reader sees it.
        clockwise: bool,
        /// Where the arc ends.
        to: Point,
    },
}

impl Segment {
    /// Where the segment ends.
    #[must_use]
    pub const fn end(self) -> Point {
        match self {
            Segment::Line { to } | Segment::Quad { to, .. } | Segment::Arc { to, .. } => to,
        }
    }
}

/// A closed outline: a start and the segments that return to it.
///
/// The closing step is implied, as SVG's `Z` implies it, so an outline
/// whose last segment does not end at its start is closed by a straight
/// line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Path {
    /// Where the outline starts.
    pub start: Point,
    /// The steps around it.
    pub segments: Vec<Segment>,
}

/// How many straight pieces a curve is flattened into for the checks: fine
/// enough that a petal's area is right to a few parts in a million, and
/// fixed so the answer is the same everywhere.
pub const CURVE_PIECES: u32 = 32;

impl Path {
    /// A closed polygon through the given points.
    #[must_use]
    pub fn polygon(points: &[Point]) -> Path {
        let (start, rest) = points
            .split_first()
            .map_or((Point::new(0.0, 0.0), &[][..]), |(first, rest)| {
                (*first, rest)
            });
        Path {
            start,
            segments: rest.iter().map(|&to| Segment::Line { to }).collect(),
        }
    }

    /// Every point the outline passes through, as a polygon, curves cut
    /// into [`CURVE_PIECES`] straight pieces.
    #[must_use]
    pub fn flatten(&self) -> Vec<Point> {
        let mut points = vec![self.start];
        let mut from = self.start;
        for segment in &self.segments {
            match *segment {
                Segment::Line { to } => points.push(to),
                Segment::Quad { control, to } => {
                    for piece in 1..=CURVE_PIECES {
                        let t = f64::from(piece) / f64::from(CURVE_PIECES);
                        points.push(from.lerp(control, t).lerp(control.lerp(to, t), t));
                    }
                }
                Segment::Arc {
                    centre,
                    clockwise,
                    to,
                } => {
                    // With y downwards, a clockwise turn increases the angle
                    // `atan2` measures. The check only reads these points; the
                    // outline itself keeps its exact endpoints.
                    let radius = (from.x - centre.x).hypot(from.y - centre.y);
                    let begin = (from.y - centre.y).atan2(from.x - centre.x);
                    let mut sweep = (to.y - centre.y).atan2(to.x - centre.x) - begin;
                    if clockwise && sweep <= 0.0 {
                        sweep += std::f64::consts::TAU;
                    } else if !clockwise && sweep >= 0.0 {
                        sweep -= std::f64::consts::TAU;
                    }
                    for piece in 1..CURVE_PIECES {
                        let angle = begin + sweep * f64::from(piece) / f64::from(CURVE_PIECES);
                        points.push(Point::new(
                            centre.x + radius * angle.cos(),
                            centre.y + radius * angle.sin(),
                        ));
                    }
                    points.push(to);
                }
            }
            from = segment.end();
        }
        if points.len() > 1 && points.last() == Some(&self.start) {
            points.pop();
        }
        points
    }

    /// The enclosed area, by the shoelace formula over the flattened
    /// outline; positive whichever way the outline runs.
    #[must_use]
    pub fn area(&self) -> f64 {
        let points = self.flatten();
        let twice: f64 = points
            .iter()
            .zip(points.iter().cycle().skip(1))
            .map(|(a, b)| a.x * b.y - b.x * a.y)
            .sum();
        (twice / 2.0).abs()
    }

    /// The centroid of the enclosed region.
    #[must_use]
    pub fn centroid(&self) -> Point {
        let points = self.flatten();
        let (mut cross_sum, mut cx, mut cy) = (0.0, 0.0, 0.0);
        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
            let cross = a.x * b.y - b.x * a.y;
            cross_sum += cross;
            cx += (a.x + b.x) * cross;
            cy += (a.y + b.y) * cross;
        }
        if cross_sum == 0.0 {
            return self.start;
        }
        Point::new(cx / (3.0 * cross_sum), cy / (3.0 * cross_sum))
    }

    /// Whether a point lies strictly inside the outline, by the even-odd
    /// rule over the flattened polygon. A point exactly on an edge may
    /// answer either way, which is why the checks sample off the grid.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        let points = self.flatten();
        let mut inside = false;
        for (a, b) in points.iter().zip(points.iter().cycle().skip(1)) {
            let crosses = (a.y > point.y) != (b.y > point.y);
            if crosses && point.x < (b.x - a.x) * (point.y - a.y) / (b.y - a.y) + a.x {
                inside = !inside;
            }
        }
        inside
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "the values are exact in binary")]

    use super::*;

    fn unit_square() -> Path {
        Path::polygon(&[
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
        ])
    }

    #[test]
    fn a_square_has_its_area_centre_and_inside() {
        let square = unit_square();
        assert_eq!(square.area(), 1.0);
        assert_eq!(square.centroid(), Point::new(0.5, 0.5));
        assert!(square.contains(Point::new(0.25, 0.75)));
        assert!(!square.contains(Point::new(1.25, 0.5)));
    }

    #[test]
    fn a_curve_bulges_and_its_area_is_the_parabola_s() {
        // A quadratic from (0,1) to (1,1) with its control at (0.5,0) peaks
        // at y = 0.5, and the region between it and its chord is a parabolic
        // segment: two thirds of base times height, 2/3 × 1 × 0.5 = 1/3. The
        // flattening lands within a thousandth of it.
        let dome = Path {
            start: Point::new(0.0, 1.0),
            segments: vec![Segment::Quad {
                control: Point::new(0.5, 0.0),
                to: Point::new(1.0, 1.0),
            }],
        };
        assert!((dome.area() - 1.0 / 3.0).abs() < 1e-3, "{}", dome.area());
        assert!(dome.contains(Point::new(0.5, 0.9)));
        assert!(!dome.contains(Point::new(0.5, 0.2)));
    }

    #[test]
    fn an_explicitly_closed_outline_is_not_counted_twice() {
        let mut closed = unit_square();
        closed.segments.push(Segment::Line {
            to: Point::new(0.0, 0.0),
        });
        assert_eq!(closed.flatten().len(), 4);
        assert_eq!(closed.area(), 1.0);
    }
}
