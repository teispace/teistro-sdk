//! The memo under sustained load (`05-testing/01-quality-bar.md`, "cache
//! bounds"): two years of almanac, month after month, through one
//! context, measured by the counting allocator. The memo's budget is a
//! count of cells (`provider.cache_cells`). Once it is full a month keeps
//! nothing more, the memory it holds is what that count of cells costs,
//! and without the memo no month keeps anything at all, so no other cache
//! grows behind it.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset};
use teistro_test_allocator::{Counting, Counts, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// A memo small enough to fill in the first months.
const CELLS: u32 = 4_096;

/// What one remembered cell may cost in bytes, its key and the map's own
/// node included: measured at 269.
const BYTES_PER_CELL: i128 = 320;

/// Twenty-eight days of each month of 2026 and 2027, each measured.
fn months(sdk: &Context) -> Vec<Counts> {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let clock = UtcOffset::literal(5, 45, 0);
    (0..24u8)
        .map(|index| {
            let (year, month) = (2026 + i32::from(index / 12), index % 12 + 1);
            let from = CalendarDate::defined(Calendar::Gregorian, year, month, 1);
            let to = CalendarDate::defined(Calendar::Gregorian, year, month, 28);
            measure(|| drop(sdk.almanac().of(&from, &to, &place, clock).unwrap())).1
        })
        .collect()
}

fn context(cells: u32) -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(format!(r#"{{"provider":{{"cache_cells":{cells}}}}}"#))
        .build()
        .unwrap()
}

#[test]
fn a_full_memo_keeps_nothing_more_and_holds_what_its_cells_cost() {
    let remembered = months(&context(CELLS));
    let held: i128 = remembered.iter().map(Counts::retained).sum();
    let filled = remembered
        .iter()
        .position(|month| month.retained() == 0)
        .unwrap();
    println!(
        "the memo of {CELLS} cells holds {held} bytes, {} a cell, full after {filled} months",
        held / i128::from(CELLS)
    );
    assert!(
        held > 0 && held <= i128::from(CELLS) * BYTES_PER_CELL,
        "a memo of {CELLS} cells holds {held} bytes: its budget is \
         {BYTES_PER_CELL} bytes a cell"
    );
    assert!(filled <= 3, "the memo filled after {filled} months");
    for (index, month) in remembered.iter().enumerate().skip(filled) {
        assert_eq!(
            month.retained(),
            0,
            "month {index} keeps {} bytes after the memo is full",
            month.retained()
        );
    }
    let peaks = remembered.iter().skip(filled).map(|month| month.peak);
    let (least, most) = peaks.fold((u64::MAX, 0), |(least, most), peak| {
        (least.min(peak), most.max(peak))
    });
    assert!(
        most <= least + least / 10,
        "a full memo's months peak from {least} to {most} bytes: \
         the same work should hold the same memory"
    );

    let forgotten = months(&context(0));
    for (index, month) in forgotten.iter().enumerate() {
        assert_eq!(
            month.retained(),
            0,
            "month {index} keeps {} bytes with the memo off: a cache \
             grows behind it",
            month.retained()
        );
    }
}
