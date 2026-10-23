//! When a body leaves the sign a divisional chart put it in.
//!
//! "When does Jupiter change navamsha" is the question a transit
//! application asks, and it is a crossing search: the varga sign is a
//! step function of the longitude, so it changes exactly at a part
//! boundary.
//!
//! Twenty of the twenty-one shipped charts, and every arbitrary D-N, cut
//! a sign into equal parts, and their boundaries are a **lattice** —
//! evenly spaced round the whole circle — which `astro::events` already
//! searches efficiently, bracketing by the body's own rate. The
//! trimshamsha's are not evenly spaced, so its changes are found by
//! bisecting on the sign index itself: correct, slower, and used for one
//! chart out of twenty-one.

use serde::{Deserialize, Serialize};
use teistro_astro::events::{Longitudes, Search};
use teistro_core::angle::Nas;
use teistro_core::catalogue::Rashi;
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{Degrees, JulianDay, Ut1};
use teistro_port_ephemeris::{Body, Lattice, Quantity};

use crate::evaluate::place;
use crate::scheme::{Scheme, Spans};

/// How closely a change is placed, days: a hundredth of a second, the
/// tolerance the boundary solver stops at (`astro::events`).
pub const TOLERANCE_DAYS: f64 = 1e-7;

/// The most halvings the unequal-span search takes.
///
/// Sixty-four exceeds a double's resolution over any window a caller can
/// ask for, so a search that needs more is not converging rather than
/// nearly there.
const MOST_HALVINGS: u32 = 64;

/// A body leaving one sign of a divisional chart for another.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Change {
    /// When it happened.
    pub at: JulianDay<Ut1>,
    /// The sign it left.
    pub from: Rashi,
    /// The sign it entered.
    pub into: Rashi,
}

/// Every change of a body's divisional sign inside a window, in order.
///
/// # Errors
///
/// The source's own refusal for an instant or a body, a window that ends
/// before it begins, or — for the trimshamsha alone — a bisection that
/// does not converge, which cannot happen for a real sky and is reported
/// as `NOT_CONVERGED`.
pub fn changes<S: Longitudes + ?Sized>(
    source: &S,
    body: Body,
    scheme: &Scheme,
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
) -> Result<Vec<Change>, Error> {
    if to.get() < from.get() {
        return Err(
            Error::invalid_arg(format!("a window ends before it begins: {from} to {to}"))
                .with_field("to"),
        );
    }
    if to.get() <= from.get() {
        // A window of no width holds no change. Refusing it would make
        // every caller that walks a range guard the degenerate step.
        return Ok(Vec::new());
    }
    if equal_spans(scheme) {
        by_lattice(source, body, scheme, from, to)
    } else {
        by_bisection(source, body, scheme, from, to)
    }
}

/// Whether every group of the chart cuts its signs evenly, which makes
/// the part boundaries a lattice.
fn equal_spans(scheme: &Scheme) -> bool {
    scheme
        .groups
        .iter()
        .all(|group| group.spans == Spans::Equal)
}

/// The lattice that holds every line a chart's sign can change at, for a
/// search that tests it beside others in one scan (the transit hit list's
/// varga ingresses, `03-design/transit-hit-list.md` §7): a part's width
/// for a chart cut evenly, and for the trimshamsha the common divisor of
/// its whole-degree widths, one degree.
///
/// Not every line of it changes the sign. The hora's line at the end of
/// Aries leaves a body in Cancer on both sides, and most of the
/// trimshamsha's one-degree lines fall inside a part, so a crossing of it
/// is a change only where [`either_side`] differs, as [`changes`] keeps
/// it. `None` for a chart mixing even and uneven groups whose even parts
/// are not whole degrees, which no chart the SDK builds does.
///
/// ```
/// use teistro_core::catalogue::Varga;
/// use teistro_vargas::Scheme;
/// use teistro_vargas::change::lines;
///
/// assert_eq!(lines(&Scheme::of(Varga::D9)).map(|l| l.step_deg), Some(30.0 / 9.0));
/// assert_eq!(lines(&Scheme::of(Varga::D30)).map(|l| l.step_deg), Some(1.0));
/// ```
#[must_use]
pub fn lines(scheme: &Scheme) -> Option<Lattice> {
    const fn gcd(a: u16, b: u16) -> u16 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    let divisions = scheme.divisions.max(1);
    if equal_spans(scheme) {
        return Some(Lattice {
            origin_deg: 0.0,
            step_deg: 30.0 / f64::from(divisions),
        });
    }
    // Uneven parts are whole degrees wide, so their edges and the signs'
    // own lie on the widths' common divisor.
    let mut step = 30_u16;
    for group in scheme.groups {
        match group.spans {
            Spans::Degrees { widths } => {
                step = widths
                    .iter()
                    .fold(step, |step, width| gcd(step, u16::from(*width)));
            }
            Spans::Equal if 30 % divisions == 0 => step = gcd(step, 30 / divisions),
            Spans::Equal => return None,
        }
    }
    Some(Lattice {
        origin_deg: 0.0,
        step_deg: f64::from(step.max(1)),
    })
}

/// The chart's sign just before a line of the zodiac and just past it: a
/// body crossing the line forward leaves the first for the second, and
/// crossing it backward the second for the first. The two are equal on a
/// line that changes nothing.
///
/// Read an arcsecond either side of the line, on the canonical angle: a
/// line a search reports a hair off still reads as itself, and no part is
/// that narrow (a D300's is six arcminutes wide).
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varga};
/// use teistro_vargas::Scheme;
/// use teistro_vargas::change::either_side;
///
/// // Ten degrees of Taurus ends the twelfth navamsha from Aries and
/// // begins the thirteenth, which is Aries again.
/// let d9 = Scheme::of(Varga::D9);
/// assert_eq!(either_side(&d9, 40.0)?, (Rashi::Pisces, Rashi::Aries));
/// // The hora's line at the end of Aries changes nothing.
/// let (before, past) = either_side(&Scheme::of(Varga::D2), 30.0)?;
/// assert_eq!(before, past);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a line that is not a finite number.
pub fn either_side(scheme: &Scheme, line_deg: f64) -> Result<(Rashi, Rashi), Error> {
    let line = Nas::try_from_degrees(line_deg)?;
    let nudge = Nas::new(Nas::PER_ARCSECOND);
    Ok((
        place(scheme, line - nudge).sign,
        place(scheme, line + nudge).sign,
    ))
}

/// The changes of an evenly cut chart, from the boundary solver.
fn by_lattice<S: Longitudes + ?Sized>(
    source: &S,
    body: Body,
    scheme: &Scheme,
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
) -> Result<Vec<Change>, Error> {
    let lattice = Lattice {
        origin_deg: 0.0,
        step_deg: 30.0 / f64::from(scheme.divisions.max(1)),
    };
    let events = Search::new(source, Quantity::Longitude(body), lattice)
        .with_tolerance_days(TOLERANCE_DAYS)
        .between(from, to)?;
    let mut changes = Vec::with_capacity(events.len());
    for event in events {
        // Either side of the boundary, by a hundredth of the tolerance's
        // own width, so the two readings cannot land on the same part.
        let step = TOLERANCE_DAYS * 100.0;
        let before = sign_at(source, body, scheme, event.instant.get() - step)?;
        let after = sign_at(source, body, scheme, event.instant.get() + step)?;
        if before != after {
            changes.push(Change {
                at: event.instant,
                from: before,
                into: after,
            });
        }
    }
    Ok(changes)
}

/// The changes of an unevenly cut chart, by bisecting on the sign.
///
/// The trimshamsha's part boundaries are not evenly spaced, so there is
/// no lattice to search. Sampling at a step no wider than the narrowest
/// part takes and halving the bracket where the sign differs finds every
/// change without one, at the cost of a reading per halving.
fn by_bisection<S: Longitudes + ?Sized>(
    source: &S,
    body: Body,
    scheme: &Scheme,
    from: JulianDay<Ut1>,
    to: JulianDay<Ut1>,
) -> Result<Vec<Change>, Error> {
    let step = sampling_step(body, scheme);
    let mut changes = Vec::new();
    let mut left = from.get();
    let mut left_sign = sign_at(source, body, scheme, left)?;
    while left < to.get() {
        let right = (left + step).min(to.get());
        let right_sign = sign_at(source, body, scheme, right)?;
        if right_sign != left_sign {
            let at = narrow(source, body, scheme, left, right, left_sign)?;
            changes.push(Change {
                at: JulianDay::try_new(at)?,
                from: left_sign,
                into: right_sign,
            });
        }
        left = right;
        left_sign = right_sign;
        if right >= to.get() {
            break;
        }
    }
    Ok(changes)
}

/// The instant a bracket's sign changes, to the tolerance.
fn narrow<S: Longitudes + ?Sized>(
    source: &S,
    body: Body,
    scheme: &Scheme,
    mut low: f64,
    mut high: f64,
    low_sign: Rashi,
) -> Result<f64, Error> {
    for _ in 0..MOST_HALVINGS {
        if high - low <= TOLERANCE_DAYS {
            return Ok(high);
        }
        let middle = f64::midpoint(low, high);
        if sign_at(source, body, scheme, middle)? == low_sign {
            low = middle;
        } else {
            high = middle;
        }
    }
    Err(Error::new(
        Status::NotConverged,
        format!("a divisional sign change between {low} and {high} did not narrow"),
    ))
}

/// A sampling step no wider than the narrowest part the chart has, in
/// days, so no change can hide inside one.
fn sampling_step(body: Body, scheme: &Scheme) -> f64 {
    let narrowest = scheme
        .groups
        .iter()
        .filter_map(|group| match group.spans {
            Spans::Equal => Some(30.0 / f64::from(scheme.divisions.max(1))),
            Spans::Degrees { widths } => widths.iter().copied().min().map(f64::from),
        })
        .fold(f64::INFINITY, f64::min);
    let rate = teistro_astro::events::greatest_rate(body).max(f64::EPSILON);
    // Half the narrowest part, so a part cannot be crossed and left
    // between two samples.
    (narrowest / rate / 2.0).max(TOLERANCE_DAYS)
}

/// The divisional sign a body stands in at an instant.
fn sign_at<S: Longitudes + ?Sized>(
    source: &S,
    body: Body,
    scheme: &Scheme,
    jd: f64,
) -> Result<Rashi, Error> {
    let (degrees, _) = source.longitude_and_speed(body, JulianDay::<Ut1>::literal(jd))?;
    let longitude = Nas::from_degrees(Degrees::try_new(degrees.rem_euclid(360.0))?);
    Ok(place(scheme, longitude).sign)
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        reason = "tests fail by panicking"
    )]

    use super::{TOLERANCE_DAYS, either_side, equal_spans, lines, sampling_step};
    use crate::evaluate::place;
    use crate::scheme::{Classifier, Group, Scheme, Spans};
    use teistro_core::angle::Nas;
    use teistro_core::catalogue::Varga;
    use teistro_port_ephemeris::Body;

    /// A chart of the consumer's own reaches every arm of [`lines`]. The
    /// shipped charts are all even, or whole degrees whose common divisor
    /// is one, and neither can tell the widths' divisor from one or
    /// nought, nor an even group beside whole-degree ones.
    #[test]
    fn a_mixed_or_whole_degree_chart_steps_by_its_widths_common_divisor() {
        let map = Scheme::of(Varga::D30).groups[0].map;
        let step = |divisions: u16, spans: &[Spans]| {
            let groups: &'static [Group] = Box::leak(
                spans
                    .iter()
                    .map(|&spans| Group { spans, map })
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            );
            lines(&Scheme {
                varga: None,
                divisions,
                classifier: Classifier::Parity,
                groups,
            })
            .map(|lattice| lattice.step_deg)
        };
        let twelve_and_eighteen = Spans::Degrees { widths: &[12, 18] };
        let ten_and_twenty = Spans::Degrees { widths: &[10, 20] };
        assert_eq!(
            step(2, &[twelve_and_eighteen, twelve_and_eighteen]),
            Some(6.0)
        );
        // Fifths of a sign are six degrees, and six beside ten and twenty
        // leaves two.
        assert_eq!(step(5, &[Spans::Equal, ten_and_twenty]), Some(2.0));
        assert_eq!(step(5, &[ten_and_twenty, Spans::Equal]), Some(2.0));
        // Sevenths are not whole degrees, so no whole-degree lattice holds
        // them beside the widths.
        assert_eq!(step(7, &[Spans::Equal, ten_and_twenty]), None);
    }

    #[test]
    fn every_change_lies_on_the_lines() {
        // Read every chart at the middle of each arcminute of the circle:
        // wherever two neighbours differ, a line of the chart's lattice
        // lies between them. No part is narrower than six arcminutes, so
        // no change hides between two readings.
        let middle =
            |minute: i32| Nas::new(i64::from(minute) * Nas::PER_ARCMINUTE + Nas::PER_ARCMINUTE / 2);
        for varga in Varga::ALL {
            let scheme = Scheme::of(varga);
            let step = lines(&scheme)
                .expect("every chart the SDK builds has one")
                .step_deg;
            let mut changes = 0;
            for minute in 0..360 * 60 {
                let here = place(&scheme, middle(minute)).sign;
                let next = place(&scheme, middle(minute + 1)).sign;
                if here == next {
                    continue;
                }
                changes += 1;
                let (low, high) = (
                    (f64::from(minute) + 0.5) / 60.0,
                    (f64::from(minute) + 1.5) / 60.0,
                );
                let line = (high / step).floor() * step;
                assert!(
                    line >= low - 1e-9,
                    "{varga:?} changes between {low} and {high}, off its lattice"
                );
                // The line between them reads as the change itself.
                let (before, past) = either_side(&scheme, line).unwrap();
                assert_eq!((before, past), (here, next), "{varga:?} at {line}");
            }
            assert!(changes > 0, "{varga:?} changes somewhere");
        }
        // The navamsha changes at every one of its 108 lines; the hora's
        // line at the end of Aries changes nothing.
        let d9 = Scheme::of(Varga::D9);
        assert!((0..108).all(|k| {
            let (before, past) = either_side(&d9, f64::from(k) * 30.0 / 9.0).unwrap();
            before != past
        }));
        let (before, past) = either_side(&Scheme::of(Varga::D2), 30.0).unwrap();
        assert_eq!(before, past);
    }

    #[test]
    fn only_the_trimshamsha_needs_the_slow_path() {
        for varga in Varga::ALL {
            let scheme = Scheme::of(varga);
            assert_eq!(equal_spans(&scheme), varga != Varga::D30, "{varga:?}");
        }
        assert!(equal_spans(&Scheme::cyclic(37).expect("inside the limit")));
    }

    #[test]
    fn the_sampling_step_cannot_skip_a_part() {
        // The narrowest trimshamsha is five degrees; the Moon covers at
        // most 15.5 a day, so the step is under a sixth of a day.
        let step = sampling_step(Body::Moon, &Scheme::of(Varga::D30));
        assert!(step > 0.0 && step < 5.0 / 15.5, "{step}");
        // A slow body takes a wider step, and a large chart a narrower.
        assert!(sampling_step(Body::Saturn, &Scheme::of(Varga::D30)) > step);
        assert!(sampling_step(Body::Moon, &Scheme::cyclic(150).unwrap()) < step);
        // And it never collapses to nothing.
        assert!(sampling_step(Body::Moon, &Scheme::cyclic(300).unwrap()) >= TOLERANCE_DAYS);
    }
}
