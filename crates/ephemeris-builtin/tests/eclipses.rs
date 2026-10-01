//! The eclipse search over the built-in sky: a known eclipse of each
//! kind, the shadow rule's effect, and the edges of a window.
//!
//! The whole canon is measured in `03-design/eclipses-measured.md`; these
//! are the few answers a reader can check by eye, held here so a change
//! to the search fails a unit test before it fails the page. The
//! catalogue's times are Terrestrial Time, so each is compared through
//! the same Delta T the search ran under.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "a test fails by panicking"
)]

use teistro_astro::eclipse::{Eclipses, LunarEclipseKind, ShadowRule, SolarEclipseKind};
use teistro_astro::{Completion, DeltaTModel, tt_of};
use teistro_core::error::Status;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::Horizon;

const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

/// The seconds, magnitude and degrees an eclipse is held to on this tier.
/// The compact tier's Moon is good to a minute of arc (ADR-0027), which
/// moves a greatest moment by seconds; the others are the canon's.
#[cfg(feature = "compact")]
const BOUNDS: Bounds = Bounds {
    seconds: 15.0,
    magnitude: 0.005,
    degrees: 0.5,
};
#[cfg(not(feature = "compact"))]
const BOUNDS: Bounds = Bounds {
    seconds: 5.0,
    magnitude: 0.001,
    degrees: 0.5,
};

struct Bounds {
    seconds: f64,
    magnitude: f64,
    degrees: f64,
}

/// NASA's greatest eclipse, Terrestrial Time: the total lunar eclipse of
/// 2025-09-07 18:12:58 and the total solar eclipse of 2024-04-08
/// 18:18:29.
const LUNAR_2025_TT: f64 = 2_460_926.259_004_63;
const SOLAR_2024_TT: f64 = 2_460_409.262_835_65;

fn seconds_from(at: JulianDay<Ut1>, tt: f64) -> f64 {
    (tt_of(at, DELTA_T).unwrap().0.get() - tt) * 86_400.0
}

fn day(jd: f64) -> JulianDay<Ut1> {
    JulianDay::literal(jd)
}

#[test]
fn the_total_lunar_eclipse_of_2025_september_is_found_to_the_second() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let found = Eclipses::new(&sky, DELTA_T)
        .lunar_between(day(LUNAR_2025_TT - 2.0), day(LUNAR_2025_TT + 2.0))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one lunar eclipse in four days, found {}", found.len());
    };
    assert_eq!(eclipse.kind, LunarEclipseKind::Total);
    assert_eq!(eclipse.shadow, ShadowRule::Danjon);
    let off = seconds_from(eclipse.greatest, LUNAR_2025_TT);
    assert!(
        off.abs() < BOUNDS.seconds,
        "greatest eclipse {off:+.1} s from NASA's"
    );
    // NASA: umbral magnitude 1.3619, totality 82.1 minutes.
    assert!(
        (eclipse.umbral_magnitude - 1.3619).abs() < BOUNDS.magnitude,
        "{}",
        eclipse.umbral_magnitude
    );
    let totality = eclipse.durations()[0].expect("a total eclipse has totality") * 1440.0;
    assert!((totality - 82.1).abs() < 1.0, "totality {totality:.1} min");
}

#[test]
fn the_total_solar_eclipse_of_2024_april_is_greatest_over_mexico() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let found = Eclipses::new(&sky, DELTA_T)
        .solar_between(day(SOLAR_2024_TT - 2.0), day(SOLAR_2024_TT + 2.0))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse in four days, found {}", found.len());
    };
    assert_eq!(eclipse.kind, SolarEclipseKind::Total);
    let off = seconds_from(eclipse.greatest, SOLAR_2024_TT);
    assert!(
        off.abs() < BOUNDS.seconds,
        "greatest eclipse {off:+.1} s from NASA's"
    );
    // NASA: magnitude 1.0566, greatest at 25.3°N 104.1°W.
    assert!(
        (eclipse.magnitude - 1.0566).abs() < BOUNDS.magnitude,
        "{}",
        eclipse.magnitude
    );
    assert!((eclipse.point.latitude.get() - 25.3).abs() < BOUNDS.degrees);
    assert!((eclipse.point.longitude.get() + 104.1).abs() < BOUNDS.degrees);
}

#[test]
fn chauvenets_shadow_is_the_larger() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let window = (day(LUNAR_2025_TT - 2.0), day(LUNAR_2025_TT + 2.0));
    let under = |rule| {
        Eclipses::new(&sky, DELTA_T)
            .with_shadow(rule)
            .lunar_between(window.0, window.1)
            .unwrap()
            .into_iter()
            .next()
            .expect("the eclipse under either rule")
    };
    let danjon = under(ShadowRule::Danjon);
    let chauvenet = under(ShadowRule::Chauvenet);
    assert_eq!(chauvenet.shadow, ShadowRule::Chauvenet);
    assert!(chauvenet.umbral_magnitude > danjon.umbral_magnitude);
    assert!(chauvenet.penumbral_magnitude > danjon.penumbral_magnitude);
    // The same eclipse: the rule moves the shadow's size, not its moment.
    assert!((chauvenet.greatest.get() - danjon.greatest.get()).abs() * 86_400.0 < 1.0);
}

#[test]
fn a_month_without_an_eclipse_is_empty_and_a_backward_window_is_refused() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    // October 2025 falls between September's pair and February's.
    let (from, to) = (day(2_460_949.5), day(2_460_980.5));
    assert!(
        eclipses.lunar_between(from, to).unwrap().is_empty(),
        "{:?}",
        eclipses.lunar_between(from, to).unwrap()
    );
    assert!(
        eclipses.solar_between(from, to).unwrap().is_empty(),
        "{:?}",
        eclipses.solar_between(from, to).unwrap()
    );
    let backward = eclipses.lunar_between(to, from).unwrap_err();
    assert_eq!(backward.status, Status::InvalidArg);
}

#[test]
fn the_next_lunar_eclipse_after_september_2025_is_march_2026s() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let next = Eclipses::new(&sky, DELTA_T)
        .next_lunar(day(LUNAR_2025_TT + 1.0))
        .unwrap();
    // NASA: 2026-03-03 11:34:52 TT, total.
    let march = 2_461_102.982_546_3;
    assert_eq!(next.kind, LunarEclipseKind::Total);
    let off = seconds_from(next.greatest, march);
    assert!(
        off.abs() < BOUNDS.seconds,
        "greatest eclipse {off:+.1} s from NASA's"
    );
}

/// Kathmandu as NASA's bulletin for 2009 July 22 places it.
fn kathmandu() -> Place {
    Place::try_from_degrees(27.0 + 43.0 / 60.0, 85.0 + 19.0 / 60.0, 1348.0).unwrap()
}

#[test]
fn kathmandu_saw_the_2009_eclipse_partial_as_the_bulletin_prints_it() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .solar_between(day(2_455_034.0), day(2_455_035.0))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse that day, found {}", found.len());
    };
    assert_eq!(eclipse.kind, SolarEclipseKind::Total);
    let view = eclipses
        .solar_seen(eclipse, kathmandu(), &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap()
        .expect("Kathmandu is in the penumbra");
    // The bulletin: partial, magnitude 0.962; first contact 00:01:10.9,
    // the maximum 00:57:43.9 and the last contact 02:00:31.5, UT.
    assert_eq!(view.kind, SolarEclipseKind::Partial);
    assert!(view.second.is_none() && view.third.is_none());
    assert!(
        (view.magnitude - 0.962).abs() < BOUNDS.magnitude,
        "{}",
        view.magnitude
    );
    for (ours, printed) in [
        (view.first.at, 2_455_034.500_820_602),
        (view.maximum.at, 2_455_034.540_091_435),
        (view.fourth.at, 2_455_034.583_697_917),
    ] {
        let off = (ours.get() - printed) * 86_400.0;
        assert!(off.abs() < BOUNDS.seconds, "{off:+.1} s from the bulletin");
    }
    // Just after sunrise, so seen from the first contact to the last.
    assert!(view.first.altitude_deg > 0.0 && view.first.altitude_deg < 6.0);
    let seen = view.seen.expect("the Sun was up");
    assert_eq!((seen.from, seen.to), (view.first.at, view.fourth.at));
}

#[test]
fn kathmandu_saw_the_whole_of_the_2025_lunar_eclipse() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .lunar_between(day(LUNAR_2025_TT - 1.0), day(LUNAR_2025_TT + 1.0))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one lunar eclipse, found {}", found.len());
    };
    let view = eclipses
        .lunar_seen(eclipse, kathmandu(), &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap();
    // The Moon rose before the penumbra touched it and set after it left,
    // and stood high at the greatest eclipse, near local midnight.
    assert!(view.p1.altitude_deg > 0.0 && view.p4.altitude_deg > 0.0);
    assert!(view.greatest.altitude_deg > 40.0);
    assert!(view.u2.is_some() && view.u3.is_some());
    let seen = view.seen.expect("the Moon was up");
    assert_eq!((seen.from, seen.to), (view.p1.at, view.p4.at));
    // And through the umbral phase, the part the eye sees.
    let umbral = view.umbral_seen.expect("the Moon was up");
    let (u1, u4) = (view.u1.unwrap().at, view.u4.unwrap().at);
    assert_eq!((umbral.from, umbral.to), (u1, u4));
}

#[test]
fn a_city_on_the_night_side_has_its_contacts_and_sees_none_of_them() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .solar_between(day(SOLAR_2024_TT - 1.0), day(SOLAR_2024_TT + 1.0))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse, found {}", found.len());
    };
    // 2024 April's eclipse was North America's and Kathmandu was on the
    // night side, where the penumbra reaches only through the Earth: the
    // contacts are there, every one below the horizon, and none is seen.
    let view = eclipses
        .solar_seen(eclipse, kathmandu(), &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap()
        .expect("the geometry reaches through the Earth");
    assert!(view.first.altitude_deg < 0.0 && view.maximum.altitude_deg < 0.0);
    assert!(view.fourth.altitude_deg < 0.0);
    assert!(view.seen.is_none());
    // Tokyo, nearer the shadow's side of the Earth, sees none of it
    // either, whether or not its geometry reaches.
    let tokyo = Place::try_from_degrees(35.68, 139.69, 40.0).unwrap();
    let far = eclipses
        .solar_seen(eclipse, tokyo, &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap();
    assert!(far.is_none_or(|view| view.seen.is_none()));
}
