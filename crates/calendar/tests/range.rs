//! A day outside a calendar's range is refused, never converted.
//!
//! Found by the fuzzer (`docs/05-testing/03-fuzzing.md`): a progression
//! asked at a Julian day of 1.2e30 reached `gregorian_from_fixed`, whose
//! year arithmetic overflowed before `date_of` checked the year it made.
//! Every shipped calendar now refuses a day outside its range first.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro_calendar::fixed::FixedDay;
use teistro_calendar::shipped;
use teistro_core::Status;
use teistro_core::catalogue::Calendar;

#[test]
fn every_shipped_calendar_refuses_a_day_beyond_its_range() {
    for calendar in [
        Calendar::Gregorian,
        Calendar::Julian,
        Calendar::Mixed,
        Calendar::IsoWeek,
        Calendar::BikramSambat,
    ] {
        let system = shipped(calendar).expect("a shipped calendar");
        let (first, last) = system.capabilities().range;
        for fixed in [first, last] {
            assert!(
                system.date_of(fixed).is_ok(),
                "{}: day {} is in range",
                calendar.key(),
                fixed.get()
            );
        }
        for day in [
            i64::MIN,
            -1_000_000_000_000,
            first.get() - 1,
            last.get() + 1,
            1_000_000_000_000,
            i64::MAX,
        ] {
            let refused = system
                .date_of(FixedDay::new(day))
                .expect_err("a day out of range");
            assert_eq!(
                refused.status,
                Status::OutOfRange,
                "{}: day {day}",
                calendar.key()
            );
        }
    }
}
