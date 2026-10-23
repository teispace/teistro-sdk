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
    for system in Calendar::ALL.into_iter().filter_map(shipped) {
        let calendar = system.id();
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

/// Every shipped calendar reads back the day it names, over its whole
/// range: a day's date converts to that day, its month is at least that
/// long, and a year beyond the range is refused by name.
#[test]
fn every_shipped_calendar_reads_back_the_days_it_names() {
    const SAMPLES: i64 = 400;
    for (calendar, system) in Calendar::ALL
        .into_iter()
        .filter_map(|calendar| shipped(calendar).map(|system| (calendar, system)))
    {
        assert_eq!(system.id(), calendar);
        let (first, last) = system.capabilities().range;
        let step = ((last.get() - first.get()) / SAMPLES).max(1);
        let days = (0..=SAMPLES)
            .map(|k| (first.get() + k * step).min(last.get()))
            .chain([last.get()]);
        for day in days {
            let date = system
                .date_of(FixedDay::new(day))
                .unwrap_or_else(|e| panic!("{}: day {day}: {e}", calendar.key()));
            assert_eq!(
                system.fixed_of(&date).map(FixedDay::get),
                Ok(day),
                "{}: {date:?}",
                calendar.key()
            );
            if date.leap.is_none() {
                let length = system
                    .month_length(date.year, date.month)
                    .unwrap_or_else(|e| panic!("{}: {date:?}: {e}", calendar.key()));
                // A Tibetan month numbers its days 1 to 30 whatever it
                // holds, a skipped number missing and a repeated one twice.
                let numbered = if calendar == Calendar::TibetanPhugpa {
                    30
                } else {
                    length
                };
                assert!(
                    date.day <= numbered,
                    "{}: {date:?} in {length}",
                    calendar.key()
                );
            }
            let _ = system.is_leap(date.year);
        }
        let (start, end) = (
            system.date_of(first).expect("in range").year,
            system.date_of(last).expect("in range").year,
        );
        for year in [start - 1, end + 1] {
            let refused = system
                .month_length(year, 1)
                .expect_err("a year beyond the range");
            assert_eq!(
                refused.status,
                Status::OutOfRange,
                "{}: {year}",
                calendar.key()
            );
        }
    }
}
