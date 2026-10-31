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
    CalendarDate, Context, FeedForm, FeedHint, FeedOptions, FeedSelector, FestivalCalendarRequest,
    FestivalPack, FestivalRequest, Status, SuppliedEvent, UtcOffset,
    calendar_feed::{Alarm, Color, Hints, When},
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

#[test]
fn a_vrata_s_parana_is_timed_from_its_first_reading_when_that_ends() {
    let sdk = Context::builder().nepal_official([]).build().unwrap();
    let (from, to) = (
        CalendarDate::defined(Calendar::Gregorian, 2026, 2, 1),
        CalendarDate::defined(Calendar::Gregorian, 2026, 9, 30),
    );
    let request = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let answer = sdk
        .almanac()
        .festivals(
            &from,
            &to,
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &request,
        )
        .unwrap()
        .value;
    let text = sdk
        .almanac()
        .festival_calendar(
            &from,
            &to,
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &request,
            &FeedOptions::new(STAMP),
        )
        .unwrap()
        .value
        .replace("\r\n ", "");
    let timed: Vec<&str> = text
        .split("BEGIN:VEVENT")
        .filter(|event| event.contains("\r\nDTSTART:"))
        .collect();
    let summary = |rule: &str| {
        let name = sdk.intl().entity(&format!("festival.{rule}")).unwrap();
        format!("SUMMARY:Parana: {}\r\n", name.form("name").unwrap())
    };
    // Shivaratri's first reading keeps a window (WITHIN, three yamas).
    let shivaratri = answer
        .observances
        .iter()
        .find(|o| o.rule == "SHIVARATRI")
        .unwrap();
    let first = &shivaratri.parana[0];
    assert!(first.to.is_some());
    let on = format!(
        "DTSTART:{:04}{:02}{:02}T",
        first.day.year, first.day.month, first.day.day
    );
    let event = timed
        .iter()
        .find(|event| event.contains(&summary("SHIVARATRI")))
        .expect("Shivaratri's parana a timed event");
    assert!(event.contains(&on), "{event}");
    assert!(event.contains("UID:"), "{event}");
    // Janmashtami's and Rama Navami's first readings are open: no event.
    for rule in ["JANMASHTAMI", "RAMA_NAVAMI"] {
        let observance = answer.observances.iter().find(|o| o.rule == rule).unwrap();
        assert!(observance.parana[0].to.is_none(), "{rule}");
        assert!(
            !timed.iter().any(|event| event.contains(&summary(rule))),
            "{rule}"
        );
    }
}

fn sdk() -> Context {
    Context::builder().nepal_official([]).build().unwrap()
}

/// The value of each `name:` line of an unfolded feed.
fn lines<'a>(unfolded: &'a str, name: &str) -> Vec<&'a str> {
    let prefix = format!("{name}:");
    unfolded
        .split("\r\n")
        .filter_map(|line| line.strip_prefix(prefix.as_str()))
        .collect()
}

fn unfolded(text: &str) -> String {
    text.replace("\r\n ", "")
}

fn gregorian_fixed(month: u8, of: u8) -> teistro::FixedDay {
    teistro_calendar::Gregorian
        .to_fixed_ymd(2025, month, of)
        .unwrap()
}

#[test]
fn a_supplied_event_is_merged_in_day_order_and_keeps_its_hints() {
    let sdk = sdk();
    let mut holiday = SuppliedEvent::new("Office closed", When::Day(gregorian_fixed(10, 3)));
    holiday.kind = Some("HOLIDAY".to_owned());
    holiday.tradition = Some("CIVIL".to_owned());
    holiday.hints.categories = vec!["Work".to_owned(), "HOLIDAY".to_owned()];
    holiday.hints.color = Some(Color::named("red").unwrap());
    // 2025-10-02T20:00:00Z is 01:45 on the 3rd at Kathmandu.
    let call = SuppliedEvent::new(
        "Call",
        When::Span {
            from_unix_seconds: 1_759_435_200,
            to_unix_seconds: 1_759_438_800,
        },
    );
    let mut options = FeedOptions::new(STAMP);
    options.events = vec![call, holiday];
    let text = unfolded(&feed(&sdk, &options));
    let plain = unfolded(&feed(&sdk, &FeedOptions::new(STAMP)));
    assert_eq!(
        lines(&text, "BEGIN").len(),
        lines(&plain, "BEGIN").len() + 2
    );
    let days = lines(&text, "DTSTART;VALUE=DATE");
    let mut sorted = days.clone();
    sorted.sort_unstable();
    assert_eq!(days, sorted, "whole days stay in order");
    let summaries = lines(&text, "SUMMARY");
    let closed = summaries
        .iter()
        .position(|s| *s == "Office closed")
        .unwrap();
    let call = summaries.iter().position(|s| *s == "Call").unwrap();
    // Whole days first on the 3rd, the timed call after them.
    assert!(closed < call);
    assert!(text.contains("CATEGORIES:HOLIDAY,CIVIL,Work\r\nCOLOR:red\r\n"));
    // Its UID is the same each time.
    assert_eq!(
        lines(&text, "UID"),
        lines(&unfolded(&feed(&sdk, &options)), "UID")
    );
}

#[test]
fn a_supplied_uid_replaces_the_computed_event_and_two_supplied_are_refused() {
    let sdk = sdk();
    let plain = unfolded(&feed(&sdk, &FeedOptions::new(STAMP)));
    let first = lines(&plain, "UID")[0].to_owned();
    let mut mine = SuppliedEvent::new("My own day", When::Day(gregorian_fixed(9, 25)));
    mine.uid = Some(first.clone());
    let mut options = FeedOptions::new(STAMP);
    options.events = vec![mine.clone()];
    let text = unfolded(&feed(&sdk, &options));
    assert_eq!(lines(&text, "UID").len(), lines(&plain, "UID").len());
    assert_eq!(
        lines(&text, "UID").iter().filter(|u| **u == first).count(),
        1
    );
    assert!(text.contains("SUMMARY:My own day\r\n"));
    options.events = vec![mine.clone(), mine];
    let refused = sdk
        .almanac()
        .festival_calendar(
            &day(9, 20),
            &day(11, 10),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
            &FestivalRequest::from(FestivalPack::NepalPanchanga),
            &options,
        )
        .unwrap_err();
    assert_eq!(refused.status, Status::InvalidArg);
}

#[test]
fn only_keeps_what_a_selector_selects_and_a_parana_answers_to_its_fast() {
    let sdk = sdk();
    let mut options = FeedOptions::new(STAMP);
    options.only = vec![FeedSelector {
        forms: vec![FeedForm::Ekadashi, FeedForm::Parana],
        ..FeedSelector::default()
    }];
    let text = unfolded(&feed(&sdk, &options));
    let plain = unfolded(&feed(&sdk, &FeedOptions::new(STAMP)));
    let fewer = lines(&text, "UID").len();
    assert!(fewer > 0 && fewer < lines(&plain, "UID").len());
    assert!(lines(&text, "SUMMARY").iter().any(|s| s.contains("Parana")));
    assert!(!text.contains("SUMMARY:Vijaya Dashami\r\n"));
    // Two lists must both hold the event: a parana of a rule nothing names
    // falls, the rule's own event stays.
    options.only = vec![FeedSelector {
        rules: vec!["VIJAYA_DASHAMI".to_owned()],
        ..FeedSelector::default()
    }];
    let one = unfolded(&feed(&sdk, &options));
    assert_eq!(lines(&one, "SUMMARY"), ["Vijaya Dashami"]);
    options.only[0].forms = vec![FeedForm::Parana];
    assert_eq!(lines(&unfolded(&feed(&sdk, &options)), "UID"), [""; 0]);
}

#[test]
fn the_first_hint_that_selects_speaks_and_the_feed_alarm_fills_in() {
    let sdk = sdk();
    let mut options = FeedOptions::new(STAMP);
    options.categories = vec!["Festivals".to_owned()];
    options.color = Some(Color::named("gold").unwrap());
    options.alarm = Some(Alarm {
        trigger_minutes: 540,
        description: None,
    });
    options.hints = vec![
        FeedHint {
            select: FeedSelector {
                forms: vec![FeedForm::Ekadashi],
                ..FeedSelector::default()
            },
            hints: Hints {
                categories: vec!["Fast".to_owned()],
                color: Some(Color::named("teal").unwrap()),
                alarm: Some(Alarm {
                    trigger_minutes: -720,
                    description: Some("Fast tomorrow".to_owned()),
                }),
            },
        },
        FeedHint {
            select: FeedSelector::default(),
            hints: Hints {
                categories: vec!["Other".to_owned()],
                ..Hints::default()
            },
        },
    ];
    let text = unfolded(&feed(&sdk, &options));
    assert!(text.contains("COLOR:gold\r\nX-APPLE-CALENDAR-COLOR:#FFD700\r\n"));
    let events = lines(&text, "UID").len();
    assert_eq!(lines(&text, "BEGIN").len(), 1 + 2 * events);
    let fasts = text.matches("Fast\r\nCOLOR:teal\r\n").count();
    assert!(fasts > 0);
    assert_eq!(text.matches("TRIGGER:-PT12H\r\n").count(), fasts);
    assert_eq!(text.matches("TRIGGER:PT9H\r\n").count(), events - fasts);
    assert_eq!(text.matches("Other\r\n").count(), events - fasts);
}

#[test]
fn a_selector_on_a_field_nothing_carries_is_refused_not_answered_empty() {
    let sdk = sdk();
    let asked =
        |festivals: &FestivalRequest, select: FeedSelector, extra: Option<SuppliedEvent>| {
            let mut options = FeedOptions::new(STAMP);
            options.only = vec![select];
            options.events.extend(extra);
            sdk.almanac().festival_calendar(
                &day(9, 20),
                &day(11, 10),
                &kathmandu(),
                UtcOffset::literal(5, 45, 0),
                festivals,
                &options,
            )
        };
    let traditions = FeedSelector {
        traditions: vec!["VAISHNAVA".to_owned()],
        ..FeedSelector::default()
    };
    let kinds = FeedSelector {
        kinds: vec!["HOLIDAY".to_owned()],
        ..FeedSelector::default()
    };
    // Rules that carry neither field are refused, not answered empty.
    let bare = FestivalRequest::of_kind(teistro::NextKind::Purnima.rules());
    let refused = asked(&bare, traditions.clone(), None).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported);
    assert_eq!(refused.field(), Some("traditions"));
    let refused = asked(&bare, kinds.clone(), None).unwrap_err();
    assert_eq!(refused.field(), Some("kinds"));
    // One event carrying the field is enough: the filter then reads it.
    let mut held = SuppliedEvent::new("Mine", When::Day(gregorian_fixed(10, 1)));
    held.tradition = Some("VAISHNAVA".to_owned());
    held.kind = Some("HOLIDAY".to_owned());
    for select in [traditions, kinds] {
        let text = unfolded(&asked(&bare, select, Some(held.clone())).unwrap().value);
        assert_eq!(lines(&text, "SUMMARY"), ["Mine"]);
    }
    // A pack's rules carry their own: a filter keeps some, not all.
    let pack = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let all = unfolded(&asked(&pack, FeedSelector::default(), None).unwrap().value);
    let gazetted = FeedSelector {
        kinds: vec!["GAZETTED".to_owned()],
        ..FeedSelector::default()
    };
    let kept = unfolded(&asked(&pack, gazetted, None).unwrap().value);
    let (all, kept) = (lines(&all, "SUMMARY").len(), lines(&kept, "SUMMARY").len());
    assert!(0 < kept && kept < all, "{kept} of {all}");
}

#[test]
fn the_record_reads_hints_and_events_and_names_a_refusal_by_its_field() {
    let base = r#""first": {"year": 2026, "month": 1, "day": 1},
        "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
        "festivals": {"rules": "NEPAL"}, "stampUnixSeconds": 1767225600"#;
    let asked = FestivalCalendarRequest::from_json(&format!(
        r#"{{{base}, "color": "Navy", "categories": ["A"],
            "alarm": {{"triggerMinutes": -30}},
            "only": [{{"forms": ["OBSERVANCE", "SUPPLIED"]}}],
            "hints": [{{"select": {{"rules": ["X"]}}, "categories": ["B"], "color": "red", "alarm": {{"triggerMinutes": 5, "description": "d"}}}}],
            "events": [
              {{"uid": "a@x", "summary": "Day", "day": {{"year": 2081, "month": 1, "day": 1}}, "calendar": "BIKRAM_SAMBAT", "kind": "K", "tradition": "T", "categories": ["C"], "color": "blue", "alarm": {{"triggerMinutes": 0}}, "description": "d"}},
              {{"summary": "Span", "fromUnixSeconds": 10, "toUnixSeconds": 20}}
            ]}}"#
    ))
    .unwrap();
    assert_eq!(asked.feed.color.unwrap().name(), "navy");
    assert_eq!(asked.feed.alarm.as_ref().unwrap().trigger_minutes, -30);
    assert_eq!(asked.feed.events.len(), 2);
    assert_eq!(asked.feed.hints[0].hints.color.unwrap().name(), "red");
    assert!(matches!(asked.feed.events[0].when, When::Day(_)));
    let refused = |rest: &str| {
        FestivalCalendarRequest::from_json(&format!("{{{base}, {rest}}}"))
            .unwrap_err()
            .field()
            .unwrap()
            .to_owned()
    };
    assert_eq!(refused(r##""color": "#ff0000""##), "festivalCalendar.color");
    assert_eq!(
        refused(r#""hints": [{"color": "nope"}]"#),
        "festivalCalendar.hints[0].color"
    );
    assert_eq!(
        refused(r#""events": [{"summary": "x"}]"#),
        "festivalCalendar.events[0]"
    );
    assert_eq!(
        refused(
            r#""events": [{"summary": "x", "color": "nope", "day": {"year": 2026, "month": 1, "day": 1}}]"#
        ),
        "festivalCalendar.events[0].color"
    );
    assert_eq!(
        refused(r#""events": [{"summary": "x", "day": {"year": 2026, "month": 13, "day": 1}}]"#),
        "festivalCalendar.events[0].month"
    );
}
