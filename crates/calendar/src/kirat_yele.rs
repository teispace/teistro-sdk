//! The Kirat Yele Sambat: the Kirat (Rai, Limbu, Yakkha, Sunuwar) era,
//! a solar count whose year turns on Maghe Sankranti, Magh 1 of the
//! Bikram Sambat (`docs/calendars/bikram-sambat.md` §Kirat Yele Sambat,
//! SR-96).
//!
//! Like Nepal Sambat and the Vira Nirvana Samvat, the era is a reading of
//! a calendar the SDK already keeps rather than a calendar of its own:
//! its days and months are the Bikram Sambat's, and only the year number
//! differs, turning on Magh 1 rather than Baisakh 1. The sources agree on
//! that shape and count the year from two epochs, and each is an
//! [`Epoch`] rather than a guess:
//!
//! - **[`Epoch::Kirati`]**, the count Kirat calendars and the press print
//!   today: Kirati Sambat 5086 began on Magh 1, 2082 BS (15 January 2026;
//!   *Kantipur*, Koshi Pradesh, 2082-10-01), and 5076 from 16 January 2016
//!   (G. Rai, "Kirant identity and Yele Sambat", *Indigenous Voice*,
//!   5 December 2021). Yele = BS + 3004 from Magh 1. The default.
//! - **[`Epoch::Yalambar`]**, the count from Yalambar's accession that
//!   J. C. Regmi derives from the Kirat dynasty's 1964 years before the
//!   Shaka era's 107th year (*Kirat Yele Samvat Abhilekh*, Kirat Yele
//!   Samvat Adhyayan Samanvaya Samiti, 2010): 3790 in 2011 and 3795 from
//!   15 January 2016 (*Indigenous Voice*, as above). Yele = BS + 1723
//!   from Magh 1.
//!
//! The Government of Nepal stated on 15 January 2010 that it would
//! recognise the era; no notice fixes either count.

use teistro_core::catalogue::{Calendar, Era};
use teistro_core::error::Error;

use crate::date::{CalendarDate, EraNumber};

/// Magh, the Bikram Sambat month the Yele year opens on.
pub const MAGH: u8 = 10;

/// Where the era's year 1 stands: the settings knob
/// `calendars.kirat_yele`, declared beside the other knobs in
/// `teistro_core` (which this crate reads, and not the reverse) and named
/// here for what it is. [`Epoch::Kirati`] is the default.
pub use teistro_core::settings::KiratYeleEpoch as Epoch;

/// The Yele year of a Bikram Sambat year and month: the next number from
/// Magh, the Bikram Sambat year's tenth month.
///
/// ```
/// use teistro_calendar::kirat_yele::{Epoch, year_of};
///
/// // 5086 opened on Magh 1, 2082 BS; Poush 2082 still closed 5085.
/// assert_eq!(year_of(2082, 10, Epoch::Kirati), 5086);
/// assert_eq!(year_of(2082, 9, Epoch::Kirati), 5085);
/// // Baisakh 2083 is still in 5086.
/// assert_eq!(year_of(2083, 1, Epoch::Kirati), 5086);
/// assert_eq!(year_of(2072, 10, Epoch::Yalambar), 3795);
/// ```
#[must_use]
pub const fn year_of(vikrama: i32, month: u8, epoch: Epoch) -> i32 {
    let opened = if month >= MAGH { 0 } else { -1 };
    vikrama + epoch.years_after_vikrama() + opened
}

/// The Bikram Sambat year and month a Yele year and Bikram Sambat month
/// fall in: [`year_of`] read back.
///
/// ```
/// use teistro_calendar::kirat_yele::{Epoch, vikrama_of, year_of};
///
/// assert_eq!(vikrama_of(5086, 10, Epoch::Kirati), 2082);
/// assert_eq!(vikrama_of(5086, 1, Epoch::Kirati), 2083);
/// for month in 1..=12 {
///     assert_eq!(year_of(vikrama_of(5086, month, Epoch::Kirati), month, Epoch::Kirati), 5086);
/// }
/// ```
#[must_use]
pub const fn vikrama_of(yele: i32, month: u8, epoch: Epoch) -> i32 {
    let opened = if month >= MAGH { 0 } else { -1 };
    yele - epoch.years_after_vikrama() - opened
}

/// A Bikram Sambat date with its Yele era number in place of the Vikrama
/// one: the date's year, month and day stay the Bikram Sambat's.
///
/// ```
/// use teistro_calendar::date::EraNumber;
/// use teistro_calendar::kirat_yele::{Epoch, view};
/// use teistro_calendar::{BikramSambat, CalendarDate, CalendarSystem, Gregorian};
/// use teistro_core::catalogue::{Calendar, Era};
///
/// // 15 January 2026 was Magh 1, 2082 BS, the first day of Yele 5086.
/// let fixed = Gregorian.fixed_of(&CalendarDate::defined(Calendar::Gregorian, 2026, 1, 15))?;
/// let bs = BikramSambat::shipped().date_of(fixed)?;
/// assert_eq!((bs.year, bs.month, bs.day), (2082, 10, 1));
/// let yele = view(&bs, Epoch::Kirati)?;
/// assert_eq!(yele.era, Some(EraNumber { era: Era::KiratYele, year: 5086 }));
/// assert_eq!((yele.year, yele.month, yele.day), (2082, 10, 1));
/// // The day before closed Yele 5085.
/// let eve = BikramSambat::shipped().date_of(fixed.plus_days(-1))?;
/// assert_eq!(view(&eve, Epoch::Kirati)?.era.map(|era| era.year), Some(5085));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a date in another calendar, or a month outside 1 to
/// 12.
pub fn view(date: &CalendarDate, epoch: Epoch) -> Result<CalendarDate, Error> {
    if date.calendar != Calendar::BikramSambat {
        return Err(Error::invalid_arg(format!(
            "the Kirat Yele Sambat reads a Bikram Sambat date, not a {} one",
            date.calendar.key()
        ))
        .with_field("calendar"));
    }
    if !(1..=12).contains(&date.month) {
        return Err(Error::invalid_arg(format!(
            "Bikram Sambat has months 1 to 12, not {}",
            date.month
        ))
        .with_field("month"));
    }
    let mut viewed = date.clone();
    viewed.era = Some(EraNumber {
        era: Era::KiratYele,
        year: year_of(date.year, date.month, epoch),
    });
    Ok(viewed)
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::{BikramSambat, CalendarSystem, Gregorian};

    fn bs(year: i32, month: u8, day: u8) -> CalendarDate {
        CalendarDate::defined(Calendar::BikramSambat, year, month, day)
    }

    #[test]
    fn every_magh_first_opens_the_next_year_on_both_epochs() {
        let calendar = BikramSambat::shipped();
        let (first, last) = calendar.years();
        for year in first..=last {
            let Ok(magh) = calendar.fixed_of(&bs(year, MAGH, 1)) else {
                continue;
            };
            let eve = calendar.date_of(magh.plus_days(-1)).unwrap();
            for epoch in [Epoch::Kirati, Epoch::Yalambar] {
                let opened = view(&bs(year, MAGH, 1), epoch).unwrap().era.unwrap().year;
                let closed = view(&eve, epoch).unwrap().era.unwrap().year;
                assert_eq!(opened, closed + 1, "{year} BS {epoch:?}");
                assert_eq!(vikrama_of(opened, MAGH, epoch), year);
            }
        }
    }

    #[test]
    fn the_sources_dated_years_hold() {
        // Kantipur 2026: 5086 from Magh 1, 2082 = 15 January 2026.
        let magh = BikramSambat::shipped().fixed_of(&bs(2082, 10, 1)).unwrap();
        let gregorian = Gregorian.date_of(magh).unwrap();
        assert_eq!(
            (gregorian.year, gregorian.month, gregorian.day),
            (2026, 1, 15)
        );
        // Indigenous Voice 2021: 5076 and 3795 from January 2016; 3790 in 2011.
        assert_eq!(year_of(2072, 10, Epoch::Kirati), 5076);
        assert_eq!(year_of(2072, 10, Epoch::Yalambar), 3795);
        assert_eq!(year_of(2067, 10, Epoch::Yalambar), 3790);
    }

    #[test]
    fn another_calendar_or_month_is_refused_by_its_field() {
        let gregorian = CalendarDate::defined(Calendar::Gregorian, 2026, 1, 15);
        assert_eq!(
            view(&gregorian, Epoch::Kirati).unwrap_err().field(),
            Some("calendar")
        );
        assert_eq!(
            view(&bs(2082, 13, 1), Epoch::Kirati).unwrap_err().field(),
            Some("month")
        );
    }
}
