//! Directions on a clock face, exact enough to be the same on every platform.
//!
//! A radial layout's sectors start on a clock hour and its labels sit on the
//! half hour between two, so every direction it needs is a multiple of 15°.
//! Those have sines and cosines built from √2, √3 and √6, and IEEE 754
//! requires a square root to be correctly rounded. So they come out bit for
//! bit the same on every architecture, where a platform's `sin` and `cos`
//! need not (`03-design/chart-geometry.md` §7).

use crate::path::Point;

/// The unit direction `half_hours` half-hours clockwise from twelve o'clock,
/// in the unit square's axes (x right, y down): twelve o'clock is `(0, -1)`,
/// three o'clock `(1, 0)`.
#[must_use]
pub fn direction(half_hours: u32) -> (f64, f64) {
    let (root2, root3, root6) = (2.0_f64.sqrt(), 3.0_f64.sqrt(), 6.0_f64.sqrt());
    // sin and cos of 0°, 15°, 30°, ... 90°, the first quadrant.
    let sines = [
        0.0,
        (root6 - root2) / 4.0,
        0.5,
        root2 / 2.0,
        root3 / 2.0,
        (root6 + root2) / 4.0,
        1.0,
    ];
    let steps = half_hours % 24;
    let quadrant = steps / 6;
    let within = usize::try_from(steps % 6).unwrap_or(0);
    let (s, c) = (
        sines.get(within).copied().unwrap_or(0.0),
        sines.get(6 - within).copied().unwrap_or(1.0),
    );
    // Clockwise from twelve: (sin θ, −cos θ), turned a quarter at a time.
    match quadrant {
        0 => (s, -c),
        1 => (c, s),
        2 => (-s, c),
        _ => (-c, -s),
    }
}

/// The point at `radius` from `centre`, `half_hours` clockwise from twelve.
#[must_use]
pub fn at(centre: Point, radius: f64, half_hours: u32) -> Point {
    let (x, y) = direction(half_hours);
    Point::new(centre.x + radius * x, centre.y + radius * y)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::float_cmp, reason = "the quarter hours are exact in binary")]

    use super::*;

    #[test]
    fn the_quarters_are_exact_and_the_rest_are_where_sin_puts_them() {
        assert_eq!(direction(0), (0.0, -1.0));
        assert_eq!(direction(6), (1.0, 0.0));
        assert_eq!(direction(12), (0.0, 1.0));
        assert_eq!(direction(18), (-1.0, 0.0));
        assert_eq!(direction(24), direction(0));
        for half_hours in 0..24 {
            let angle = f64::from(half_hours) * 15f64.to_radians();
            let (x, y) = direction(half_hours);
            assert!((x - angle.sin()).abs() < 1e-15, "{half_hours}");
            assert!((y + angle.cos()).abs() < 1e-15, "{half_hours}");
        }
    }
}
