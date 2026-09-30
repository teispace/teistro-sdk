//! KP through the façade: the chart's own cusps and grahas read with their
//! lords, the house system the settings name, and the ayanamsha a KP
//! reading takes (`docs/03-design/kp.md`).

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::cast_precision_loss,
    reason = "tests fail by panicking, index what they found and measure arcs below 2^53"
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

/// Read back through a founded chart: the horary chart for a number is
/// the chart of the moment that day when the lagna itself reaches the
/// number's start, found here by founding charts, to within the drift of
/// the obliquity, nutation and ayanamsha over the hours between, which
/// measured under 0.1″ on every cusp of these five numbers; and its
/// planets are the moment of judgement's.
#[test]
fn a_horary_chart_is_the_moment_the_lagna_reaches_the_number() {
    use teistro::KpNumber;
    let sdk = context("kp-default", "{}");
    let moment = chart_at(&sdk, 27.7172);
    let now = sdk.chart().kp(&moment, &KpRequest::new()).unwrap();
    let place = moment.foundation.place;
    let lagna_at = |jd: f64| -> Document {
        sdk.chart()
            .reading(
                JulianDay::<Utc>::literal(jd),
                &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
            )
            .unwrap()
            .value
    };
    let forward = |from: Nas, to: Nas| from.arc_to(to).get();
    for number in [1, 48, 74, 229, 249] {
        let number = KpNumber::new(number).unwrap();
        let horary = sdk
            .chart()
            .kp_horary(&moment, number, &KpRequest::new())
            .unwrap();
        assert_eq!(horary.cusp(1).unwrap().longitude, number.start());
        // The moment's planets, housed by the horary cusps.
        let placed = |chart: &teistro::KpChart| -> Vec<_> {
            chart
                .planets
                .iter()
                .map(|planet| {
                    (
                        planet.graha,
                        planet.longitude,
                        planet.retrograde,
                        planet.lords,
                    )
                })
                .collect()
        };
        assert_eq!(placed(&horary), placed(&now), "{number}");
        let cusps = horary.cusps.map(|cusp| cusp.longitude);
        assert!(
            horary
                .planets
                .iter()
                .all(|planet| planet.house == teistro::kp::house_of(&cusps, planet.longitude))
        );
        assert_eq!(horary.system, now.system);

        // The lagna goes round once a sidereal day: bisect the day after
        // the moment for when it reaches the number's start.
        let rising = nas(moment.foundation.lagna_deg);
        let wanted = forward(rising, number.start());
        let (mut low, mut high) = (BIRTH, BIRTH + 0.997_27);
        for _ in 0..40 {
            let middle = f64::midpoint(low, high);
            if forward(rising, nas(lagna_at(middle).foundation.lagna_deg)) < wanted {
                low = middle;
            } else {
                high = middle;
            }
        }
        let then = lagna_at(f64::midpoint(low, high));
        let founded = sdk.chart().kp(&then, &KpRequest::new()).unwrap();
        for (a, b) in horary.cusps.iter().zip(&founded.cusps) {
            let apart = a.longitude.signed_difference(b.longitude).abs();
            assert!(
                apart < Nas::PER_ARCSECOND / 10,
                "{number}: cusp {} is {apart} nas from the founded chart's",
                a.house
            );
        }
    }
}

/// Beyond the polar circle a number is raised where its start crosses the
/// horizon and refused where it never does. At 69.65° north (Tromsø) the
/// colatitude is 20.35°, so a start whose declination passes it circles
/// without rising or setting: about 37° to 95° and 217° to 275° of
/// Krishnamurti's zodiac. The ascendant jumps across those arcs as the
/// meridian turns, which a single bisection over the day mistakes for a
/// crossing, so the read-back scans the day too: the founded chart whose
/// own lagna reaches the start has the horary chart's cusps.
#[test]
fn a_polar_place_raises_what_rises_and_refuses_what_never_does() {
    use teistro::KpNumber;
    let sdk = context("kp-default", "{}");
    let moment = chart_at(&sdk, 69.65);
    let number = |degrees: f64| KpNumber::of(nas(degrees));
    for degrees in [66.0, 246.0] {
        let refused = sdk
            .chart()
            .kp_horary(&moment, number(degrees), &KpRequest::new())
            .unwrap_err();
        assert_eq!(refused.field(), Some("place.latitude"), "{degrees}°");
    }
    for degrees in [0.5, 150.0] {
        let horary = sdk
            .chart()
            .kp_horary(&moment, number(degrees), &KpRequest::new())
            .unwrap();
        assert_eq!(
            horary.cusps[0].longitude,
            number(degrees).start(),
            "{degrees}°"
        );
    }

    // The founded chart that day whose lagna reaches Virgo's number: its
    // lagna less the start, scanned every four minutes for a crossing that
    // is not the jump across a circumpolar arc, then bisected.
    let target = number(150.0);
    let horary = sdk
        .chart()
        .kp_horary(&moment, target, &KpRequest::new())
        .unwrap();
    let place = moment.foundation.place;
    let short = |jd: f64| -> f64 {
        let founded = sdk
            .chart()
            .reading(
                JulianDay::<Utc>::literal(jd),
                &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
            )
            .unwrap()
            .value;
        target
            .start()
            .signed_difference(nas(founded.foundation.lagna_deg)) as f64
    };
    let step = 4.0 / 1440.0;
    let mut before = short(BIRTH);
    let crossing = (1..=360)
        .map(|k| BIRTH + step * f64::from(k))
        .find(|jd| {
            let now = short(*jd);
            let crosses = (before < 0.0) != (now < 0.0)
                && (now - before).abs() < 90.0 * Nas::PER_DEGREE as f64;
            before = now;
            crosses
        })
        .unwrap(); // Virgo rises at Tromsø that day.
    let (mut low, mut high) = (crossing - step, crossing);
    let rising = short(low) < 0.0;
    for _ in 0..40 {
        let middle = f64::midpoint(low, high);
        if (short(middle) < 0.0) == rising {
            low = middle;
        } else {
            high = middle;
        }
    }
    let then = sdk
        .chart()
        .reading(
            JulianDay::<Utc>::literal(f64::midpoint(low, high)),
            &ChartRequest::at(place, UtcOffset::literal(5, 45, 0)),
        )
        .unwrap()
        .value;
    let founded = sdk.chart().kp(&then, &KpRequest::new()).unwrap();
    assert_eq!(founded.system, horary.system);
    for (a, b) in horary.cusps.iter().zip(&founded.cusps) {
        let apart = a.longitude.signed_difference(b.longitude).abs();
        assert!(
            apart < Nas::PER_ARCSECOND / 10,
            "cusp {} is {apart} nas from the founded chart's",
            a.house
        );
    }
}

/// A reading whole is its three parts: the chart the request names, its
/// significators, and the ruling planets of the moment, which a horary
/// number does not move.
#[test]
fn a_reading_is_the_chart_its_significators_and_the_moment_s_rulers() {
    use teistro::KpNumber;
    let sdk = context("kp-default", "{}");
    let moment = chart_at(&sdk, 27.7172);
    let plain = sdk.chart().kp_reading(&moment, &KpRequest::new()).unwrap();
    assert_eq!(
        plain.chart,
        sdk.chart().kp(&moment, &KpRequest::new()).unwrap()
    );
    assert_eq!(
        plain.significators,
        sdk.chart().kp_significators(&plain.chart)
    );
    assert_eq!(
        plain.ruling,
        sdk.chart().kp_ruling(&moment, &KpRequest::new()).unwrap()
    );

    let number = KpNumber::new(74).unwrap();
    let asked = KpRequest::new().for_number(number);
    let horary = sdk.chart().kp_reading(&moment, &asked).unwrap();
    assert_eq!(
        horary.chart,
        sdk.chart()
            .kp_horary(&moment, number, &KpRequest::new())
            .unwrap()
    );
    assert_eq!(horary.chart, sdk.chart().kp(&moment, &asked).unwrap());
    assert_eq!(
        horary.significators,
        sdk.chart().kp_significators(&horary.chart)
    );
    assert_eq!(horary.ruling, plain.ruling);
    assert_eq!(
        sdk.chart().kp_ruling(&moment, &asked).unwrap(),
        plain.ruling
    );
}
