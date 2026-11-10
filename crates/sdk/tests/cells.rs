//! A range of calendar cells against the full days that hold them
//! (`03-design/month-cells.md`, SR-84): the same sunrise and sunrise tithi
//! to the bit on every date of a year at Kathmandu on its zone, a Bikram
//! Sambat month, a polar winter answered or refused as the full day is,
//! the kshaya and vriddhi dates the full days show, and the festivals the
//! festival request answers.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and compares the values it was handed"
)]

use teistro::catalogue::{Calendar, Tithi};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, CellKind, CellsRequest, Context, DayCell, DayClock, Ephemeris,
    FestivalPack, FestivalRequest, Panchanga, Status, UtcOffset, ZoneSpec,
};

fn context(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

fn place(latitude: f64, longitude: f64, altitude: f64) -> Place {
    Place::new(
        Latitude::literal(latitude),
        Longitude::literal(longitude),
        Altitude::literal(altitude),
    )
}

fn kathmandu() -> Place {
    place(27.7172, 85.324, 1400.0)
}

fn gregorian(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

fn nepal() -> ZoneSpec {
    ZoneSpec::iana("Asia/Kathmandu")
}

fn asked(first: CalendarDate, last: CalendarDate, place: Place, clock: DayClock) -> CellsRequest {
    CellsRequest {
        first,
        last,
        place,
        clock,
        festivals: None,
    }
}

/// The sunrise tithi of a full day.
fn opening(day: &Panchanga) -> Tithi {
    day.tithi_at(day.day.sunrise)
        .expect("a sunrise tithi")
        .member
}

/// The kind the full days give a date: its sunrise tithi held the sunrise
/// before, and a tithi of its own began after its sunrise and ended before
/// the next.
fn kind_of(before: &Panchanga, day: &Panchanga) -> CellKind {
    let vriddhi =
        before.day.sunrise.get() < day.day.sunrise.get() && opening(before) == opening(day);
    let kshaya = day.limbs.tithi.iter().any(|span| {
        span.sunrises(day.day.sunrise, day.day.next_sunrise)
            .is_kshaya()
    });
    CellKind::of(vriddhi, kshaya)
}

/// Each cell holds its full day's sunrise, next sunrise, vara, month and
/// sunrise tithi, to the bit, and the tithis ending before the next
/// sunrise; `before` is the day before the first cell's.
fn agrees(cells: &[DayCell], before: &Panchanga, days: &[Panchanga]) {
    assert_eq!(cells.len(), days.len());
    let mut last = before;
    for (cell, day) in cells.iter().zip(days) {
        let what = cell.date.to_string();
        assert_eq!(cell.date, day.day.date, "{what}");
        assert_eq!(cell.sunrise, day.day.sunrise, "{what}");
        assert_eq!(cell.next_sunrise, day.day.next_sunrise, "{what}");
        assert_eq!(cell.vara, day.day.vara, "{what}");
        assert_eq!(cell.month, day.month, "{what}");
        assert_eq!(cell.tithi, opening(day), "{what}");
        assert_eq!(cell.paksha, opening(day).attributes().paksha, "{what}");
        let ends: Vec<(Tithi, f64)> = day
            .limbs
            .tithi
            .iter()
            .filter(|span| span.whole.to.get() <= day.day.next_sunrise.get())
            .map(|span| (span.member, span.whole.to.get()))
            .collect();
        let found: Vec<(Tithi, f64)> = cell
            .ends
            .iter()
            .map(|end| (end.tithi, end.at.get()))
            .collect();
        assert_eq!(found, ends, "{what}");
        assert_eq!(cell.kind, kind_of(last, day), "{what}");
        last = day;
    }
}

/// The full days of a range on a zone's clock, and the day before it.
fn full_days(
    sdk: &Context,
    (first, last): (&CalendarDate, &CalendarDate),
    place: &Place,
    zone: &ZoneSpec,
) -> (Panchanga, Vec<Panchanga>) {
    let almanac = sdk.almanac();
    let request = AlmanacRequest::new();
    let days = almanac
        .in_zone(first, last, place, zone, &request)
        .unwrap()
        .days
        .value;
    let calendar = sdk.calendar();
    let yesterday = calendar
        .date_of(
            first.calendar,
            calendar.fixed_of(first).unwrap().plus_days(-1),
        )
        .expect("the day before");
    let before = almanac
        .in_zone(&yesterday, &yesterday, place, zone, &request)
        .unwrap()
        .days
        .value
        .remove(0);
    (before, days)
}

#[test]
fn a_year_of_cells_is_the_full_days_own_to_the_bit() {
    let sdk = context("nepali-drik");
    let (first, last) = (gregorian(2026, 1, 1), gregorian(2026, 12, 31));
    let cells = sdk
        .almanac()
        .cells(&asked(
            first.clone(),
            last.clone(),
            kathmandu(),
            DayClock::Zone(nepal()),
        ))
        .unwrap()
        .value;
    assert!(cells.observances.is_none());
    let (before, days) = full_days(&sdk, (&first, &last), &kathmandu(), &nepal());
    agrees(&cells.cells, &before, &days);
    // Two the full days above already agree with, pinned: 21 November's
    // sunrise falls in Shukla Ekadashi and the Dwadashi after it ends
    // before the next sunrise, touching none; Shukla Navami holds both
    // 18 and 19 November's sunrises.
    let on = |ordinal: usize| {
        let cell = cells.cells.get(ordinal - 1).expect("a day of 2026");
        (cell.date.to_string(), cell)
    };
    let (date, kshaya_day) = on(325);
    assert!(date.ends_with("2026-11-21"), "{date}");
    assert_eq!(kshaya_day.kind, CellKind::Kshaya);
    assert_eq!(kshaya_day.tithi, Tithi::ShuklaEkadashi);
    let skipped: Vec<_> = kshaya_day
        .ends
        .iter()
        .map(|end| (end.tithi, end.kshaya))
        .collect();
    assert_eq!(
        skipped,
        [
            (Tithi::ShuklaEkadashi, false),
            (Tithi::ShuklaDwadashi, true)
        ]
    );
    let (date, vriddhi_day) = on(323);
    assert!(date.ends_with("2026-11-19"), "{date}");
    assert_eq!(vriddhi_day.kind, CellKind::Vriddhi);
    assert_eq!(
        (on(322).1.tithi, vriddhi_day.tithi),
        (Tithi::ShuklaNavami, Tithi::ShuklaNavami)
    );
    // A year holds both, and every tithi once: a kshaya cell ends two, a
    // vriddhi none, so the ends outnumber the days by the kshayas less the
    // vriddhis, about six a year (a tithi is 0.984 of a day on average).
    let count = |kind: CellKind| cells.cells.iter().filter(|cell| cell.kind == kind).count();
    let (kshaya, vriddhi) = (count(CellKind::Kshaya), count(CellKind::Vriddhi));
    assert!(
        kshaya > 0 && vriddhi > 0,
        "{kshaya} kshaya, {vriddhi} vriddhi"
    );
    let ends: usize = cells.cells.iter().map(|cell| cell.ends.len()).sum();
    assert_eq!(ends + vriddhi, cells.cells.len() + kshaya);
    assert!(
        (5..=7).contains(&(kshaya - vriddhi)),
        "{kshaya} kshaya, {vriddhi} vriddhi"
    );
}

#[test]
fn a_bikram_sambat_month_is_its_full_days_own() {
    let sdk = context("nepali-drik");
    let date = |day| CalendarDate::defined(Calendar::BikramSambat, 2083, 1, day);
    let (first, last) = (date(1), date(30));
    let cells = sdk
        .almanac()
        .cells(&asked(
            first.clone(),
            last.clone(),
            kathmandu(),
            DayClock::Zone(nepal()),
        ))
        .unwrap()
        .value
        .cells;
    assert!(
        cells
            .iter()
            .all(|cell| cell.date.calendar == Calendar::BikramSambat)
    );
    let (before, days) = full_days(&sdk, (&first, &last), &kathmandu(), &nepal());
    agrees(&cells, &before, &days);
}

/// Tromsø's polar winter: under a nearest-event policy the cells are the
/// full days' own; under an undefined one both refuse alike.
#[test]
fn a_polar_winter_is_answered_or_refused_as_the_full_day_is() {
    let tromso = place(69.6492, 18.9553, 0.0);
    let (first, last) = (gregorian(2026, 12, 10), gregorian(2026, 12, 31));
    let zone = ZoneSpec::iana("Europe/Oslo");
    let sdk = context("nepali-drik");
    let cells = sdk
        .almanac()
        .cells(&asked(
            first.clone(),
            last.clone(),
            tromso,
            DayClock::Zone(zone.clone()),
        ))
        .unwrap()
        .value
        .cells;
    assert!(cells.iter().any(|cell| !cell.normal));
    let (before, days) = full_days(&sdk, (&first, &last), &tromso, &zone);
    agrees(&cells, &before, &days);
    let strict = context("parashari-classical");
    let offset = UtcOffset::literal(1, 0, 0);
    let refused = strict
        .almanac()
        .cells(&asked(
            first.clone(),
            last.clone(),
            tromso,
            DayClock::Offset(offset),
        ))
        .unwrap_err();
    let full = strict
        .almanac()
        .of(&first, &last, &tromso, offset)
        .unwrap_err();
    assert_eq!(refused.status, full.status, "{refused} against {full}");
}

#[test]
fn the_festivals_are_the_festival_requests_own() {
    let sdk = context("nepali-drik");
    let (first, last) = (gregorian(2026, 10, 1), gregorian(2026, 10, 31));
    let offset = UtcOffset::literal(5, 45, 0);
    let festivals = FestivalRequest::from(FestivalPack::Nepal);
    let mut request = asked(
        first.clone(),
        last.clone(),
        kathmandu(),
        DayClock::Offset(offset),
    );
    request.festivals = Some(festivals.clone());
    let answer = sdk.almanac().cells(&request).unwrap();
    let found = sdk
        .almanac()
        .festivals(&first, &last, &kathmandu(), offset, &festivals)
        .unwrap()
        .value;
    let observances = answer.value.observances.clone().expect("the observances");
    assert_eq!(observances, found);
    assert_ne!(
        found.observances.len(),
        0,
        "a Nepal October keeps festivals"
    );
    for cell in &answer.value.cells {
        assert_eq!(cell.festivals, found.keys_on(&cell.date), "{}", cell.date);
    }
    assert!(
        answer
            .value
            .cells
            .iter()
            .any(|cell| !cell.festivals.is_empty())
    );
    // Asked without them, the cells are the same but for their keys.
    request.festivals = None;
    let bare = sdk.almanac().cells(&request).unwrap();
    assert!(
        bare.value
            .cells
            .iter()
            .all(|cell| cell.festivals.is_empty())
    );
    assert_ne!(bare.provenance.input_hash, answer.provenance.input_hash);
}

#[test]
fn a_request_is_read_strictly_and_held_to_a_year() {
    let refused = CellsRequest::from_json(
        r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 27.7172,
            "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "kinds": []}"#,
    )
    .unwrap_err();
    assert_eq!(refused.field(), Some("cells.kinds"));
    let sdk = context("nepali-drik");
    let long = asked(
        gregorian(2026, 1, 1),
        gregorian(2027, 1, 2),
        kathmandu(),
        DayClock::Offset(UtcOffset::literal(5, 45, 0)),
    );
    let refused = sdk.almanac().cells(&long).unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::OutOfRange, Some("to"))
    );
    let backwards = asked(
        gregorian(2026, 1, 2),
        gregorian(2026, 1, 1),
        kathmandu(),
        DayClock::Offset(UtcOffset::literal(5, 45, 0)),
    );
    assert!(sdk.almanac().cells(&backwards).is_err());
}
