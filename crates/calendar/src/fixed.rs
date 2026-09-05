//! The fixed day number of Reingold and Dershowitz: day 1 is Monday,
//! 1 January 1 CE in the proleptic Gregorian calendar. Every calendar
//! converts through it, and the Julian day meets it at midnight.

use core::fmt;

use teistro_core::catalogue::Vara;
use teistro_core::quantity::{InvalidValue, JulianDay, Utc};

/// A fixed day number.
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(transparent)]
pub struct FixedDay(i64);

impl FixedDay {
    /// The Julian day at the midnight that begins fixed day 0.
    pub const JD_EPOCH: f64 = 1_721_424.5;
    /// Fixed day 1: Monday, 1 January 1 CE.
    pub const EPOCH: FixedDay = FixedDay(1);

    /// A fixed day.
    #[must_use]
    pub const fn new(day: i64) -> FixedDay {
        FixedDay(day)
    }

    /// The number.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The day `days` later, or earlier when negative.
    #[must_use]
    pub const fn plus_days(self, days: i64) -> FixedDay {
        FixedDay(self.0 + days)
    }

    /// The days from `self` to `other`.
    #[must_use]
    pub const fn days_until(self, other: FixedDay) -> i64 {
        other.0 - self.0
    }

    /// The Julian day at the midnight that begins this day.
    ///
    /// # Errors
    ///
    /// A day too far for a finite Julian day, which no calendar reaches.
    #[allow(clippy::cast_precision_loss, reason = "days are far below 2^53")]
    pub fn jd_at_midnight(self) -> Result<JulianDay<Utc>, InvalidValue> {
        JulianDay::try_new(self.0 as f64 + FixedDay::JD_EPOCH)
    }

    /// The fixed day a Julian day falls in, and the fraction of that day
    /// elapsed since its midnight.
    #[must_use]
    #[allow(
        clippy::cast_possible_truncation,
        reason = "the floor of a finite day count"
    )]
    pub fn from_jd(jd: JulianDay<Utc>) -> (FixedDay, f64) {
        let shifted = jd.get() - FixedDay::JD_EPOCH;
        let day = shifted.floor();
        (FixedDay(day as i64), shifted - day)
    }

    /// The weekday.
    #[must_use]
    #[allow(clippy::cast_possible_truncation, reason = "a residue below seven")]
    pub const fn weekday(self) -> Weekday {
        match self.0.rem_euclid(7) as u8 {
            0 => Weekday::Sunday,
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            _ => Weekday::Saturday,
        }
    }

    /// The most recent day on or before `self` that falls on `weekday`.
    #[must_use]
    pub const fn on_or_before(self, weekday: Weekday) -> FixedDay {
        FixedDay(self.0 - (self.0 - weekday as i64).rem_euclid(7))
    }
}

impl fmt::Display for FixedDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RD {}", self.0)
    }
}

/// A weekday; Sunday is 0, as fixed day 0 was a Sunday.
#[repr(u8)]
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Weekday {
    /// Sunday.
    Sunday = 0,
    /// Monday.
    Monday = 1,
    /// Tuesday.
    Tuesday = 2,
    /// Wednesday.
    Wednesday = 3,
    /// Thursday.
    Thursday = 4,
    /// Friday.
    Friday = 5,
    /// Saturday.
    Saturday = 6,
}

impl Weekday {
    /// Every weekday from Sunday.
    pub const ALL: [Weekday; 7] = [
        Weekday::Sunday,
        Weekday::Monday,
        Weekday::Tuesday,
        Weekday::Wednesday,
        Weekday::Thursday,
        Weekday::Friday,
        Weekday::Saturday,
    ];

    /// The ISO number, Monday 1 to Sunday 7.
    #[must_use]
    pub const fn iso_number(self) -> u8 {
        match self {
            Weekday::Sunday => 7,
            other => other as u8,
        }
    }

    /// The weekday with an ISO number.
    #[must_use]
    pub const fn from_iso_number(number: u8) -> Option<Weekday> {
        match number {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }

    /// The vara of the same day under the sunrise-anchored reckoning,
    /// which the panchanga applies; the number is the same.
    #[must_use]
    pub fn vara(self) -> Vara {
        Vara::from_id(self as u16).unwrap_or(Vara::Ravivara)
    }

    /// The English name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Weekday::Sunday => "Sunday",
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    #[test]
    fn the_epoch_and_its_weekday() {
        assert_eq!(FixedDay::EPOCH.weekday(), Weekday::Monday);
        assert_eq!(FixedDay::new(0).weekday(), Weekday::Sunday);
        assert_eq!(FixedDay::new(-1).weekday(), Weekday::Saturday);
        let jd = FixedDay::EPOCH.jd_at_midnight().unwrap();
        assert!((jd.get() - 1_721_425.5).abs() < 1e-9);
        let (day, fraction) = FixedDay::from_jd(JulianDay::try_new(2_451_545.0).unwrap());
        assert_eq!(day, FixedDay::new(730_120));
        assert!((fraction - 0.5).abs() < 1e-9);
        assert_eq!(FixedDay::new(730_120).weekday(), Weekday::Saturday);
        assert_eq!(
            FixedDay::new(10).on_or_before(Weekday::Sunday),
            FixedDay::new(7)
        );
        assert_eq!(Weekday::Sunday.iso_number(), 7);
        assert_eq!(Weekday::from_iso_number(7), Some(Weekday::Sunday));
        assert_eq!(Weekday::Tuesday.vara().key(), "MANGALAVARA");
    }
}
