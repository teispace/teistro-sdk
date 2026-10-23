//! What a panchanga day allocates, counted rather than guessed
//! (`05-testing/01-quality-bar.md`, "allocation counts").
//!
//! A day is some 250 provider completions, for sunrise, the limbs'
//! crossings and the lunation's new moons and sankrantis, and each
//! completion allocates its 13 (`crates/astro/tests/allocations.rs`): the
//! eight columns, the steps and the request. So a day's count is the
//! number of times it asks the sky, and what this test holds is its
//! shape: a day keeps nothing once dropped, and a range shares what its
//! days have in common, the lunation above all, so a later day costs a
//! fraction of the first: 318 allocations against 3,340, and 807 with the
//! lunation's memo taken out.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, Gregorian};
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::quantity::{Altitude, Latitude, Longitude, Place};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_panchanga::Almanac;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_test_allocator::{Counting, Counts, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

fn june(day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2024, 6, day)
}

/// A call's allocations, made once first so that what is counted is the
/// call's own and not a lazily built table's, and dropped inside the
/// measurement so that what it keeps is what it leaves behind.
fn counted<T>(call: impl Fn() -> T) -> Counts {
    drop(call());
    measure(|| drop(call())).1
}

/// What each day of a range costs past its first.
fn per_later_day(one: &Counts, days: u64, range: &Counts) -> u64 {
    (range.allocations - one.allocations) / (days - 1)
}

#[test]
fn a_day_keeps_nothing_and_a_range_shares_its_lunation() {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );

    let day = counted(|| almanac.day(&june(21), &place).unwrap());
    let one = counted(|| almanac.between(&june(1), &june(1), &place).unwrap());
    let month = counted(|| almanac.between(&june(1), &june(28), &place).unwrap());
    let festival_one = counted(|| almanac.festival_days(&june(1), &june(1), &place).unwrap());
    let festival_month = counted(|| almanac.festival_days(&june(1), &june(28), &place).unwrap());
    let later = per_later_day(&one, 28, &month);
    let festival_later = per_later_day(&festival_one, 28, &festival_month);
    println!(
        "a day: {} allocations, {} bytes; 28 days: {} ({later} a later day); \
         28 festival days: {} ({festival_later} a later day)",
        day.allocations, day.bytes, month.allocations, festival_month.allocations
    );

    for (what, counts) in [
        ("a day", &day),
        ("28 days", &month),
        ("28 festival days", &festival_month),
    ] {
        assert_eq!(
            counts.retained(),
            0,
            "{what} keeps {} bytes once dropped: an almanac caches nothing between calls",
            counts.retained()
        );
    }
    assert!(
        day.allocations <= 3_500,
        "a day allocates {} times, measured at 2,837: some 250 completions of 13 \
         each; far above that is a search asking the sky more often than it did",
        day.allocations
    );
    assert!(
        later * 8 <= one.allocations,
        "a later day of a range allocates {later} times against the first day's {}, \
         measured at 318 against 3,340: the lunation, found once, is shared; without \
         its memo a later day costs 807",
        one.allocations
    );
    assert!(
        festival_later * 2 <= later && festival_month.allocations < month.allocations,
        "a festival day allocates {festival_later} times past the first against a \
         panchanga day's {later}, measured at 133 against 318: it does not compute \
         the parts festival rules do not read, the Moon's rises and sets least of all"
    );
}
