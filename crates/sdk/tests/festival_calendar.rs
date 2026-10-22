//! A festival feed (`03-design/icalendar.md`): Dashain and Tihar 2025 at
//! Kathmandu under Nepal's official reckoning, written as iCalendar and
//! read back against the festival answer it was written from.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use std::collections::BTreeSet;

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, FeedOptions, FestivalCalendarRequest, FestivalPack, FestivalRequest,
    UtcOffset,
};
use teistro_calendar::CalendarSystem as _;
use teistro_core::envelope::Hash;

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn day(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2025, month, day)
}

const STAMP: i64 = 1_767_225_600;

fn feed(sdk: &Context, options: &FeedOptions) -> String {
    sdk.almanac()
        .festival_calendar(
            &day(9, 20),
            &day(11, 10),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
            options,
        )
        .unwrap()
        .value
}

/// The value of each line naming `property`, unfolded.
fn values<'a>(unfolded: &'a str, property: &str) -> Vec<&'a str> {
    unfolded
        .split("\r\n")
        .filter_map(|line| line.strip_prefix(property))
        .collect()
}

#[test]
fn every_observance_and_fast_is_one_event_on_its_own_day() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let answer = sdk
        .almanac()
        .festivals(
            &day(9, 20),
            &day(11, 10),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
        )
        .unwrap()
        .value;
    let text = feed(&sdk, &FeedOptions::new(STAMP));
    let unfolded = text.replace("\r\n ", "");
    let starts = values(&unfolded, "DTSTART;VALUE=DATE:");
    assert_eq!(
        starts.len(),
        answer.observances.len() + answer.ekadashis.len() + answer.dated.len()
    );
    assert!(
        starts.len() > 20,
        "a festival season holds more than {}",
        starts.len()
    );
    // Ordered by day.
    assert!(starts.windows(2).all(|pair| pair[0] <= pair[1]));
    // Every occurrence its own UID.
    let uids = values(&unfolded, "UID:");
    assert_eq!(uids.iter().collect::<BTreeSet<_>>().len(), uids.len());
    // Vijaya Dashami on the day the answer gives.
    let dashami = answer
        .observances
        .iter()
        .find(|o| o.rule == "VIJAYA_DASHAMI")
        .unwrap();
    let on = format!(
        "DTSTART;VALUE=DATE:{:04}{:02}{:02}\r\nDTEND;VALUE=DATE:",
        dashami.day.year, dashami.day.month, dashami.day.day
    );
    let event = unfolded
        .split("BEGIN:VEVENT")
        .find(|event| event.contains("SUMMARY:Vijaya Dashami\r\n"))
        .unwrap();
    assert!(event.contains(&on), "{event}");
    assert!(
        event.contains("under the nepali-committee profile"),
        "{event}"
    );
}

#[test]
fn a_fast_s_parana_is_a_timed_event_inside_its_window() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let answer = sdk
        .almanac()
        .festivals(
            &day(9, 20),
            &day(11, 10),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
        )
        .unwrap()
        .value;
    let kept: Vec<_> = answer
        .ekadashis
        .iter()
        .filter_map(|fast| Some((fast, fast.parana.as_ref()?.kept?)))
        .collect();
    assert!(kept.len() >= 3, "{} fasts keep a parana", kept.len());
    let unfolded = feed(&sdk, &FeedOptions::new(STAMP)).replace("\r\n ", "");
    let timed: Vec<&str> = unfolded
        .split("BEGIN:VEVENT")
        .filter(|event| event.contains("\r\nDTSTART:"))
        .collect();
    assert_eq!(timed.len(), kept.len());
    let unix = |jd: f64| (jd - 2_440_587.5) * 86_400.0;
    for (fast, window) in kept {
        let name = sdk
            .intl()
            .entity(&format!("festival.{}", fast.rule))
            .unwrap();
        let summary = format!("SUMMARY:Parana: {}\r\n", name.form("name").unwrap());
        let parana = fast.parana.as_ref().unwrap();
        let on = format!(
            "{:04}{:02}{:02}T",
            parana.day.year, parana.day.month, parana.day.day
        );
        let event = timed
            .iter()
            .find(|event| event.contains(&summary) && event.contains(&format!("DTSTART:{on}")))
            .expect("each fast's parana an event on its day");
        // The shown minutes lie inside the computed window, and each end
        // is within the minute of it.
        let [from, to] = ["DTSTART:", "DTEND:"].map(|property| {
            let at = values(event, property)[0];
            let (h, m, s) = (&at[9..11], &at[11..13], &at[13..15]);
            assert_eq!(s, "00", "{at}: an end on the minute");
            let date = teistro_calendar::Gregorian
                .to_fixed_ymd(
                    at[..4].parse().unwrap(),
                    at[4..6].parse().unwrap(),
                    at[6..8].parse().unwrap(),
                )
                .unwrap();
            f64::from(i32::try_from(date.get() - 719_163).unwrap()) * 86_400.0
                + h.parse::<f64>().unwrap() * 3_600.0
                + m.parse::<f64>().unwrap() * 60.0
        });
        let (computed_from, computed_to) =
            (unix(window.span.from.get()), unix(window.span.to.get()));
        assert!(from >= computed_from - 1e-3 && from - computed_from < 60.0);
        assert!(to <= computed_to + 1e-3 && computed_to - to < 60.0);
    }
    // A rule without a parana writes no timed event.
    let request = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let without: Vec<_> = request
        .ekadashis()
        .iter()
        .cloned()
        .map(|mut rule| {
            rule.parana = None;
            rule
        })
        .collect();
    let request = request.with_ekadashis(without);
    let none = sdk
        .almanac()
        .festival_calendar(
            &day(9, 20),
            &day(11, 10),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &request,
            &FeedOptions::new(STAMP),
        )
        .unwrap()
        .value;
    assert!(!none.contains("\r\nDTSTART:"));
}

#[test]
fn every_line_is_crlf_and_at_most_75_octets() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    sdk.intl().set_locale("ne-Deva-NP").unwrap();
    let text = feed(&sdk, &FeedOptions::new(STAMP));
    assert!(text.ends_with("END:VCALENDAR\r\n"));
    assert_eq!(text.matches('\n').count(), text.matches("\r\n").count());
    for line in text.split("\r\n") {
        assert!(line.len() <= 75, "{} octets: {line}", line.len());
    }
    // The summary is the name in the context's locale.
    let name = sdk.intl().entity("festival.VIJAYA_DASHAMI").unwrap();
    let unfolded = text.replace("\r\n ", "");
    assert!(unfolded.contains(&format!("SUMMARY:{}\r\n", name.form("name").unwrap())));
    // And a parana's in its words.
    assert!(
        unfolded.contains("को पारणा\r\n"),
        "a parana named in Nepali"
    );
}

#[test]
fn the_same_request_writes_the_same_bytes_and_the_stamp_moves_only_dtstamp() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let first = feed(&sdk, &FeedOptions::new(STAMP));
    assert_eq!(first, feed(&sdk, &FeedOptions::new(STAMP)));
    let later = feed(&sdk, &FeedOptions::new(STAMP + 86_400));
    let differing: Vec<_> = first
        .split("\r\n")
        .zip(later.split("\r\n"))
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(
        differing.len(),
        first.matches("BEGIN:VEVENT").count(),
        "every DTSTAMP moved"
    );
    assert!(differing.iter().all(|(a, _)| a.starts_with("DTSTAMP:")));
}

#[test]
fn a_request_a_binding_sends_writes_what_the_call_does_and_seals_the_text() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let asked = FestivalCalendarRequest::from_json(&format!(
        r#"{{"first": {{"year": 2025, "month": 9, "day": 20}},
            "last": {{"year": 2025, "month": 11, "day": 10}},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, "festivals": {{"rules": "NEPAL_PANCHANGA"}},
            "stampUnixSeconds": {STAMP}, "name": "Nepal", "refreshMinutes": 0}}"#
    ))
    .unwrap();
    let sealed = sdk.almanac().festival_calendar_request(&asked).unwrap();
    let options = FeedOptions::new(STAMP)
        .with_name("Nepal")
        .with_refresh_minutes(None);
    assert_eq!(sealed.value, feed(&sdk, &options));
    assert!(sealed.value.contains("X-WR-CALNAME:Nepal\r\n"));
    assert!(!sealed.value.contains("REFRESH-INTERVAL"));
    assert_eq!(
        sealed.provenance.content_hash,
        Hash::of(teistro_core::envelope::canonical_json(&sealed.value).as_bytes())
    );
}

#[test]
fn a_request_without_a_stamp_or_festivals_is_refused_by_name() {
    for text in [
        r#"{"first": {"year": 2025, "month": 9, "day": 20}, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
            "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"}}"#,
        r#"{"first": {"year": 2025, "month": 9, "day": 20}, "latitudeDeg": 27.7, "longitudeDeg": 85.3,
            "utcOffsetSeconds": 20700, "stampUnixSeconds": 0}"#,
        r#"{"first": {"year": 2025, "month": 9, "day": 20}, "latitudeDeg": 91, "longitudeDeg": 85.3,
            "utcOffsetSeconds": 20700, "festivals": {"rules": "NEPAL"}, "stampUnixSeconds": 0}"#,
    ] {
        let why = FestivalCalendarRequest::from_json(text).unwrap_err();
        assert!(
            why.field().unwrap().starts_with("festivalCalendar"),
            "{:?}",
            why.field()
        );
    }
}
