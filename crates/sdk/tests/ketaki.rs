//! Ketaki through the façade (`docs/03-design/ketaki.md`, phase K1): the
//! `ketaki` profile over a modern engine takes the catalogue's `KETAKI`,
//! which is its `CUSTOM` restatement to the bit in the chart and in the
//! solar months. Ketaki's origin stands about a minute of arc behind
//! Lahiri's, so the Sun reaches it, and its sankranti falls, about 24
//! minutes earlier; the Tilak panchang's Revati origin stands 3°58′ further
//! back, about four days earlier still.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they asked for"
)]

use teistro::catalogue::{Ayanamsha, Calendar};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::settings::AyanamshaChoice;
use teistro::{CalendarDate, ChartRequest, Context, Document, Ephemeris, UtcOffset};

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
