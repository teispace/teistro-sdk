//! The ashauch at Kathmandu in January 2026 (`03-design/ashauch.md` §5.2,
//! K1–K10), counted in the place's own sunrises: every instant at least
//! half an hour from the boundary it is tested against. The night of
//! Sunday 11 to Monday 12 January runs from sunset 17:27 to sunrise 06:56
//! Nepal time; its thirds turn at 21:57 and 02:26, its middle is 00:12, and
//! its last quarter begins at 03:34.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index the JSON they read"
)]

use teistro::ashauch::{
    Ashauch, AshauchEvent, AshauchRequest, AshauchTable, Clause, Deceased, Nearness, NightRule,
    Overlapped, Relation, Running,
};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{AshauchQuery, Context, Ephemeris, UtcOffset};

/// Nepal time, five hours forty-five minutes east.
const NPT: f64 = (5.0 + 45.0 / 60.0) / 24.0;

/// The UTC Julian day of a Nepal-time instant in 2026: `day` of `month`
/// (January or February) at `hour`:`minute`.
fn npt(month: u8, day: u8, hour: u8, minute: u8) -> JulianDay<Utc> {
    // 2026 January 1, 00:00 UTC.
    let first = 2_461_041.5;
    let date = f64::from(day - 1) + if month == 2 { 31.0 } else { 0.0 };
    let time = (f64::from(hour) + f64::from(minute) / 60.0) / 24.0;
    JulianDay::try_new(first + date + time - NPT).unwrap()
}

/// The Nepal-time clock reading of an instant, hours and minutes.
fn clock(at: JulianDay<Utc>) -> (u32, u32) {
    let minutes = ((at.get() + NPT + 0.5).fract() * 1440.0).round();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a minute of one day"
    )]
    let minutes = minutes as u32;
    (minutes / 60, minutes % 60)
}

fn context() -> Context {
    Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .settings_json(
            r#"{"day": {"sunrise": {"kind": "NAMED", "which": "UPPER_LIMB_REFRACTION"}}}"#,
        )
        .build()
        .unwrap()
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1337.0),
    )
}

fn death(at: JulianDay<Utc>) -> AshauchEvent {
    AshauchEvent::Death {
        at,
        cremated_at: None,
        ahitagni: false,
        deceased: Deceased::default(),
    }
}

fn request(key: &str, event: AshauchEvent) -> AshauchRequest {
    AshauchRequest {
        key: key.to_owned(),
        event,
        relation: Relation::Sapinda,
        heard_at: None,
        abroad: false,
        nearness: Nearness::SameVillage,
        full_days: None,
        during: Vec::new(),
    }
}

fn under(night: NightRule, asked: &AshauchRequest) -> Ashauch {
    let table = AshauchTable {
        night,
        ..AshauchTable::dharmasindhu()
    };
    context()
        .almanac()
        .ashauch(&table, asked, &kathmandu(), UtcOffset::literal(5, 45, 0))
        .unwrap()
        .value
}

fn answer(asked: &AshauchRequest) -> Ashauch {
    under(NightRule::Thirds, asked)
}

/// The first day and the last as (month, day).
fn span(found: &Ashauch) -> ((u8, u8), (u8, u8)) {
    let last = found.last_day.as_ref().unwrap();
    (
        (found.day_one.month, found.day_one.day),
        (last.month, last.day),
    )
}

/// An end at a January sunrise in Kathmandu, 06:5x.
fn at_sunrise(found: &Ashauch) {
    let (hour, minute) = clock(found.end);
    assert!(
        hour == 6 && (50..=59).contains(&minute),
        "{hour}:{minute:02} is not a Kathmandu sunrise in January"
    );
}

#[test]
fn k1_to_k3_a_night_death_takes_its_day_by_each_rule() {
    let k1 = request("k1", death(npt(1, 12, 1, 0)));
    let thirds = answer(&k1);
    assert_eq!(span(&thirds), ((1, 11), (1, 20)));
    at_sunrise(&thirds);
    assert_eq!(span(&under(NightRule::HalfNight, &k1)), ((1, 12), (1, 21)));
    assert_eq!(span(&under(NightRule::Sunrise, &k1)), ((1, 11), (1, 20)));
    assert_eq!(
        span(&under(NightRule::CivilMidnight, &k1)),
        ((1, 12), (1, 21))
    );
    let k2 = request("k2", death(npt(1, 12, 3, 30)));
    assert_eq!(span(&answer(&k2)), ((1, 12), (1, 21)));
    assert_eq!(span(&under(NightRule::Sunrise, &k2)), ((1, 11), (1, 20)));
    let k3 = request("k3", death(npt(1, 11, 22, 30)));
    for night in [NightRule::Thirds, NightRule::HalfNight, NightRule::Sunrise] {
        assert_eq!(span(&under(night, &k3)), ((1, 11), (1, 20)), "{night:?}");
    }
}

#[test]
fn k4_a_night_births_period_ends_at_its_naming_days_sunrise() {
    let birth = request(
        "k4",
        AshauchEvent::Birth {
            at: npt(1, 12, 1, 0),
            child: None,
        },
    );
    let found = answer(&birth);
    // The period's last day is the 20th, so the naming is on the 21st.
    assert_eq!(span(&found), ((1, 11), (1, 20)));
    assert_eq!(span(&under(NightRule::HalfNight, &birth)).1, (1, 21));
}

#[test]
fn k5_to_k8_a_second_death_during_the_first() {
    let first = answer(&request("first", death(npt(1, 11, 10, 0))));
    assert_eq!(span(&first), ((1, 11), (1, 20)));
    let second = |at| {
        let mut asked = request("second", death(at));
        asked.during = vec![Running::from(&first)];
        answer(&asked)
    };
    let k5 = second(npt(1, 15, 14, 0));
    assert_eq!(k5.end, first.end);
    assert_eq!(
        k5.overlap,
        Some(Overlapped::EndsWithEarlier {
            earlier: "first".to_owned()
        })
    );
    let k6 = second(npt(1, 20, 23, 0));
    assert_eq!(span(&k6).1, (1, 22));
    at_sunrise(&k6);
    assert!(matches!(
        k6.overlap,
        Some(Overlapped::LastNight { days: 2, .. })
    ));
    let k7 = second(npt(1, 21, 4, 15));
    assert_eq!(span(&k7).1, (1, 23));
    assert!(matches!(
        k7.overlap,
        Some(Overlapped::LastNight { days: 3, .. })
    ));
    let k8 = second(npt(1, 21, 8, 0));
    assert_eq!((span(&k8), k8.overlap), (((1, 21), (1, 30)), None));
}

#[test]
fn k9_k10_news_inside_the_period_and_after_it() {
    let died = death(npt(1, 1, 10, 0));
    let mut k9 = request("k9", died);
    k9.heard_at = Some(npt(1, 5, 12, 0));
    let found = answer(&k9);
    assert_eq!(found.clause, Clause::Remainder);
    assert_eq!(span(&found), ((1, 1), (1, 10)));
    at_sunrise(&found);
    let mut k10 = request("k10", died);
    k10.heard_at = Some(npt(2, 20, 12, 0));
    let found = answer(&k10);
    assert_eq!(
        found.clause,
        Clause::Late {
            within_days: 90,
            abroad: false
        }
    );
    assert_eq!(span(&found), ((2, 20), (2, 22)));
}

#[test]
fn a_binding_query_is_the_same_answer_and_names_a_refusal_under_its_record() {
    let asked = AshauchQuery::from_json(
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "altitudeM": 1337,
            "utcOffsetSeconds": 20700,
            "request": {"key": "first", "relation": "SAPINDA",
                        "event": {"event": "DEATH", "at": 2461051.4375}}}"#,
    )
    .unwrap();
    let sdk = context();
    let found = sdk.almanac().ashauch_query(&asked).unwrap();
    let kernel = answer(&request(
        "first",
        death(JulianDay::literal(2_461_051.437_5)),
    ));
    assert_eq!(found.value, kernel.in_full().unwrap());
    assert_eq!(found.value["dayOne"]["calendar"], "calendar.GREGORIAN");
    // Every member written in full is the serde spelling under its kind.
    let plain = serde_json::to_value(&kernel).unwrap();
    for (path, _) in Ashauch::MEMBERS {
        let at = format!("/{}", path.replace('.', "/"));
        if let (Some(full), Some(bare)) = (found.value.pointer(&at), plain.pointer(&at)) {
            assert_eq!(
                full.as_str().unwrap().split_once('.').unwrap().1,
                bare.as_str().unwrap(),
                "{path}"
            );
        }
    }
    let refused = AshauchQuery::from_json(
        r#"{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
            "request": {"key": "child", "relation": "SAPINDA",
                        "event": {"event": "DEATH", "at": 2461051.4375,
                                  "deceased": {"born": 2459000.5, "sex": "FEMALE"}}}}"#,
    )
    .unwrap();
    let error = sdk.almanac().ashauch_query(&refused).unwrap_err();
    assert_eq!(
        error.field(),
        Some("ashauch.request.event.deceased.married"),
        "{error}"
    );
}

#[test]
fn a_query_names_the_mitakshara_by_its_tradition_and_never_beside_a_table() {
    let at = |tradition: &str| {
        format!(
            r#"{{"latitudeDeg": 27.7172, "longitudeDeg": 85.324, "utcOffsetSeconds": 20700,
                {tradition}
                "request": {{"key": "k", "relation": "SAPINDA",
                            "event": {{"event": "DEATH", "at": 2461051.4375}}}}}}"#
        )
    };
    let named = AshauchQuery::from_json(&at(r#""tradition": "MITAKSHARA","#)).unwrap();
    assert_eq!(named.table, AshauchTable::mitakshara());
    let table = serde_json::to_string(&AshauchTable::dharmasindhu()).unwrap();
    let both = AshauchQuery::from_json(&at(&format!(
        r#""tradition": "MITAKSHARA", "table": {table},"#
    )))
    .unwrap_err();
    assert_eq!(both.field(), Some("ashauch.tradition"), "{both}");
    let found = context().almanac().ashauch_query(&named).unwrap().value;
    assert_eq!(found["table"], "MITAKSHARA");
}
