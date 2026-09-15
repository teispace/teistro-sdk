//! The balance at birth: how much of the first lord's period is still to
//! run, and how it is written (`03-design/dasha-measured.md`).
//!
//! Two methods, both measured exact over the corpus, and both reading the
//! lord's window of nakshatras the same way: the whole nakshatras of the
//! window behind the seed are gone, and the Moon's own nakshatra is gone by
//! how far into it the Moon is (`03-design/dasha-systems-measured.md`).
//!
//! - **spatial**: how far by longitude;
//! - **temporal**: how far by time, from the Moon entering the nakshatra to
//!   birth over the whole of its stay.
//!
//! The balance is that fraction of the first lord's years, and it is
//! written as whole years of the year length, whole months of a twelfth of
//! it, whole days, and the rest rounded to the minute.

use serde::{Deserialize, Serialize};
use teistro_core::angle::Nas;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_core::settings::Balance;

use crate::row::{Seat, UduRow};

/// The minutes in a day.
const MINUTES_PER_DAY: f64 = 1440.0;

/// What remains of the first period, and how long it runs.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct BalanceAtBirth {
    /// How it was measured.
    pub method: Balance,
    /// The fraction of the first lord's period still to run, 0 to 1.
    pub remaining: f64,
    /// That fraction of the first lord's years, in days.
    pub days: f64,
    /// The same, written as a reader writes it.
    pub written: Written,
}

/// A balance written as years, months, days, hours and minutes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Written {
    /// Whole years of the year length.
    pub years: u32,
    /// Whole months of a twelfth of it.
    pub months: u8,
    /// Whole days.
    pub days: u8,
    /// Hours of the rest, rounded to the minute.
    pub hours: u8,
    /// Minutes of the rest, rounded.
    pub minutes: u8,
}

impl Written {
    /// A length in days, written against a year of `year_days`.
    ///
    /// Only the minutes are rounded, and a rest that rounds to a whole day
    /// carries into the days, which can never reach a month: a remainder
    /// below a twelfth of a year of whole days is at most 30 and a fraction.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "each part is floored or rounded first, and bounded by the part above it"
    )]
    pub fn of(days: f64, year_days: f64) -> Written {
        let month = year_days / 12.0;
        let years = (days / year_days).floor().max(0.0);
        let mut rest = days - years * year_days;
        let months = (rest / month).floor().max(0.0);
        rest -= months * month;
        let mut whole_days = rest.floor().max(0.0);
        let mut minutes = ((rest - whole_days) * MINUTES_PER_DAY).round();
        if minutes >= MINUTES_PER_DAY {
            whole_days += 1.0;
            minutes -= MINUTES_PER_DAY;
        }
        let minutes = minutes as u32;
        Written {
            years: years as u32,
            months: months as u8,
            days: whole_days as u8,
            hours: (minutes / 60) as u8,
            minutes: (minutes % 60) as u8,
        }
    }
}

/// What remains of a lord's window when the Moon is `elapsed` (0 to 1) of
/// the way through its own nakshatra.
///
/// The window is the lord's `span` of nakshatras, so for a lord of three
/// the fraction is of the three and not of the one the Moon is in, which is
/// the correction `dasha-kernels.md` records for Ashtottari.
fn window(row: &UduRow, seat: Seat, elapsed: f64) -> f64 {
    let span = f64::from(row.span.max(1));
    ((span - f64::from(seat.within) - elapsed) / span).clamp(0.0, 1.0)
}

/// What remains of the Moon's window of nakshatras, spatially.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "both are below a nakshatra's nanoarcseconds, 4.8e13, far inside the 2^53 a double holds exactly"
)]
pub fn spatial(row: &UduRow, moon: Nas, seat: Seat) -> f64 {
    window(
        row,
        seat,
        moon.in_nakshatra().get() as f64 / Nas::PER_NAKSHATRA as f64,
    )
}

/// What remains of the Moon's window of nakshatras, temporally: the time
/// from the Moon entering its nakshatra to birth over the whole of its
/// stay, taken as that nakshatra's part of the window. For a lord of one
/// nakshatra that is the time from birth to the Moon leaving it.
///
/// The other reading of a window, the Moon's time across all of it, is not
/// recorded anywhere this crate is measured against, and is not built.
///
/// # Errors
///
/// A span that does not hold the birth, or that is empty, named as
/// `moon_span`.
pub fn temporal(
    row: &UduRow,
    seat: Seat,
    birth: JulianDay<Utc>,
    moon_span: Interval,
) -> Result<f64, Error> {
    if moon_span.is_empty() || !moon_span.contains_inclusive(birth) {
        return Err(Error::invalid_arg(format!(
            "the Moon's nakshatra span {} to {} does not hold the birth at {birth}",
            moon_span.from, moon_span.to
        ))
        .with_field("moon_span"));
    }
    Ok(window(
        row,
        seat,
        (birth.get() - moon_span.from.get()) / moon_span.days(),
    ))
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        reason = "tests fail by panicking and compare values exact in binary"
    )]

    use teistro_core::quantity::Degrees;

    use super::*;
    use crate::row::VIMSHOTTARI;

    #[test]
    fn a_balance_is_written_with_its_minutes_rounded() {
        // The corpus's first chart: six years, eleven months, thirteen days,
        // 11:20.
        let written = Written::of(2_539.784_984_723_759_4, 365.25);
        assert_eq!(
            written,
            Written {
                years: 6,
                months: 11,
                days: 13,
                hours: 11,
                minutes: 20
            }
        );
        // Half a minute rounds up.
        assert_eq!(Written::of(10.0 + 30.5 / 1440.0, 365.25).minutes, 31);
        // A rest that rounds to a whole day carries into the days.
        let carried = Written::of(10.0 + 1439.6 / 1440.0, 365.25);
        assert_eq!((carried.days, carried.hours, carried.minutes), (11, 0, 0));
    }

    #[test]
    fn spatially_what_remains_is_the_rest_of_the_window() {
        let seat = VIMSHOTTARI.seat(0);
        let start = Nas::from_degrees(Degrees::try_new(0.0).unwrap());
        assert_eq!(spatial(&VIMSHOTTARI, start, seat), 1.0);
        let half = Nas::from_degrees(Degrees::try_new(360.0 / 54.0).unwrap());
        assert!((spatial(&VIMSHOTTARI, half, seat) - 0.5).abs() < 1e-12);
        // A window of three, one nakshatra and a half in, has half remaining.
        let wide = UduRow {
            span: 3,
            ..VIMSHOTTARI
        };
        let seat = Seat {
            lord: 0,
            within: 1,
            overflow: false,
        };
        assert!((spatial(&wide, half, seat) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn temporally_what_remains_is_the_rest_of_the_stay() {
        let span = Interval::literal(100.0, 101.0);
        let seat = VIMSHOTTARI.seat(0);
        let remaining = temporal(&VIMSHOTTARI, seat, JulianDay::literal(100.25), span).unwrap();
        assert_eq!(remaining, 0.75);
        let outside = temporal(&VIMSHOTTARI, seat, JulianDay::literal(99.0), span).unwrap_err();
        assert_eq!(outside.field(), Some("moon_span"));
        // In a window of three, a quarter of the way through its second
        // nakshatra by time, what remains is the rest of that one and the
        // third: 1.75 of 3.
        let wide = UduRow {
            span: 3,
            ..VIMSHOTTARI
        };
        let second = Seat {
            lord: 0,
            within: 1,
            overflow: false,
        };
        let remaining = temporal(&wide, second, JulianDay::literal(100.25), span).unwrap();
        assert!((remaining - 1.75 / 3.0).abs() < 1e-15);
    }
}
