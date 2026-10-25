//! Easter Sunday by the computus: the first Sunday after the paschal full
//! moon, the ecclesiastical moon's fourteenth day on or after 21 March.
//!
//! The ecclesiastical moon is a table, not the sky: a nineteen-year cycle
//! of epacts. The Western churches reckon it in the Gregorian calendar,
//! with the reform's corrections for the Sun and the Moon (Clavius, 1582);
//! the Eastern churches keep the Alexandrian cycle in the Julian calendar,
//! which repeats every 532 years. Both put Easter between 22 March and 25
//! April of their own calendar, so the Julian reckoning's Sunday falls 4
//! April to 8 May on the Gregorian calendar between 1900 and 2099.
//!
//! The Gregorian arithmetic is the anonymous algorithm published in
//! *Nature* in 1876 and given by Butcher; the Julian is Meeus's (*Astronomical
//! Algorithms*, 2nd ed., ch. 8, both). The tests hold the Gregorian one to
//! the Clavian epact rules as Knuth derives them (*The Art of Computer
//! Programming*, vol. 1, §1.3.2, exercise 14), year by year.

use teistro_core::error::Error;

use crate::fixed::FixedDay;
use crate::gregorian::{YEAR_RANGE, fixed_from_gregorian};
use crate::julian::fixed_from_julian;

/// Which reckoning of Easter.
///
/// ```
/// use teistro_calendar::easter::{Computus, easter};
/// use teistro_calendar::gregorian::fixed_from_gregorian;
///
/// // In 2024 the two reckonings were five weeks apart.
/// assert_eq!(easter(2024, Computus::Gregorian)?, fixed_from_gregorian(2024, 3, 31));
/// assert_eq!(easter(2024, Computus::Julian)?, fixed_from_gregorian(2024, 5, 5));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Computus {
    /// The Gregorian computus, the Western churches': from 1583, the
    /// first whole year of the reform.
    Gregorian,
    /// The Julian computus, the Eastern churches': the Alexandrian cycle in
    /// the Julian calendar, from 326, the year after Nicaea.
    Julian,
}

impl Computus {
    /// The first year the reckoning answers for.
    ///
    /// ```
    /// use teistro_calendar::easter::{Computus, easter};
    ///
    /// assert_eq!(Computus::Gregorian.first_year(), 1583);
    /// assert!(easter(1582, Computus::Gregorian).is_err());
    /// assert!(easter(1582, Computus::Julian).is_ok());
    /// ```
    #[must_use]
    pub const fn first_year(self) -> i32 {
        match self {
            Computus::Gregorian => 1583,
            Computus::Julian => 326,
        }
    }

    /// The reckoning's catalogue spelling.
    ///
    /// ```
    /// use teistro_calendar::easter::Computus;
    ///
    /// assert_eq!(Computus::Julian.key(), "JULIAN");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Computus::Gregorian => "GREGORIAN",
            Computus::Julian => "JULIAN",
        }
    }
}

/// Easter Sunday of `year` (the Gregorian year it falls in, which under
/// either reckoning is the year it is reckoned for) as a fixed day.
///
/// ```
/// use teistro_calendar::easter::{Computus, easter};
/// use teistro_calendar::gregorian::fixed_from_gregorian;
///
/// assert_eq!(easter(2025, Computus::Gregorian)?, fixed_from_gregorian(2025, 4, 20));
/// assert_eq!(easter(2026, Computus::Julian)?, fixed_from_gregorian(2026, 4, 12));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` for a year before the reckoning's
/// [`Computus::first_year`] or past the calendars' range.
pub fn easter(year: i32, computus: Computus) -> Result<FixedDay, Error> {
    if year < computus.first_year() || year > YEAR_RANGE.1 {
        return Err(Error::invalid_arg(format!(
            "the {} computus answers {} to {}, not {year}",
            computus.key(),
            computus.first_year(),
            YEAR_RANGE.1
        ))
        .with_field("year"));
    }
    let (month, day) = match computus {
        Computus::Gregorian => gregorian_month_day(year),
        Computus::Julian => julian_month_day(year),
    };
    Ok(match computus {
        Computus::Gregorian => fixed_from_gregorian(year, month, day),
        Computus::Julian => fixed_from_julian(year, month, day),
    })
}

/// The month and day of Gregorian Easter: the 1876 algorithm.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    reason = "a month of 3 or 4 and a day of 1 to 31, from a year the caller checked positive; \
              the letters are Meeus's, so the arithmetic reads against the book"
)]
const fn gregorian_month_day(year: i32) -> (u8, u8) {
    let year = year as i64;
    // The golden number less one, the century and the year in it.
    let (a, b, c) = (year % 19, year / 100, year % 100);
    // The solar and lunar corrections, and the epact's complement.
    let (d, e) = (b / 4, b % 4);
    let f = (b + 8) / 25;
    let g = (b - f + 1) / 3;
    let h = (19 * a + b - d - g + 15) % 30;
    // The days from the paschal full moon to the Sunday after.
    let (i, k) = (c / 4, c % 4);
    let l = (32 + 2 * e + 2 * i - h - k) % 7;
    // The two epacts the reform moves a day.
    let m = (a + 11 * h + 22 * l) / 451;
    let p = h + l - 7 * m + 114;
    ((p / 31) as u8, (p % 31 + 1) as u8)
}

/// The month and day of Julian Easter, in the Julian calendar: Meeus.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    reason = "a month of 3 or 4 and a day of 1 to 31, from a year the caller checked positive; \
              the letters are Meeus's, so the arithmetic reads against the book"
)]
const fn julian_month_day(year: i32) -> (u8, u8) {
    let year = year as i64;
    let (a, b, c) = (year % 4, year % 7, year % 19);
    // The paschal full moon's days past 21 March, then to the Sunday.
    let d = (19 * c + 15) % 30;
    let e = (2 * a + 4 * b - d + 34) % 7;
    let p = d + e + 114;
    ((p / 31) as u8, (p % 31 + 1) as u8)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;
    use crate::fixed::Weekday;
    use crate::gregorian::gregorian_from_fixed;
    use crate::julian::julian_from_fixed;

    type MonthDay = (u8, u8);

    /// The dates Wikipedia's "List of dates for Easter" gives, read
    /// 2026-10-23: the year, the Western Easter and the Eastern, both on
    /// the Gregorian calendar.
    const LISTED: [(i32, MonthDay, MonthDay); 21] = [
        (2016, (3, 27), (5, 1)),
        (2017, (4, 16), (4, 16)),
        (2018, (4, 1), (4, 8)),
        (2019, (4, 21), (4, 28)),
        (2020, (4, 12), (4, 19)),
        (2021, (4, 4), (5, 2)),
        (2022, (4, 17), (4, 24)),
        (2023, (4, 9), (4, 16)),
        (2024, (3, 31), (5, 5)),
        (2025, (4, 20), (4, 20)),
        (2026, (4, 5), (4, 12)),
        (2027, (3, 28), (5, 2)),
        (2028, (4, 16), (4, 16)),
        (2029, (4, 1), (4, 8)),
        (2030, (4, 21), (4, 28)),
        (2031, (4, 13), (4, 13)),
        (2032, (3, 28), (5, 2)),
        (2033, (4, 17), (4, 24)),
        (2034, (4, 9), (4, 9)),
        (2035, (3, 25), (4, 29)),
        (2036, (4, 13), (4, 20)),
    ];

    #[test]
    fn the_listed_easters_of_both_churches() {
        for (year, west, east) in LISTED {
            let on = |computus| {
                let (_, month, day) = gregorian_from_fixed(easter(year, computus).unwrap());
                (month, day)
            };
            assert_eq!(on(Computus::Gregorian), west, "{year} Western");
            assert_eq!(on(Computus::Julian), east, "{year} Eastern");
        }
    }

    /// Knuth's derivation of the Gregorian date from the Clavian rules:
    /// the golden number, the century's solar and lunar equations, the
    /// epact with its two exceptions, the full moon and the Sunday after.
    fn clavian(year: i64) -> (u8, u8) {
        let golden = year % 19 + 1;
        let century = year / 100 + 1;
        let solar = 3 * century / 4 - 12;
        let lunar = (8 * century + 5) / 25 - 5;
        let sunday = 5 * year / 4 - solar - 10;
        let mut epact = (11 * golden + 20 + lunar - solar).rem_euclid(30);
        if (epact == 25 && golden > 11) || epact == 24 {
            epact += 1;
        }
        let mut moon = 44 - epact;
        if moon < 21 {
            moon += 30;
        }
        let day = moon + 7 - (sunday + moon) % 7;
        if day > 31 {
            (4, u8::try_from(day - 31).unwrap())
        } else {
            (3, u8::try_from(day).unwrap())
        }
    }

    #[test]
    fn every_gregorian_easter_is_the_clavian_one() {
        for year in 1583..=YEAR_RANGE.1 {
            assert_eq!(
                gregorian_month_day(year),
                clavian(i64::from(year)),
                "{year}"
            );
        }
    }

    #[test]
    fn every_easter_is_a_sunday_from_22_march_to_25_april_of_its_calendar() {
        let window = |month: u8, day: u8| (month == 3 && day >= 22) || (month == 4 && day <= 25);
        for year in 1583..=YEAR_RANGE.1 {
            let west = easter(year, Computus::Gregorian).unwrap();
            let (_, month, day) = gregorian_from_fixed(west);
            assert!(
                window(month, day) && west.weekday() == Weekday::Sunday,
                "{year}"
            );
        }
        for year in 326..=YEAR_RANGE.1 {
            let east = easter(year, Computus::Julian).unwrap();
            let (on, month, day) = julian_from_fixed(east);
            assert!(
                window(month, day) && east.weekday() == Weekday::Sunday,
                "{year}"
            );
            assert_eq!(on, year);
        }
    }

    #[test]
    fn the_extremes_and_the_julian_cycle() {
        let on = |year| gregorian_from_fixed(easter(year, Computus::Gregorian).unwrap());
        assert_eq!(on(1818), (1818, 3, 22));
        assert_eq!(on(2285), (2285, 3, 22));
        assert_eq!(on(1943), (1943, 4, 25));
        assert_eq!(on(2038), (2038, 4, 25));
        // The Alexandrian cycle repeats its dates every 532 Julian years.
        for year in 326..1000 {
            assert_eq!(
                julian_month_day(year),
                julian_month_day(year + 532),
                "{year}"
            );
        }
    }

    #[test]
    fn a_year_outside_the_reckoning_is_refused_by_its_field() {
        for (year, computus) in [
            (1582, Computus::Gregorian),
            (325, Computus::Julian),
            (YEAR_RANGE.1 + 1, Computus::Gregorian),
        ] {
            let refused = easter(year, computus).unwrap_err();
            assert_eq!(refused.field(), Some("year"));
        }
        assert_eq!(Computus::Julian.key(), "JULIAN");
    }
}
