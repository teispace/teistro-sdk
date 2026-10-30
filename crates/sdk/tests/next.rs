//! The next occurrence through the façade (`03-design/next-occurrence.md`):
//! each kind and a pack's rule read back against the festival reckoning
//! over the whole range, the instant's own day counted, the windows past
//! the first, and every refusal by its field.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, FixedDay, NextKind,
    NextRequest, NextTarget, Status, UtcOffset,
};

fn committee() -> Context {
    Context::builder()
        .profile("nepali-committee")
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// Kathmandu's clock.
fn npt() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

/// The UTC Julian day at local noon of a Gregorian date at Kathmandu.
fn noon(year: i32, month: u8, day: u8) -> f64 {
    let fixed = fixed_of(&gregorian(year, month, day));
    #[allow(clippy::cast_precision_loss, reason = "days are far below 2^53")]
    let midnight = fixed.get() as f64 + 1_721_424.5;
    midnight + 0.5 - 20_700.0 / 86_400.0
}

fn asked(after: f64, target: &str) -> NextRequest {
    NextRequest::from_json(&format!(
        r#"{{"after": {after}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, {target}}}"#
    ))
    .unwrap()
}

fn gregorian(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A context that computes nothing, for the calendars alone.
fn calendars() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap()
}

fn fixed_of(date: &CalendarDate) -> FixedDay {
    calendars().calendar().fixed_of(date).unwrap()
}

fn date_of(fixed: FixedDay) -> CalendarDate {
    calendars()
        .calendar()
        .date_of(Calendar::Gregorian, fixed)
        .unwrap()
}

/// The earliest day a reckoning over the whole range gives `keys`.
fn first_in_range(
    sdk: &Context,
    from: &CalendarDate,
    to: &CalendarDate,
    request: &FestivalRequest,
    keys: &[&str],
) -> CalendarDate {
    let found = sdk
        .almanac()
        .festivals(from, to, &kathmandu(), npt(), request)
        .unwrap()
        .value;
    let mut days: Vec<CalendarDate> = found
        .observances
        .iter()
        .filter(|o| keys.contains(&o.rule.as_str()))
        .map(|o| o.day.clone())
        .chain(
            found
                .ekadashis
                .iter()
                .filter(|f| keys.contains(&f.rule.as_str()))
                .map(|f| f.day.clone()),
        )
        .chain(
            found
                .dated
                .iter()
                .filter(|d| keys.contains(&d.rule.as_str()))
                .map(|d| d.day.clone()),
        )
        .collect();
    days.sort_by_key(fixed_of);
    days.remove(0)
}

#[test]
fn a_sankranti_is_the_first_of_the_next_bikram_sambat_month() {
    let sdk = committee();
    let next = sdk
        .almanac()
        .next(&asked(noon(2026, 1, 1), r#""kind": "SANKRANTI""#))
        .unwrap()
        .value;
    // 1 Magh 2082 is 15 January 2026: Maghe sankranti.
    assert_eq!(next.keys, ["MAKARA_SANKRANTI"]);
    let magh = sdk
        .calendar()
        .convert(
            &CalendarDate::defined(Calendar::BikramSambat, 2082, 10, 1),
            Calendar::Gregorian,
        )
        .unwrap();
    assert_eq!(
        (next.day.year, next.day.month, next.day.day),
        (magh.year, magh.month, magh.day)
    );
    assert_eq!((next.day.month, next.day.day), (1, 15));
    assert_eq!(next.festivals.dated.len(), 1);
    assert_eq!(next.searched_days, 32);
    assert_eq!(next.limit_days, 400);
}

#[test]
fn each_kind_is_the_reckonings_first_day_over_the_range() {
    let sdk = committee();
    let from = gregorian(2026, 3, 1);
    let to = gregorian(2026, 4, 30);
    for kind in NextKind::ALL {
        let next = sdk
            .almanac()
            .next(&asked(
                noon(2026, 3, 1),
                &format!(r#""kind": "{}""#, kind.key()),
            ))
            .unwrap()
            .value;
        let rules = kind.rules();
        let keys: Vec<&str> = rules
            .rules
            .iter()
            .map(|r| r.key.as_str())
            .chain(rules.ekadashis.iter().map(|r| r.key.as_str()))
            .chain(rules.dated.iter().map(|r| r.key.as_str()))
            .collect();
        let whole = first_in_range(
            &sdk,
            &from,
            &to,
            &FestivalRequest::of_kind(kind.rules()),
            &keys,
        );
        assert_eq!(next.day, whole, "{kind:?}");
        assert_ne!(next.keys.len(), 0);
        assert_eq!(fixed_of(&next.from), fixed_of(&from));
    }
}

#[test]
fn a_packs_festival_a_season_away_is_found_past_the_first_windows() {
    let sdk = committee();
    let pack = FestivalRequest::from(FestivalPack::NepalPanchanga);
    let next = sdk
        .almanac()
        .next(&asked(
            noon(2026, 1, 1),
            r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap();
    let whole = first_in_range(
        &sdk,
        &gregorian(2026, 1, 1),
        &gregorian(2026, 12, 31),
        &pack,
        &["VIJAYA_DASHAMI"],
    );
    assert_eq!(next.value.day, whole);
    assert_eq!(next.value.keys, ["VIJAYA_DASHAMI"]);
    assert_eq!(next.value.festivals.observances.len(), 1);
    // Past 32, 64 and 128 days: the fourth window held it.
    assert_eq!(next.value.searched_days, 400);
    let said = next
        .provenance
        .applied_conventions
        .iter()
        .find(|c| c.knob == "festival.next")
        .unwrap();
    assert!(said.value.ends_with("+400d"), "{}", said.value);
    // The same request answers the same, sealed alike.
    let again = sdk
        .almanac()
        .next(&asked(
            noon(2026, 1, 1),
            r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap();
    assert_eq!(again.provenance.content_hash, next.provenance.content_hash);
    assert_eq!(again.provenance.input_hash, next.provenance.input_hash);
    let written = sdk
        .almanac()
        .next_in_full(&asked(
            noon(2026, 1, 1),
            r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}"#,
        ))
        .unwrap()
        .value;
    assert_eq!(written["day"]["calendar"], "calendar.GREGORIAN");
    assert_eq!(
        written["festivals"]["observances"][0]["month"],
        "masa.ASHWINA"
    );
}

#[test]
fn the_instants_own_day_counts_and_the_next_one_is_after_it() {
    let sdk = committee();
    let first = sdk
        .almanac()
        .next(&asked(noon(2026, 3, 1), r#""kind": "PURNIMA""#))
        .unwrap()
        .value;
    let day = fixed_of(&first.day);
    // Asked at noon of that day, the day itself; the day after, a month on.
    let (y, m, d) = (first.day.year, first.day.month, first.day.day);
    let same = sdk
        .almanac()
        .next(&asked(noon(y, m, d), r#""kind": "PURNIMA""#))
        .unwrap()
        .value;
    assert_eq!(same.day, first.day);
    let after = date_of(day.plus_days(1));
    let later = sdk
        .almanac()
        .next(&asked(
            noon(after.year, after.month, after.day),
            r#""kind": "PURNIMA""#,
        ))
        .unwrap()
        .value;
    let gap = day.days_until(fixed_of(&later.day));
    assert!((28..=31).contains(&gap), "{gap}");
    // In Bikram Sambat, the days are written so.
    let bs = sdk
        .almanac()
        .next(&NextRequest {
            calendar: Calendar::BikramSambat,
            ..asked(noon(2026, 3, 1), r#""kind": "PURNIMA""#)
        })
        .unwrap()
        .value;
    assert_eq!(bs.day.calendar, Calendar::BikramSambat);
    assert_eq!(
        sdk.calendar()
            .convert(&bs.day, Calendar::Gregorian)
            .unwrap()
            .day,
        first.day.day
    );
}

#[test]
fn nothing_in_the_limit_and_every_bad_request_is_refused_by_its_field() {
    let sdk = committee();
    let short = asked(
        noon(2026, 1, 1),
        r#""rule": "VIJAYA_DASHAMI", "festivals": {"rules": "NEPAL_PANCHANGA"}, "limitDays": 20"#,
    );
    let refused = sdk.almanac().next(&short).unwrap_err();
    assert_eq!(
        (refused.status, refused.field()),
        (Status::Limit, Some("limitDays"))
    );
    let missing = asked(
        noon(2026, 1, 1),
        r#""rule": "NO_SUCH_DAY", "festivals": {"rules": "NEPAL"}"#,
    );
    assert_eq!(
        sdk.almanac().next(&missing).unwrap_err().field(),
        Some("rule")
    );
    // A rule given without its festivals, built by hand.
    let bare = NextRequest {
        target: NextTarget::Rule("HOLI".to_owned()),
        festivals: None,
        ..asked(noon(2026, 1, 1), r#""kind": "PURNIMA""#)
    };
    assert_eq!(
        sdk.almanac().next(&bare).unwrap_err().field(),
        Some("festivals")
    );
    let common = r#""after": 2461042.0, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700"#;
    for (body, field, status) in [
        (
            r#""kind": "PURNIMA", "rule": "HOLI""#,
            "next.rule",
            Status::InvalidArg,
        ),
        (
            r#""kind": "PURNIMA", "festivals": {"rules": "NEPAL"}"#,
            "next.festivals",
            Status::InvalidArg,
        ),
        (r#""rule": "HOLI""#, "next.festivals", Status::InvalidArg),
        ("\"limitDays\": 30", "next.kind", Status::InvalidArg),
        (
            r#""kind": "PURNIMA", "limitDays": 0"#,
            "next.limitDays",
            Status::OutOfRange,
        ),
        (
            r#""kind": "PURNIMA", "limitDays": 3661"#,
            "next.limitDays",
            Status::OutOfRange,
        ),
        (
            r#""rule": "HOLI", "festivals": {"rules": "NEPL"}"#,
            "next.festivals.rules",
            Status::InvalidArg,
        ),
    ] {
        let refused = NextRequest::from_json(&format!("{{{common}, {body}}}")).unwrap_err();
        assert_eq!(
            (refused.field(), refused.status),
            (Some(field), status),
            "{body}"
        );
    }
    let far = NextRequest::from_json(
        r#"{"after": 2461042.0, "latitudeDeg": 95, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "kind": "PURNIMA"}"#,
    )
    .unwrap_err();
    assert_eq!(far.field(), Some("next.latitudeDeg"));
    // A kind's request reads as a festival request does.
    assert_eq!(
        asked(noon(2026, 1, 1), r#""kind": "AMAVASYA""#).target,
        NextTarget::Kind(NextKind::Amavasya)
    );
}
