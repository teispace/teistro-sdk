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

use teistro_astro::eclipse::{
    Eclipses, LunarEclipseKind, SeenAt, ShadowRule, SolarEclipseKind, SolarView,
};
use teistro_astro::{Completion, DeltaTModel, tt_of};
use teistro_core::error::Status;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{HorizonModel, OverridePolicy};
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{DiscPoint, Horizon, Refraction};

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
    // So the maximum itself is the largest phase seen.
    assert_eq!(view.seen_maximum.why, SeenAt::Maximum);
    assert_eq!(view.seen_maximum.magnitude, Some(view.magnitude));
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
    let largest = view.seen_maximum;
    assert_eq!(largest.why, SeenAt::Maximum);
    assert_eq!(largest.moment, Some(view.greatest));
    assert_eq!(largest.umbral_magnitude, Some(eclipse.umbral_magnitude));
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
    // Nothing seen is a state of its own, not a phase of zero.
    assert_eq!(view.seen_maximum.why, SeenAt::NotSeen);
    assert!(view.seen_maximum.moment.is_none() && view.seen_maximum.magnitude.is_none());
    // Tokyo, nearer the shadow's side of the Earth, sees none of it
    // either, whether or not its geometry reaches.
    let tokyo = Place::try_from_degrees(35.68, 139.69, 40.0).unwrap();
    let far = eclipses
        .solar_seen(eclipse, tokyo, &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap();
    assert!(far.is_none_or(|view| view.seen.is_none()));
}

/// The sunset and sunrise NASA's JavaScript Solar Eclipse Explorer
/// (O'Byrne and Espenak, `eclipse.gsfc.nasa.gov/JSEX/program.js`) times
/// an eclipse in progress by: the Sun's centre 0.00524 rad (18′) below the
/// geometric horizon, which is the upper limb on it with no refraction.
const NASA_HORIZON: Horizon = Horizon {
    disc: DiscPoint::UpperLimb,
    refraction: Refraction::None,
    altitude_deg: 0.0,
    model: HorizonModel::STANDARD,
};

/// One eclipse at one place, as NASA's explorer prints its largest phase
/// seen: the UT hour and the magnitude.
fn held_to_nasa(view: &SolarView, why: SeenAt, ut_hours: f64, magnitude: f64) {
    let seen = view.seen_maximum;
    assert_eq!(seen.why, why);
    let at = seen.moment.expect("seen").at.get();
    // Hours from the civil day's midnight, UT.
    let ours = (at + 0.5).fract() * 24.0;
    let off = (ours - ut_hours) * 3_600.0;
    // The explorer sets its line from the Sun's declination alone and
    // prints it 0.05° lower than the line it solved for: its times stand
    // within a minute of the line's.
    assert!(off.abs() < 60.0, "{off:+.1} s from NASA's explorer");
    let ours = seen.magnitude.expect("seen");
    assert!((ours - magnitude).abs() < 0.005, "magnitude {ours}");
}

#[test]
fn kathmandu_saw_the_2022_october_eclipse_largest_at_sunset() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .solar_between(day(2_459_877.5), day(2_459_878.5))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse that day, found {}", found.len());
    };
    // The geometric maximum: magnitude 0.490 with the Sun 5.7° down.
    let almanac = eclipses
        .solar_seen(eclipse, kathmandu(), &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap()
        .expect("Kathmandu is in the penumbra");
    assert!((almanac.magnitude - 0.490).abs() < BOUNDS.magnitude);
    assert!((almanac.maximum.altitude_deg + 5.7).abs() < 0.05);
    // Under NASA's explorer's sunset, the largest phase seen is 0.356 at
    // 11:38:23 UT (its "(s)" row), and the consumer's 0.354 (SR-106).
    let nasa = eclipses
        .solar_seen(eclipse, kathmandu(), &NASA_HORIZON)
        .unwrap()
        .unwrap();
    held_to_nasa(&nasa, SeenAt::Setting, 11.639_66, 0.3556);
    assert!((nasa.seen_maximum.magnitude.unwrap() - 0.354).abs() < 0.005);
    // Under the almanac's own horizon the eye loses the upper limb later,
    // as timeanddate's 17:25 sunset has it, and so sees more: the edge
    // of what is seen and the largest phase are the same instant.
    let seen = almanac.seen_maximum;
    assert_eq!(seen.why, SeenAt::Setting);
    let moment = seen.moment.unwrap();
    assert_eq!(Some(moment.at), almanac.seen.map(|seen| seen.to));
    assert!((moment.altitude_deg + 0.835).abs() < 0.01);
    let magnitude = seen.magnitude.unwrap();
    assert!((magnitude - 0.376).abs() < BOUNDS.magnitude, "{magnitude}");
    assert!(seen.obscuration.unwrap() < almanac.obscuration);
}

#[test]
fn washington_saw_the_2021_june_eclipse_rise_in_progress() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .solar_between(day(2_459_375.5), day(2_459_376.5))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse that day, found {}", found.len());
    };
    let washington = Place::try_from_degrees(38.8951, -77.0364, 20.0).unwrap();
    let view = eclipses
        .solar_seen(eclipse, washington, &NASA_HORIZON)
        .unwrap()
        .unwrap();
    // The explorer: in progress at sunrise, 09:45:31 UT, magnitude 0.680,
    // the maximum below the horizon.
    assert!(view.maximum.altitude_deg < 0.0);
    held_to_nasa(&view, SeenAt::Rising, 9.758_52, 0.6804);
    assert_eq!(
        view.seen_maximum.moment.map(|moment| moment.at),
        view.seen.map(|seen| seen.from)
    );
}

#[test]
fn delhi_saw_the_2019_december_maximum_itself() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .solar_between(day(2_458_843.5), day(2_458_844.5))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one solar eclipse that day, found {}", found.len());
    };
    let delhi = Place::try_from_degrees(28.6139, 77.209, 216.0).unwrap();
    let view = eclipses
        .solar_seen(eclipse, delhi, &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap()
        .unwrap();
    // The explorer: the maximum at 04:00:48 UT, magnitude 0.555, the Sun
    // 23.7° up. Seen, it is the geometric maximum to the bit.
    held_to_nasa(&view, SeenAt::Maximum, 4.013_33, 0.5547);
    let seen = view.seen_maximum;
    assert_eq!(seen.moment, Some(view.maximum));
    assert_eq!(seen.magnitude, Some(view.magnitude));
    assert_eq!(seen.obscuration, Some(view.obscuration));
}

#[test]
fn kathmandu_saw_the_2022_november_lunar_eclipse_rise_in_totality() {
    let provider = Builtin::new();
    let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DELTA_T);
    let eclipses = Eclipses::new(&sky, DELTA_T);
    let found = eclipses
        .lunar_between(day(2_459_891.5), day(2_459_892.5))
        .unwrap();
    let [eclipse] = found.as_slice() else {
        panic!("one lunar eclipse that day, found {}", found.len());
    };
    let view = eclipses
        .lunar_seen(eclipse, kathmandu(), &Horizon::UPPER_LIMB_REFRACTION)
        .unwrap();
    // The greatest eclipse fell with the Moon 6.4° down; it rose at
    // 17:11 NPT, 11:26 UT (timeanddate), still in totality.
    assert!(view.greatest.altitude_deg < -6.0);
    let seen = view.seen_maximum;
    assert_eq!(seen.why, SeenAt::Rising);
    let moment = seen.moment.unwrap();
    assert_eq!(Some(moment.at), view.seen.map(|seen| seen.from));
    let rose = ((moment.at.get() + 0.5).fract() * 24.0 - (11.0 + 26.0 / 60.0)) * 60.0;
    assert!((0.0..1.0).contains(&rose), "{rose:+.2} min from 11:26 UT");
    assert!(
        view.u2.unwrap().at.get() < moment.at.get() && moment.at.get() < view.u3.unwrap().at.get()
    );
    // So the largest phase seen is total, and less than the greatest.
    let umbral = seen.umbral_magnitude.unwrap();
    assert!(
        1.0 < umbral && umbral < eclipse.umbral_magnitude,
        "{umbral}"
    );
    assert!(seen.penumbral_magnitude.unwrap() < eclipse.penumbral_magnitude);
}
