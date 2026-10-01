//! The Nepal Sambat date of a day: its year, its month and its half
//! (`03-design/calendar-indian-lunisolar.md` §11).
//!
//! Nepal Sambat is a reading of the lunisolar calendar the almanac
//! already keeps, not a calendar of its own. Its months are the amanta
//! months under their own names, counted from Kartika: Kachhala is
//! amanta Kartika and Kaula amanta Ashwina, and an adhika month, whichever
//! it repeats, is Anala. Its halves are the pakshas, *thwa* the bright
//! and *gā* the dark. Its year opens at Kachhala's first day, the sunrise
//! after Kartika's new moon. Nepal's panchanga committee prints all four
//! in every page header, and VS 2082's and 2083's 51 headers are the
//! measured page (`03-design/nepal-sambat-measured.md`).
//!
//! So a day carries everything its date needs but the year's number, and
//! that is the civil year the Nepal Sambat year's middle falls in, less
//! 880: the year 1146 opened on 2025-10-22, and its middle is April 2026.

use serde::{Deserialize, Serialize};
use teistro_calendar::FixedDay;
use teistro_calendar::gregorian::year_from_fixed;
use teistro_calendar::lunisolar::{MonthKind, SYNODIC_DAYS};
use teistro_core::catalogue::{Kind, Masa, Paksha, write_in_full};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::almanac::Panchanga;

/// The amanta month Nepal Sambat counts its year from: Kachhala.
const FIRST: Masa = Masa::Kartika;

/// The Nepal Sambat year whose middle falls in civil year 0. The year
/// 1146 opened at Kartika's new moon in October 2025, so its middle is in
/// 2026.
const YEAR_AT_ZERO: i32 = -880;

/// A day's Nepal Sambat date: the committee's "ने.सं. ११४६ (कछलाथ्व)".
///
/// The tithi that completes the date is the day's own
/// (`Panchanga::limbs`), and `sdk.calendar.nepalSambatMonth`,
/// `nepalSambatHalf` and `nepalSambatDate` say the rest in every locale.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct NepalSambatDate {
    /// The year, which opens at Kachhala's first day: 1146 from
    /// 2025-10-22, 1147 from 2026-11-10.
    pub year: i32,
    /// The month, counted from 1 for Kachhala (amanta Kartika) to 12 for
    /// Kaula (amanta Ashwina). An adhika month keeps the number of the
    /// month it repeats, and `kind` says it is Anala.
    pub month: u8,
    /// Whether the month is ordinary, intercalary (Anala) or omitted: the
    /// day's own lunar month's kind.
    pub kind: MonthKind,
    /// The half: Shukla is *thwa* and Krishna *gā*.
    pub paksha: Paksha,
}

impl NepalSambatDate {
    /// The catalogue members a date holds, by JSON path, with their
    /// kinds: what [`NepalSambatDate::in_full`] writes in full at the
    /// boundary.
    pub const MEMBERS: [(&'static str, Kind); 1] = [("paksha", Kind::Paksha)];

    /// The Nepal Sambat date of a day, read off its lunar month and its
    /// sunrise.
    ///
    /// ```no_run
    /// # fn day() -> teistro_panchanga::almanac::Panchanga { unimplemented!() }
    /// use teistro_panchanga::nepal_sambat::NepalSambatDate;
    ///
    /// let date = NepalSambatDate::of(&day());
    /// println!("NS {} month {} ({:?})", date.year, date.month, date.paksha);
    /// ```
    #[must_use]
    pub fn of(day: &Panchanga) -> NepalSambatDate {
        NepalSambatDate {
            year: year_of(day.month.amanta, day.day.sunrise),
            month: month_of(day.month.amanta),
            kind: day.month.kind,
            paksha: day.month.paksha,
        }
    }

    /// Dates as JSON, every catalogue member written as its full key
    /// where [`NepalSambatDate::MEMBERS`] says: what a boundary section
    /// carries and seals.
    ///
    /// # Errors
    ///
    /// `INTERNAL` if the dates do not serialise, which a value this
    /// crate built cannot do.
    pub fn in_full(dates: &[NepalSambatDate]) -> Result<serde_json::Value, Error> {
        let mut value = serde_json::to_value(dates).map_err(|error| {
            Error::internal(format!("the Nepal Sambat dates did not serialise: {error}"))
        })?;
        for (path, kind) in NepalSambatDate::MEMBERS {
            write_in_full(&mut value, path, kind);
        }
        Ok(value)
    }
}

/// The Nepal Sambat year of a day in an amanta month, by its sunrise:
/// the civil year its [`middle_of`] falls in, less 880.
#[must_use]
pub fn year_of(amanta: Masa, sunrise: JulianDay<Utc>) -> i32 {
    year_from_fixed(middle_of(amanta, sunrise)) + YEAR_AT_ZERO
}

/// An estimate of the middle of the Nepal Sambat year a day in an amanta
/// month falls in: its sunrise moved by the months between.
///
/// An adhika month moves the estimate by a month at most, and the middle
/// stands about four months from either end of its civil year: that is
/// the margin the count has for the sidereal months' drift through the
/// civil year, a day in seventy years. The measured page holds the count
/// to the lunar years the almanac itself finds, and reports how near a
/// civil new year the estimate came.
#[must_use]
pub fn middle_of(amanta: Masa, sunrise: JulianDay<Utc>) -> FixedDay {
    let (day, _) = FixedDay::from_jd(sunrise);
    let to_middle = (5.5 - f64::from(month_of(amanta) - 1)) * SYNODIC_DAYS;
    #[allow(
        clippy::cast_possible_truncation,
        reason = "at most six months of days"
    )]
    day.plus_days(to_middle.round() as i64)
}

/// The Nepal Sambat month an amanta month is, 1 for Kachhala.
#[must_use]
#[allow(clippy::cast_possible_truncation, reason = "at most 12")]
pub const fn month_of(amanta: Masa) -> u8 {
    ((amanta.id() + 12 - FIRST.id()) % 12 + 1) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The committee's header names each amanta month: Kachhala is
    /// Kartika, Pwanhela Pausha, Chaula Chaitra, Kaula Ashwina.
    #[test]
    fn the_months_count_from_kartika() {
        assert_eq!(month_of(Masa::Kartika), 1);
        assert_eq!(month_of(Masa::Pausha), 3);
        assert_eq!(month_of(Masa::Phalguna), 5);
        assert_eq!(month_of(Masa::Chaitra), 6);
        assert_eq!(month_of(Masa::Ashwina), 12);
    }
}
