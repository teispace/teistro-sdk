//! A moving body's parallels to fixed points over time: a progressed
//! planet's to the radical chart (`03-design/western-progressions.md`).
//!
//! ```
//! use teistro_western::{SpanSearch, parallel_spans};
//!
//! // A body rising 0.5° a day from 20° S, against a point 10° N.
//! let at = |t: f64| -20.0 + 0.5 * t;
//! let samples: Vec<(f64, f64)> = (0..=100).map(|t| (f64::from(t), at(f64::from(t)))).collect();
//! let search = SpanSearch { fixed_deg: 10.0, orb_deg: 1.0, tolerance: 1e-6 };
//! let spans = parallel_spans(&samples, search, |t| Ok::<_, ()>(at(t))).unwrap();
//! // Contra-parallel at 10° S (day 20), parallel at 10° N (day 60).
//! assert_eq!(spans.len(), 2);
//! assert!(spans[0].contrary && !spans[1].contrary);
//! assert!((spans[0].closest - 20.0).abs() < 1e-5 && (spans[1].closest - 60.0).abs() < 1e-5);
//! assert!((spans[1].enters.unwrap() - 58.0).abs() < 1e-5);
//! ```

use crate::declination::{contrary, gap_deg};

/// A stretch of time over which a moving body stands within a parallel's
/// orb of a fixed distance from the equator: a progressed planet's
/// parallel to a radical point, which Leo holds "in force for a period of
/// several years" (*The Progressed Horoscope*, p. 34).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ParallelSpan {
    /// When the body came within the orb; none when it was already within
    /// it at the first instant searched.
    pub enters: Option<f64>,
    /// When it left the orb; none when it was still within it at the last.
    pub leaves: Option<f64>,
    /// The first instant of the span at which the two distances stood
    /// closest: the exact parallel, when the span reaches it.
    pub closest: f64,
    /// How far apart the two distances stood then, degrees.
    pub closest_deg: f64,
    /// Whether the two stood on opposite sides of the equator then.
    pub contrary: bool,
}

/// Instants with the moving body's declination at each.
type Points = Vec<(f64, f64)>;

/// What [`parallel_spans`] searches for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpanSearch {
    /// The fixed distance from the equator, degrees, north positive.
    pub fixed_deg: f64,
    /// The parallel's orb, degrees.
    pub orb_deg: f64,
    /// How close in time an edge or the closest instant is found, in the
    /// samples' unit.
    pub tolerance: f64,
}

/// Every span of `samples` — instants in increasing order, each with the
/// moving body's declination there — within the orb of the fixed one.
///
/// `declination_at` gives the moving body's declination at any instant
/// between the first sample and the last. Each edge is narrowed to the
/// tolerance between the two samples it falls between. The closest
/// instant is the first exact parallel in the span, or where the distances
/// come nearest when they never meet. A span that falls wholly between two
/// samples is found when the distances cross there and missed when they
/// only graze the orb, so the caller samples finer than the shortest
/// graze it needs.
///
/// # Errors
///
/// Whatever `declination_at` refuses.
pub fn parallel_spans<E>(
    samples: &[(f64, f64)],
    search: SpanSearch,
    mut declination_at: impl FnMut(f64) -> Result<f64, E>,
) -> Result<Vec<ParallelSpan>, E> {
    let gap = |moving: f64| gap_deg(moving, search.fixed_deg);
    let within = |moving: f64| gap(moving).abs() <= search.orb_deg;
    let tolerance = search.tolerance;
    // Each span as the points it was read at: its edges and the samples
    // between them, and the edges that are edges of the orb.
    let mut found: Vec<(Option<f64>, Points, Option<f64>)> = Vec::new();
    let mut open: Option<(Option<f64>, Points)> = None;
    if let Some(&first) = samples.first().filter(|first| within(first.1)) {
        open = Some((None, vec![first]));
    }
    for pair in samples.windows(2) {
        let [(a, da), (b, db)] = *pair else {
            continue;
        };
        match open.take() {
            Some((enters, mut points)) if within(db) => {
                points.push((b, db));
                open = Some((enters, points));
            }
            Some((enters, mut points)) => {
                let leaves = narrow(a, b, tolerance, |t| declination_at(t).map(|d| !within(d)))?;
                points.push((leaves, declination_at(leaves)?));
                found.push((enters, points, Some(leaves)));
            }
            None if within(db) => {
                let enters = narrow(a, b, tolerance, |t| declination_at(t).map(within))?;
                open = Some((
                    Some(enters),
                    vec![(enters, declination_at(enters)?), (b, db)],
                ));
            }
            // Through the whole orb between two samples: the exact
            // instant first, then each edge on its side of it.
            None if gap(da).is_sign_negative() != gap(db).is_sign_negative() => {
                let far = gap(db).is_sign_negative();
                let exact = narrow(a, b, tolerance, |t| {
                    declination_at(t).map(|d| gap(d).is_sign_negative() == far)
                })?;
                let enters = narrow(a, exact, tolerance, |t| declination_at(t).map(within))?;
                let leaves = narrow(exact, b, tolerance, |t| {
                    declination_at(t).map(|d| !within(d))
                })?;
                let points = vec![
                    (enters, declination_at(enters)?),
                    (exact, declination_at(exact)?),
                    (leaves, declination_at(leaves)?),
                ];
                found.push((Some(enters), points, Some(leaves)));
            }
            None => {}
        }
    }
    if let Some((enters, points)) = open {
        found.push((enters, points, None));
    }
    found
        .into_iter()
        .map(|(enters, points, leaves)| {
            let (closest, declination) = closest(&points, search, &mut declination_at)?;
            Ok(ParallelSpan {
                enters,
                leaves,
                closest,
                closest_deg: gap(declination).abs(),
                contrary: contrary(declination, search.fixed_deg),
            })
        })
        .collect()
}

/// The first instant of a span's points where the distances meet, or
/// where they come nearest, with the moving declination there.
fn closest<E>(
    points: &[(f64, f64)],
    search: SpanSearch,
    declination_at: &mut impl FnMut(f64) -> Result<f64, E>,
) -> Result<(f64, f64), E> {
    let gap = |moving: f64| gap_deg(moving, search.fixed_deg);
    for pair in points.windows(2) {
        let [(a, da), (b, db)] = *pair else {
            continue;
        };
        if gap(da) == 0.0 {
            return Ok((a, da));
        }
        if gap(da).is_sign_negative() != gap(db).is_sign_negative() {
            let far = gap(db).is_sign_negative();
            let exact = narrow(a, b, search.tolerance, |t| {
                declination_at(t).map(|d| gap(d).is_sign_negative() == far)
            })?;
            return Ok((exact, declination_at(exact)?));
        }
    }
    if let [only] = points {
        return Ok(*only);
    }
    // Never met: the nearest point, then the least distance between its
    // neighbours, by golden section.
    let nearest = (0..points.len())
        .min_by(|&i, &j| {
            let at = |k: usize| points.get(k).map_or(f64::INFINITY, |p| gap(p.1).abs());
            at(i).total_cmp(&at(j))
        })
        .unwrap_or(0);
    let time = |k: usize| points.get(k).map_or(f64::NAN, |p| p.0);
    let (mut low, mut high) = (
        time(nearest.saturating_sub(1)),
        time((nearest + 1).min(points.len().saturating_sub(1))),
    );
    let ratio = (5.0_f64.sqrt() - 1.0) / 2.0;
    let mut distance = |t: f64| declination_at(t).map(|d| gap(d).abs());
    let ends = (low, high);
    while high - low > search.tolerance {
        let left = high - ratio * (high - low);
        let right = low + ratio * (high - low);
        if distance(left)? <= distance(right)? {
            high = right;
        } else {
            low = left;
        }
    }
    // A span closest at its first or last instant, such as one still
    // separating when the search starts, is closest at that instant.
    // The earliest of equals wins.
    let mut best = (ends.0, distance(ends.0)?);
    for at in [f64::midpoint(low, high), ends.1] {
        let here = distance(at)?;
        if here < best.1 {
            best = (at, here);
        }
    }
    Ok((best.0, declination_at(best.0)?))
}

/// The first instant in `[low, high]` at which `turned` holds, to within
/// `tolerance`, when it does not hold at `low` and holds at `high`.
fn narrow<E>(
    mut low: f64,
    mut high: f64,
    tolerance: f64,
    mut turned: impl FnMut(f64) -> Result<bool, E>,
) -> Result<f64, E> {
    while high - low > tolerance {
        let middle = low + (high - low) / 2.0;
        if turned(middle)? {
            high = middle;
        } else {
            low = middle;
        }
    }
    Ok(high)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use super::*;

    /// `at` sampled every `step`, `count` steps from 0.
    fn sampled(at: impl Fn(f64) -> f64, step: f64, count: u32) -> Vec<(f64, f64)> {
        (0..=count)
            .map(|k| f64::from(k) * step)
            .map(|t| (t, at(t)))
            .collect()
    }

    fn search(fixed_deg: f64, orb_deg: f64) -> SpanSearch {
        SpanSearch {
            fixed_deg,
            orb_deg,
            tolerance: 1e-9,
        }
    }

    fn spans(
        at: impl Fn(f64) -> f64 + Copy,
        step: f64,
        count: u32,
        asked: SpanSearch,
    ) -> Vec<ParallelSpan> {
        parallel_spans(&sampled(at, step, count), asked, |t| Ok::<_, ()>(at(t))).unwrap()
    }

    #[test]
    fn a_span_open_at_either_end_has_no_edge_there() {
        let rising = |t: f64| 9.5 + 0.1 * t;
        let found = spans(rising, 1.0, 10, search(10.0, 1.0));
        assert_eq!(found.len(), 1);
        let span = found[0];
        assert_eq!(span.enters, None);
        assert_eq!(span.leaves, None);
        assert!((span.closest - 5.0).abs() < 1e-8 && span.closest_deg < 1e-8);
        assert!(!span.contrary);
    }

    #[test]
    fn a_span_crossed_between_two_samples_is_found_by_its_crossing() {
        // 0.05° wide on either side, crossed at 2° a day, sampled daily.
        let falling = |t: f64| 30.0 - 2.0 * t;
        let found = spans(falling, 1.0, 10, search(-20.5, 0.05));
        assert_eq!(found.len(), 1);
        let span = found[0];
        assert!((span.closest - 4.75).abs() < 1e-8, "{span:?}");
        assert!((span.enters.unwrap() - 4.725).abs() < 1e-8);
        assert!((span.leaves.unwrap() - 4.775).abs() < 1e-8);
        assert!(span.contrary);
    }

    #[test]
    fn a_graze_that_never_meets_is_closest_where_the_distances_come_nearest() {
        // Turns at 10.4° on day 5, never reaching 10°.
        let turning = |t: f64| 10.4 + 0.05 * (t - 5.0).powi(2);
        let found = spans(turning, 1.0, 10, search(10.0, 1.0));
        assert_eq!(found.len(), 1);
        let span = found[0];
        assert!((span.closest - 5.0).abs() < 1e-4, "{span:?}");
        assert!((span.closest_deg - 0.4).abs() < 1e-9);
        let edge = 5.0 - (0.6_f64 / 0.05).sqrt();
        assert!((span.enters.unwrap() - edge).abs() < 1e-8);
        assert!((span.leaves.unwrap() - (10.0 - edge)).abs() < 1e-8);
    }

    #[test]
    fn two_meetings_are_one_span_closest_at_the_first() {
        let turning = |t: f64| 10.0 - 0.01 * (t - 5.0).powi(2) + 0.04;
        let found = spans(turning, 0.5, 20, search(10.0, 1.0));
        assert_eq!(found.len(), 1);
        assert!((found[0].closest - 3.0).abs() < 1e-8, "{:?}", found[0]);
        assert!(found[0].closest_deg < 1e-8);
    }

    #[test]
    fn no_samples_no_spans_and_a_refusal_comes_through() {
        assert_eq!(
            parallel_spans(&[], search(1.0, 1.0), |_| Ok::<_, ()>(0.0)),
            Ok(vec![])
        );
        let samples = [(0.0, 5.0), (1.0, 15.0)];
        let refused = parallel_spans(&samples, search(10.0, 1.0), |_| Err("no sky"));
        assert_eq!(refused, Err("no sky"));
    }
}
