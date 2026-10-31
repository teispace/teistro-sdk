//! The day's markers through the façade (`03-design/day-markers.md`):
//! ganda mula, gandanta and vinchudo when asked, none otherwise, and
//! Muhurta Chintamani's yoga tables under `panchanga.muhurta_tables`, and
//! the Gowri panchangam (`03-design/gowri-panchangam.md`).
//!
//! Each gandanta is read back from a chart at its middle: the Moon there
//! must stand at a junction of a water and a fire sign, 0°, 120° or 240°
//! sidereal, so a span placed at the wrong boundary fails here.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Calendar, DayMarker, Graha, MuhurtaYoga, Nakshatra, Rashi, Vara};
use teistro::omen::GANDANTA_REACH_DAYS;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    AlmanacRequest, CalendarDate, ChartRequest, Context, DaysRequest, Ephemeris, UtcOffset,
};

/// The six stars at a junction of a water and a fire sign.
const GANDA_MULA: [Nakshatra; 6] = [
    Nakshatra::Ashwini,
    Nakshatra::Ashlesha,
    Nakshatra::Magha,
    Nakshatra::Jyeshtha,
    Nakshatra::Mula,
    Nakshatra::Revati,
];

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

fn context(patch: &str) -> Context {
    Context::builder()
        .profile("nepali-drik")
        .ephemeris([Ephemeris::Builtin])
        .settings_json(patch)
        .build()
        .unwrap()
}

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

#[test]
fn markers_follow_the_moons_stars_and_sign_and_gandanta_reads_back_at_a_junction() {
    let sdk = context("{}");
    let place = kathmandu();
    let offset = UtcOffset::literal(5, 45, 0);
    let answer = sdk
        .almanac()
        .asked(
            &date(1, 1),
            &date(3, 31),
            &place,
            offset,
            &AlmanacRequest::new().with_markers(),
        )
        .unwrap();
    let charts = ChartRequest::at(place, offset);
    let mut junctions = Vec::new();
    for day in &answer.days.value {
        let of = |kind| -> Vec<_> {
            day.markers
                .iter()
                .filter(|m| m.member == kind)
                .map(|m| m.whole)
                .collect()
        };
        let stars: Vec<_> = day
            .limbs
            .nakshatra
            .iter()
            .filter(|s| GANDA_MULA.contains(&s.member))
            .map(|s| s.whole)
            .collect();
        assert_eq!(of(DayMarker::GandaMula), stars, "{:?}", day.day.date);
        let scorpio: Vec<_> = day
            .moon
            .signs
            .iter()
            .filter(|s| s.member == Rashi::Scorpio)
            .map(|s| s.whole)
            .collect();
        assert_eq!(of(DayMarker::Vinchudo), scorpio, "{:?}", day.day.date);
        assert!(
            day.markers
                .windows(2)
                .all(|w| w[0].whole.from.get() <= w[1].whole.from.get()),
            "in the order they begin"
        );
        for gandanta in of(DayMarker::Gandanta) {
            assert!((gandanta.days() - 2.0 * GANDANTA_REACH_DAYS).abs() < 1e-9);
            let middle = gandanta.from.get().midpoint(gandanta.to.get());
            if junctions.last() == Some(&middle) {
                continue; // the same junction from the day before
            }
            junctions.push(middle);
            let chart = sdk
                .chart()
                .reading(JulianDay::<Utc>::literal(middle), &charts)
                .unwrap()
                .value;
            let moon = chart.foundation.graha(Graha::Moon).unwrap().longitude_deg;
            // A chart may read the Moon from another centre, about a
            // degree apart by parallax.
            let off = (moon.rem_euclid(120.0) + 60.0).rem_euclid(120.0) - 60.0;
            assert!(off.abs() < 1.2, "{:?}: the Moon at {moon}", day.day.date);
        }
    }
    // A junction every nine days or so: a quarter holds about ten.
    assert!((9..=11).contains(&junctions.len()), "{}", junctions.len());
}

#[test]
fn a_day_not_asked_carries_no_markers_and_hashes_as_before() {
    let sdk = context("{}");
    let place = kathmandu();
    let offset = UtcOffset::literal(5, 45, 0);
    let ask = |request: &AlmanacRequest| {
        sdk.almanac()
            .asked(&date(1, 1), &date(1, 31), &place, offset, request)
            .unwrap()
    };
    let (plain, asked) = (
        ask(&AlmanacRequest::new()),
        ask(&AlmanacRequest::new().with_markers()),
    );
    assert!(plain.days.value.iter().all(|day| day.markers.is_empty()));
    assert!(asked.days.value.iter().any(|day| !day.markers.is_empty()));
    let document = serde_json::to_value(&plain.days.value[0]).unwrap();
    assert!(document.get("markers").is_none());
    for (plain_day, asked_day) in plain.days.value.iter().zip(&asked.days.value) {
        let mut stripped = asked_day.clone();
        stripped.markers.clear();
        assert_eq!(&stripped, plain_day);
    }
    // The JSON request the bindings and the agent server send asks the same.
    let json = DaysRequest::from_json(
        r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 27.7172,
            "longitudeDeg": 85.324, "utcOffsetSeconds": 20700, "markers": true}"#,
    )
    .unwrap();
    assert!(json.beside.markers());
    assert!(DaysRequest::from_json(r#"{"first": {"year": 2026, "month": 1, "day": 1}, "latitudeDeg": 0, "longitudeDeg": 0, "utcOffsetSeconds": 0, "markers": 1}"#).is_err());
}

/// Under Muhurta Chintamani's tables a day's yogas read every star it
/// held; a Thursday in Pushya carries Guru Pushya over the star, and the
/// classical tables, read at sunrise, name no Pushya yoga.
#[test]
fn muhurta_chintamani_tables_give_guru_pushya_on_a_thursday_in_pushya() {
    let place = kathmandu();
    let offset = UtcOffset::literal(5, 45, 0);
    let days = |patch| {
        context(patch)
            .almanac()
            .asked(
                &date(1, 1),
                &date(6, 30),
                &place,
                offset,
                &AlmanacRequest::new(),
            )
            .unwrap()
            .days
            .value
    };
    let (classical, full) = (
        days("{}"),
        days(r#"{"panchanga": {"muhurta_tables": "MUHURTA_CHINTAMANI"}}"#),
    );
    let mut thursdays = 0;
    for (plain, day) in classical.iter().zip(&full) {
        let pushya = day
            .limbs
            .nakshatra
            .iter()
            .find(|s| s.member == Nakshatra::Pushya);
        let guru: Vec<_> = day
            .omens
            .yogas
            .iter()
            .filter(|y| y.yoga == MuhurtaYoga::GuruPushya)
            .collect();
        match (day.day.vara, pushya) {
            (Vara::Guruvara, Some(star)) => {
                thursdays += 1;
                assert_eq!(guru.len(), 1, "{:?}", day.day.date);
                assert_eq!(guru[0].at, star.inside);
            }
            _ => assert!(guru.is_empty(), "{:?}", day.day.date),
        }
        assert!(
            !plain
                .omens
                .yogas
                .iter()
                .any(|y| matches!(y.yoga, MuhurtaYoga::GuruPushya | MuhurtaYoga::RaviPushya)),
            "the classical tables name none"
        );
        // Every yoga lies inside the day.
        assert!(
            day.omens
                .yogas
                .iter()
                .all(|y| day.window.from <= y.at.from && y.at.to <= day.window.to)
        );
    }
    assert!(thursdays >= 1, "half a year holds a Guru Pushya");
}

/// The Gowri panchangam (`03-design/gowri-panchangam.md`): asked, each
/// day's sixteen parts are eighths of its daylight and night, Visham by
/// day is the day's own rahu kaala, and a day not asked carries none.
#[test]
fn gowri_divides_the_day_and_night_and_visham_is_rahu_kaala() {
    use teistro::catalogue::{Gowri, Kaala};
    let sdk = context("{}");
    let chennai = Place::new(
        Latitude::literal(13.0827),
        Longitude::literal(80.2707),
        Altitude::literal(6.0),
    );
    let offset = UtcOffset::literal(5, 30, 0);
    let ask = |request: &AlmanacRequest| {
        sdk.almanac()
            .asked(&date(11, 1), &date(11, 7), &chennai, offset, request)
            .unwrap()
            .days
            .value
    };
    let (plain, asked) = (
        ask(&AlmanacRequest::new()),
        ask(&AlmanacRequest::new().with_gowri()),
    );
    assert!(plain.iter().all(|day| day.gowri.is_empty()));
    assert!(
        serde_json::to_value(&plain[0])
            .unwrap()
            .get("gowri")
            .is_none()
    );
    for day in &asked {
        assert_eq!(day.gowri.len(), 16, "{:?}", day.day.date);
        let (daylight, night) = (
            day.day.sunset.get() - day.day.sunrise.get(),
            day.day.next_sunrise.get() - day.day.sunset.get(),
        );
        for (k, part) in day.gowri.iter().enumerate() {
            let arc = if part.is_daytime { daylight } else { night };
            assert!(
                (part.at.days() - arc / 8.0).abs() < 1e-9,
                "{:?} {k}",
                day.day.date
            );
            assert_eq!(part.is_daytime, k < 8);
        }
        let rahu = day
            .kaalas
            .iter()
            .find(|k| k.kaala == Kaala::RahuKaala)
            .unwrap();
        let visham: Vec<_> = day
            .gowri
            .iter()
            .filter(|p| p.gowri == Gowri::Visham)
            .collect();
        assert_eq!(visham.len(), 2, "one by day, one by night");
        assert_eq!(visham[0].at, rahu.at, "{:?}", day.day.date);
        // Seven grahas and Rahu, each once a half.
        for half in [&day.gowri[..8], &day.gowri[8..]] {
            let mut names: Vec<_> = half.iter().map(|p| p.gowri).collect();
            names.sort_by_key(|g| g.id());
            assert_eq!(names, Gowri::ALL);
        }
    }
    // A Sunday's daylight opens with the Sun's Uthi, its night with Jupiter's Dhanam.
    let sunday = asked
        .iter()
        .find(|day| day.day.vara == Vara::Ravivara)
        .unwrap();
    assert_eq!(
        (sunday.gowri[0].gowri, sunday.gowri[8].gowri),
        (Gowri::Uthi, Gowri::Dhanam)
    );
    let json = DaysRequest::from_json(
        r#"{"first": {"year": 2026, "month": 11, "day": 1}, "latitudeDeg": 13.08,
            "longitudeDeg": 80.27, "utcOffsetSeconds": 19800, "gowri": true}"#,
    )
    .unwrap();
    assert!(json.beside.gowri());
}
