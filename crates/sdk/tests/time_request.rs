//! A civil time, an instant, a scale and a date read from JSON (the agent
//! server's `time.*` and `calendar.convert`): each is the request its
//! area takes, and each refusal names the field written.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::catalogue::Calendar;
use teistro::{CalendarDate, CalendarRequest, CivilRequest, ResolveRequest, Scale, ScaleRequest};

#[test]
fn each_record_is_the_request_it_spells() {
    let resolve = ResolveRequest::from_json(
        r#"{"calendar": "calendar.BIKRAM_SAMBAT", "date": {"year": 2047, "month": 1, "day": 1},
            "zone": {"kind": "FIXED", "offset": 20700}}"#,
    )
    .unwrap();
    assert_eq!(
        resolve.civil.date,
        CalendarDate::defined(Calendar::BikramSambat, 2047, 1, 1)
    );
    assert_eq!(resolve.civil.time, None, "a time left out is not known");
    let civil = CivilRequest::from_json(
        r#"{"instant": 2451545.0, "zone": {"kind": "LOCAL_MEAN", "longitude": 85.324}}"#,
    )
    .unwrap();
    assert_eq!(civil.calendar, Calendar::Gregorian);
    let scale = ScaleRequest::from_json(r#"{"jd": 2451545.0, "from": "UT1", "to": "TT"}"#).unwrap();
    assert_eq!((scale.from, scale.to), (Scale::Ut1, Scale::Tt));
    let date = CalendarRequest::from_json(
        r#"{"date": {"year": 2026, "month": 10, "day": 10}, "into": "BIKRAM_SAMBAT"}"#,
    )
    .unwrap();
    assert_eq!(date.into, Calendar::BikramSambat);
}

#[test]
fn a_refusal_names_the_field_written() {
    let zone = r#""zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#;
    for (refused, field) in [
        (
            ResolveRequest::from_json(&format!(
                r#"{{"date": {{"year": 1990, "month": 4, "day": 5}}, "tme": {{}}, {zone}}}"#
            ))
            .unwrap_err(),
            "tme",
        ),
        (
            ResolveRequest::from_json(&format!(
                r#"{{"date": {{"year": 1990, "month": 4, "day": 5}},
                    "time": {{"hour": 25, "minute": 0, "second": 0}}, {zone}}}"#
            ))
            .unwrap_err(),
            "time",
        ),
        (
            ResolveRequest::from_json(r#"{"date": {"year": 1990, "month": 4, "day": 5}}"#)
                .unwrap_err(),
            "zone",
        ),
        (
            CivilRequest::from_json(&format!(r#"{{"instant": "noon", {zone}}}"#)).unwrap_err(),
            "instant",
        ),
        (
            ScaleRequest::from_json(r#"{"jd": 2451545.0, "from": "GPS", "to": "TT"}"#).unwrap_err(),
            "from",
        ),
        (
            CalendarRequest::from_json(
                r#"{"date": {"year": 2026, "month": 10, "day": 10}, "into": "MAYAN"}"#,
            )
            .unwrap_err(),
            "into",
        ),
    ] {
        assert_eq!(refused.field(), Some(field), "{refused}");
    }
}
