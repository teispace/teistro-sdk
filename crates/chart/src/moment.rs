//! What a chart's instant falls in: the span of the sign rising and the
//! eclipse, if any, seen where the chart is cast.
//!
//! Neither is a limb of the day. The day's almanac holds the tithi's and
//! the nakshatra's spans, but the lagna moves at its own rate through a
//! sign in two hours or so, and an eclipse is a condition of the sky at
//! one place rather than of a civil day. A rule that asks whether a birth
//! fell in the last half ghatika of a water sign (BPHS ch. 92 v. 4) or in
//! a lunar eclipse (Brihat Jataka ch. 6 v. 9) reads them here.

use serde::{Deserialize, Serialize};
use teistro_core::Error;
use teistro_core::catalogue::Rashi;
use teistro_core::quantity::{JulianDay, Ut1, Utc};

/// How far the sign's edges are looked for either side of the instant,
/// days. A sign rises in under six hours outside the polar circles; inside
/// them one can stay up for days, and its edge is then left unknown.
const REACH_DAYS: f64 = 1.0;

/// The step the walk to an edge takes, days: five minutes, under which no
/// sign rises outside the polar circles.
const STEP_DAYS: f64 = 5.0 / 1440.0;

/// How finely an edge is found, days: a second.
const TOLERANCE_DAYS: f64 = 1.0 / 86_400.0;

/// The instant's own facts.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Moment {
    /// The sign rising at the instant and when it rose and sets.
    pub rising: RisingSign,
    /// The eclipse the instant falls in, as seen where the chart is cast.
    pub eclipse: BirthEclipse,
}

/// The sign rising at an instant, and its span.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct RisingSign {
    /// The sign.
    pub sign: Rashi,
    /// When it began to rise, UTC; `None` when it was rising a day before,
    /// which only a place inside a polar circle sees.
    pub rose: Option<JulianDay<Utc>>,
    /// When the next sign begins to rise, UTC; `None` likewise.
    pub sets: Option<JulianDay<Utc>>,
}

/// The eclipse an instant falls in, at one place.
///
/// An eclipse counts while it can be seen there: a lunar eclipse's umbral
/// phase with the Moon up, which a penumbral eclipse never has, and a
/// solar eclipse's partial phase with the Sun up. That is how the
/// observances count an eclipse's time (*Dharmasindhu*), and the almanac's
/// own eclipses report the same spans (`astro::eclipse::local`, C378).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BirthEclipse {
    /// No eclipse is seen at the instant.
    None,
    /// A lunar eclipse's umbral phase, the Moon up.
    Lunar {
        /// Its greatest moment, UT1.
        greatest: JulianDay<Ut1>,
    },
    /// A solar eclipse, the Sun up.
    Solar {
        /// Its greatest moment, UT1.
        greatest: JulianDay<Ut1>,
    },
    /// Not reckoned: a classical astronomy's eclipse is its own method,
    /// which the SDK does not build (crux C188), and a modern eclipse
    /// beside a text's sky would answer a question the chart did not ask.
    Unjudged,
}

/// The sign rising at `at` and when it rose and sets, from `lagna_at`, the
/// lagna's longitude in degrees at an instant.
///
/// Each edge is walked to in `STEP_DAYS` and found to a second by
/// bisection; an edge beyond `REACH_DAYS` is `None`.
///
/// # Errors
///
/// Whatever `lagna_at` returns.
pub fn rising<F>(at: JulianDay<Utc>, mut lagna_at: F) -> Result<RisingSign, Error>
where
    F: FnMut(JulianDay<Utc>) -> Result<f64, Error>,
{
    let mut sign_at = |t: f64| lagna_at(JulianDay::literal(t)).map(Rashi::of_longitude);
    let sign = sign_at(at.get())?;
    let rose = edge(at.get(), -1.0, sign, &mut sign_at)?;
    let sets = edge(at.get(), 1.0, sign, &mut sign_at)?;
    Ok(RisingSign {
        sign,
        rose: rose.map(JulianDay::literal),
        sets: sets.map(JulianDay::literal),
    })
}

/// The edge of `sign` from `from` in the direction `toward` (−1 back, +1
/// on): the first instant in the sign walking back, the first instant out
/// of it walking on.
fn edge<F>(from: f64, toward: f64, sign: Rashi, sign_at: &mut F) -> Result<Option<f64>, Error>
where
    F: FnMut(f64) -> Result<Rashi, Error>,
{
    let mut inside = from;
    let mut walked = 0.0;
    while walked < REACH_DAYS {
        walked += STEP_DAYS;
        let next = from + toward * walked;
        if sign_at(next)? == sign {
            inside = next;
            continue;
        }
        let mut outside = next;
        loop {
            let mid = f64::midpoint(inside, outside);
            let (low, high) = (inside.min(outside), inside.max(outside));
            // Converged, or below the last bit at this magnitude (a midpoint
            // that rounds onto an end cannot split the interval further).
            if high - low <= TOLERANCE_DAYS || mid <= low || mid >= high {
                break;
            }
            if sign_at(mid)? == sign {
                inside = mid;
            } else {
                outside = mid;
            }
        }
        return Ok(Some(if toward < 0.0 { inside } else { outside }));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// A lagna turning a degree every four minutes from 0° at day 10:
    /// each sign rises in exactly two hours.
    fn even(t: JulianDay<Utc>) -> f64 {
        ((t.get() - 10.0) * 360.0).rem_euclid(360.0)
    }

    #[test]
    fn a_sign_is_found_to_a_second_either_side() {
        // 45° is Taurus, which rose at 30° and sets at 60°.
        let at = JulianDay::literal(10.0 + 45.0 / 360.0);
        let found = rising(at, |t| Ok(even(t))).unwrap();
        assert_eq!(found.sign, Rashi::Taurus);
        let second = 1.0 / 86_400.0;
        assert!((found.rose.unwrap().get() - (10.0 + 30.0 / 360.0)).abs() <= second);
        assert!((found.sets.unwrap().get() - (10.0 + 60.0 / 360.0)).abs() <= second);
        // The rose edge is inside the sign and the sets edge outside it.
        assert_eq!(
            Rashi::of_longitude(even(found.rose.unwrap())),
            Rashi::Taurus
        );
        assert_eq!(
            Rashi::of_longitude(even(found.sets.unwrap())),
            Rashi::Gemini
        );
    }

    #[test]
    fn a_sign_that_does_not_set_within_a_day_has_no_edge() {
        let still = |_: JulianDay<Utc>| Ok(100.0);
        let found = rising(JulianDay::literal(10.0), still).unwrap();
        assert_eq!(
            (found.sign, found.rose, found.sets),
            (Rashi::Cancer, None, None)
        );
    }

    #[test]
    fn an_eclipse_is_written_by_its_kind() {
        let lunar = BirthEclipse::Lunar {
            greatest: JulianDay::literal(2_460_000.5),
        };
        assert_eq!(
            serde_json::to_value(lunar).unwrap(),
            serde_json::json!({"kind": "LUNAR", "greatest": 2_460_000.5})
        );
        assert_eq!(
            serde_json::to_value(BirthEclipse::None).unwrap(),
            serde_json::json!({"kind": "NONE"})
        );
    }
}
