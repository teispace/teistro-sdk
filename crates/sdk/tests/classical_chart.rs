//! A chart founded on the Surya Siddhanta through the façade: the text's
//! chart under the default policy, the hybrid only when asked for by name,
//! and a refusal where the text defines no answer
//! (`docs/03-design/classical-chart.md`).
//!
//! `classical-chart-measured.md` holds every recorded birth to the text
//! part by part under `prefer-native`; these hold the other policies and
//! the refusals, which a pass over one profile cannot reach.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests fail by panicking"
)]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, Status, UtcOffset};
use teistro_siddhanta::SuryaSiddhanta;

/// A Kathmandu birth, 14 April 1990.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// The text's own zodiac, and whatever else a test patches in.
fn over_the_text(extra: &str) -> Context {
    Context::builder()
        .settings_json(format!(
            r#"{{"frame": {{"ayanamsha": {{"kind": "CATALOGUED", "id": "SURYASIDDHANTA"}}}}{extra}}}"#
        ))
        .ephemeris([Ephemeris::SuryaSiddhanta])
        .build()
        .unwrap()
}

fn chart(sdk: &Context) -> Result<Document, teistro::Error> {
    let request = ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0));
    sdk.chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request)
        .map(|envelope| envelope.value)
}

fn text_lagna() -> (f64, f64) {
    let place = kathmandu();
    let lagna = SuryaSiddhanta::text()
        .lagna(
            JulianDay::<Ut1>::literal(BIRTH),
            place.latitude,
            place.longitude,
        )
        .unwrap();
    (lagna.sidereal_deg, lagna.meridian_sidereal_deg)
}

#[test]
fn the_default_policy_founds_the_texts_chart() {
    let sdk = over_the_text("");
    let chart = chart(&sdk).unwrap();
    let (lagna, meridian) = text_lagna();
    assert!((chart.foundation.lagna_deg - lagna).abs() < 1e-9);
    let angles = sdk.chart().angles(&chart).unwrap();
    assert!((angles.midheaven_deg - meridian).abs() < 1e-9);
    let sun = SuryaSiddhanta::text()
        .graha(Graha::Sun, JulianDay::<Ut1>::literal(BIRTH))
        .unwrap();
    let founded = chart.foundation.graha(Graha::Sun).unwrap();
    assert!((founded.longitude_deg - sun.longitude.get()).abs() < 1e-9);
    for step in ["zodiac:NATIVE", "angles:NATIVE", "corrections:NATIVE"] {
        assert!(
            chart.foundation.steps.iter().any(|s| s == step),
            "{step} in {:?}",
            chart.foundation.steps
        );
    }
    assert!(chart.foundation.angles_are_the_providers());
}

#[test]
fn the_envelope_says_which_parts_the_text_defined() {
    let request = ChartRequest::at(kathmandu(), UtcOffset::literal(5, 45, 0));
    let text = over_the_text("")
        .chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request)
        .unwrap();
    let deviation = text
        .provenance
        .deviation
        .expect("a classical chart deviates");
    assert_eq!(deviation.model, "SURYA_SIDDHANTA");
    assert_eq!(
        deviation.detail,
        "the zodiac, the places, the angles and the day are the provider's own"
    );
    let modern = Context::builder()
        .ephemeris([Ephemeris::Test])
        .build()
        .unwrap()
        .chart()
        .reading(JulianDay::<Utc>::literal(BIRTH), &request)
        .unwrap();
    assert_eq!(modern.provenance.deviation, None);
}

#[test]
fn sdk_only_is_the_hybrid_asked_for_by_name() {
    let sdk = over_the_text(r#", "provider": {"overrides": "SDK_ONLY"}"#);
    let chart = chart(&sdk).unwrap();
    let (lagna, _) = text_lagna();
    assert!(
        (chart.foundation.lagna_deg - lagna).abs() > 0.1,
        "the sphere's lagna {} is the text's {lagna}",
        chart.foundation.lagna_deg
    );
    assert!(!chart.foundation.angles_are_the_providers());
    assert!(chart.foundation.steps.iter().any(|s| s == "angles:SDK"));
}

#[test]
fn a_system_that_needs_the_sphere_is_refused_over_the_texts_angles() {
    let sdk = over_the_text(r#", "houses": {"placement_system": "PLACIDUS"}"#);
    let refused = chart(&sdk).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported, "{refused}");
    assert_eq!(refused.field(), Some("houses"));
    assert!(refused.message.contains("PLACIDUS"), "{refused}");
    assert!(refused.message.contains("SRIPATI"), "{refused}");
}

#[test]
fn a_stored_classical_charts_angles_need_its_provider() {
    let founded = chart(&over_the_text("")).unwrap();
    // A context over another ephemeris cannot answer them by the sphere.
    let elsewhere = Context::builder()
        .ephemeris([Ephemeris::Test])
        .build()
        .unwrap();
    let refused = elsewhere.chart().angles(&founded).unwrap_err();
    assert_eq!(refused.status, Status::Unsupported, "{refused}");
    // And one with no ephemeris at all says which option it needs.
    let none = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .unwrap();
    let missing = none.chart().angles(&founded).unwrap_err();
    assert_eq!(missing.field(), Some("ephemeris"), "{missing}");
}
