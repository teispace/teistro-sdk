//! The most memory each operation holds at once, measured by the counting
//! allocator rather than guessed (`05-testing/01-quality-bar.md`, "peak
//! memory per operation"): a context, a chart's foundation, a full chart,
//! a month of days and a month's muhurta search.
//!
//! A peak is the shape of the code, as a count is. The full chart's was
//! 36 MB while its document was hashed through a `serde_json::Value` tree,
//! ten times the document's own text; written straight from serde it is a
//! few times the text, and the test holds it there.

#![allow(
    clippy::unwrap_used,
    clippy::print_stdout,
    reason = "tests fail by panicking, and report what they measured"
)]

use teistro::catalogue::{Calendar, ChartKind};
use teistro::muhurta::ActivityRules;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    CalendarDate, ChartRequest, Context, Ephemeris, MuhurtaRequest, UtcOffset, canonical_json,
};
use teistro_test_allocator::{Counting, Counts, measure};

#[global_allocator]
static ALLOCATOR: Counting = Counting::system();

/// A call measured after it has run once, so a table built on first use
/// is not counted, and dropped inside the measurement, so what it keeps
/// is what it leaves behind.
fn counted<T>(call: impl Fn() -> T) -> Counts {
    drop(call());
    measure(|| drop(call())).1
}

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

#[test]
fn each_operation_holds_no_more_than_its_documented_peak() {
    let build = || {
        Context::builder()
            .ephemeris([Ephemeris::Builtin])
            .build()
            .unwrap()
    };
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let birth = JulianDay::<Utc>::literal(2_460_483.2);
    let sdk = build();
    let full = ChartRequest::at(place, clock).with_everything();
    let marriage = MuhurtaRequest::new(ActivityRules::raman_marriage()).with_windows_on(2);

    let measured = [
        ("a context", counted(build), 40_000),
        (
            "a foundation",
            counted(|| {
                sdk.chart()
                    .found(birth, &place, clock, ChartKind::Natal)
                    .unwrap()
            }),
            40_000,
        ),
        (
            "a full chart",
            counted(|| sdk.chart().reading(birth, &full).unwrap()),
            13_000_000,
        ),
        (
            "thirty days",
            counted(|| {
                sdk.almanac()
                    .of(&date(6, 1), &date(6, 30), &place, clock)
                    .unwrap()
            }),
            1_200_000,
        ),
        (
            "a month's muhurta search",
            counted(|| {
                sdk.almanac()
                    .muhurta(&date(11, 1), &date(11, 30), &place, clock, &marriage)
                    .unwrap()
            }),
            900_000,
        ),
    ];
    for (what, counts, budget) in &measured {
        println!(
            "{what}: peak {} bytes, {} asked for in {} allocations",
            counts.peak, counts.bytes, counts.allocations
        );
        assert_eq!(
            counts.retained(),
            0,
            "{what} keeps {} bytes once dropped",
            counts.retained()
        );
        assert!(
            counts.peak <= *budget,
            "{what} held {} bytes at once against a budget of {budget}",
            counts.peak
        );
    }

    // The full chart against its own document: the tree it was hashed
    // through held ten times the text, and the text twice over is what
    // writing it costs.
    let document = sdk.chart().reading(birth, &full).unwrap().value;
    let text = canonical_json(&document).len() as u64;
    let chart = measured[2].1.peak;
    println!("a full chart's document: {text} bytes of canonical text");
    assert!(
        chart <= 4 * text,
        "a full chart held {chart} bytes at once for a document of {text}: \
         more than four times the text is a tree built beside it"
    );
}
