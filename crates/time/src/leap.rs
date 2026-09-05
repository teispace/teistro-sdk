//! The leap-second table: TAI less UTC from 1972, which days had a
//! sixty-first second, and when the table's word expires.

use teistro_calendar::FixedDay;
use teistro_core::quantity::{JulianDay, Utc};

use crate::generated::{LEAP_EXPIRES_MJD, LEAP_ROWS, LEAP_SOURCE, LEAP_UPDATED};

/// The Julian day of the Modified Julian Date's zero.
const MJD_EPOCH: f64 = 2_400_000.5;

/// The table's source.
#[must_use]
pub const fn source() -> &'static str {
    LEAP_SOURCE
}

/// The table's version: the date the IANA list was last updated.
#[must_use]
pub const fn version() -> &'static str {
    LEAP_UPDATED
}

/// How many rows the table has (the first is the 1972 offset, the rest
/// leap seconds).
#[must_use]
pub const fn rows() -> usize {
    LEAP_ROWS.len()
}

/// TAI less UTC in whole seconds at an instant; `None` before 1972,
/// when UTC had no whole-second relation to TAI.
#[must_use]
pub fn tai_minus_utc(instant: JulianDay<Utc>) -> Option<u8> {
    let mjd = instant.get() - MJD_EPOCH;
    let after = LEAP_ROWS.partition_point(|row| f64::from(row.0) <= mjd);
    LEAP_ROWS.get(after.checked_sub(1)?).map(|row| row.1)
}

/// The day a leap-second row takes effect, as a fixed day.
fn effective_day(row_mjd: i32) -> FixedDay {
    FixedDay::from_local_jd(f64::from(row_mjd) + MJD_EPOCH).0
}

/// Whether a UTC day ended with a leap second (23:59:60).
#[must_use]
pub fn is_leap_second_day(day: FixedDay) -> bool {
    LEAP_ROWS
        .iter()
        .skip(1)
        .any(|row| effective_day(row.0) == day.plus_days(1))
}

/// The day the table's word expires: an instant after it may be missing
/// a leap second announced since the list was fetched.
#[must_use]
pub fn expires() -> FixedDay {
    effective_day(LEAP_EXPIRES_MJD)
}

/// Whether an instant lies beyond the table's word.
#[must_use]
pub fn is_expired_at(instant: JulianDay<Utc>) -> bool {
    FixedDay::from_jd(instant).0 >= expires()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

    use teistro_calendar::gregorian::{fixed_from_gregorian, gregorian_from_fixed};

    use super::*;

    fn at(year: i32, month: u8, day: u8, fraction: f64) -> JulianDay<Utc> {
        fixed_from_gregorian(year, month, day)
            .jd_at_midnight()
            .unwrap()
            .plus_days(fraction)
            .unwrap()
    }

    #[test]
    fn the_table_is_the_iana_list() {
        assert_eq!(rows(), 28);
        assert!(source().contains("IANA"));
        assert!(version().starts_with("20"));
        assert_eq!(tai_minus_utc(at(1971, 12, 31, 0.5)), None);
        assert_eq!(tai_minus_utc(at(1972, 1, 1, 0.0)), Some(10));
        assert_eq!(tai_minus_utc(at(1972, 6, 30, 0.999)), Some(10));
        assert_eq!(tai_minus_utc(at(1972, 7, 1, 0.0)), Some(11));
        assert_eq!(tai_minus_utc(at(2000, 1, 1, 0.5)), Some(32));
        assert_eq!(tai_minus_utc(at(2016, 12, 31, 0.9)), Some(36));
        assert_eq!(tai_minus_utc(at(2017, 1, 1, 0.0)), Some(37));
        assert_eq!(tai_minus_utc(at(2026, 9, 5, 0.0)), Some(37));
    }

    #[test]
    fn leap_second_days_and_the_expiry() {
        assert!(is_leap_second_day(fixed_from_gregorian(2016, 12, 31)));
        assert!(is_leap_second_day(fixed_from_gregorian(1972, 6, 30)));
        assert!(is_leap_second_day(fixed_from_gregorian(2015, 6, 30)));
        assert!(!is_leap_second_day(fixed_from_gregorian(2016, 12, 30)));
        assert!(!is_leap_second_day(fixed_from_gregorian(1971, 12, 31)));
        assert!(!is_leap_second_day(fixed_from_gregorian(2017, 12, 31)));
        let expiry = gregorian_from_fixed(expires());
        assert!(expiry.0 >= 2026, "{expiry:?}");
        assert!(!is_expired_at(at(2026, 9, 5, 0.0)));
        assert!(is_expired_at(at(2100, 1, 1, 0.0)));
    }
}
