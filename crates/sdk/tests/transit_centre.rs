//! Where a transit is seen from (`gochar.transit_centre`, crux C778): one
//! context casting a topocentric birth and geocentric transits answers as
//! a second, geocentric context does, to the bit, and stamps its own
//! settings hash.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]
#![cfg_attr(coverage_nightly, coverage(off))]

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{
    ChartRequest, Context, Document, Ephemeris, GocharRequest, HitRequest, SadeSatiRequest,
    UtcOffset,
};

/// A Kathmandu birth, 14 April 1990, the corpus's first.
const BIRTH: f64 = 2_447_995.489_583_333_5;

fn context(profile: &str, patch: &str) -> Context {
    Context::builder()
        .profile(profile)
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn natal(sdk: &Context) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(BIRTH),
            &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

fn json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).unwrap()
}

/// Every transit request of `one` against `natal` answers as `other`'s
/// does, and each carries its own context's settings hash.
fn transits_agree(one: &Context, other: &Context, natal: &Document) {
    let instants: Vec<_> = (0_u32..24)
        .map(|k| JulianDay::<Utc>::literal(2_460_676.5 + 15.25 * f64::from(k)))
        .collect();
    let gochar = GocharRequest::over(instants);
    let (a, b) = (
        one.chart().gochar(natal, &gochar).unwrap(),
        other.chart().gochar(natal, &gochar).unwrap(),
    );
    assert_eq!(json(&a.value), json(&b.value), "gochar");
    assert_eq!(a.provenance.settings_hash, one.settings_hash());

    let window = (
        JulianDay::<Utc>::literal(2_460_676.5),
        JulianDay::<Utc>::literal(2_461_041.5),
    );
    let hits = HitRequest::between(window.0, window.1);
    let (a, b) = (
        one.chart().hits(natal, &hits).unwrap(),
        other.chart().hits(natal, &hits).unwrap(),
    );
    assert_eq!(json(&a.value), json(&b.value), "hits");
    assert_eq!(a.provenance.settings_hash, one.settings_hash());

    let spells = SadeSatiRequest::between(window.0, JulianDay::literal(window.1.get() + 3650.0));
    let (a, b) = (
        one.chart().sade_sati(natal, &spells).unwrap(),
        other.chart().sade_sati(natal, &spells).unwrap(),
    );
    assert_eq!(json(&a.value), json(&b.value), "sade sati");
    assert_eq!(a.provenance.settings_hash, one.settings_hash());
}

/// The conformance profile: a topocentric birth, geocentric transits, as
/// a geocentric second context casts them.
#[test]
fn a_geocentric_transit_of_a_topocentric_birth_is_a_geocentric_context_s() {
    let one = context("conformance-baseline", "{}");
    let geocentric = context(
        "conformance-baseline",
        r#"{"frame": {"centre": "GEOCENTRIC"}}"#,
    );
    assert_ne!(one.settings_hash(), geocentric.settings_hash());
    let natal = natal(&one);
    // The birth is the topocentric one: its Moon is not the Earth's.
    let moon = |natal: &Document| {
        natal
            .foundation
            .graha(teistro::catalogue::Graha::Moon)
            .unwrap()
            .longitude_deg
    };
    assert!((moon(&natal) - moon(&self::natal(&geocentric))).abs() > 1e-3);
    transits_agree(&one, &geocentric, &natal);
}

/// The other way about: a geocentric birth's transits from the birthplace.
#[test]
fn a_topocentric_transit_of_a_geocentric_birth_is_a_topocentric_context_s() {
    let one = context(
        "parashari-classical",
        r#"{"gochar": {"transit_centre": "TOPOCENTRIC"}}"#,
    );
    let topocentric = context(
        "parashari-classical",
        r#"{"frame": {"centre": "TOPOCENTRIC"}}"#,
    );
    let natal = natal(&one);
    transits_agree(&one, &topocentric, &natal);
}

/// `AS_CHART` and a knob naming the chart's own centre are the context's
/// own transits.
#[test]
fn the_chart_s_own_centre_is_the_context_s_transit() {
    let natal = natal(&context("parashari-classical", "{}"));
    for (centre, named) in [("GEOCENTRIC", "GEOCENTRIC"), ("TOPOCENTRIC", "TOPOCENTRIC")] {
        let own = context(
            "parashari-classical",
            &format!(
                r#"{{"frame": {{"centre": "{centre}"}}, "gochar": {{"transit_centre": "AS_CHART"}}}}"#
            ),
        );
        let named = context(
            "parashari-classical",
            &format!(
                r#"{{"frame": {{"centre": "{centre}"}}, "gochar": {{"transit_centre": "{named}"}}}}"#
            ),
        );
        transits_agree(&own, &named, &natal);
    }
}
