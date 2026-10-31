//! The calls the addendum's first wave added, across the boundary: a date
//! moved and a span counted (`ts_calendar_add`, `ts_calendar_between`), an
//! amount in words (`ts_intl_in_words`) and the next day of a rule
//! (`ts_almanac_next`), each with every refusal its contract names, held
//! to the status and the field it says it gives.

#![allow(
    unsafe_code,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests cross the boundary and fail by panicking"
)]

use core::ptr;
use std::ffi::CString;

use serde_json::Value;
use teistro_calendar::CalendarDate;
use teistro_core::Status;
use teistro_core::catalogue::Calendar;
use teistro_ffi::TS_CONTEXT_TEST_PROVIDER;
use teistro_ffi::calendar::{
    TsCalendarDate, TsDateSpan, TsOverflow, ts_calendar_add, ts_calendar_between,
};
use teistro_ffi::intl::{
    ts_intl_date, ts_intl_in_words, ts_intl_lexicon, ts_intl_lookup, ts_intl_time,
};
use teistro_ffi::panchanga::{ts_almanac_anniversaries, ts_almanac_next, ts_almanac_sentences};
use teistro_ffi::string::TsString;

mod common;

use common::{Ctx, owned, sized};

fn date(calendar: Calendar, year: i32, month: u8, day: u8) -> TsCalendarDate {
    sized(
        TsCalendarDate::of(&CalendarDate::defined(calendar, year, month, day)),
        |date, size| date.struct_size = size,
    )
}

fn blank_date() -> TsCalendarDate {
    date(Calendar::Gregorian, 1, 1, 1)
}

fn blank_span() -> TsDateSpan {
    sized(
        TsDateSpan {
            struct_size: 0,
            years: 0,
            months: 0,
            days: 0,
        },
        |span, size| span.struct_size = size,
    )
}

fn add(
    ctx: &Ctx,
    from: &TsCalendarDate,
    (years, months, days): (i32, i32, i32),
    overflow: u8,
) -> (Status, TsCalendarDate) {
    let mut out = blank_date();
    // SAFETY: a live context, a sized date and a sized slot.
    let status = unsafe {
        ts_calendar_add(
            ctx.handle,
            ptr::from_ref(from),
            years,
            months,
            days,
            overflow,
            &raw mut out,
        )
    };
    (status, out)
}

fn between(
    ctx: &Ctx,
    from: &TsCalendarDate,
    to: &TsCalendarDate,
    overflow: u8,
) -> (Status, TsDateSpan) {
    let mut out = blank_span();
    // SAFETY: a live context, two sized dates and a sized slot.
    let status = unsafe {
        ts_calendar_between(
            ctx.handle,
            ptr::from_ref(from),
            ptr::from_ref(to),
            overflow,
            &raw mut out,
        )
    };
    (status, out)
}

/// A JSON call's status, and its answer or `None` when refused.
fn json_call(
    ctx: &Ctx,
    call: unsafe extern "C" fn(
        *const teistro_ffi::context::TsContext,
        *const core::ffi::c_char,
        *mut TsString,
    ) -> Status,
    request: &Value,
) -> (Status, Option<Value>) {
    let text = CString::new(request.to_string()).unwrap();
    let mut out = TsString::empty();
    // SAFETY: a live context, a NUL-terminated request and a writable slot.
    let status = unsafe { call(ctx.handle, text.as_ptr(), &raw mut out) };
    let answer = (status == Status::Ok).then(|| serde_json::from_str(&owned(out)).unwrap());
    (status, answer)
}

/// Refused with `status`, on a field that holds `field`.
fn refused(ctx: &Ctx, got: &(Status, Option<Value>), status: Status, field: &str) {
    assert_eq!(got.0, status, "{:?}", ctx.last_error());
    let named = ctx.last_error().2.unwrap_or_default();
    assert!(named.contains(field), "{named} does not name {field}");
}

fn test_ctx() -> Ctx {
    Ctx::new(TS_CONTEXT_TEST_PROVIDER, None, None, None).unwrap()
}

/// **The three rules for a day the month lacks**: Jestha 2081 has 32 days
/// and Ashadh 31, so the 32nd plus a month is the 31st, or the 1st of
/// Shrawan carried, or refused on `day`.
#[test]
fn a_month_short_of_the_day_is_resolved_by_the_rule_asked() {
    let ctx = test_ctx();
    let jestha = date(Calendar::BikramSambat, 2081, 2, 32);
    let (status, kept) = add(&ctx, &jestha, (0, 1, 0), TsOverflow::Constrain as u8);
    assert_eq!(status, Status::Ok);
    assert_eq!((kept.year, kept.month, kept.day), (2081, 3, 31));
    let (status, carried) = add(&ctx, &jestha, (0, 1, 0), TsOverflow::Carry as u8);
    assert_eq!(status, Status::Ok);
    assert_eq!((carried.year, carried.month, carried.day), (2081, 4, 1));
    let (status, _) = add(&ctx, &jestha, (0, 1, 0), TsOverflow::Reject as u8);
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("day"));
}

/// **A span added back gives the end date**, as the contract says, and
/// a count backwards is negative or zero in each part.
#[test]
fn a_span_counts_as_an_age_and_adds_back() {
    let ctx = test_ctx();
    let born = date(Calendar::BikramSambat, 2050, 2, 32);
    let birthday = date(Calendar::BikramSambat, 2082, 2, 31);
    let (status, age) = between(&ctx, &born, &birthday, TsOverflow::Constrain as u8);
    assert_eq!(status, Status::Ok);
    assert_eq!((age.years, age.months, age.days), (32, 0, 0));
    let (status, back) = add(
        &ctx,
        &born,
        (age.years, age.months, age.days),
        TsOverflow::Constrain as u8,
    );
    assert_eq!(status, Status::Ok);
    assert_eq!((back.year, back.month, back.day), (2082, 2, 31));
    let (status, before) = between(&ctx, &birthday, &born, TsOverflow::Constrain as u8);
    assert_eq!(status, Status::Ok);
    assert!(before.years <= 0 && before.months <= 0 && before.days <= 0);
}

/// **Each refusal of the two date calls**: no such rule, a calendar with
/// no twelve months, dates in two calendars, and a null date.
#[test]
fn the_date_calls_refuse_what_their_contracts_name() {
    let ctx = test_ctx();
    let gregorian = date(Calendar::Gregorian, 2026, 1, 31);
    let (status, _) = add(&ctx, &gregorian, (0, 1, 0), 3);
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("overflow"));
    let (status, _) = between(&ctx, &gregorian, &gregorian, 3);
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("overflow"));

    let week = date(Calendar::IsoWeek, 2026, 1, 1);
    let (status, _) = add(&ctx, &week, (0, 1, 0), 0);
    assert_eq!(status, Status::Unsupported);

    let other = date(Calendar::BikramSambat, 2082, 1, 1);
    let (status, _) = between(&ctx, &gregorian, &other, 0);
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("calendar"));

    let mut out = blank_date();
    // SAFETY: a live context and a slot; the null date is what is tested.
    let status = unsafe { ts_calendar_add(ctx.handle, ptr::null(), 0, 0, 0, 0, &raw mut out) };
    assert_eq!(status, Status::InvalidArg);
    let mut span = blank_span();
    // SAFETY: as above, with the second date null.
    let status = unsafe {
        ts_calendar_between(
            ctx.handle,
            &raw const gregorian,
            ptr::null(),
            0,
            &raw mut span,
        )
    };
    assert_eq!(status, Status::InvalidArg);
    assert_eq!(ctx.last_error().2.as_deref(), Some("to"));
}

/// **A number and an amount in words**, and each refusal the contract
/// names.
#[test]
fn an_amount_is_spelt_and_a_bad_request_is_refused() {
    let ctx = Ctx::defaults();
    let (status, answer) = json_call(
        &ctx,
        ts_intl_in_words,
        &serde_json::json!({ "number": 1_500_000, "locale": "ne-Deva-NP" }),
    );
    assert_eq!(status, Status::Ok);
    let answer = answer.unwrap();
    assert_eq!(answer["text"], "पन्ध्र लाख");
    assert_eq!(answer["figure"], "१५,००,०००");

    let (status, amount) = json_call(
        &ctx,
        ts_intl_in_words,
        &serde_json::json!({ "locale": "ne-Deva-NP", "amount":
            { "currency": "NPR", "value": { "decimal": "1234.567" } } }),
    );
    assert_eq!(status, Status::Ok);
    assert!(amount.unwrap()["amount"].is_object());

    for (request, status) in [
        (serde_json::json!({}), Status::InvalidArg),
        (
            serde_json::json!({ "number": 1, "amount": { "currency": "NPR", "value": { "minor": 1 } } }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "number": 1, "unknown": 1 }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "amount": { "currency": "NPR", "value": { "decimal": "1.2.3" } } }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "number": 1, "locale": "sa-Deva-IN" }),
            Status::Unsupported,
        ),
        (
            serde_json::json!({ "locale": "ne-Deva-NP", "amount": { "currency": "XXZ", "value": { "minor": 1 } } }),
            Status::Unsupported,
        ),
    ] {
        refused(
            &ctx,
            &json_call(&ctx, ts_intl_in_words, &request),
            status,
            "",
        );
    }
}

/// **A time and a date as text and parts**, and each refusal.
#[test]
fn a_time_and_a_date_are_written_and_a_bad_request_is_refused() {
    let ctx = Ctx::defaults();
    let (status, answer) = json_call(
        &ctx,
        ts_intl_time,
        &serde_json::json!({ "at": { "hour": 6, "minute": 5 }, "hour12": true, "locale": "hi-Deva-IN" }),
    );
    assert_eq!(status, Status::Ok);
    let answer = answer.unwrap();
    assert_eq!(answer["text"], "सुबह ६:०५");
    assert_eq!(answer["parts"][0]["kind"], "DAY_PERIOD");
    assert_eq!(answer["isDraft"], true);

    let (status, day) = json_call(
        &ctx,
        ts_intl_date,
        &serde_json::json!({ "at": { "calendar": "calendar.GREGORIAN", "year": 2026, "month": 10, "day": 31 }, "style": "LONG", "locale": "en-Latn" }),
    );
    assert_eq!(status, Status::Ok);
    assert_eq!(day.unwrap()["text"], "31 October 2026");

    for (request, status) in [
        (serde_json::json!({}), Status::InvalidArg),
        (
            serde_json::json!({ "at": { "hour": 24, "minute": 0 } }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "at": { "hour": 1, "minute": 0 }, "locale": "xx" }),
            Status::Unsupported,
        ),
    ] {
        refused(&ctx, &json_call(&ctx, ts_intl_time, &request), status, "");
    }
    for (request, status) in [
        (serde_json::json!({}), Status::InvalidArg),
        (
            serde_json::json!({ "at": { "calendar": "calendar.GREGORIAN", "year": 2026, "month": 2, "day": 30 } }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "at": { "calendar": "calendar.TAMIL_SOLAR", "year": 2026, "month": 1, "day": 1 } }),
            Status::Unsupported,
        ),
    ] {
        refused(&ctx, &json_call(&ctx, ts_intl_date, &request), status, "");
    }
}

/// **A locale's names, and the names a text holds** (A2-3), and each
/// refusal.
#[test]
fn a_lexicon_is_listed_and_a_text_is_looked_up() {
    let ctx = Ctx::defaults();
    let (status, answer) = json_call(
        &ctx,
        ts_intl_lexicon,
        &serde_json::json!({ "locale": "hi", "kinds": ["graha"] }),
    );
    assert_eq!(status, Status::Ok);
    let answer = answer.unwrap();
    assert_eq!(answer["locale"], "hi-Deva-IN");
    assert_eq!(answer["entries"][0]["key"], "graha.SUN");
    assert_eq!(answer["entries"][0]["forms"][0]["text"], "सूर्य");

    let (status, found) = json_call(
        &ctx,
        ts_intl_lookup,
        &serde_json::json!({ "text": "चन्द्रमा रोहिणीमा", "kinds": ["nakshatra"] }),
    );
    assert_eq!(status, Status::Ok);
    let found = found.unwrap();
    assert_eq!(
        found["matches"][0]["candidates"][0]["key"],
        "nakshatra.ROHINI"
    );
    assert_eq!(found["matches"][0]["start"], 9);

    for (request, status) in [
        (
            serde_json::json!({ "kinds": ["planet"] }),
            Status::InvalidArg,
        ),
        (serde_json::json!({ "locale": "xx" }), Status::Unsupported),
        (serde_json::json!({ "locales": [] }), Status::InvalidArg),
    ] {
        refused(
            &ctx,
            &json_call(&ctx, ts_intl_lexicon, &request),
            status,
            "",
        );
    }
    for (request, status) in [
        (serde_json::json!({}), Status::InvalidArg),
        (
            serde_json::json!({ "text": "x", "kinds": ["planet"] }),
            Status::InvalidArg,
        ),
        (
            serde_json::json!({ "text": "x", "locale": "xx" }),
            Status::Unsupported,
        ),
    ] {
        refused(&ctx, &json_call(&ctx, ts_intl_lookup, &request), status, "");
    }
}

fn kathmandu() -> Value {
    serde_json::json!({ "after": 2_461_042.0, "latitudeDeg": 27.7172,
                        "longitudeDeg": 85.324, "utcOffsetSeconds": 20_700 })
}

fn asking(extra: &Value) -> Value {
    let mut request = kathmandu();
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    request
}

/// **The next day of a kind and of a rule** read on the test sky: each
/// is found, the first day searched is the one the instant falls in, and
/// a rule asks for the same day its pack answers.
#[test]
fn the_next_day_of_a_kind_or_a_rule_is_found() {
    let ctx = test_ctx();
    for kind in ["EKADASHI", "PURNIMA", "AMAVASYA", "SANKRANTI"] {
        let request = asking(&serde_json::json!({ "kind": kind, "limitDays": 400 }));
        let (status, answer) = json_call(&ctx, ts_almanac_next, &request);
        assert_eq!(status, Status::Ok, "{kind}: {:?}", ctx.last_error());
        let value = &answer.unwrap()["value"];
        assert_eq!(value["limitDays"], 400);
        assert!(value["searchedDays"].as_u64().unwrap() >= 1, "{value}");
        assert!(!value["keys"].as_array().unwrap().is_empty(), "{value}");
    }
}

/// **Each refusal of the next-occurrence request**: what it looks for,
/// said wrongly each way, and a limit out of range.
#[test]
fn a_next_request_is_refused_as_its_contract_says() {
    let ctx = test_ctx();
    let pack = serde_json::json!({ "rules": "NEPAL_PANCHANGA" });
    for (extra, status, field) in [
        (serde_json::json!({}), Status::InvalidArg, "kind"),
        (
            serde_json::json!({ "kind": "PURNIMA", "rule": "X" }),
            Status::InvalidArg,
            "rule",
        ),
        (
            serde_json::json!({ "kind": "PURNIMA", "festivals": pack }),
            Status::InvalidArg,
            "festivals",
        ),
        (
            serde_json::json!({ "rule": "X" }),
            Status::InvalidArg,
            "festivals",
        ),
        (
            serde_json::json!({ "kind": "PURNIMA", "limitDays": 0 }),
            Status::OutOfRange,
            "limitDays",
        ),
        (
            serde_json::json!({ "kind": "PURNIMA", "limitDays": 3661 }),
            Status::OutOfRange,
            "limitDays",
        ),
        (
            serde_json::json!({ "kind": "PURNIMA", "unknown": 1 }),
            Status::InvalidArg,
            "next",
        ),
    ] {
        let got = json_call(&ctx, ts_almanac_next, &asking(&extra));
        refused(&ctx, &got, status, field);
    }
    // A context with no ephemeris has no sky to search.
    let bare = Ctx::defaults();
    let request = asking(&serde_json::json!({ "kind": "PURNIMA" }));
    assert_eq!(
        json_call(&bare, ts_almanac_next, &request).0,
        Status::Capability
    );
}

/// **A day in sentences** across the boundary: a day answers in English
/// and Nepali, and each refusal its contract names has its status and field.
#[test]
fn a_day_is_said_across_the_boundary_and_refused_as_its_contract_says() {
    let ctx = test_ctx();
    let asked = |locale: &str| {
        serde_json::json!({
            "date": {"year": 2026, "month": 3, "day": 18},
            "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, "locale": locale,
        })
    };
    let (status, answer) = json_call(&ctx, ts_almanac_sentences, &asked("ne-Deva-NP"));
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    let value = &answer.unwrap()["value"];
    assert_eq!(value["draft"], true);
    assert_eq!(value["sentences"][0]["key"], "sdk.day.vara");

    let hindi = json_call(&ctx, ts_almanac_sentences, &asked("hi-Deva-IN"));
    refused(&ctx, &hindi, Status::Unsupported, "locale");
    let wrong = json_call(&ctx, ts_almanac_sentences, &serde_json::json!({"date": 1}));
    refused(&ctx, &wrong, Status::InvalidArg, "sentences");
}

fn anniversaries(extra: &Value) -> Value {
    let mut request = serde_json::json!({
        "fromYear": 2026, "years": 2, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
        "utcOffsetSeconds": 20_700, "anniversaries": [{
            "key": "BARKHI",
            "origin": { "by": "LUNAR", "month": "KARTIKA", "tithi": "SHUKLA_DASHAMI" },
        }],
    });
    for (key, value) in extra.as_object().unwrap() {
        request[key] = value.clone();
    }
    request
}

/// **Anniversaries over a span** on the test sky: the answer is the
/// festival answer, each day naming its rule, and each refusal its contract
/// names is named by its field.
#[test]
fn anniversaries_over_a_span_cross_the_boundary() {
    let ctx = test_ctx();
    let (status, answer) = json_call(
        &ctx,
        ts_almanac_anniversaries,
        &anniversaries(&serde_json::json!({})),
    );
    assert_eq!(status, Status::Ok, "{:?}", ctx.last_error());
    let answer = answer.unwrap();
    let days = answer["value"]["anniversaries"].as_array().unwrap();
    assert!(!days.is_empty(), "{answer}");
    assert_eq!(days[0]["key"], "BARKHI");
    assert!(days[0]["dayRule"]["clause"].is_string(), "{answer}");
    let origin = serde_json::json!({ "by": "LUNAR", "month": "KARTIKA", "tithi": "PURNIMA" });
    let many: Vec<Value> = (0..=teistro::MOST_ANNIVERSARIES)
        .map(|at| serde_json::json!({ "key": format!("K{at}"), "origin": origin }))
        .collect();
    for (extra, status, field) in [
        (
            serde_json::json!({ "years": 0 }),
            Status::OutOfRange,
            "anniversaries.years",
        ),
        (
            serde_json::json!({ "anniversaries": many }),
            Status::Limit,
            "anniversaries.anniversaries",
        ),
        (
            serde_json::json!({ "unknown": 1 }),
            Status::InvalidArg,
            "anniversaries",
        ),
    ] {
        let got = json_call(&ctx, ts_almanac_anniversaries, &anniversaries(&extra));
        refused(&ctx, &got, status, field);
    }
    let bare = Ctx::defaults();
    assert_eq!(
        json_call(
            &bare,
            ts_almanac_anniversaries,
            &anniversaries(&serde_json::json!({}))
        )
        .0,
        Status::Capability
    );
}
