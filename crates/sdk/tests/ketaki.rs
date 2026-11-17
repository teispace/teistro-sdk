//! Ketaki through the façade (`docs/03-design/ketaki.md`, phase K1): the
//! `ketaki` profile over a modern engine takes the catalogue's `KETAKI`,
//! which is its `CUSTOM` restatement to the bit in the chart and in the
//! solar months. Ketaki's origin stands about a minute of arc behind
//! Lahiri's, so the Sun reaches it, and its sankranti falls, about 24
//! minutes earlier; the Tilak panchang's Revati origin stands 3°58′ further
//! back, about four days earlier still, which moves its adhika months and
//! reproduces the two Tilak dates the sources print (T10).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Ayanamsha, Calendar, Masa, Tithi};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::AyanamshaChoice;
use teistro::{CalendarDate, ChartRequest, Context, Document, Ephemeris, MonthKind, UtcOffset};

fn context(profile: &str, patch: &str) -> Context {
    Context::builder()
        .profile(profile)
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

/// The text's rule as `CUSTOM` (§4.1).
const CUSTOM: &str = r#"{"frame": {"ayanamsha": {"kind": "CUSTOM", "epoch_jd_tt": 2407086.5, "value_deg": 22.1425, "rate_deg_per_year": 0.013952138}}}"#;

/// Solapur, where the Date Panchang is made.
fn solapur() -> Place {
    Place::new(
        Latitude::literal(17.6599),
        Longitude::literal(75.9064),
        Altitude::literal(457.0),
    )
}

fn ist() -> UtcOffset {
    UtcOffset::literal(5, 30, 0)
}

/// A Solapur birth, 14 April 1990.
fn chart(sdk: &Context) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            &ChartRequest::at(solapur(), ist()),
        )
        .unwrap()
        .value
}

/// The instants of the sankrantis April 2026 holds at Solapur.
fn sankrantis(sdk: &Context) -> Vec<f64> {
    let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 4, day);
    sdk.almanac()
        .of(&date(1), &date(30), &solapur(), ist())
        .unwrap()
        .value
        .iter()
        .filter_map(|day| day.punya_kala.as_ref().map(|kala| kala.sankranti.get()))
        .collect()
}

/// The profile names the member, and the member is its `CUSTOM` form in
/// every graha and in the sankranti, to the bit.
#[test]
fn the_profile_takes_the_member_which_is_its_custom_form() {
    let ketaki = context("ketaki", "{}");
    let custom = context("ketaki", CUSTOM);
    assert_ne!(ketaki.settings_hash(), custom.settings_hash());

    let (member, restated) = (chart(&ketaki), chart(&custom));
    assert_eq!(
        member.foundation.zodiac.ayanamsha,
        Some(AyanamshaChoice::Catalogued {
            id: Ayanamsha::Ketaki
        })
    );
    assert_eq!(
        member.foundation.zodiac.offset_deg.to_bits(),
        restated.foundation.zodiac.offset_deg.to_bits()
    );
    for (a, b) in member
        .foundation
        .grahas
        .iter()
        .zip(&restated.foundation.grahas)
    {
        assert_eq!(
            a.longitude_deg.to_bits(),
            b.longitude_deg.to_bits(),
            "{:?}",
            a.graha
        );
    }
    let (a, b) = (sankrantis(&ketaki), sankrantis(&custom));
    assert_eq!(a.len(), 1, "{a:?}");
    assert_eq!(a, b);
}

/// Ketaki stands about 61″ below Lahiri in 2026: a sidereal longitude is
/// the tropical one less the ayanamsha, so the Sun's is 61″ greater and
/// reaches Mesha's 0° 20 to 30 minutes before Lahiri's. The Revati origin
/// of the Tilak panchang stands 3°58′ below Ketaki, about four days
/// earlier (research §7).
#[test]
fn the_sankranti_moves_by_the_gap_between_the_origins() {
    let ketaki = sankrantis(&context("ketaki", "{}"));
    let lahiri = sankrantis(&context(
        "ketaki",
        r#"{"frame": {"ayanamsha": {"kind": "CATALOGUED", "id": "LAHIRI"}}}"#,
    ));
    let tilak = sankrantis(&context("tilak-revati", "{}"));
    assert_eq!((ketaki.len(), lahiri.len(), tilak.len()), (1, 1, 1));
    let minutes = (lahiri[0] - ketaki[0]) * 1440.0;
    assert!(
        (20.0..30.0).contains(&minutes),
        "{minutes} min before Lahiri's"
    );
    let days = ketaki[0] - tilak[0];
    assert!((3.8..4.3).contains(&days), "{days} days after Tilak's");
}

/// Pune, where the Tilak panchang is made.
fn pune() -> Place {
    Place::new(
        Latitude::literal(18.5204),
        Longitude::literal(73.8567),
        Altitude::literal(560.0),
    )
}

/// The amanta month, its kind and whether Shukla Chaturthi touches a day
/// of 2015 at Pune.
fn month_on(sdk: &Context, month: u8, day: u8) -> (Masa, MonthKind, bool) {
    let date = CalendarDate::defined(Calendar::Gregorian, 2015, month, day);
    let days = sdk
        .almanac()
        .of(&date, &date, &pune(), ist())
        .unwrap()
        .value;
    let day = &days[0];
    let chaturthi = day
        .limbs
        .tithi
        .iter()
        .any(|span| span.member == Tithi::ShuklaChaturthi);
    (day.month.amanta, day.month.kind, chaturthi)
}

/// T10's 2015 case: the Tilak panchang kept Ganesh Chaturthi (Bhadrapada
/// Shukla Chaturthi) on 18 August, the other almanacs on 17 September
/// (*Lokmat*, 18 August 2015). Under `tilak-revati` the month holding 18
/// August is already Bhadrapada; under `ketaki` it is Shravana, and
/// Bhadrapada's Chaturthi is 17 September. The four days between the
/// origins move the adhika month, and so the whole month.
#[test]
fn ganesh_chaturthi_of_2015_is_a_month_apart_under_the_two_origins() {
    let (ketaki, tilak) = (context("ketaki", "{}"), context("tilak-revati", "{}"));
    assert_eq!(
        month_on(&tilak, 8, 18),
        (Masa::Bhadrapada, MonthKind::Nija, true)
    );
    assert_eq!(month_on(&ketaki, 8, 18).0, Masa::Shravana);
    assert_eq!(
        month_on(&ketaki, 9, 17),
        (Masa::Bhadrapada, MonthKind::Nija, true)
    );
}

/// The adhika months of the lunar year Shaka 1848 (March 1926 to March
/// 1927; a year is the most one request holds) at Pune, by amanta name.
fn adhika_of_shaka_1848(sdk: &Context) -> Vec<Masa> {
    let from = CalendarDate::defined(Calendar::Gregorian, 1926, 3, 1);
    let to = CalendarDate::defined(Calendar::Gregorian, 1927, 2, 28);
    let mut months: Vec<Masa> = sdk
        .almanac()
        .of(&from, &to, &pune(), ist())
        .unwrap()
        .value
        .iter()
        .filter(|day| day.month.kind == MonthKind::Adhika)
        .map(|day| day.month.amanta)
        .collect();
    months.dedup();
    months
}

/// T10's 1926 case: in Shaka 1848 the Tilak panchang had an adhika
/// Jyeshtha where the Grahalaghava almanacs had an adhika Chaitra (the
/// panchang-unity board's report, 1926, p. 1). `tilak-revati` gives the
/// Jyeshtha. Ketaki's origin, four degrees on, gives an adhika Phalguna in
/// February 1927 instead, a third answer the report does not print.
#[test]
fn shaka_1848_has_tilaks_adhika_jyeshtha() {
    let tilak = adhika_of_shaka_1848(&context("tilak-revati", "{}"));
    let ketaki = adhika_of_shaka_1848(&context("ketaki", "{}"));
    assert_eq!(tilak, [Masa::Jyeshtha]);
    assert_eq!(ketaki, [Masa::Phalguna]);
}
