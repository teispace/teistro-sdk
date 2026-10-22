//! The `GAUDIYA` pack's mahadvadashis (`03-design/festival-rules.md` §8.9)
//! held to the published Gaudiya lists (rank 3): Drik Panchang's ISKCON
//! Ekadashi list for New Delhi, 2021 to 2027, and the printed calendars of
//! ISKCON Bangalore (2024–25 to 2026–27) and ISKCON Melbourne (2026),
//! read 2026-10-22.
//!
//! Every labelled row is the day and the kind found. At Delhi a kind found
//! on a day the list leaves unlabelled is a margin case (C712): its
//! margin is under two minutes. The lists' sunrise is the refracted upper
//! limb's, as in `vrata_parana.rs`.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::catalogue::Calendar;
use teistro::festival::{EkadashiFast, MahadvadashiKind};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

use MahadvadashiKind::{Pakshavardhini, Trisprisha, Unmilani, Vyanjuli};

/// A fast's day, as (year, month, day), and the kind printed for it;
/// `None` for a row printing "Mahadvadasi" alone.
type Row = ((i32, u8, u8), Option<MahadvadashiKind>);

/// Drik Panchang's ISKCON list, New Delhi (§8.9 rows 1 to 18).
const DELHI: [Row; 18] = [
    ((2021, 5, 23), Some(Trisprisha)),
    ((2021, 7, 6), Some(Pakshavardhini)),
    ((2022, 2, 27), Some(Trisprisha)),
    ((2022, 6, 11), Some(Trisprisha)),
    ((2022, 8, 23), Some(Unmilani)),
    ((2023, 4, 2), Some(Vyanjuli)),
    ((2023, 9, 11), Some(Pakshavardhini)),
    ((2023, 12, 9), Some(Vyanjuli)),
    ((2023, 12, 23), Some(Trisprisha)),
    ((2024, 8, 30), Some(Pakshavardhini)),
    ((2024, 10, 14), Some(Trisprisha)),
    ((2025, 6, 7), Some(Vyanjuli)),
    ((2025, 11, 2), Some(Trisprisha)),
    ((2025, 12, 16), Some(Pakshavardhini)),
    ((2026, 8, 24), Some(Vyanjuli)),
    ((2027, 1, 19), Some(Trisprisha)),
    ((2027, 3, 4), Some(Unmilani)),
    ((2027, 6, 15), Some(Pakshavardhini)),
];

/// ISKCON Bangalore's calendars (rows 19 to 25, less row 21, a margin
/// case below).
const BANGALORE: [Row; 6] = [
    ((2024, 8, 30), Some(Pakshavardhini)),
    ((2024, 10, 14), Some(Trisprisha)),
    ((2025, 6, 7), None),
    ((2025, 11, 2), Some(Trisprisha)),
    ((2025, 12, 16), Some(Pakshavardhini)),
    ((2026, 11, 21), Some(Trisprisha)),
];

/// The lists' sunrise is the upper limb's, refracted (§8.6).
fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}}}"#,
        )
        .build()
        .unwrap()
}

fn place(latitude: f64, longitude: f64) -> Place {
    Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(0.0),
    )
}

/// The `GAUDIYA` fasts at `place` from `from` to `to`, local `offset`,
/// asked a calendar year at a time.
fn fasts(
    place: &Place,
    offset: UtcOffset,
    from: (i32, u8, u8),
    to: (i32, u8, u8),
) -> Vec<EkadashiFast> {
    let date = |(year, month, day): (i32, u8, u8)| {
        CalendarDate::defined(Calendar::Gregorian, year, month, day)
    };
    let sdk = context();
    (from.0..=to.0)
        .flat_map(|year| {
            let first = if year == from.0 { from } else { (year, 1, 1) };
            let last = if year == to.0 { to } else { (year, 12, 31) };
            let found = sdk
                .almanac()
                .festivals(
                    &date(first),
                    &date(last),
                    place,
                    offset,
                    &FestivalRequest::from(FestivalPack::Gaudiya),
                )
                .unwrap()
                .value;
            assert!(found.unjudged.is_empty(), "{:?}", found.unjudged);
            found.ekadashis
        })
        .collect()
}

fn day_of(fast: &EkadashiFast) -> (i32, u8, u8) {
    (fast.day.year, fast.day.month, fast.day.day)
}

fn kind_of(fast: &EkadashiFast) -> Option<MahadvadashiKind> {
    fast.mahadvadashi.as_ref().map(|held| held.kind)
}

/// Each printed row that the fasts do not hold, by day and kind.
fn missed(rows: &[Row], fasts: &[EkadashiFast]) -> Vec<String> {
    rows.iter()
        .filter(|&&(day, kind)| {
            !fasts.iter().any(|fast| {
                day_of(fast) == day
                    && kind_of(fast).is_some()
                    && kind.is_none_or(|kind| kind_of(fast) == Some(kind))
            })
        })
        .map(|(day, kind)| {
            let near: Vec<_> = fasts
                .iter()
                .filter(|fast| {
                    let (year, month, _) = day_of(fast);
                    (year, month) == (day.0, day.1)
                })
                .map(|fast| (day_of(fast), kind_of(fast), fast.margin_seconds.round()))
                .collect();
            format!("{day:?} {kind:?}: found {near:?}")
        })
        .collect()
}

#[test]
fn every_labelled_delhi_row_is_the_kind_found_and_every_other_kind_a_margin() {
    let fasts = fasts(
        &place(28.6356, 77.2244),
        UtcOffset::literal(5, 30, 0),
        (2021, 1, 1),
        (2027, 12, 31),
    );
    let missed = missed(&DELHI, &fasts);
    assert!(missed.is_empty(), "{}", missed.join("\n"));
    let unlabelled: Vec<_> = fasts
        .iter()
        .filter(|fast| {
            kind_of(fast).is_some() && !DELHI.iter().any(|&(day, _)| day == day_of(fast))
        })
        .map(|fast| (day_of(fast), kind_of(fast), fast.margin_seconds.round()))
        .collect();
    assert!(
        unlabelled.iter().all(|&(_, _, margin)| margin < 120.0),
        "kinds the list does not label, each with its margin: {unlabelled:?}"
    );
}

#[test]
fn every_labelled_bangalore_and_melbourne_row_is_the_kind_found() {
    let bangalore = fasts(
        &place(12.9716, 77.5946),
        UtcOffset::literal(5, 30, 0),
        (2024, 3, 1),
        (2027, 2, 28),
    );
    let mut misses = missed(&BANGALORE, &bangalore);
    // Row 21, Vyanjuli on 27 November 2024: here the dark 12th ends 18
    // seconds before the sunrise after its own, so neither stands in
    // excess and the fast is the 11th's own day, the 26th; the margin says
    // the day turns on those seconds.
    let parted: Vec<_> = bangalore
        .iter()
        .filter(|fast| fast.days[1].month == 11 && fast.days[1].day == 27)
        .map(|fast| (day_of(fast), kind_of(fast), fast.margin_seconds))
        .collect();
    assert!(
        matches!(parted[..], [((2024, 11, 26), None, margin)] if margin < 60.0),
        "{parted:?}"
    );
    let melbourne = place(-37.8136, 144.9631);
    let summer = fasts(
        &melbourne,
        UtcOffset::literal(11, 0, 0),
        (2026, 1, 1),
        (2026, 2, 28),
    );
    let winter = fasts(
        &melbourne,
        UtcOffset::literal(10, 0, 0),
        (2026, 6, 1),
        (2026, 8, 31),
    );
    misses.extend(missed(&[((2026, 1, 15), Some(Pakshavardhini))], &summer));
    misses.extend(missed(&[((2026, 7, 11), Some(Trisprisha))], &winter));
    assert!(misses.is_empty(), "{}", misses.join("\n"));
}
