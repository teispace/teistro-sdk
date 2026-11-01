//! The `JAIN` pack and the Vira Nirvana Samvat
//! (`03-design/vira-nirvana-and-jain-festivals.md`, SR-59), at Delhi over
//! the built-in ephemeris, against the days published Jain calendars print
//! for 2025 and 2026.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro::catalogue::Calendar;
use teistro::festival::{MetaFilter, Observances};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// A day a published calendar prints: year, key, (month, day), and who
/// printed it.
type Printed = (i32, &'static str, (u8, u8), &'static str);

/// Drik Panchang's Jain calendar for New Delhi
/// (drikpanchang.com/jain/calendar/jain-calendar.html, 2025 and 2026).
const DRIK: &str = "Drik Panchang, Jain calendar, New Delhi";

/// The days the published calendars print, each the acceptance value of a
/// rule.
const PRINTED: [Printed; 21] = [
    (2025, "MAHAVIR_JAYANTI", (4, 10), DRIK),
    (2025, "CHAITRA_AYAMBIL_OLI", (4, 4), DRIK),
    (2025, "CHAITRA_AYAMBIL_OLI_END", (4, 12), DRIK),
    (2025, "PARYUSHANA_TRAYODASHI", (8, 21), DRIK),
    (2025, "SAMVATSARI_PANCHAMI", (8, 28), DRIK),
    (
        2025,
        "SAMVATSARI",
        (8, 27),
        "Jain Center of Northern California, Shwetamber Paryushan 2025",
    ),
    (2025, "ASHVINA_AYAMBIL_OLI", (9, 29), DRIK),
    (2025, "ASHVINA_AYAMBIL_OLI_END", (10, 7), DRIK),
    (
        2025,
        "MAHAVIR_NIRVANA",
        (10, 21),
        "Harvard FAS calendar, Diwali / Mahavir Nirvana 2025",
    ),
    (
        2025,
        "VIRA_NIRVANA_NEW_YEAR",
        (10, 22),
        "Harvard FAS calendar, Diwali / Mahavir Nirvana 2025",
    ),
    (2026, "MAHAVIR_JAYANTI", (3, 31), DRIK),
    (2026, "CHAITRA_AYAMBIL_OLI", (3, 25), DRIK),
    (2026, "CHAITRA_AYAMBIL_OLI_END", (4, 2), DRIK),
    (2026, "PARYUSHANA", (9, 8), DRIK),
    (2026, "SAMVATSARI", (9, 15), DRIK),
    (
        2026,
        "DASA_LAKSHANA",
        (9, 16),
        "Jain Society of Metropolitan Washington, Das Lakshan 2026",
    ),
    (
        2026,
        "ANANTA_CHATURDASHI",
        (9, 25),
        "Jain Society of Metropolitan Washington, Das Lakshan 2026",
    ),
    (2026, "ASHVINA_AYAMBIL_OLI", (10, 17), DRIK),
    (2026, "ASHVINA_AYAMBIL_OLI_END", (10, 26), DRIK),
    (
        2026,
        "MAHAVIR_NIRVANA",
        (11, 9),
        "University of Southern California, Jain holy days 2026",
    ),
    (2026, "VIRA_NIRVANA_NEW_YEAR", (11, 10), DRIK),
];

/// New Delhi, the place Drik Panchang's calendar is reckoned for, on
/// India's clock.
fn delhi() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(28.6139),
            Longitude::literal(77.209),
            Altitude::literal(216.0),
        ),
        UtcOffset::literal(5, 30, 0),
    )
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn year(context: &Context, request: &FestivalRequest, year: i32) -> Observances {
    let (place, offset) = delhi();
    context
        .almanac()
        .festivals(
            &date(year, 1, 1),
            &date(year, 12, 31),
            &place,
            offset,
            request,
        )
        .unwrap()
        .value
}

/// Every day a rule finds for `key` in an answer.
fn days(answer: &Observances, key: &str) -> Vec<(u8, u8)> {
    answer
        .observances
        .iter()
        .filter(|found| found.rule == key)
        .map(|found| (found.day.month, found.day.day))
        .collect()
}

#[test]
fn the_pack_finds_the_days_the_jain_calendars_print() {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let request = FestivalRequest::from(FestivalPack::Jain);
    let mut parted = Vec::new();
    for reckoned in [2025, 2026] {
        let answer = year(&context, &request, reckoned);
        for &(_, key, printed, by) in PRINTED.iter().filter(|row| row.0 == reckoned) {
            let found = days(&answer, key);
            if found != [printed] {
                parted.push(format!(
                    "{reckoned} {key}: {by} prints {printed:?}, the rule finds {found:?}"
                ));
            }
        }
    }
    assert!(parted.is_empty(), "{}", parted.join("\n"));
}

/// A sect's filter keeps that sect's days: the Digambaras keep Das
/// Lakshana and the 8th and 14th, and no Ayambil Oli.
#[test]
fn a_community_filter_keeps_one_sects_days() {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let digambara = FestivalRequest::from(FestivalPack::Jain).keeping(MetaFilter {
        communities: vec!["DIGAMBARA".to_owned()],
        ..MetaFilter::default()
    });
    let answer = year(&context, &digambara, 2026);
    let kept = |key: &str| !days(&answer, key).is_empty();
    assert!(kept("DASA_LAKSHANA") && kept("MAHAVIR_JAYANTI") && kept("JAIN_PARVA_ASHTAMI"));
    assert!(!kept("CHAITRA_AYAMBIL_OLI") && !kept("SAMVATSARI") && !kept("JAIN_PARVA_DVITIYA"));
    // Both halves' 8th, about twenty-five a year.
    assert!((23..=27).contains(&days(&answer, "JAIN_PARVA_ASHTAMI").len()));
}

/// The lunar years carry the Vira Nirvana year their Kartika opens.
#[test]
fn a_lunar_year_carries_the_vira_nirvana_year() {
    let context = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap();
    let (place, offset) = delhi();
    let years = context
        .almanac()
        .years(&date(2025, 8, 1), &date(2025, 8, 1), &place, offset)
        .unwrap()
        .value;
    assert_eq!((years[0].vikrama, years[0].vira_nirvana), (2082, 2552));
}
