//! The upagrahas the day divides: Gulika and Mandi, which are the
//! ascendant at the two ends of Saturn's eighth of the arc.
//!
//! The arc a birth falls in — the daylight, or the night that follows
//! it — is cut into eight. Which eighth is Saturn's the catalogue
//! carries, measured against the recorded panchanga on all 55 days
//! (`Kaala::GulikaKaala`); a night birth walks that table five weekdays
//! on, the same walk the choghadiya take.
//!
//! **Gulika begins the portion and Mandi ends it.** The research page
//! marks both halves of that "verify" — where inside the portion, and
//! whether the two names differ at all — and the falsification pass
//! settled it by trying all twenty-four candidate instants against every
//! recorded value rather than proposing one: two readings of one
//! portion, start against end and not start against middle, on all 71
//! fixtures (`03-design/points-measured.md` §6).
//!
//! What this needs and a foundation does not carry is the **ascendant at
//! an instant that is not the birth**, which is why [`Ascendant`] is a
//! trait rather than a dependency: the ascendant is the sidereal time
//! and the latitude, so no ephemeris is involved and a test implements
//! it in four lines.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Kaala, Point, Vara};
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};

use crate::derived::Derived;

/// How many parts an arc is cut into.
pub const EIGHTHS: u16 = 8;

/// How many weekdays on the night's own sequence begins: the fifth lord
/// from the day's, which is four steps.
pub const NIGHT_WALK: u8 = 4;

/// Something that can say where the ecliptic rises at an instant.
///
/// The two day-division points are read off the ascendant at the ends of
/// a portion, which is not the birth, so this module cannot take those
/// from a founded chart. It takes them from whatever the caller has:
/// `chart` implements this over `astro::houses`, and a test implements
/// it over a straight line.
pub trait Ascendant {
    /// The ascendant at an instant, in the chart's own zodiac, degrees.
    ///
    /// # Errors
    ///
    /// Whatever the implementation cannot answer, unchanged. This module
    /// does not wrap it.
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error>;
}

impl<F> Ascendant for F
where
    F: Fn(JulianDay<Utc>) -> Result<f64, Error>,
{
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        self(at)
    }
}

/// Saturn's eighth of an arc: which part it is, and when it runs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Portion {
    /// Which eighth, counted from one as an almanac prints it.
    pub eighth: u8,
    /// When it runs.
    pub at: Interval,
}

/// Which eighth of its arc is Saturn's, on a chart of this vara and
/// half of the day, counted from one.
///
/// The daylight takes the catalogue's table as it stands; the night
/// takes it walked [`NIGHT_WALK`] weekdays on.
#[must_use]
pub fn saturns_eighth(vara: Vara, is_day: bool) -> Option<u8> {
    let weekday = vara.attributes().weekday;
    let index = if is_day {
        weekday
    } else {
        (weekday + NIGHT_WALK) % 7
    };
    Kaala::GulikaKaala
        .attributes()
        .eighth_by_vara
        .get(usize::from(index))
        .copied()
}

/// Saturn's portion of an arc.
///
/// # Errors
///
/// `UNSUPPORTED` for an arc with no length, which is what a polar day or
/// a polar night gives: an eighth of nothing is not an instant, and an
/// empty portion would answer "Gulika is at sunrise" for a place where
/// the Sun did not rise.
pub fn saturns(arc: Interval, vara: Vara, is_day: bool) -> Result<Portion, Error> {
    if arc.is_empty() {
        return Err(Error::unsupported(format!(
            "the {} arc has no length, so it has no eighths; a polar day or night \
             has no Gulika",
            if is_day { "daylight" } else { "night" }
        ))
        .with_field("points.arc"));
    }
    let eighth = saturns_eighth(vara, is_day).ok_or_else(|| {
        Error::unsupported(format!("no eighth is Saturn's on {}", vara.key()))
            .with_field("points.vara")
    })?;
    let at = arc.part(u16::from(eighth.saturating_sub(1)), EIGHTHS)?;
    Ok(Portion { eighth, at })
}

/// Gulika and Mandi: the ascendant where Saturn's portion begins and
/// where it ends.
///
/// # Errors
///
/// As [`saturns`], and whatever the [`Ascendant`] returns.
pub fn day_division(
    arc: Interval,
    vara: Vara,
    is_day: bool,
    ascendant: &dyn Ascendant,
) -> Result<[Derived; 2], Error> {
    let portion = saturns(arc, vara, is_day)?;
    Ok([
        Derived::at(Point::Gulika, ascendant.ascendant_deg(portion.at.from)?)?,
        Derived::at(Point::Mandi, ascendant.ascendant_deg(portion.at.to)?)?,
    ])
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::float_cmp,
        clippy::unnecessary_wraps,
        reason = "tests fail by panicking, index a fixed-length pair, compare longitudes computed the same way, and match the `Ascendant` trait's shape"
    )]

    use super::{EIGHTHS, day_division, saturns, saturns_eighth};
    use teistro_core::catalogue::{Point, Vara};
    use teistro_core::error::Error;
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Utc};

    /// An ascendant that turns once through the day, which is enough to
    /// tell one instant from another.
    fn rising(at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok((at.get() * 360.0).rem_euclid(360.0))
    }

    #[test]
    fn saturns_eighth_is_the_catalogue_s_by_day_and_walked_by_night() {
        // The daylight table, Sunday first.
        let day: Vec<u8> = Vara::ALL
            .into_iter()
            .map(|vara| saturns_eighth(vara, true).unwrap())
            .collect();
        assert_eq!(day, vec![7, 6, 5, 4, 3, 2, 1]);
        // The night's is the same table five weekdays on.
        let night: Vec<u8> = Vara::ALL
            .into_iter()
            .map(|vara| saturns_eighth(vara, false).unwrap())
            .collect();
        assert_eq!(night, vec![3, 2, 1, 7, 6, 5, 4]);
        for (index, vara) in Vara::ALL.into_iter().enumerate() {
            assert_eq!(
                saturns_eighth(vara, false),
                Some(day[(index + 4) % 7]),
                "{vara:?}"
            );
        }
    }

    #[test]
    fn the_eight_portions_partition_the_arc() {
        let arc = Interval::literal(2_460_000.25, 2_460_000.75);
        for vara in Vara::ALL {
            for is_day in [true, false] {
                let portion = saturns(arc, vara, is_day).unwrap();
                assert!((1..=8).contains(&portion.eighth), "{vara:?}");
                // The portion is one eighth of the arc, inside it.
                assert!(
                    (portion.at.days() - arc.days() / f64::from(EIGHTHS)).abs() < 1e-12,
                    "{vara:?}"
                );
                assert!(portion.at.from.get() >= arc.from.get() - 1e-12);
                assert!(portion.at.to.get() <= arc.to.get() + 1e-12);
            }
        }
    }

    #[test]
    fn gulika_begins_the_portion_and_mandi_ends_it() {
        let arc = Interval::literal(2_460_000.25, 2_460_000.75);
        let vara = Vara::ALL[0];
        let portion = saturns(arc, vara, true).unwrap();
        let found = day_division(arc, vara, true, &rising).unwrap();
        assert_eq!(found[0].point, Point::Gulika);
        assert_eq!(found[1].point, Point::Mandi);
        assert!((found[0].longitude_deg - rising(portion.at.from).unwrap()).abs() < 1e-9);
        assert!((found[1].longitude_deg - rising(portion.at.to).unwrap()).abs() < 1e-9);
        assert_ne!(found[0].longitude_deg, found[1].longitude_deg);
    }

    #[test]
    fn an_arc_with_no_length_has_no_gulika() {
        let empty = Interval::literal(2_460_000.25, 2_460_000.25);
        let error = saturns(empty, Vara::ALL[0], true).expect_err("a polar day");
        assert_eq!(error.field(), Some("points.arc"));
        assert!(error.message.contains("daylight"), "{error}");
        let night = saturns(empty, Vara::ALL[0], false).expect_err("a polar night");
        assert!(night.message.contains("night"), "{night}");
        assert!(day_division(empty, Vara::ALL[0], true, &rising).is_err());
    }

    #[test]
    fn an_ascendant_that_refuses_is_reported_unchanged() {
        let arc = Interval::literal(2_460_000.25, 2_460_000.75);
        let refusing = |_: JulianDay<Utc>| {
            Err(
                Error::unsupported(String::from("the provider has no positions there"))
                    .with_field("provider.instant"),
            )
        };
        let error = day_division(arc, Vara::ALL[0], true, &refusing).expect_err("refused");
        assert_eq!(error.field(), Some("provider.instant"), "unwrapped");
    }
}
