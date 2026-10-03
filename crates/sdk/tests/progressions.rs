//! Progressions and directions through the façade, held to Leo's own
//! nativity and his worked figures in *The Progressed Horoscope* (1906)
//! (`docs/03-design/western-progressions.md`).

#![allow(clippy::panic, clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro::catalogue::Graha;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::western::{Rate, YearMeasure};
use teistro::{
    AngleMethod, ArcMeasure, ChartRequest, ContactRequest, Context, DirectionArc, Document,
    Ephemeris, NatalPoint, Progression, ProgressionRequest, UtcOffset,
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

/// 0h UT on the first of a Gregorian month.
fn first_of(year: i32, month: u8) -> f64 {
    teistro_calendar::gregorian::fixed_from_gregorian(year, month, 1)
        .jd_at_midnight()
        .unwrap()
        .get()
}

#[test]
fn leos_contact_falls_due_on_each_measures_day() {
    let sdk = context();
    let born = birth(&sdk);
    let asked = ContactRequest::default()
        .with_grahas([Graha::Moon])
        .with_points([NatalPoint::Graha {
            graha: Graha::Mercury,
        }])
        .with_aspects([135]);
    // "The sesquiquadrate of the Moon to Mercury" (p. 305): the 21st of
    // October 1906 by a tropical year, the 22nd by Leo's sidereal time.
    for (progression, day) in [(Progression::SECONDARY, 21.0), (Progression::LEO, 22.0)] {
        let found = sdk
            .chart()
            .progressed_contacts(
                &born,
                jd(first_of(1906, 10)),
                jd(first_of(1906, 11)),
                &asked.clone().with_progression(progression),
            )
            .unwrap();
        let [contact] = found.as_slice() else {
            panic!("{found:?}");
        };
        assert_eq!((contact.graha, contact.angle), (Graha::Moon, 135));
        // The sky's instant is his 10.40 a.m. to within two minutes.
        assert!(
            (contact.sky.get() - CONTACT).abs() < 2.0 / 1440.0,
            "{}",
            contact.sky.get()
        );
        let into_month = contact.life.get() - first_of(1906, 10);
        assert!(
            (day - 1.0..day).contains(&into_month),
            "{progression:?}: {into_month}"
        );
    }
}

#[test]
fn leos_lunar_year_is_found_month_by_month() {
    let sdk = context();
    let born = birth(&sdk);
    let moon = ContactRequest::default().with_grahas([Graha::Moon]);
    let found = sdk
        .chart()
        .progressed_contacts(&born, jd(first_of(1906, 10)), jd(first_of(1908, 1)), &moon)
        .unwrap();
    // His lunar list (p. 41), the contacts to the seven planets. He dates
    // them by counting a month for each degree the Moon passes, which runs
    // up to a month late: the exact contact falls in his month or in the
    // month before (Saturn on 23 May, the Sun on 28 July).
    let printed = [
        (Graha::Mercury, 135, (1906, 10)),
        (Graha::Jupiter, 150, (1907, 1)),
        (Graha::Saturn, 135, (1907, 6)),
        (Graha::Sun, 150, (1907, 8)),
        (Graha::Venus, 180, (1907, 11)),
    ];
    for (graha, angle, (year, month)) in printed {
        let contact = found
            .iter()
            .find(|c| c.to == NatalPoint::Graha { graha } && c.angle == angle)
            .unwrap_or_else(|| panic!("{graha:?} {angle} in {found:?}"));
        let next = if month == 12 {
            (year + 1, 1)
        } else {
            (year, month + 1)
        };
        let before = if month == 1 {
            (year - 1, 12)
        } else {
            (year, month - 1)
        };
        assert!(
            (first_of(before.0, before.1)..first_of(next.0, next.1)).contains(&contact.life.get()),
            "{graha:?} {angle} at {}",
            contact.life.get()
        );
    }
    // In his order, as each falls due.
    assert!(found.windows(2).all(|pair| match pair {
        [a, b] => a.life.get() <= b.life.get(),
        _ => true,
    }));
}

#[test]
fn leos_solar_contacts_fall_in_his_forty_seventh_year() {
    let sdk = context();
    let born = birth(&sdk);
    let sun = NatalPoint::Graha { graha: Graha::Sun };
    let asked = ContactRequest::default()
        .with_grahas([Graha::Sun, Graha::Mercury])
        .with_points([sun]);
    // "☉ ∠ ☉" and "☿ ∠ ☉", progressed to radical (p. 41), in the year from
    // his birthday in 1906.
    let year = (BIRTH + 46.0 * 365.242_189, BIRTH + 47.0 * 365.242_189);
    let found = sdk
        .chart()
        .progressed_contacts(&born, jd(year.0), jd(year.1), &asked)
        .unwrap();
    for graha in [Graha::Sun, Graha::Mercury] {
        assert!(
            found
                .iter()
                .any(|c| c.graha == graha && c.to == sun && c.angle == 45),
            "{graha:?} in {found:?}"
        );
    }
}

#[test]
fn a_contact_window_that_runs_backwards_is_named() {
    let sdk = context();
    let born = birth(&sdk);
    let error = sdk
        .chart()
        .progressed_contacts(
            &born,
            jd(first_of(1907, 1)),
            jd(first_of(1906, 1)),
            &ContactRequest::default(),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("contacts.to"));
    let error = sdk
        .chart()
        .progressed_contacts(
            &born,
            jd(first_of(1906, 1)),
            jd(first_of(1907, 1)),
            &ContactRequest::default().with_aspects([200]),
        )
        .unwrap_err();
    assert_eq!(error.field(), Some("contacts.aspects"));
}
