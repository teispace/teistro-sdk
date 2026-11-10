//! `INDIA_TAMIL_NADU` against each year's order of the Public
//! (Miscellaneous) Department, 2019 to 2026, at Chennai under the
//! `tamil-vakya` profile the pack ships for (`03-design/tamil-nadu-holidays.md`,
//! C1350): every reckoned day of every year meets the print but the four
//! named partings, and the days set by the moon stand as printed.

#![cfg(feature = "builtin-ephemeris")]
#![allow(
    clippy::unwrap_used,
    missing_docs,
    reason = "a test fails by panicking"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{Observances, SubjectTo};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// The keys of each year's row, in this order.
const KEYS: [&str; 13] = [
    "THAI_PONGAL",
    "THIRUVALLUVAR_DAY",
    "UZHAVAR_THIRUNAL",
    "THAI_POOSAM",
    "SAMVATSARA_ARAMBHA",
    "MAHAVIR_JAYANTI",
    "GOOD_FRIDAY",
    "PUTHANDU",
    "JANMASHTAMI",
    "GANESHA_CHATURTHI",
    "AYUDHA_PUJA",
    "VIJAYA_DASHAMI",
    "NARAK_CHATURDASHI",
];

/// Each order's days as (month, day), in the order of [`KEYS`]; `None`
/// where the year's order has no such row (Thai Poosam before 2021).
/// Vinayakar Chathurthi 2023 is the day G.O.(Ms.) No. 528 declared.
type Year = (i32, [Option<(u8, u8)>; 13]);

const PRINTED: [Year; 8] = [
    (
        2019,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 17)),
            None,
            Some((4, 6)),
            Some((4, 17)),
            Some((4, 19)),
            Some((4, 14)),
            Some((8, 23)),
            Some((9, 2)),
            Some((10, 7)),
            Some((10, 8)),
            Some((10, 27)),
        ],
    ),
    (
        2020,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 17)),
            None,
            Some((3, 25)),
            Some((4, 6)),
            Some((4, 10)),
            Some((4, 14)),
            Some((8, 11)),
            Some((8, 22)),
            Some((10, 25)),
            Some((10, 26)),
            Some((11, 14)),
        ],
    ),
    (
        2021,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 28)),
            Some((4, 13)),
            Some((4, 25)),
            Some((4, 2)),
            Some((4, 14)),
            Some((8, 30)),
            Some((9, 10)),
            Some((10, 14)),
            Some((10, 15)),
            Some((11, 4)),
        ],
    ),
    (
        2022,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 18)),
            Some((4, 2)),
            Some((4, 14)),
            Some((4, 15)),
            Some((4, 14)),
            Some((8, 19)),
            Some((8, 31)),
            Some((10, 4)),
            Some((10, 5)),
            Some((10, 24)),
        ],
    ),
    (
        2023,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 17)),
            Some((2, 5)),
            Some((3, 22)),
            Some((4, 4)),
            Some((4, 7)),
            Some((4, 14)),
            Some((9, 6)),
            Some((9, 18)),
            Some((10, 23)),
            Some((10, 24)),
            Some((11, 12)),
        ],
    ),
    (
        2024,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 17)),
            Some((1, 25)),
            Some((4, 9)),
            Some((4, 21)),
            Some((3, 29)),
            Some((4, 14)),
            Some((8, 26)),
            Some((9, 7)),
            Some((10, 11)),
            Some((10, 12)),
            Some((10, 31)),
        ],
    ),
    (
        2025,
        [
            Some((1, 14)),
            Some((1, 15)),
            Some((1, 16)),
            Some((2, 11)),
            Some((3, 30)),
            Some((4, 10)),
            Some((4, 18)),
            Some((4, 14)),
            Some((8, 16)),
            Some((8, 27)),
            Some((10, 1)),
            Some((10, 2)),
            Some((10, 20)),
        ],
    ),
    (
        2026,
        [
            Some((1, 15)),
            Some((1, 16)),
            Some((1, 17)),
            Some((2, 1)),
            Some((3, 19)),
            Some((3, 31)),
            Some((4, 3)),
            Some((4, 14)),
            Some((9, 4)),
            Some((9, 14)),
            Some((10, 19)),
            Some((10, 20)),
            Some((11, 8)),
        ],
    ),
];

/// A row that parts: its year, key and the days the answer gives it.
type Parted = (i32, &'static str, Vec<(u8, u8)>);

/// The rows that part from the print under the vakya sky, each with the
/// day the rule gives (`festival-measured.md` §13).
const PARTS: [(i32, &str, (u8, u8)); 4] = [
    // The chaturdashi holds neither sunrise, ending at 05:41 on the 4th
    // (C1356).
    (2021, "NARAK_CHATURDASHI", (11, 3)),
    // Pushya's first sunrise is the 26th's; the print keeps the full
    // moon's day (C1352).
    (2024, "THAI_POOSAM", (1, 26)),
    // The dashami holds the 21st's sunrise; the print keeps the 20th
    // (C1357), and Ayutha Pooja is counted from it (C1353).
    (2026, "AYUDHA_PUJA", (10, 20)),
    (2026, "VIJAYA_DASHAMI", (10, 21)),
];

fn chennai() -> Place {
    Place::new(
        Latitude::literal(13.0827),
        Longitude::literal(80.2707),
        Altitude::literal(0.0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn vakya_sky() -> Context {
    Context::builder()
        .profile("tamil-vakya")
        .ephemeris([Ephemeris::SuryaSiddhanta, Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// A year's answer at Chennai.
fn year(context: &Context, year: i32) -> Observances {
    context
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &chennai(),
            UtcOffset::literal(5, 30, 0),
            &FestivalRequest::from(FestivalPack::IndiaTamilNadu),
        )
        .unwrap()
        .value
}

/// The days the answer gives `key`, of whichever kind.
fn days_of(answer: &Observances, key: &str) -> Vec<(u8, u8)> {
    let lunar = answer
        .observances
        .iter()
        .filter(|found| found.rule == key)
        .map(|found| (found.day.month, found.day.day));
    let dated = answer
        .dated
        .iter()
        .filter(|found| found.rule == key)
        .map(|found| (found.day.month, found.day.day));
    lunar.chain(dated).collect()
}

#[test]
fn every_order_of_2019_to_2026_meets_the_rule_but_the_named_four() {
    let context = vakya_sky();
    let mut parted: Vec<Parted> = Vec::new();
    for (printed_year, printed) in PRINTED {
        let answer = year(&context, printed_year);
        for (key, day) in KEYS.iter().zip(printed) {
            let Some(day) = day else { continue };
            let found = days_of(&answer, key);
            if found != [day] {
                parted.push((printed_year, *key, found));
            }
        }
        // The days set by the moon stand as printed, each marked.
        assert_eq!(answer.added.len(), 4, "{printed_year}");
        assert!(
            answer
                .added
                .iter()
                .all(|added| added.published.subject_to == Some(SubjectTo::MoonSighting)),
            "{printed_year}"
        );
    }
    let named: Vec<Parted> = PARTS
        .iter()
        .map(|&(year, key, day)| (year, key, vec![day]))
        .collect();
    assert_eq!(parted, named);
}
