//! KP through the façade: the chart's own cusps and grahas read with their
//! lords, the house system the settings name, and the ayanamsha a KP
//! reading takes (`docs/03-design/kp.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they found"
)]

use teistro::catalogue::{Ayanamsha, HouseSystem};
use teistro::kp::{house_of, lords};
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{ChartRequest, Context, Document, Ephemeris, KpRequest, Nas, UtcOffset};

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

fn chart_at(sdk: &Context, latitude: f64) -> Document {
    let place = Place::new(
        Latitude::literal(latitude),
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

fn nas(degrees: f64) -> Nas {
    Nas::try_from_degrees(degrees).unwrap()
}

/// Read back through the chart itself: the 1st cusp is its lagna and the
/// 10th its midheaven, every graha is where the chart put it, and each
/// lord and house is the crate's own reading of that longitude.
#[test]
fn a_kp_chart_is_the_founded_chart_read_by_its_lords() {
    let sdk = context("kp-default", "{}");
    let chart = chart_at(&sdk, 27.7172);
    let kp = sdk.chart().kp(&chart, &KpRequest::new()).unwrap();
    assert_eq!(kp.system, HouseSystem::Placidus);
    let foundation = &chart.foundation;
    let angles = sdk.chart().angles(&chart).unwrap();
    assert_eq!(kp.cusp(1).unwrap().longitude, nas(foundation.lagna_deg));
    assert_eq!(kp.cusp(10).unwrap().longitude, nas(angles.midheaven_deg));
    let cusps = kp.cusps.map(|cusp| cusp.longitude);
    assert_eq!(kp.planets.len(), foundation.grahas.len());
    for (planet, graha) in kp.planets.iter().zip(&foundation.grahas) {
        assert_eq!(planet.graha, graha.graha);
        assert_eq!(planet.longitude, nas(graha.longitude_deg));
        assert_eq!(planet.retrograde, graha.is_retrograde());
        assert_eq!(planet.house, house_of(&cusps, planet.longitude));
        assert_eq!(planet.lords, lords(planet.longitude));
    }
    for cusp in &kp.cusps {
        assert_eq!(cusp.lords, lords(cusp.longitude));
    }
}

/// C157: a chart in another zodiac is refused by name unless the request
/// says to read it as it is.
#[test]
fn a_chart_outside_the_kp_ayanamshas_is_refused_unless_asked_for() {
    let sdk = context("nepali-default", "{}");
    let chart = chart_at(&sdk, 27.7172);
    let refused = sdk.chart().kp(&chart, &KpRequest::new()).unwrap_err();
    assert_eq!(refused.field(), Some("frame.ayanamsha"));
    assert!(refused.to_string().contains("LAHIRI"), "{refused}");
    let read = sdk
        .chart()
        .kp(&chart, &KpRequest::new().under_any_ayanamsha())
        .unwrap();
    // No override names KP's system here, so it is the Readers' Placidus.
    assert_eq!(read.system, HouseSystem::Placidus);

    let vp291 = context(
        "kp-default",
        &format!(
            r#"{{"frame": {{"ayanamsha": {{"kind": "CATALOGUED", "id": "{}"}}}}}}"#,
            Ayanamsha::KrishnamurtiVp291.key()
        ),
    );
    let chart = chart_at(&vp291, 27.7172);
    assert!(vp291.chart().kp(&chart, &KpRequest::new()).is_ok());
}

/// The settings' override for the `kp` module chooses the system, and
/// inside the polar circle the profile's policy says what stood in.
#[test]
fn the_settings_choose_the_system_and_the_polar_policy_what_stands_in() {
    let koch = context(
        "kp-default",
        r#"{"houses": {"module_overrides": {"kp": "KOCH"}}}"#,
    );
    let chart = chart_at(&koch, 27.7172);
    let kp = koch.chart().kp(&chart, &KpRequest::new()).unwrap();
    assert_eq!(kp.system, HouseSystem::Koch);
    assert_eq!(
        kp.cusp(1).unwrap().longitude,
        nas(chart.foundation.lagna_deg)
    );
    let placidus = context("kp-default", "{}");
    let other = placidus.chart().kp(&chart, &KpRequest::new()).unwrap();
    assert_ne!(kp.cusp(2), other.cusp(2));

    // kp-default falls back to Porphyry where Placidus is undefined.
    let polar = chart_at(&placidus, 75.0);
    let kp = placidus.chart().kp(&polar, &KpRequest::new()).unwrap();
    assert_eq!(kp.system, HouseSystem::Porphyry);
}

/// Through the façade the significators are the crate's, with the node
/// aspects the settings give, and every house has a lord from the chart's
/// own cusp.
#[test]
fn the_significators_are_read_with_the_settings_node_aspects() {
    let sdk = context("kp-default", "{}");
    let chart = chart_at(&sdk, 27.7172);
    let kp = sdk.chart().kp(&chart, &KpRequest::new()).unwrap();
    let significators = sdk.chart().kp_significators(&kp);
    assert_eq!(
        significators,
        teistro::kp::Significators::of(&kp, teistro::settings::NodeAspects::None)
    );
    for (house, cusp) in significators.houses.iter().zip(&kp.cusps) {
        assert_eq!(house.lord, cusp.lords.sign);
        // Every planet is an occupant of exactly one house.
        assert!(
            house
                .occupants
                .iter()
                .all(|graha| kp.planet(*graha).unwrap().house == house.house)
        );
    }
    let occupied: usize = significators
        .houses
        .iter()
        .map(|house| house.occupants.len())
        .sum();
    assert_eq!(occupied, kp.planets.len());
    assert_eq!(significators.nodes.len(), 2);

    let nodes = context(
        "kp-default",
        r#"{"aspect": {"node_aspects": "FIVE_SEVEN_NINE"}}"#,
    );
    assert_eq!(
        nodes.chart().kp_significators(&kp),
        teistro::kp::Significators::of(&kp, teistro::settings::NodeAspects::FiveSevenNine)
    );
}

/// 04:00 in Kathmandu on Saturday 14 April 1990 (`BIRTH` is 05:30 there):
/// before sunrise, so the
/// chart's own day (C151's default) is still Friday's, Venus's, while the
/// civil weekday is Saturday, Saturn's.
const BEFORE_DAWN: f64 = BIRTH - 1.5 / 24.0;

fn before_dawn(sdk: &Context) -> Document {
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    sdk.chart()
        .reading(
            JulianDay::<Utc>::literal(BEFORE_DAWN),
            &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value
}

/// The ruling planets are the crate's, with the chart's own day lord and
/// the settings' rules; the civil reading takes its weekday from the clock
/// the request names, and without one says so.
#[test]
fn the_ruling_planets_take_the_day_from_sunrise_or_from_the_named_clock() {
    use teistro::catalogue::Graha;
    use teistro::kp::{Reason, RulingPlanets, RulingRules};

    let sdk = context("kp-default", "{}");
    let chart = before_dawn(&sdk);
    let kp = sdk.chart().kp(&chart, &KpRequest::new()).unwrap();
    let ruling = sdk.chart().kp_ruling(&chart, &KpRequest::new()).unwrap();
    assert_eq!(
        ruling,
        RulingPlanets::of(&kp, Graha::Venus, RulingRules::READER)
    );
    let day_lord = |ruling: &RulingPlanets| {
        ruling
            .rulers
            .iter()
            .find(|ruler| ruler.reasons.contains(&Reason::DayLord))
            .map(|ruler| ruler.graha)
    };
    assert_eq!(day_lord(&ruling), Some(Graha::Venus));

    let civil = context("kp-default", r#"{"kp": {"day_lord_day": "CIVIL"}}"#);
    let refused = civil
        .chart()
        .kp_ruling(&chart, &KpRequest::new())
        .unwrap_err();
    assert_eq!(refused.field(), Some("kp.day_lord_day"));
    let on_clock = KpRequest::new().on_clock(UtcOffset::literal(5, 45, 0));
    let ruling = civil.chart().kp_ruling(&chart, &on_clock).unwrap();
    assert_eq!(day_lord(&ruling), Some(Graha::Saturn));
    // The same moment on a clock behind UT is still Friday.
    let behind = KpRequest::new().on_clock(UtcOffset::literal(-5, 0, 0));
    let ruling = civil.chart().kp_ruling(&chart, &behind).unwrap();
    assert_eq!(day_lord(&ruling), Some(Graha::Venus));

    // The settings' other readings reach the reading.
    let seven = context("kp-default", r#"{"kp": {"ruling_count": "WITH_SUBS"}}"#);
    let ruling = seven.chart().kp_ruling(&chart, &KpRequest::new()).unwrap();
    assert!(
        ruling
            .rulers
            .iter()
            .any(|ruler| ruler.reasons.contains(&Reason::LagnaSub))
    );
}
