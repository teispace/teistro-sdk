//! Where a wheel's bodies are written: at their degrees, spread apart round
//! the ring where they would overlap (`03-design/render-svg.md` §6).
//!
//! A conjunction is the common case, not the rare one: three grahas within
//! a few degrees write on top of each other at their true places. Every
//! Western wheel resolves it the same way, by spreading a cluster round the
//! ring about its middle, in order, and that is what this does.
//!
//! The spreading needs a bearing and its sine and cosine, which a platform's
//! library computes. As the wheel's own placement does
//! (`chart-geometry.md` §7), every such value is rounded to the geometry's
//! [`GRAIN`], so no platform difference reaches the drawing, and the hash
//! matrix checks it.

use teistro_geometry::place::GRAIN;
use teistro_geometry::{Mark, Placed, Point};

const CENTRE: Point = Point::new(0.5, 0.5);

/// How many times clusters are merged and laid out again: each pass can
/// only join clusters, so a ring of `n` marks settles within `n` passes.
fn passes(count: usize) -> usize {
    count.max(1)
}

fn grain(value: f64) -> f64 {
    (value / GRAIN).round() * GRAIN
}

fn distance(at: Point) -> f64 {
    let (dx, dy) = (at.x - CENTRE.x, at.y - CENTRE.y);
    (dx * dx + dy * dy).sqrt()
}

/// The radii a ring's marks are drawn at: their words where the ring's
/// bodies are anchored, clear of the house numbers nearer the centre, and
/// their ticks at the ring's outer edge.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Radii {
    pub(crate) words: f64,
    pub(crate) outer: f64,
}

impl Radii {
    /// The radii of the ring a chart's marks are in, read off its cells;
    /// nothing when the chart has no cell in that ring.
    pub(crate) fn of(placed: &Placed, ring: u8) -> Option<Radii> {
        let cell = placed.cells.iter().find(|cell| cell.ring == ring)?;
        let outer = cell
            .outline
            .flatten()
            .into_iter()
            .map(distance)
            .fold(0.0, f64::max);
        Some(Radii {
            words: distance(cell.anchor),
            outer: grain(outer),
        })
    }
}

/// The point at `radius` in the direction of `at`: along its own radius, so
/// it needs no trigonometry.
fn towards(at: Point, radius: f64) -> Point {
    let from = distance(at);
    if from == 0.0 {
        return at;
    }
    let scale = radius / from;
    Point::new(
        grain(CENTRE.x + (at.x - CENTRE.x) * scale),
        grain(CENTRE.y + (at.y - CENTRE.y) * scale),
    )
}

/// Each mark's position at the words' radius: in its own direction where
/// it is clear, or its cluster's spread round the ring about the cluster's
/// middle, keeping the marks' order. `room` is the length of ring each
/// mark's words need.
#[expect(
    clippy::indexing_slicing,
    reason = "every index is a slot or a mark number below the marks' own count"
)]
pub(crate) fn spread(marks: &[Mark], radii: Radii, room: f64) -> Vec<Point> {
    let radius = radii.words;
    if marks.len() < 2 || radius == 0.0 {
        return marks.iter().map(|mark| towards(mark.at, radius)).collect();
    }

    // Bearings clockwise from twelve o'clock, in degrees.
    let bearings: Vec<f64> = marks.iter().map(|mark| bearing(mark.at)).collect();
    let count = marks.len();
    let separation = (room / radius)
        .to_degrees()
        .min(360.0 / crate::fit::to_unit(count));

    // The marks in bearing order, ties kept in the chart's order.
    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|&a, &b| bearings[a].total_cmp(&bearings[b]).then(a.cmp(&b)));
    let mut laid: Vec<f64> = order.iter().map(|&index| bearings[index]).collect();

    for _ in 0..passes(count) {
        let mut moved = false;
        for cluster in clusters(&laid, separation) {
            if cluster.len() < 2 {
                continue;
            }
            // Unwrap the cluster so it reads increasing across twelve
            // o'clock, then centre its spread on its middle.
            let start = laid[cluster[0]];
            let unwrapped: Vec<f64> = cluster
                .iter()
                .map(|&slot| {
                    let value = laid[slot];
                    if value < start { value + 360.0 } else { value }
                })
                .collect();
            let middle = unwrapped.iter().sum::<f64>() / crate::fit::to_unit(unwrapped.len());
            let half = separation * crate::fit::to_unit(cluster.len() - 1) / 2.0;
            for (step, &slot) in cluster.iter().enumerate() {
                let value =
                    (middle - half + separation * crate::fit::to_unit(step)).rem_euclid(360.0);
                let value = grain(value);
                if value.to_bits() != laid[slot].to_bits() {
                    moved = true;
                }
                laid[slot] = value;
            }
        }
        if !moved {
            break;
        }
    }

    let mut points = vec![CENTRE; count];
    for (slot, &index) in order.iter().enumerate() {
        points[index] = if laid[slot].to_bits() == bearings[index].to_bits() {
            towards(marks[index].at, radius)
        } else {
            at_bearing(radius, laid[slot])
        };
    }
    points
}

/// A mark's tick: a short stroke inside the ring's outer edge at the body's
/// true degree, so a body spread off its place still shows where it stands.
pub(crate) fn tick(mark: &Mark, radii: Radii, length: f64) -> (Point, Point) {
    (
        towards(mark.at, radii.outer - length),
        towards(mark.at, radii.outer),
    )
}

/// Runs of consecutive bearings, round the ring, no further apart than
/// `separation`. Each run is a list of slots in order; a run may cross
/// twelve o'clock.
///
/// Marks exactly one separation apart touch and stay one run: a cluster
/// already spread is still a cluster, so the next pass merges it whole with
/// a neighbour it has spread into, rather than splitting it into pairs.
#[expect(
    clippy::indexing_slicing,
    reason = "every slot is reduced modulo the bearings' own count"
)]
fn clusters(laid: &[f64], separation: f64) -> Vec<Vec<usize>> {
    let count = laid.len();
    // Allow for the grain the laid bearings were rounded to.
    let close = |from: usize, to: usize| {
        let gap = (laid[to] - laid[from]).rem_euclid(360.0);
        gap < separation + GRAIN * 10.0
    };
    // Start after a gap, so no run is split at the list's end.
    let Some(start) = (0..count).find(|&slot| !close((slot + count - 1) % count, slot)) else {
        // Every mark is close to the next: one run round the whole ring.
        return vec![(0..count).collect()];
    };
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for step in 0..count {
        let slot = (start + step) % count;
        match runs.last_mut() {
            Some(run) if close((slot + count - 1) % count, slot) => run.push(slot),
            _ => runs.push(vec![slot]),
        }
    }
    runs
}

/// A point's bearing from the centre, clockwise from twelve o'clock, in
/// degrees from 0 up to 360, on the grain.
fn bearing(at: Point) -> f64 {
    grain(
        (at.x - CENTRE.x)
            .atan2(CENTRE.y - at.y)
            .to_degrees()
            .rem_euclid(360.0),
    )
}

fn at_bearing(radius: f64, degrees: f64) -> Point {
    let angle = degrees.to_radians();
    Point::new(
        grain(CENTRE.x + radius * angle.sin()),
        grain(CENTRE.y - radius * angle.cos()),
    )
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
    use teistro_core::catalogue::Graha;

    use super::*;

    fn mark(degrees: f64) -> Mark {
        Mark {
            body: Graha::Sun.key_id(),
            ring: 0,
            at: at_bearing(0.3, degrees),
            longitude_deg: degrees,
        }
    }

    const RADII: Radii = Radii {
        words: 0.35,
        outer: 0.4,
    };

    /// The length of ring each mark needs in these tests, and the
    /// separation in degrees it makes at the words' radius.
    const ROOM: f64 = 0.04;

    fn separation() -> f64 {
        (ROOM / RADII.words).to_degrees()
    }

    fn gap(a: Point, b: Point) -> f64 {
        (bearing(b) - bearing(a)).rem_euclid(360.0)
    }

    #[test]
    fn clear_marks_stay_in_their_directions_at_the_words_radius() {
        let marks = [mark(10.0), mark(90.0), mark(200.0)];
        for (mark, at) in marks.iter().zip(spread(&marks, RADII, ROOM)) {
            assert!((bearing(at) - mark.longitude_deg).abs() < 1e-6);
            assert!((distance(at) - RADII.words).abs() < 1e-8);
        }
    }

    #[test]
    fn a_conjunction_spreads_about_its_middle_in_order() {
        let marks = [mark(101.0), mark(103.5), mark(104.0), mark(250.0)];
        let spread = spread(&marks, RADII, ROOM);
        assert!(gap(spread[0], spread[1]) >= separation() - 1e-6);
        assert!(gap(spread[1], spread[2]) >= separation() - 1e-6);
        let middle = f64::midpoint(bearing(spread[0]), bearing(spread[2]));
        assert!(
            (middle - (101.0 + 103.5 + 104.0) / 3.0).abs() < 1e-6,
            "{middle}"
        );
        assert!(
            (bearing(spread[3]) - 250.0).abs() < 1e-6,
            "a mark clear of the cluster stays"
        );
    }

    #[test]
    fn a_cluster_across_twelve_o_clock_is_one_cluster() {
        let marks = [mark(359.0), mark(1.0)];
        let spread = spread(&marks, RADII, ROOM);
        assert!((gap(spread[0], spread[1]) - separation()).abs() < 1e-6);
        assert!(
            gap(spread[1], spread[0]) > 180.0,
            "it spread round, not across"
        );
    }

    #[test]
    fn clusters_that_spread_into_each_other_merge() {
        let marks = [mark(100.0), mark(102.0), mark(111.0), mark(113.0)];
        let spread = spread(&marks, RADII, ROOM);
        for pair in spread.windows(2) {
            assert!(gap(pair[0], pair[1]) >= separation() - 1e-6, "{spread:?}");
        }
    }

    #[test]
    fn a_tick_sits_inside_the_outer_edge_at_the_true_degree() {
        let mark = mark(101.0);
        let (from, to) = tick(&mark, RADII, 0.015);
        assert!((bearing(from) - 101.0).abs() < 1e-6 && (bearing(to) - 101.0).abs() < 1e-6);
        assert!((distance(from) - 0.385).abs() < 1e-8);
        assert!((distance(to) - 0.4).abs() < 1e-8);
    }

    #[test]
    fn every_spread_coordinate_sits_on_the_grain() {
        let marks = [mark(101.0), mark(103.5), mark(104.0)];
        for at in spread(&marks, RADII, ROOM) {
            for value in [at.x, at.y] {
                assert_eq!(value, grain(value));
            }
        }
    }
}
