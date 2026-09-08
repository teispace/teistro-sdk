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
            Spans::Degrees(widths) => widths.iter().copied().min().map(f64::from),
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

    use super::{TOLERANCE_DAYS, equal_spans, sampling_step};
    use crate::scheme::Scheme;
    use teistro_core::catalogue::Varga;
    use teistro_port_ephemeris::Body;

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
