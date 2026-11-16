//! Festival groups through the façade (`03-design/festival-groups.md`,
//! SR-125): BS 2083's Tihar with its gap day, Dashain in a year that
//! doubles a tithi and one that loses one, a range that cuts a span, a
//! consumer's own grouping, the meta filter, the next span, and every
//! refusal by its field.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::festival::GroupSpan;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, NextRequest, UtcOffset,
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

fn day(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// The spans a request's reckoning gives over a range at Kathmandu.
fn spans(from: CalendarDate, to: CalendarDate, request: &FestivalRequest) -> Vec<GroupSpan> {
    committee()
        .almanac()
        .festivals(&from, &to, &kathmandu(), UtcOffset::literal(5, 45, 0), request)
        .unwrap()
        .value
        .groups
}

/// A span written as its days: `d:MEMBER+MEMBER`, `d:-` for a gap.
fn written(span: &GroupSpan) -> String {
    span.days
        .iter()
        .map(|d| {
            let on = if d.gap {
                "-".to_owned()
            } else {
                d.members.join("+")
            };
            format!("{}:{on}", d.day.day)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// A date as its year, month and day.
fn ymd(date: &CalendarDate) -> (i32, u8, u8) {
    (date.year, date.month, date.day)
}

fn nepal() -> FestivalRequest {
    FestivalRequest::from(FestivalPack::NepalPanchanga)
}

#[test]
fn every_pack_checks_with_its_groups_each_member_a_rule_of_the_pack() {
    for pack in FestivalPack::ALL {
        let asked = FestivalRequest::from(pack);
        asked.check().unwrap();
        assert_eq!(asked.groups(), pack.groups().as_slice(), "{}", pack.key());
    }
}

#[test]
fn bs_2083_tihar_holds_kartik_23_as_a_gap_however_short_the_range() {
    // Kartik 21 to 25, 2083 are 7 to 11 November 2026. The new moon holds
    // the 8th's evening and the 9th's sunrise, so Lakshmi puja falls on
    // the 8th with the dogs' day, and no member falls on the 9th.
    let whole = "7:KAG_TIHAR 8:KUKUR_TIHAR+LAKSHMI_PUJA 9:- 10:GOVARDHAN_PUJA+MHA_PUJA 11:YAMA_DWITIYA";
    for (from, to) in [
        (day(2026, 1, 1), day(2026, 12, 31)),
        (day(2026, 11, 9), day(2026, 11, 9)),
        (day(2026, 11, 11), day(2026, 11, 20)),
        (day(2026, 10, 30), day(2026, 11, 7)),
    ] {
        let found = spans(from.clone(), to.clone(), &nepal());
        let tihar: Vec<&GroupSpan> = found.iter().filter(|s| s.group == "TIHAR").collect();
        assert_eq!(tihar.len(), 1, "{from}..{to}");
        assert_eq!(written(tihar[0]), whole, "{from}..{to}");
        assert_eq!((tihar[0].first.day, tihar[0].last.day), (7, 11));
        assert_eq!(tihar[0].main_day().map(ymd), Some((2026, 11, 8)));
        assert!(tihar[0].is_whole());
        let gap = &tihar[0].days[2];
        assert!(gap.gap && gap.members.is_empty() && ymd(&gap.day) == (2026, 11, 9));
    }
}

#[test]
fn dashain_spans_a_doubled_tithi_and_a_lost_one() {
    // 2026: the 8th holds two sunrises, so 19 October lies between the
    // Mahashtami and the Mahanavami, a gap of the span.
    let doubled = spans(day(2026, 10, 20), day(2026, 10, 30), &nepal());
    let dashain = doubled.iter().find(|s| s.group == "DASHAIN").unwrap();
    assert_eq!((ymd(&dashain.first), ymd(&dashain.last)), ((2026, 10, 11), (2026, 10, 25)));
    assert_eq!(dashain.days.len(), 15);
    let around: Vec<String> = dashain.days[7..10]
        .iter()
        .map(|d| format!("{}:{}", d.day.day, d.members.join("+")))
        .collect();
    assert_eq!(around, ["18:MAHA_ASHTAMI", "19:", "20:MAHA_NAVAMI"]);
    assert!(dashain.days[8].gap);
    // 2024: the 8th holds no sunrise, so the Mahashtami and the Mahanavami
    // share 11 October and the span is a day shorter.
    let lost = spans(day(2024, 10, 1), day(2024, 10, 31), &nepal());
    let dashain = lost.iter().find(|s| s.group == "DASHAIN").unwrap();
    assert_eq!((ymd(&dashain.first), ymd(&dashain.last)), ((2024, 10, 3), (2024, 10, 16)));
    let eleventh = dashain.days.iter().find(|d| ymd(&d.day) == (2024, 10, 11)).unwrap();
    assert_eq!(eleventh.members, ["MAHA_ASHTAMI", "MAHA_NAVAMI"]);
    assert_eq!(dashain.days.iter().filter(|d| d.gap).count(), 14 - 5);
}

#[test]
fn a_request_regroups_by_key_and_a_filter_keeps_a_group_by_its_members() {
    // Dashain to the Tika alone, as the consumer's own grouping.
    let text = r#"{"rules": "NEPAL_PANCHANGA", "groups": [{"key": "DASHAIN",
        "members": ["NAVARATRA_ARAMBHA", "VIJAYA_DASHAMI"], "main": "VIJAYA_DASHAMI",
        "source": "the consumer's grouping"}]}"#;
    let asked = FestivalRequest::from_json(text).unwrap();
    assert_eq!(asked.groups().len(), 2);
    let found = spans(day(2026, 10, 1), day(2026, 11, 30), &asked);
    let dashain = found.iter().find(|s| s.group == "DASHAIN").unwrap();
    assert_eq!(ymd(&dashain.last), (2026, 10, 21));
    assert_eq!(found.iter().map(|s| s.group.as_str()).collect::<Vec<_>>(), ["DASHAIN", "TIHAR"]);
    // The Newar filter keeps Tihar, whose Mha puja is the Newars'.
    let newar = r#"{"rules": "NEPAL_PANCHANGA", "only": {"communities": ["NEWAR"]}}"#;
    let found = spans(day(2026, 10, 1), day(2026, 11, 30), &FestivalRequest::from_json(newar).unwrap());
    assert_eq!(found.iter().map(|s| s.group.as_str()).collect::<Vec<_>>(), ["TIHAR"]);
    // A pack without groups answers none.
    let plain = spans(day(2026, 10, 1), day(2026, 11, 30), &FestivalRequest::from(FestivalPack::Dharmasindhu));
    assert!(plain.is_empty());
}

#[test]
fn a_group_is_refused_by_its_field() {
    for (group, field) in [
        (
            r#"{"key": "X", "members": ["VIJAYA_DASHAMI", "NO_SUCH_RULE"], "main": "VIJAYA_DASHAMI", "source": "s"}"#,
            "festivals.groups[0].members[1]",
        ),
        (
            r#"{"key": "X", "members": ["VIJAYA_DASHAMI"], "main": "LAKSHMI_PUJA", "source": "s"}"#,
            "festivals.groups[0].main",
        ),
        (
            r#"{"key": "X", "members": [], "main": "LAKSHMI_PUJA", "source": "s"}"#,
            "festivals.groups[0].members",
        ),
        (
            r#"{"key": "X", "members": ["VIJAYA_DASHAMI", "VIJAYA_DASHAMI"], "main": "VIJAYA_DASHAMI", "source": "s"}"#,
            "festivals.groups[0].members[1]",
        ),
        (r#"{"key": "X", "members": ["VIJAYA_DASHAMI"]}"#, "festivals.groups[0]"),
    ] {
        let text = format!(r#"{{"rules": "NEPAL_PANCHANGA", "groups": [{group}]}}"#);
        let refused = FestivalRequest::from_json(&text).unwrap_err();
        assert!(
            refused.field().is_some_and(|at| at.starts_with(field)),
            "{group}: {:?}",
            refused.field()
        );
    }
}

#[test]
fn bihar_keeps_chhath_from_kharna_to_the_morning_arghya() {
    let bihar = FestivalRequest::from(FestivalPack::IndiaBihar);
    let found = spans(day(2026, 11, 1), day(2026, 11, 30), &bihar);
    let chhath = found.iter().find(|s| s.group == "CHHATH").unwrap();
    assert_eq!(written(chhath), "14:CHHATH_KHARNA 15:CHHATH 16:CHHATH_SECOND_DAY");
}

fn next(after: f64, group: &str) -> NextRequest {
    NextRequest::from_json(&format!(
        r#"{{"after": {after}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1400,
            "utcOffsetSeconds": 20700, "group": "{group}", "festivals": {{"rules": "NEPAL_PANCHANGA"}}}}"#
    ))
    .unwrap()
}

#[test]
fn the_next_span_of_a_group_counts_one_under_way() {
    let sdk = committee();
    // Noon of 1 January 2026 at Kathmandu: the next Dashain.
    let found = sdk.almanac().next(&next(2_461_041.76, "DASHAIN")).unwrap().value;
    let span = found.group.clone().unwrap();
    assert_eq!((ymd(&found.day), ymd(&span.last)), ((2026, 10, 11), (2026, 10, 25)));
    assert_eq!(found.keys.len(), 6);
    assert_eq!(found.festivals.observances.len(), 6);
    // Noon of 9 November 2026, Tihar's gap day: Tihar under way.
    let under_way = sdk.almanac().next(&next(2_461_353.76, "TIHAR")).unwrap().value;
    assert_eq!(ymd(&under_way.day), (2026, 11, 7));
    assert_eq!(ymd(&under_way.group.unwrap().days[2].day), (2026, 11, 9));
    // A key no group of the request has.
    let refused = sdk.almanac().next(&next(2_461_041.76, "HOLI")).unwrap_err();
    assert_eq!(refused.field(), Some("group"));
    // A group and a rule both.
    let both = NextRequest::from_json(
        r#"{"after": 2461041.76, "latitudeDeg": 27.7172, "longitudeDeg": 85.324,
            "utcOffsetSeconds": 20700, "group": "TIHAR", "kind": "PURNIMA"}"#,
    )
    .unwrap_err();
    assert_eq!(both.field(), Some("next.group"));
}
