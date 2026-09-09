//! The lunar months of a span, with the sankrantis that fall in each,
//! written as JSON for the pass that measures them
//! (`cargo xtask lunisolar`).
//!
//! An Indian lunisolar month runs from one new moon to the next and takes
//! its name from the solar month it belongs to. Two cases break the
//! one-name-per-month correspondence, and the calendar has to say which:
//! a month in which **no** sankranti falls is **adhika**, intercalary,
//! and one in which **two** fall is **kshaya**, omitted. That is the rule
//! this emits the evidence for; it does not apply it, because the point
//! of the measurement is to test the rule against what the sky does
//! rather than to assume it.
//!
//! Everything here is the Surya Siddhanta's own Sun and Moon, so the
//! calendar is computed from the text as the Bikram Sambat engine is, and
//! needs no ephemeris. What the text cannot settle — whether a modern
//! *drik* reckoning classifies a borderline month the same way — is the
//! deferred adapter harness's, and the page says so rather than guessing.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "an example fails by panicking and reports what it measured"
)]

use teistro_calendar::fixed::FixedDay;
use teistro_calendar::solar::SolarModel;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_siddhanta::{Parameters, SuryaSiddhanta, Trig};

/// Ujjain, the prime meridian of Indian astronomy and the place the
/// tradition reckons its day from.
fn ujjain() -> Place {
    Place::new(
        Latitude::literal(23.1765),
        Longitude::literal(75.7885),
        Altitude::literal(494.0),
    )
}

/// The tithi running at an instant, 0 to 29: the elongation in
/// twelfth-parts of the circle.
fn tithi_at(model: &SuryaSiddhanta, jd: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "an elongation over twelve is 0 to 29"
    )]
    let index = (elongation(model, jd) / 12.0) as u8;
    index.min(29)
}

/// A synodic month, days: the mean interval between new moons.
const SYNODIC_DAYS: f64 = 29.530_588_9;

/// How finely a crossing is bracketed before it is narrowed, days.
const STEP_DAYS: f64 = 0.25;

/// The tolerance a crossing is found to, days: under a tenth of a second.
const TOLERANCE_DAYS: f64 = 1e-6;

/// The Moon's elongation from the Sun at an instant, degrees in [0, 360).
///
/// Nought at a new moon, which is what a lunar month begins at.
fn elongation(model: &SuryaSiddhanta, jd: f64) -> f64 {
    let at = JulianDay::<Ut1>::literal(jd);
    let sun = model.sun(at).longitude.get();
    let moon = model.moon(at).longitude.get();
    (moon - sun).rem_euclid(360.0)
}

/// The Sun's sidereal longitude, degrees.
fn sun(model: &SuryaSiddhanta, jd: f64) -> f64 {
    model.sun_longitude_deg(jd)
}

/// The first instant at or after `from` where `f` crosses zero upward
/// after wrapping, found by scanning and then bisecting.
///
/// The elongation and the Sun's longitude both run forwards and wrap, so
/// a crossing is a step where the value falls: the scan looks for that
/// fall and the bisection narrows it.
fn next_wrap(f: impl Fn(f64) -> f64, from: f64, limit: f64) -> Option<f64> {
    let mut lo = from;
    let mut previous = f(lo);
    while lo < limit {
        let hi = lo + STEP_DAYS;
        let value = f(hi);
        if value < previous {
            // The wrap is between `lo` and `hi`; bisect on "has it
            // wrapped yet", which is monotone across the step.
            let (mut a, mut b) = (lo, hi);
            while b - a > TOLERANCE_DAYS {
                let mid = f64::midpoint(a, b);
                if f(mid) < previous {
                    b = mid;
                } else {
                    a = mid;
                }
            }
            return Some(f64::midpoint(a, b));
        }
        previous = value;
        lo = hi;
    }
    None
}

/// The sign the Sun stands in at an instant, 0 for Mesha.
fn sign(model: &SuryaSiddhanta, jd: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a longitude divided by thirty is 0 to 11"
    )]
    let index = (sun(model, jd).rem_euclid(360.0) / 30.0) as u8;
    index.min(11)
}

fn main() {
    let model = SuryaSiddhanta::new(Parameters::TEXT, Trig::Exact);

    // The span the frequencies are measured over, and two dates the
    // corpus marks adhika, so the rule is tested where an authority
    // disagrees with silence.
    let args: Vec<String> = std::env::args().skip(1).collect();
    let from_jd: f64 = args
        .first()
        .and_then(|a| a.parse().ok())
        .unwrap_or(2_415_020.5); // 1900-01-01
    let to_jd: f64 = args
        .get(1)
        .and_then(|a| a.parse().ok())
        .unwrap_or(2_488_069.5); // 2100-01-01

    // Every new moon in the span, which is every lunar month's opening.
    let mut new_moons = Vec::new();
    let mut at = from_jd;
    while at < to_jd {
        match next_wrap(|jd| elongation(&model, jd), at, to_jd + SYNODIC_DAYS) {
            Some(found) => {
                new_moons.push(found);
                at = found + SYNODIC_DAYS * 0.5;
            }
            None => break,
        }
    }

    // Every sankranti in the span: the Sun's longitude wrapping past a
    // sign boundary is a sign change, so the sign at each end of a step
    // says when one happened.
    let mut months = Vec::new();
    for pair in new_moons.windows(2) {
        // `windows(2)` yields exactly two, but say so rather than index.
        let [start, end] = pair else { continue };
        let (start, end) = (*start, *end);
        let mut crossings = Vec::new();
        let mut previous = sign(&model, start);
        let mut walk = start + STEP_DAYS;
        while walk < end {
            let now = sign(&model, walk);
            if now != previous {
                crossings.push(now);
                previous = now;
            }
            walk += STEP_DAYS;
        }
        // The sign at the very end, in case the change lands in the last
        // partial step.
        let last = sign(&model, end);
        if last != previous {
            crossings.push(last);
        }
        months.push(serde_json::json!({
            "start_jd": start,
            "end_jd": end,
            "sign_at_start": sign(&model, start),
            "sign_at_end": last,
            "sankrantis": crossings,
        }));
    }

    // The tithi at each sunrise over the span, which is what a lunisolar
    // date's *day* is. A tithi runs about 23 to 26 hours, so it can span
    // two sunrises or none: the first repeats a date's day and the second
    // skips one, and how often each happens decides whether the date can
    // be written without a flag for it.
    let mut sunrise_tithis = Vec::new();
    let (mut repeated, mut skipped, mut days) = (0u32, 0u32, 0u32);
    let place = ujjain();
    let mut previous: Option<u8> = None;
    let mut fixed = FixedDay::from_jd(JulianDay::literal(from_jd)).0;
    let last = FixedDay::from_jd(JulianDay::literal(to_jd)).0;
    while fixed < last {
        if let Ok(light) = SolarModel::day_light(&model, fixed, &place)
            && let Some(arc) = light.arc()
        {
            let tithi = tithi_at(&model, arc.sunrise.get());
            days += 1;
            if let Some(before) = previous {
                if tithi == before {
                    repeated += 1;
                } else if u32::from((tithi + 30 - before) % 30) > 1 {
                    skipped += 1;
                }
            }
            previous = Some(tithi);
            if sunrise_tithis.len() < 64 {
                sunrise_tithis.push(tithi);
            }
        }
        fixed = fixed.plus_days(1);
    }

    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "model": model.describe(),
            "sunrise_days": days,
            "repeated_tithis": repeated,
            "skipped_tithis": skipped,
            "from_jd": from_jd,
            "to_jd": to_jd,
            "months": months,
        }))
        .expect("the months serialise")
    );
}
