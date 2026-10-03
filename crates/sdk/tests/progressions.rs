//! Progressions and directions through the façade, held to Leo's own
//! nativity and his worked figures in *The Progressed Horoscope* (1906)
//! (`docs/03-design/western-progressions.md`).

#![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::western::{Rate, YearMeasure};
use teistro::{
    AngleMethod, ArcMeasure, ChartRequest, Context, DirectionArc, Document, Ephemeris, Progression,
    ProgressionRequest, UtcOffset,
};

/// 7 August 1860, 5.49 a.m. at London (p. 305). Leo reckons it as
/// Greenwich time and his sidereal time at birth as Greenwich's, so the
/// place is on the Greenwich meridian.
const BIRTH: f64 = 2_400_629.742_361_111;
/// 10.40 a.m. on 22 September 1860, when the progressed Moon is
/// sesquiquadrate the radical Mercury (p. 304).
const CONTACT: f64 = 2_400_675.944_444_444;

fn context() -> Context {
    Context::builder()
        .profile("western-tropical-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .unwrap()
}

fn request() -> ChartRequest {
    let london = Place::new(
        Latitude::literal(51.5),
        Longitude::literal(0.0),
        Altitude::literal(0.0),
    );
    ChartRequest::at(london, UtcOffset::UTC)
}

fn jd(value: f64) -> JulianDay<Utc> {
    JulianDay::try_new(value).unwrap()
}

fn birth(sdk: &Context) -> Document {
    sdk.chart().reading(jd(BIRTH), &request()).unwrap().value
}

fn tropical(chart: &Document, graha: Graha) -> f64 {
    chart
        .foundation
        .grahas
        .iter()
        .find(|at| at.graha == graha)
        .unwrap()
        .tropical_deg
}

/// The instant of life a day for a tropical year matches to `sky`.
fn life_of(sky: f64) -> JulianDay<Utc> {
    Progression::SECONDARY.life_at(jd(BIRTH), jd(sky)).unwrap()
}

/// Hours of sidereal time, as Leo prints them.
fn hours(h: f64, m: f64, s: f64) -> f64 {
    h + m / 60.0 + s / 3600.0
}

#[test]
fn leos_progressed_map_is_cast_at_his_sidereal_time() {
    let sdk = context();
    let born = birth(&sdk);
    let forty_sixth = life_of(BIRTH + 46.0);
    let asked = ProgressionRequest::default();
    let progressed = sdk
        .chart()
        .progressed(&born, forty_sixth, &asked, &request())
        .unwrap();
    assert!((progressed.sky.get() - (BIRTH + 46.0)).abs() < 1e-9);
    // "Gives S.T. for Progressed Horoscope 5 54 16" (p. 35). The SDK's
    // sidereal time is the apparent one, which the equation of the
    // equinoxes moves by about a second from Leo's mean time.
    let printed = hours(5.0, 54.0, 16.0);
    assert!(
        (progressed.armc_deg / 15.0 - printed).abs() < 2.0 / 3600.0,
        "{}",
        progressed.armc_deg / 15.0
    );
    // On the birth's own clock time the chart at the instant is the same
    // meridian: Leo's map is the quotidian of a whole day.
    let quotidian = sdk
        .chart()
        .progressed(
            &born,
            forty_sixth,
            &asked.with_angles(AngleMethod::Quotidian),
            &request(),
        )
        .unwrap();
    assert!((quotidian.armc_deg - progressed.armc_deg).abs() < 0.001);
    assert!((quotidian.angles.midheaven_deg - progressed.angles.midheaven_deg).abs() < 0.001);
}

#[test]
fn leos_moon_and_mercury_are_where_he_prints_them() {
    let sdk = context();
    let born = birth(&sdk);
    let minute = 1.0 / 60.0;
    // Mercury at birth, Leo 20° 11′ (p. 304).
    let mercury = tropical(&born, Graha::Mercury);
    assert!(
        (mercury - (120.0 + 20.0 + 11.0 * minute)).abs() < minute,
        "{mercury}"
    );
    // The Moon at Greenwich noon on 21 and 22 September 1860: Sagittarius
    // 22° 58′ and Capricorn 5° 54′.
    let asked = ProgressionRequest::default();
    for (sky, printed) in [
        (2_400_675.0, 240.0 + 22.0 + 58.0 * minute),
        (2_400_676.0, 270.0 + 5.0 + 54.0 * minute),
    ] {
        let progressed = sdk
            .chart()
            .progressed(&born, life_of(sky), &asked, &request())
            .unwrap();
        let moon = tropical(&progressed.chart.value, Graha::Moon);
        assert!((moon - printed).abs() < minute, "{moon} against {printed}");
    }
    // At 10.40 a.m. she is 135° from Mercury, to within his minute.
    let at_contact = sdk
        .chart()
        .progressed(&born, life_of(CONTACT), &asked, &request())
        .unwrap();
    let moon = tropical(&at_contact.chart.value, Graha::Moon);
    assert!(
        (moon - mercury - 135.0).abs() < 2.0 * minute,
        "{}",
        moon - mercury
    );
}

#[test]
fn every_angle_method_moves_the_meridian_as_it_says() {
    let sdk = context();
    let born = birth(&sdk);
    let natal = sdk.chart().angles(&born).unwrap();
    // Forty years and four months: off the birth's clock time, so the
    // quotidian is not Leo's map.
    let sky_days = 40.33;
    let life = life_of(BIRTH + sky_days);
    let by = |method| {
        sdk.chart()
            .progressed(
                &born,
                life,
                &ProgressionRequest::default().with_angles(method),
                &request(),
            )
            .unwrap()
    };
    let solar = by(AngleMethod::SolarArcLongitude);
    let arc = tropical(&solar.chart.value, Graha::Sun) - tropical(&born, Graha::Sun);
    let moved = (solar.angles.midheaven_deg - natal.midheaven_deg).rem_euclid(360.0);
    assert!((moved - arc).abs() < 1e-6, "{moved} against {arc}");
    let naibod = by(AngleMethod::NaibodLongitude);
    let moved = (naibod.angles.midheaven_deg - natal.midheaven_deg).rem_euclid(360.0);
    assert!((moved - sky_days * 360.0 / teistro::western::TROPICAL_YEAR_DAYS).abs() < 1e-6);
    // Every method puts the midheaven somewhere else.
    let methods = [
        AngleMethod::NaibodRightAscension,
        AngleMethod::NaibodLongitude,
        AngleMethod::SolarArcLongitude,
        AngleMethod::SolarArcRightAscension,
        AngleMethod::Quotidian,
    ];
    let midheavens: Vec<f64> = methods
        .iter()
        .map(|method| by(*method).angles.midheaven_deg)
        .collect();
    for (i, one) in midheavens.iter().enumerate() {
        for other in midheavens.iter().skip(i + 1) {
            assert!((one - other).abs() > 0.01, "{midheavens:?}");
        }
    }
}

#[test]
fn a_direction_moves_every_point_by_one_arc() {
    let sdk = context();
    let born = birth(&sdk);
    let natal = sdk.chart().angles(&born).unwrap();
    let life = life_of(BIRTH + 46.0);
    let solar = sdk
        .chart()
        .directed(&born, life, &DirectionArc::default(), &request())
        .unwrap();
    // The directed Sun is the progressed Sun.
    let progressed = sdk
        .chart()
        .progressed(&born, life, &ProgressionRequest::default(), &request())
        .unwrap();
    let sun = solar
        .planets
        .iter()
        .find(|at| at.graha == Graha::Sun)
        .unwrap();
    let progressed_sun = progressed
        .chart
        .value
        .foundation
        .grahas
        .iter()
        .find(|at| at.graha == Graha::Sun)
        .unwrap();
    assert!((sun.longitude_deg - progressed_sun.longitude_deg).abs() < 1e-9);
    assert!(
        ((solar.midheaven_deg - natal.midheaven_deg).rem_euclid(360.0) - solar.arc_deg).abs()
            < 1e-9
    );
    assert_eq!(solar.planets.len(), born.foundation.grahas.len());
    // Naibod's arc needs no ephemeris: 46 years of 0° 59′ 8″.
    let naibod = sdk
        .chart()
        .directed(
            &born,
            life,
            &DirectionArc::Measure(ArcMeasure::Naibod),
            &request(),
        )
        .unwrap();
    assert!((naibod.arc_deg - 46.0 * 360.0 / teistro::western::TROPICAL_YEAR_DAYS).abs() < 1e-6);
}

#[test]
fn a_refused_measure_is_named_under_the_call() {
    let sdk = context();
    let born = birth(&sdk);
    let lunar_leo = ProgressionRequest::default()
        .with_year(YearMeasure::NoonSiderealTime)
        .with_rate(Rate::MINOR);
    let error = sdk
        .chart()
        .progressed(&born, jd(BIRTH + 10_000.0), &lunar_leo, &request())
        .unwrap_err();
    assert_eq!(error.field(), Some("progression.year"));
    let error = sdk
        .chart()
        .directed(
            &born,
            jd(BIRTH + 10_000.0),
            &DirectionArc::Measure(ArcMeasure::PerYear(0.0)),
            &request(),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("direction.arc"));
}
