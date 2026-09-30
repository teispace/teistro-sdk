//! The sixty-year cycle: the Jovian year running at an instant, and the
//! name a lunar year carries.
//!
//! Two readings of the cycle are in use, and they name the same year
//! differently (`03-design/samvatsara-measured.md`):
//!
//! - **Barhaspatya**, north India's and Nepal's: a year is the time
//!   Jupiter's *mean* place takes to cross one sign. The Surya Siddhanta's
//!   rule (I.55) counts the signs its mean Jupiter has crossed since the
//!   Kali age began and reads the count modulo sixty from Vijaya. A
//!   Jovian year is 361 days and a little over eleven hours, so it
//!   starts about four days earlier each solar year, and a name that
//!   begins and ends inside one lunar year never names a year: it is
//!   *lupta*, expunged, about once in eighty-six years.
//! - **Chandramana**, the south's: the names follow the lunar years in
//!   order and nothing is skipped, so a year's name is its Shaka number
//!   modulo sixty.
//!
//! Which Jovian year names a lunar year Nepal's panchanga committee
//! states in terms: the one that meets the day of Chaitra Shukla
//! Pratipada is said in the year's rites. That rule is not here: it
//! needs the day, which the almanac founds, so this module gives the
//! Jovian year at any instant and the almanac asks it at the day the
//! year begins.
//!
//! The mean Jupiter is the text's, whatever sky the almanac is otherwise
//! founded on. The cycle is a count the text defines, not a measurement,
//! and a modern mean Jupiter changes sign about two months later: read
//! against the named years it is simply wrong, which the page measures.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Samvatsara;
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_siddhanta::{Ahargana, Planet};
// Re-exported: the parameters a count is made under are the caller's choice,
// the text's own or with a bija.
pub use teistro_siddhanta::{Bija, Parameters};

/// Signs in a revolution.
const SIGNS: i128 = 12;

/// The cycle's length in years.
const CYCLE: i64 = 60;

/// Vijaya's place in the cycle counted from Prabhava: the Surya Siddhanta
/// counts from Vijaya (I.55), and the names are catalogued from Prabhava.
const VIJAYA: i64 = 26;

/// The Shaka year the southern count puts first in a cycle, less sixty:
/// Shaka 1947 (2025–26) is Vishvavasu, the 39th, so the count is the
/// Shaka year plus eleven.
const SHAKA_TO_CYCLE: i64 = 11;

/// One Jovian year: which name it carries and when it ran.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct JovianYear {
    /// The year's name.
    pub member: Samvatsara,
    /// The signs mean Jupiter had crossed since the Kali age began: the
    /// year's number, counted from 0. Burgess's worked example (I.55)
    /// calls the year Jupiter entered in February 1859 "the 5019th since
    /// the epoch", which is 5018 here.
    pub count: i64,
    /// When mean Jupiter entered the sign.
    pub from: JulianDay<Utc>,
    /// When it entered the next one, which is when the next year began.
    pub to: JulianDay<Utc>,
}

impl JovianYear {
    /// The Jovian year running at an instant under a set of parameters:
    /// the text's own ([`Parameters::TEXT`]), or the text with a bija,
    /// which is how the page measures the corrected reading.
    ///
    /// # Errors
    ///
    /// Only an instant whose year's bounds leave the Julian-day range,
    /// which no calendar date reaches.
    pub fn at(params: &Parameters, at: JulianDay<Utc>) -> Result<JovianYear, Error> {
        let (per_sign, days_from_epoch) = Self::rates(params);
        let day = Ahargana::at(at.get(), params);
        // The signs crossed, in integers: the whole days times the signs
        // an age holds, over the days in it, and the day's fraction added
        // to the remainder so that only it is floating.
        let whole = i128::from(day.days) * per_sign;
        let crossed = whole.div_euclid(days_from_epoch);
        let rest = whole.rem_euclid(days_from_epoch);
        #[allow(
            clippy::cast_precision_loss,
            reason = "a remainder below the days in an age, and signs in one"
        )]
        let spill =
            ((rest as f64 + day.fraction * per_sign as f64) / days_from_epoch as f64).floor();
        #[allow(clippy::cast_possible_truncation, reason = "a floored count of 0 or 1")]
        let count = crossed + spill as i128;
        // The bounds are rounded to a Julian day's last bit, and the count
        // is exact, so an instant a year reports as its end can count as
        // inside it. The bounds are what a caller holds, so they decide.
        let year = Self::numbered(params, count)?;
        if at.get() >= year.to.get() {
            Self::numbered(params, count + 1)
        } else if at.get() < year.from.get() {
            Self::numbered(params, count - 1)
        } else {
            Ok(year)
        }
    }

    /// The Jovian year with a count: its name and its bounds.
    ///
    /// # Errors
    ///
    /// As [`JovianYear::at`].
    pub fn numbered(params: &Parameters, count: impl Into<i128>) -> Result<JovianYear, Error> {
        let count: i128 = count.into();
        let from = Self::entry(params, count)?;
        let to = Self::entry(params, count + 1)?;
        #[allow(
            clippy::cast_possible_truncation,
            reason = "the signs since the Kali age are far inside i64"
        )]
        let count = count as i64;
        Ok(JovianYear {
            member: in_cycle(count + VIJAYA),
            count,
            from,
            to,
        })
    }

    /// When mean Jupiter entered its `count`th sign since the Kali age:
    /// the days since the epoch are `count × days / signs` for an age,
    /// its whole part in integers.
    fn entry(params: &Parameters, count: i128) -> Result<JulianDay<Utc>, Error> {
        let (per_sign, days_from_epoch) = Self::rates(params);
        let scaled = count * days_from_epoch;
        let days = scaled.div_euclid(per_sign);
        let fraction = scaled.rem_euclid(per_sign);
        #[allow(
            clippy::cast_precision_loss,
            reason = "days since the epoch and a remainder below the signs in an age"
        )]
        let jd = params.epoch_jd_ut + days as f64 + fraction as f64 / per_sign as f64;
        JulianDay::try_new(jd).map_err(Error::from)
    }

    /// The signs mean Jupiter crosses in its cycle, and the days in that
    /// cycle.
    fn rates(params: &Parameters) -> (i128, i128) {
        let motion = params.motion(Planet::Jupiter);
        (
            i128::from(motion.revolutions) * SIGNS,
            i128::from(params.cycle_days(motion.cycle)),
        )
    }
}

/// The southern name of a lunar year, from its Shaka number (the year
/// that began at its Chaitra Shukla Pratipada, counted from Shaka 0).
#[must_use]
pub fn chandramana(shaka: i64) -> Samvatsara {
    in_cycle(shaka + SHAKA_TO_CYCLE)
}

/// The name at a place in the cycle counted from Prabhava, any integer
/// read modulo sixty.
fn in_cycle(place: i64) -> Samvatsara {
    let index = usize::try_from(place.rem_euclid(CYCLE)).unwrap_or_default();
    // A residue modulo sixty always indexes the sixty names.
    Samvatsara::ALL
        .get(index)
        .copied()
        .unwrap_or(Samvatsara::Prabhava)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    fn jd(y: i32, m: u8, d: u8) -> JulianDay<Utc> {
        crate::gregorian::fixed_from_gregorian(y, m, d)
            .jd_at_midnight()
            .unwrap()
    }

    #[test]
    fn burgess_worked_example_names_1859_prajapati() {
        // I.55's note: the current year is "the 5019th since the epoch",
        // remainder 39 from Vijaya, Prajapati, begun on 23 February 1859
        // by the text or 3 April with the bija.
        let year = JovianYear::at(&Parameters::TEXT, jd(1859, 6, 1)).unwrap();
        assert_eq!(year.count + 1, 5019);
        assert_eq!(year.member, Samvatsara::Prajotpatti);
        let began = year.from.get() - jd(1859, 2, 23).get();
        assert!(began.abs() < 2.0, "began {began} days from 23 February");
        let bija = Parameters::TEXT.with_bija(&Bija {
            jupiter: -8,
            ..Bija::default()
        });
        let corrected = JovianYear::at(&bija, jd(1859, 6, 1)).unwrap();
        assert_eq!(corrected.member, Samvatsara::Prajotpatti);
        let began = corrected.from.get() - jd(1859, 4, 3).get();
        assert!(began.abs() < 2.0, "began {began} days from 3 April");
    }

    #[test]
    fn a_year_is_bounded_by_its_neighbours_and_names_the_next() {
        let year = JovianYear::at(&Parameters::TEXT, jd(2025, 1, 1)).unwrap();
        let next = JovianYear::at(&Parameters::TEXT, year.to).unwrap();
        assert_eq!(next.count, year.count + 1);
        assert_eq!(next.from, year.to);
        let before = JovianYear::at(
            &Parameters::TEXT,
            JulianDay::try_new(year.from.get() - 1e-6).unwrap(),
        )
        .unwrap();
        assert_eq!(before.count, year.count - 1);
        let length = year.to.get() - year.from.get();
        assert!((length - 361.026_7).abs() < 1e-3, "{length}");
    }

    #[test]
    fn the_southern_count_names_2025_vishvavasu() {
        assert_eq!(chandramana(1947), Samvatsara::Vishvavasu);
        assert_eq!(chandramana(1946), Samvatsara::Krodhi);
        assert_eq!(chandramana(1947 + 60), Samvatsara::Vishvavasu);
    }
}
