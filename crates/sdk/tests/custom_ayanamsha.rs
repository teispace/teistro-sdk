//! Custom ayanamshas through the façade (SR-112,
//! `docs/03-design/astro-custom-ayanamsha.md`): a settings document names
//! a value carried by a precession model, or a star held at a longitude,
//! and the chart and the solar months both take it. Until SR-112 the
//! solar model put Lahiri in place of any custom definition, so the
//! sankrantis kept Lahiri's zodiac while the chart took the custom one.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::AyanamshaChoice;
use teistro::{CalendarDate, ChartRequest, Context, Document, Ephemeris, UtcOffset};

fn context(ayanamsha: &str) -> Context {
    Context::builder()
        .profile("nepali-drik")
        .settings_json(format!(r#"{{"frame": {{"ayanamsha": {ayanamsha}}}}}"#))
        .ephemeris([Ephemeris::Builtin])
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

/// A Kathmandu birth, 14 April 1990.
fn chart(sdk: &Context) -> Document {
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(2_447_995.489_583_333_5),
            &ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

/// The instants of the sankrantis April 2026 holds at Kathmandu.
fn sankrantis(sdk: &Context) -> Vec<f64> {
    let date = |day| CalendarDate::defined(Calendar::Gregorian, 2026, 4, day);
    sdk.almanac()
        .of(
            &date(1),
            &date(30),
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .unwrap()
        .value
        .iter()
        .filter_map(|day| day.punya_kala.as_ref().map(|kala| kala.sankranti.get()))
        .collect()
}

/// Spica held at 180° is True Chitra in the chart and in the solar
/// months, and the chart says which definition it took.
#[test]
fn a_held_star_is_the_catalogued_member_in_the_chart_and_the_months() {
    let spica = context(r#"{"kind": "ANCHORED", "star": "SPICA", "longitude_deg": 180.0}"#);
    let chitra = context(r#"{"kind": "CATALOGUED", "id": "TRUE_CHITRA"}"#);
    let lahiri = context(r#"{"kind": "CATALOGUED", "id": "LAHIRI"}"#);
    assert_ne!(spica.settings_hash(), chitra.settings_hash());

    let (held, member) = (chart(&spica), chart(&chitra));
    assert!(matches!(
        held.foundation.zodiac.ayanamsha,
        Some(AyanamshaChoice::Anchored { .. })
    ));
    let offset = (held.foundation.zodiac.offset_deg - member.foundation.zodiac.offset_deg).abs();
    assert!(offset < 1e-9, "{offset}°");
    for (a, b) in held.foundation.grahas.iter().zip(&member.foundation.grahas) {
        assert!(
            (a.longitude_deg - b.longitude_deg).abs() < 1e-9,
            "{:?}",
            a.graha
        );
    }

    let (held, member, other) = (sankrantis(&spica), sankrantis(&chitra), sankrantis(&lahiri));
    assert_eq!(held.len(), 1, "{held:?}");
    assert_eq!(held.len(), member.len());
    let seconds = (held[0] - member[0]).abs() * 86_400.0;
    assert!(seconds < 0.01, "{seconds} s from True Chitra's");
    // Not Lahiri's, which a custom zodiac took before SR-112.
    let apart = (held[0] - other[0]).abs() * 86_400.0;
    assert!(apart > 60.0, "{apart} s from Lahiri's");
}

/// SR-112's acceptance through the façade: Lahiri's value at its epoch
/// carried by IAU 2006 precession is Lahiri, in the chart's zodiac to
/// 0.01″ and in the sankranti to a hundredth of a second, on the
/// profile's true basis.
#[test]
fn a_precessed_lahiri_is_lahiri_through_the_facade() {
    let lahiri = context(r#"{"kind": "CATALOGUED", "id": "LAHIRI"}"#);
    // The catalogue's mean Lahiri at 1956-03-21 0h TT under Vondrák 2011,
    // the root's precession, to the last digit a double keeps.
    let precessed = context(
        r#"{"kind": "PRECESSED", "epoch_jd_tt": 2435553.5, "value_deg": 23.245560954594648, "basis": "MEAN", "precession": "IAU_2006"}"#,
    );
    let offset = (chart(&precessed).foundation.zodiac.offset_deg
        - chart(&lahiri).foundation.zodiac.offset_deg)
        .abs()
        * 3600.0;
    assert!(offset < 0.01, "{offset}″");
    let (a, b) = (sankrantis(&precessed), sankrantis(&lahiri));
    assert_eq!(a.len(), b.len());
    let seconds = (a[0] - b[0]).abs() * 86_400.0;
    assert!(seconds < 0.01, "{seconds} s");
}
