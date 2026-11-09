//! The `MITHILA` pack over the `mithila-makaranda` profile
//! (`03-design/makaranda.md` §8, SR-16): the days the Vishwavidyalaya
//! Panchang prints for 2022-23 and 2009-10, reckoned in the print's own
//! sky at the place it states, on India's clock.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and indexes what it built"
)]

use teistro::catalogue::Calendar;
use teistro::festival::Observances;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};

/// A printed day: key, (year, month, day), and the page.
type Printed = (&'static str, (i32, u8, u8), &'static str);

/// The days the print gives, each the acceptance value of a rule.
const PRINTED: [Printed; 14] = [
    ("MAUNA_PANCHAMI", (2022, 7, 18), "2022-23 PDF p. 11"),
    ("MADHUSHRAVANI_ARAMBHA", (2022, 7, 18), "2022-23 PDF p. 11"),
    ("MADHUSHRAVANI_SAMAPTI", (2022, 7, 31), "2022-23 PDF p. 12"),
    ("CHAURCHAN", (2022, 8, 30), "2022-23 PDF p. 14"),
    ("OTHGAN", (2022, 9, 16), "2022-23 PDF p. 15"),
    ("JITIYA", (2022, 9, 17), "2022-23 PDF p. 15"),
    ("CHHATH", (2022, 10, 30), "2022-23 PDF p. 18"),
    ("SAMA_CHAKEVA_ARAMBHA", (2022, 10, 31), "2022-23 PDF p. 18"),
    ("VIDYAPATI_SMRITI_DIVAS", (2022, 11, 6), "2022-23 PDF p. 18"),
    ("SAMA_CHAKEVA_VISARJANA", (2022, 11, 7), "2022-23 PDF p. 18"),
    ("JUR_SITAL", (2023, 4, 15), "2022-23 PDF p. 29"),
    ("CHAURCHAN", (2009, 8, 23), "2009-10 PDF p. 14"),
    ("OTHGAN", (2009, 9, 10), "2009-10 PDF p. 15"),
    ("JITIYA", (2009, 9, 11), "2009-10 PDF p. 15"),
];

/// The place the print states every Mithila panchang is computed for,
/// 85°30′ E, 26°35′ N, on India's clock.
fn mithila() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(26.0 + 35.0 / 60.0),
            Longitude::literal(85.5),
            Altitude::literal(50.0),
        ),
        UtcOffset::literal(5, 30, 0),
    )
}

/// The print's sky: the Makaranda over the text.
fn makaranda() -> Context {
    Context::builder()
        .profile("mithila-makaranda")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap()
}

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// The answer over a Gregorian range.
fn between(
    context: &Context,
    request: &FestivalRequest,
    from: (i32, u8, u8),
    to: (i32, u8, u8),
) -> Observances {
    let (place, offset) = mithila();
    context
        .almanac()
        .festivals(
            &date(from.0, from.1, from.2),
            &date(to.0, to.1, to.2),
            &place,
            offset,
            request,
        )
        .unwrap()
        .value
}

/// Every day the answer gives `key`, by the tithi, a count or a date.
fn days(answer: &Observances, key: &str) -> Vec<(i32, u8, u8)> {
    let lunar = answer.observances.iter().map(|o| (&o.rule, &o.day));
    let dated = answer.dated.iter().map(|d| (&d.rule, &d.day));
    lunar
        .chain(dated)
        .filter(|(rule, _)| rule.as_str() == key)
        .map(|(_, day)| (day.year, day.month, day.day))
        .collect()
}

#[test]
fn the_pack_finds_the_days_the_mithila_print_gives() {
    let context = makaranda();
    let request = FestivalRequest::from(FestivalPack::Mithila);
    let years = [
        between(&context, &request, (2022, 7, 1), (2023, 4, 30)),
        between(&context, &request, (2009, 8, 1), (2009, 9, 30)),
    ];
    let mut parted = Vec::new();
    for &(key, printed, page) in &PRINTED {
        let answer = &years[usize::from(printed.0 == 2009)];
        let found = days(answer, key);
        if found != [printed] {
            parted.push(format!(
                "{key}: {page} prints {printed:?}, the rule finds {found:?}"
            ));
        }
    }
    assert!(parted.is_empty(), "{}", parted.join("\n"));
}

/// The committee's Jitiya stays the udaya day, and naming the Mithila pack
/// after it puts the print's day in its place.
#[test]
fn the_mithila_jitiya_replaces_the_committees_only_when_asked() {
    let context = makaranda();
    let nepal = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let udaya = between(&context, &nepal, (2022, 9, 1), (2022, 9, 30));
    assert_eq!(days(&udaya, "JITIYA"), [(2022, 9, 18)]);
    let laid = nepal.with_pack(FestivalPack::Mithila);
    laid.check().unwrap();
    let print = between(&context, &laid, (2022, 9, 1), (2022, 9, 30));
    assert_eq!(days(&print, "JITIYA"), [(2022, 9, 17)]);
    assert_eq!(days(&print, "OTHGAN"), [(2022, 9, 16)]);
}
