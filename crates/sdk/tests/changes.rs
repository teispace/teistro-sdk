//! A range's change points against the days that hold them
//! (`03-design/change-points.md`), and a range on a zone's clock
//! (`03-design/time-and-timezone.md` §3.4).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::float_cmp,
    clippy::indexing_slicing,
    missing_docs,
    reason = "a test fails by panicking and compares the values it was handed"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{
    AlmanacRequest, CalendarDate, ChangeKind, ChangesRequest, Context, DayClock, Ephemeris,
    FestivalPack, FestivalRequest, UtcOffset, ZoneSource, ZoneSpec,
};

fn context() -> Context {
    Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .expect("a context")
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn new_york() -> Place {
    Place::new(
        Latitude::literal(40.7128),
        Longitude::literal(-74.006),
        Altitude::literal(10.0),
    )
}

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

fn nepal() -> UtcOffset {
    UtcOffset::literal(5, 45, 0)
}

fn month() -> ChangesRequest {
    ChangesRequest {
        first: date(11, 1),
        last: date(11, 30),
        place: kathmandu(),
        clock: DayClock::Offset(nepal()),
        kinds: Vec::new(),
    }
}

#[test]
fn every_change_is_the_days_own_once_and_in_order() {
    let sdk = context();
    let asked = month();
    let changes = sdk.almanac().changes(&asked).unwrap().value;
    let days = sdk
        .almanac()
        .of(&asked.first, &asked.last, &asked.place, nepal())
        .unwrap()
        .value;
    assert!(
        changes
            .points
            .windows(2)
            .all(|pair| (pair[0].at.get(), pair[0].kind) <= (pair[1].at.get(), pair[1].kind))
    );
    assert_eq!(changes.window.from, days[0].window.from);
    assert_eq!(changes.window.to, days[days.len() - 1].window.to);

    // The same members in the same order as the days' own spans, end to end.
    let of = |kind| {
        changes
            .points
            .iter()
            .filter(move |point| point.kind == kind)
            .map(|point| (point.at, point.from.clone(), point.to.clone()))
            .collect::<Vec<_>>()
    };
    let chained = |spans: Vec<(&'static str, teistro_core::interval::Interval)>| {
        let mut out = Vec::new();
        let mut last: Option<(&str, teistro_core::interval::Interval)> = None;
        for span in spans {
            if let Some(before) = last.filter(|before| before.0 != span.0) {
                out.push((span.1.from, before.0.to_string(), span.0.to_string()));
            }
            last = Some(span);
        }
        out.retain(|point| {
            changes.window.from.get() <= point.0.get() && point.0.get() < changes.window.to.get()
        });
        out
    };
    macro_rules! limb {
        ($kind:expr, $field:ident) => {
            assert_eq!(
                of($kind),
                chained(
                    days.iter()
                        .flat_map(|day| &day.limbs.$field)
                        .map(|span| (span.member.key(), span.whole))
                        .collect()
                ),
                "{:?}",
                $kind
            );
        };
    }
    limb!(ChangeKind::Tithi, tithi);
    limb!(ChangeKind::Nakshatra, nakshatra);
    limb!(ChangeKind::Yoga, yoga);
    limb!(ChangeKind::Karana, karana);
    assert_eq!(
        of(ChangeKind::MoonSign),
        chained(
            days.iter()
                .flat_map(|day| &day.moon.signs)
                .map(|span| (span.member.key(), span.whole))
                .collect()
        )
    );

    // Every sankranti of the range, to the bit.
    let sankrantis: Vec<_> = days.iter().filter_map(|day| day.sun.sankranti).collect();
    assert_ne!(sankrantis.len(), 0);
    let suns: Vec<_> = of(ChangeKind::SunSign).into_iter().map(|p| p.0).collect();
    assert_eq!(suns, sankrantis);

    // A vara a day, at that day's sunrise, the first day's not a change.
    let varas: Vec<_> = of(ChangeKind::Vara).into_iter().map(|p| p.0).collect();
    let sunrises: Vec<_> = days[1..].iter().map(|day| day.day.sunrise).collect();
    assert_eq!(varas, sunrises);
}

#[test]
fn a_kind_asked_is_all_that_answers() {
    let sdk = context();
    let mut asked = month();
    let all = sdk.almanac().changes(&asked).unwrap();
    asked.kinds = vec![ChangeKind::Tithi, ChangeKind::SunSign];
    let some = sdk.almanac().changes(&asked).unwrap();
    assert!(
        some.value
            .points
            .iter()
            .all(|point| matches!(point.kind, ChangeKind::Tithi | ChangeKind::SunSign))
    );
    assert!(some.value.points.len() < all.value.points.len());
    assert_ne!(some.provenance.content_hash, all.provenance.content_hash);
}

#[test]
fn the_record_reads_one_clock() {
    let text = |clock: &str| {
        format!(
            r#"{{"first": {{"year": 2026, "month": 11, "day": 1}}, "latitudeDeg": 27.7172, "longitudeDeg": 85.324{clock}}}"#
        )
    };
    assert_eq!(
        ChangesRequest::from_json(&text("")).unwrap_err().field(),
        Some("changes.utcOffsetSeconds")
    );
    let both = r#", "utcOffsetSeconds": 20700, "zone": {"kind": "IANA", "zone": "Asia/Kathmandu"}"#;
    assert_eq!(
        ChangesRequest::from_json(&text(both)).unwrap_err().field(),
        Some("changes.zone")
    );
    let bad = r#", "utcOffsetSeconds": 999999"#;
    assert!(ChangesRequest::from_json(&text(bad)).is_err());
    let unknown = text(r#", "utcOffsetSeconds": 20700, "kind": "TITHI""#);
    assert!(ChangesRequest::from_json(&unknown).is_err());
}

#[test]
fn a_zone_reads_each_day_on_its_own_offset_across_a_change() {
    let sdk = context();
    let zone = ZoneSpec::iana("America/New_York");
    // New York's clocks go back at 02:00 on 1 November 2026. A day is on
    // the offset of its noon, so that day already reads standard time,
    // which is the clock its sunrise and sunset are printed on.
    let answer = sdk
        .almanac()
        .in_zone(
            &date(10, 31),
            &date(11, 2),
            &new_york(),
            &zone,
            &AlmanacRequest::new(),
        )
        .unwrap();
    let offsets: Vec<_> = answer
        .days
        .value
        .iter()
        .map(|day| day.clock.as_ref().unwrap().offset.seconds())
        .collect();
    assert_eq!(offsets, [-14_400, -18_000, -18_000]);
    let clock = answer.days.provenance.time.clock.as_ref().unwrap();
    assert_eq!(
        (clock.source, clock.offset.seconds()),
        (ZoneSource::Iana, -14_400)
    );
    assert_eq!(clock.abbreviation.as_deref(), Some("EDT"));
    assert_eq!(
        answer.days.provenance.time.tzdb_version,
        sdk.time().zone_database().version
    );

    // The same day asked by the offset it had is the same sky, and says
    // only that the caller stated its clock.
    let stated = sdk
        .almanac()
        .of(
            &date(10, 31),
            &date(10, 31),
            &new_york(),
            UtcOffset::literal(-4, 0, 0),
        )
        .unwrap();
    assert_eq!(
        stated.value[0].day.sunrise,
        answer.days.value[0].day.sunrise
    );
    assert_eq!(stated.value[0].clock, None);
    let manual = stated.provenance.time.clock.as_ref().unwrap();
    assert_eq!(
        (manual.source, manual.offset.seconds()),
        (ZoneSource::Manual, -14_400)
    );

    // A section reckons on one offset, so it is refused across the change
    // and read on a span with none.
    let festivals =
        AlmanacRequest::new().with_festivals(FestivalRequest::from(FestivalPack::Nepal));
    let refused = sdk
        .almanac()
        .in_zone(&date(10, 31), &date(11, 2), &new_york(), &zone, &festivals)
        .unwrap_err();
    assert_eq!(refused.field(), Some("zone"));
    let steady = sdk
        .almanac()
        .in_zone(
            &date(11, 1),
            &date(11, 3),
            &kathmandu(),
            &ZoneSpec::iana("Asia/Kathmandu"),
            &festivals,
        )
        .unwrap();
    assert!(steady.festivals.is_some());
}

#[test]
fn local_mean_time_says_so_and_a_fixed_zone_is_a_stated_clock() {
    let sdk = context();
    let asked = AlmanacRequest::new();
    let lmt = sdk
        .almanac()
        .in_zone(
            &date(11, 1),
            &date(11, 1),
            &kathmandu(),
            &ZoneSpec::local_mean(Longitude::literal(85.324)),
            &asked,
        )
        .unwrap();
    assert_eq!(
        lmt.days.value[0].clock.as_ref().unwrap().source,
        ZoneSource::LocalMean
    );
    let fixed = sdk
        .almanac()
        .in_zone(
            &date(11, 1),
            &date(11, 1),
            &kathmandu(),
            &ZoneSpec::fixed(nepal()),
            &asked,
        )
        .unwrap();
    assert_eq!(fixed.days.value[0].clock, None);
    let unknown = ZoneSpec::iana("Asia/Kathmandoo");
    let refused = sdk
        .almanac()
        .in_zone(&date(11, 1), &date(11, 1), &kathmandu(), &unknown, &asked);
    assert_eq!(refused.unwrap_err().field(), Some("zone"));
}

#[test]
fn a_change_point_asked_by_zone_is_the_same_instant() {
    let sdk = context();
    let mut asked = month();
    let stated = sdk.almanac().changes(&asked).unwrap();
    asked.clock = DayClock::Zone(ZoneSpec::iana("Asia/Kathmandu"));
    let zoned = sdk.almanac().changes(&asked).unwrap();
    assert_eq!(stated.value, zoned.value);
    assert_eq!(
        zoned.provenance.time.clock.unwrap().source,
        ZoneSource::Iana
    );
}
