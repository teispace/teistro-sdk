//! `intl.time` and `intl.date`: the locale's pattern and parts of the day,
//! as text and typed parts (`docs/03-design/intl-engine-and-packs.md`).
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro_core::Status;
use teistro_intl::civil::{CivilPartKind, CivilText, DateTextRequest, TimeTextRequest};
use teistro_intl::source::Tree;
use teistro_intl::{Intl, sdk_root};

fn intl() -> Intl {
    Intl::from_tree(&Tree::load(&sdk_root()).expect("the sources")).expect("the engine")
}

fn time(intl: &Intl, json: &str) -> CivilText {
    intl.time(&TimeTextRequest::from_json(json).unwrap())
        .unwrap()
}

fn date(intl: &Intl, json: &str) -> CivilText {
    intl.date(&DateTextRequest::from_json(json).unwrap())
        .unwrap()
}

fn kinds(answer: &CivilText) -> Vec<CivilPartKind> {
    answer.parts.iter().map(|part| part.kind).collect()
}

fn joined(answer: &CivilText) -> String {
    answer.parts.iter().map(|part| part.text.as_str()).collect()
}

const LOCALES: [&str; 3] = ["en-Latn", "hi-Deva-IN", "ne-Deva-NP"];

#[test]
fn the_clock_each_locale_writes() {
    let intl = intl();
    let at = |hour: u8, locale: &str| {
        time(
            &intl,
            &format!(
                r#"{{"at": {{"hour": {hour}, "minute": 5}}, "hour12": true, "locale": "{locale}"}}"#
            ),
        )
        .text
    };
    // CLDR 48 `hi`: morning 04-12, afternoon 12-16, evening 16-20, night 20-04.
    assert_eq!(at(6, "hi-Deva-IN"), "सुबह ६:०५");
    assert_eq!(at(13, "hi-Deva-IN"), "दोपहर १:०५");
    assert_eq!(at(18, "hi-Deva-IN"), "शाम ६:०५");
    assert_eq!(at(21, "hi-Deva-IN"), "रात ९:०५");
    assert_eq!(at(0, "hi-Deva-IN"), "रात १२:०५");
    assert_eq!(at(6, "sa-Deva"), "प्रातः ६:०५");
    assert_eq!(at(18, "sa-Deva"), "सायम् ६:०५");
    assert_eq!(at(21, "sa-Deva"), "रात्रौ ९:०५");
    assert_eq!(at(6, "ne-Deva-NP"), "बिहान ६:०५");
    assert_eq!(at(18, "en-Latn"), "6:05 pm");
    // The twenty-four-hour clock is the default, and `LONG` adds seconds.
    let long = time(
        &intl,
        r#"{"at": {"hour": 6, "minute": 5, "second": 9}, "style": "LONG", "locale": "hi-Deva-IN"}"#,
    );
    assert_eq!(long.text, "०६:०५:०९");
    assert_eq!(
        kinds(&long),
        [
            CivilPartKind::Hour,
            CivilPartKind::Literal,
            CivilPartKind::Minute,
            CivilPartKind::Literal,
            CivilPartKind::Second,
        ]
    );
    let twelve = time(
        &intl,
        r#"{"at": {"hour": 14, "minute": 5, "second": 9}, "style": "LONG", "hour12": true}"#,
    );
    assert_eq!(twelve.text, "2:05:09 pm");
}

#[test]
fn a_part_is_found_wherever_the_locale_writes_it() {
    let intl = intl();
    let english = time(&intl, r#"{"at": {"hour": 6, "minute": 5}, "hour12": true}"#);
    assert_eq!(english.parts.last().unwrap().kind, CivilPartKind::DayPeriod);
    assert_eq!(english.parts.last().unwrap().text, "am");
    let hindi = time(
        &intl,
        r#"{"at": {"hour": 6, "minute": 5}, "hour12": true, "locale": "hi-Deva-IN"}"#,
    );
    assert_eq!(hindi.parts[0].kind, CivilPartKind::DayPeriod);
    assert_eq!(hindi.parts[0].text, "सुबह");
    assert_eq!(hindi.parts[2].text, "६");
    assert!(hindi.is_draft && !hindi.is_fallback);
    assert_eq!(
        (hindi.locale.as_str(), hindi.resolved_from.as_str()),
        ("hi-Deva-IN", "hi-Deva-IN")
    );
    assert!(!english.is_draft);
}

#[test]
fn parts_join_to_the_text_in_every_style_and_calendar() {
    let intl = intl();
    let calendars = [
        ("GREGORIAN", 2026, 10, 31),
        ("JULIAN", 2026, 10, 18),
        ("MIXED", 2026, 10, 31),
        ("ISO_WEEK", 2026, 44, 6),
        ("BIKRAM_SAMBAT", 2083, 7, 14),
        ("INDIAN_NATIONAL", 1948, 8, 9),
        ("HIJRI_TABULAR", 1448, 5, 20),
        ("TIBETAN_PHUGPA", 2153, 9, 20),
        ("BANGLADESHI", 1433, 7, 15),
    ];
    for locale in ["en-Latn", "hi-Deva-IN", "ne-Deva-NP", "sa-Deva"] {
        for (calendar, year, month, day) in calendars {
            for style in ["NUMERIC", "LONG", "FULL"] {
                let answer = date(
                    &intl,
                    &format!(
                        r#"{{"at": {{"calendar": "calendar.{calendar}", "year": {year}, "month": {month}, "day": {day}}}, "style": "{style}", "locale": "{locale}"}}"#
                    ),
                );
                assert_eq!(joined(&answer), answer.text);
                assert!(
                    answer.warnings.is_empty(),
                    "{locale} {calendar} {style}: {:?}",
                    answer.warnings
                );
                assert!(!answer.is_fallback, "{locale} {calendar} {style} fell back");
                assert!(
                    kinds(&answer).contains(&CivilPartKind::Year),
                    "{locale} {calendar}: {}",
                    answer.text
                );
                assert!(!answer.text.contains('{'), "{}", answer.text);
            }
        }
    }
}

#[test]
fn the_dates_read_as_cldr_orders_them() {
    let intl = intl();
    let day = r#""at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 10, "day": 31}"#;
    let hindi = date(
        &intl,
        &format!(r#"{{{day}, "style": "FULL", "locale": "hi-Deva-IN"}}"#),
    );
    assert_eq!(hindi.text, "शनिवार, ३१ अक्टूबर २०२६");
    assert_eq!(hindi.parts[0].kind, CivilPartKind::Weekday);
    assert!(hindi.is_draft);
    let numeric = date(&intl, &format!(r#"{{{day}, "locale": "hi-Deva-IN"}}"#));
    assert_eq!(numeric.text, "३१/१०/२०२६");
    let sanskrit = date(
        &intl,
        &format!(r#"{{{day}, "style": "FULL", "locale": "sa-Deva"}}"#),
    );
    assert_eq!(sanskrit.text, "शनिवासरः, ३१ अक्तूबरमासः २०२६");
    let bikram = date(
        &intl,
        r#"{"at": {"calendar": "calendar.BIKRAM_SAMBAT", "year": 2081, "month": 1, "day": 1}, "style": "LONG", "locale": "hi-Deva-IN"}"#,
    );
    assert_eq!(bikram.text, "१ वैशाख वि.सं. २०८१");
    assert_eq!(kinds(&bikram)[0], CivilPartKind::Day);
    assert!(kinds(&bikram).contains(&CivilPartKind::Month));
}

#[test]
fn a_duration_in_sanskrit_keeps_its_dual() {
    let mut intl = intl();
    intl.set_locale("sa-Deva").unwrap();
    let text = |n: i64| {
        let params = teistro_intl::params([("n", teistro_intl::Value::Int(n))]);
        intl.render("sdk.calendar.duration.day", &params).text
    };
    assert_eq!(text(1), "१ दिनम्");
    assert_eq!(text(2), "२ दिने");
    assert_eq!(text(3), "३ दिनानि");
}

#[test]
fn digits_can_be_asked_for() {
    let intl = intl();
    let latin = time(
        &intl,
        r#"{"at": {"hour": 6, "minute": 5}, "digits": "LATN", "locale": "ne-Deva-NP"}"#,
    );
    assert_eq!(latin.text, "06:05");
    let devanagari = time(
        &intl,
        r#"{"at": {"hour": 6, "minute": 5}, "digits": "DEVA"}"#,
    );
    assert_eq!(devanagari.text, "०६:०५");
    assert_eq!(devanagari.parts[0].text, "०६");
    // The numbers of a name stay in the digits asked for too.
    let month = date(
        &intl,
        r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 1, "day": 2}, "digits": "LATN", "locale": "hi-Deva-IN"}"#,
    );
    assert_eq!(month.text, "02/01/2026");
}

#[test]
fn the_engines_own_locale_stands_in_for_one_left_out() {
    let mut intl = intl();
    intl.set_locale("ne-Deva-NP").unwrap();
    assert_eq!(
        time(&intl, r#"{"at": {"hour": 1, "minute": 2}}"#).locale,
        "ne-Deva-NP"
    );
}

#[test]
fn a_locale_without_the_text_falls_back_and_says_so() {
    let intl = intl();
    for locale in LOCALES {
        assert!(
            !time(
                &intl,
                &format!(r#"{{"at": {{"hour": 1, "minute": 2}}, "locale": "{locale}"}}"#)
            )
            .is_fallback
        );
    }
    let derived = time(
        &intl,
        r#"{"at": {"hour": 1, "minute": 2}, "hour12": true, "locale": "sa-Latn"}"#,
    );
    assert!(derived.is_fallback);
    assert_eq!(derived.resolved_from, "en-Latn");
    assert_eq!(derived.locale, "sa-Latn");
}

#[test]
fn a_refusal_names_what_to_change() {
    let intl = intl();
    let refused = |json: &str| {
        intl.time(&TimeTextRequest::from_json(json).unwrap())
            .unwrap_err()
    };
    for (json, field) in [
        (r#"{"at": {"hour": 24, "minute": 0}}"#, "time.at.hour"),
        (r#"{"at": {"hour": 0, "minute": 60}}"#, "time.at.minute"),
        (
            r#"{"at": {"hour": 0, "minute": 0, "second": 61}}"#,
            "time.at.second",
        ),
    ] {
        let error = refused(json);
        assert_eq!(
            (error.status, error.field()),
            (Status::InvalidArg, Some(field))
        );
    }
    assert!(
        intl.time(
            &TimeTextRequest::from_json(r#"{"at": {"hour": 0, "minute": 0, "second": 60}}"#)
                .unwrap()
        )
        .is_ok()
    );
    let error = refused(r#"{"at": {"hour": 0, "minute": 0}, "locale": "xx"}"#);
    assert_eq!(
        (error.status, error.field()),
        (Status::Unsupported, Some("time.locale"))
    );

    let refused = |json: &str| {
        intl.date(&DateTextRequest::from_json(json).unwrap())
            .unwrap_err()
    };
    let no_day = refused(
        r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 2, "day": 30}}"#,
    );
    assert_eq!(no_day.status, Status::InvalidArg);
    assert!(
        no_day.field().is_some_and(|f| f.starts_with("date.at")),
        "{:?}",
        no_day.field()
    );
    let unshipped = refused(
        r#"{"at": {"calendar": "calendar.INDIAN_LUNISOLAR", "year": 2026, "month": 2, "day": 3}}"#,
    );
    assert_eq!(
        (unshipped.status, unshipped.field()),
        (Status::Unsupported, Some("date.at.calendar"))
    );
    let no_pattern = refused(
        r#"{"at": {"calendar": "calendar.TAMIL_SOLAR", "year": 2026, "month": 2, "day": 3}}"#,
    );
    assert_eq!(
        (no_pattern.status, no_pattern.field()),
        (Status::Unsupported, Some("date.at.calendar"))
    );
    let locale = refused(
        r#"{"at": {"calendar": "calendar.GREGORIAN", "year": 2026, "month": 2, "day": 3}, "locale": "xx"}"#,
    );
    assert_eq!(locale.field(), Some("date.locale"));
}

#[test]
fn a_datetime_in_full_shows_a_time() {
    let mut intl = intl();
    let at = teistro_intl::params([(
        "at",
        teistro_intl::Value::DateTime(
            teistro_calendar::CalendarDate::defined(
                teistro_core::catalogue::Calendar::Gregorian,
                2026,
                10,
                31,
            ),
            teistro_intl::ClockTime::new(6, 5, 9),
        ),
    )]);
    intl.set_locale("en-Latn").unwrap();
    let full = intl
        .render_source("{$at :datetime style=full}", &at)
        .unwrap();
    assert_eq!(full.text, "Saturday, 31 October 2026, 06:05:09");
    assert!(full.warnings.is_empty(), "{:?}", full.warnings);
}
