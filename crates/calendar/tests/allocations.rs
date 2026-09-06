//! What a calendar conversion allocates: nothing
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! Every shipped calendar is integer arithmetic over a fixed day, and the
//! Bikram Sambat engine reads a static table. A conversion that allocated
//! would mean one of them had grown a `Vec` or a `String` on the path a
//! chart takes for every date it shows.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro_calendar::fixed::FixedDay;
use teistro_calendar::shipped;
use teistro_core::catalogue::Calendar;
use teistro_test_allocator::{Counting, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

#[test]
fn a_conversion_allocates_nothing_in_any_shipped_calendar() {
    for calendar in [
        Calendar::Gregorian,
        Calendar::Julian,
        Calendar::Mixed,
        Calendar::IsoWeek,
        Calendar::BikramSambat,
    ] {
        let system = shipped(calendar).expect("a shipped calendar");
        // 14 April 2015, which is 1 Baisakh 2072 BS.
        let fixed = FixedDay::new(735_702);
        let (date, counts) = measure(|| system.date_of(fixed).unwrap());
        assert_eq!(
            counts.allocations,
            0,
            "{}: reading a date allocated {} times",
            calendar.key(),
            counts.allocations
        );
        let (back, counts) = measure(|| system.fixed_of(&date).unwrap());
        assert_eq!(back, fixed, "{}: the day round-trips", calendar.key());
        assert_eq!(
            counts.allocations,
            0,
            "{}: writing a date allocated {} times",
            calendar.key(),
            counts.allocations
        );
    }
}

#[test]
fn a_month_of_dates_allocates_nothing() {
    let system = shipped(Calendar::BikramSambat).expect("a shipped calendar");
    let (days, counts) = measure(|| {
        (735_702..735_732)
            .filter_map(|day| system.date_of(FixedDay::new(day)).ok())
            .count()
    });
    assert_eq!(days, 30);
    assert_eq!(
        counts.allocations, 0,
        "thirty conversions allocated {} times",
        counts.allocations
    );
}
