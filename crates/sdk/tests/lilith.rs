//! The Black Moon Lilith beside the nine (`03-design/lilith.md`): asked
//! for, placed in the same request and the chart's own frame, the mean
//! apogee held to Meeus's worked example and to its motion, and a chart that
//! does not ask is the chart it always was.

#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking"
)]

use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::{Apogee, ChartRequest, Context, Document, Ephemeris, Status, UtcOffset};

/// 1 January 2000, 12.00 UT: a date the corpus brackets.
const J2000: f64 = 2_451_545.0;

/// Days in a Julian year, the unit the apogee's yearly motion is quoted in.
const JULIAN_YEAR: f64 = 365.25;

fn context(profile: &str) -> Context {
    Context::builder()
        .profile(profile)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn kathmandu() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::try_from_seconds(20_700).unwrap(),
    )
}

fn greenwich() -> ChartRequest {
    ChartRequest::at(
        Place::new(
            Latitude::literal(51.4769),
            Longitude::literal(0.0),
            Altitude::literal(0.0),
        ),
        UtcOffset::UTC,
    )
}

fn read(sdk: &Context, at: f64, request: &ChartRequest) -> teistro::Envelope<Document> {
    sdk.chart()
        .reading(JulianDay::<Utc>::try_new(at).unwrap(), request)
        .unwrap()
}

/// `a` less `b` the short way round, degrees in `(-180, 180]`.
fn apart(a: f64, b: f64) -> f64 {
    let d = (a - b).rem_euclid(360.0);
    if d > 180.0 { d - 360.0 } else { d }
}

#[test]
fn a_chart_that_does_not_ask_is_unchanged() {
    let sdk = context("conformance-baseline");
    let chart = read(&sdk, J2000, &kathmandu()).value;
    assert_eq!(chart.foundation.lilith, None);
    let json = serde_json::to_value(&chart).unwrap();
    assert!(
        json["foundation"].get("lilith").is_none(),
        "an unasked Lilith is left out of the document"
    );
    assert_eq!(kathmandu().lilith(), None);
}

#[test]
fn asking_adds_lilith_and_moves_none_of_the_nine() {
    for profile in ["conformance-baseline", "western-tropical-default"] {
        let sdk = context(profile);
        let bare = read(&sdk, J2000, &kathmandu());
        let mut hashes = vec![bare.provenance.input_hash];
        for apogee in Apogee::ALL {
            let asked = read(&sdk, J2000, &kathmandu().with_lilith(apogee));
            let foundation = &asked.value.foundation;
            assert_eq!(
                foundation.grahas, bare.value.foundation.grahas,
                "{profile}, {apogee:?}: the nine are the same to the bit"
            );
            assert_eq!(
                foundation.lagna_deg.to_bits(),
                bare.value.foundation.lagna_deg.to_bits()
            );
            let lilith = foundation.lilith.unwrap();
            assert_eq!(lilith.apogee, apogee);
            // The chart's own zodiac, as the nine.
            let shifted = foundation.zodiac.of_tropical(lilith.tropical_deg);
            assert!(
                apart(shifted, lilith.longitude_deg).abs() < 1e-9,
                "{profile}"
            );
            assert!((1..=12).contains(&lilith.house.bhava), "{profile}");
            assert!((1..=12).contains(&lilith.placement.bhava), "{profile}");
            if apogee == Apogee::Mean {
                // A direction on the ecliptic, with no distance.
                assert!(lilith.distance_au.abs() < 1e-12, "{profile}");
                assert!(lilith.latitude_deg.abs() < 1e-6, "{profile}");
            } else {
                // The far end of the Moon's orbit, some 404 000 km out.
                assert!(
                    (0.0024..=0.0030).contains(&lilith.distance_au),
                    "{profile}: {} au",
                    lilith.distance_au
                );
            }
            hashes.push(asked.provenance.input_hash);
        }
        // No Lilith, the mean one and the osculating one are three
        // questions, and each is stamped as its own.
        for (k, first) in hashes.iter().enumerate() {
            for second in hashes.iter().skip(k + 1) {
                assert_ne!(first, second, "{profile}");
            }
        }
    }
}

/// Meeus, *Astronomical Algorithms* (2nd ed., 1998), ch. 47, Example 47.a:
/// 1992 April 12 at 0h TD. He prints the Moon's mean longitude L′ =
/// 134.290182° and its mean anomaly M′ = 5.150833°, and the mean anomaly is
/// counted from the perigee, so the mean perigee is L′ − M′ and the mean
/// apogee half a circle on: 309.139349°. He prints the nutation in longitude
/// for the instant too, Δψ = +0.004610°, and a chart's tropical zodiac is
/// of the true equinox of date, so the chart's apogee is 309.143959°.
///
/// The SDK's apogee is ELP2000-82B's `W2`, whose constants are not
/// Meeus's (his are the later ELP 2000-85 fit), and at this date the two
/// polynomials stand 0.00019° apart; Delta T at 1992, about a minute, moves
/// the apogee 5e-6°. So the printed answer is asserted to a thousandth of a
/// degree, five times what the two fits differ by.
#[test]
fn the_mean_apogee_is_meeus_example_47a() {
    const L_PRIME: f64 = 134.290_182;
    const M_PRIME: f64 = 5.150_833;
    const NUTATION: f64 = 0.004_610;
    let perigee = L_PRIME - M_PRIME + NUTATION;
    let apogee = (perigee + 180.0).rem_euclid(360.0);
    assert!((apogee - 309.143_959).abs() < 1e-9, "the arithmetic itself");

    let sdk = context("western-tropical-default");
    // 0h TD less a Delta T of about 59 seconds.
    let at = 2_448_724.5 - 59.0 / 86_400.0;
    let chart = read(&sdk, at, &greenwich().with_lilith(Apogee::Mean)).value;
    let lilith = chart.foundation.lilith.unwrap();
    assert!(
        apart(lilith.tropical_deg, apogee).abs() < 1e-3,
        "the mean apogee is {}°, and Meeus's L′ − M′ + 180° + Δψ is {apogee}°",
        lilith.tropical_deg
    );
    assert!(
        apart(lilith.perigee_deg(), perigee).abs() < 1e-3,
        "the mean perigee is {}°, and Meeus's L′ − M′ + Δψ is {perigee}°",
        lilith.perigee_deg()
    );
    // A tropical chart's longitude is the tropical one.
    assert!(apart(lilith.longitude_deg, lilith.tropical_deg).abs() < 1e-12);
}

/// The apogee is the perigee turned half a circle, both apogees, in both
/// zodiacs.
#[test]
fn the_apogee_is_the_perigee_plus_half_a_circle() {
    for profile in ["conformance-baseline", "western-tropical-default"] {
        let sdk = context(profile);
        for apogee in Apogee::ALL {
            let chart = read(&sdk, J2000, &kathmandu().with_lilith(apogee)).value;
            let lilith = chart.foundation.lilith.unwrap();
            let turned = apart(lilith.longitude_deg, lilith.perigee_deg());
            assert!(
                (turned.abs() - 180.0).abs() < 1e-9,
                "{profile}, {apogee:?}: apogee and perigee are {turned}° apart"
            );
        }
    }
}

/// The mean apogee advances 40.69° a Julian year: Meeus's rates for L′
/// and M′ (481267.88123421° and 477198.8675055° a century) differ by
/// 4069.0137° a century. Read two ways: the speed the chart reports, and
/// the longitude a year later. The nutation is in both, and moves a year's
/// difference by at most a few thousandths of a degree.
#[test]
fn the_mean_apogee_advances_forty_degrees_a_year() {
    const RATE_PER_YEAR: f64 = (481_267.881_234_21 - 477_198.867_505_5) / 100.0;
    let sdk = context("western-tropical-default");
    let request = greenwich().with_lilith(Apogee::Mean);
    let now = read(&sdk, J2000, &request).value.foundation.lilith.unwrap();
    let later = read(&sdk, J2000 + JULIAN_YEAR, &request)
        .value
        .foundation
        .lilith
        .unwrap();
    let per_year = now.speed_deg_per_day * JULIAN_YEAR;
    assert!(
        (per_year - RATE_PER_YEAR).abs() < 0.01,
        "the mean apogee moves {per_year}° a year, not {RATE_PER_YEAR}°"
    );
    let moved = (later.tropical_deg - now.tropical_deg).rem_euclid(360.0);
    assert!(
        (moved - RATE_PER_YEAR).abs() < 0.005,
        "a year moved the mean apogee {moved}°, not {RATE_PER_YEAR}°"
    );
    assert!(!now.is_retrograde() && !later.is_retrograde());
}

/// The osculating apogee is the same apse as the mean one, swung by the
/// Sun: within forty degrees of it over a month, and away from it by more
/// than a degree somewhere, or it would not be osculating.
#[test]
fn the_osculating_apogee_swings_about_the_mean_one() {
    let sdk = context("western-tropical-default");
    let mut widest = 0.0_f64;
    for day in (0..30).step_by(3) {
        let at = J2000 + f64::from(day);
        let mean = read(&sdk, at, &greenwich().with_lilith(Apogee::Mean))
            .value
            .foundation
            .lilith
            .unwrap();
        let osculating = read(&sdk, at, &greenwich().with_lilith(Apogee::Osculating))
            .value
            .foundation
            .lilith
            .unwrap();
        let swing = apart(osculating.tropical_deg, mean.tropical_deg).abs();
        assert!(swing < 40.0, "day {day}: {swing}° from the mean apogee");
        widest = widest.max(swing);
    }
    assert!(
        widest > 1.0,
        "the osculating apogee never left the mean one by more than {widest}°"
    );
}

/// A provider that does not compute the apogee is refused on `lilith`,
/// naming it, and the chart without Lilith still founds over it.
#[test]
fn a_provider_without_the_apogee_is_refused_on_lilith() {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Test])
        .build()
        .unwrap();
    let at = JulianDay::<Utc>::try_new(J2000).unwrap();
    assert!(sdk.chart().reading(at, &kathmandu()).is_ok());
    let refused = sdk
        .chart()
        .reading(at, &kathmandu().with_lilith(Apogee::Mean))
        .unwrap_err();
    assert_eq!(refused.status, Status::Unsupported, "{refused}");
    assert_eq!(refused.field(), Some("lilith"), "{refused}");
    assert!(refused.message.contains("MEAN_APOGEE"), "{refused}");
    assert!(refused.hint().is_some(), "{refused}");
}
