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

use teistro_astro::eclipse::{Eclipses, LunarKind, ShadowRule, SolarKind};
use teistro_astro::{Completion, DeltaTModel, tt_of};
use teistro_core::error::Status;
use teistro_core::quantity::{JulianDay, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_builtin::provider::Builtin;

const DELTA_T: DeltaTModel = DeltaTModel::TableThenModel;

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
    assert_eq!(eclipse.kind, LunarKind::Total);
    assert_eq!(eclipse.shadow, ShadowRule::Danjon);
    let off = seconds_from(eclipse.greatest, LUNAR_2025_TT);
    assert!(off.abs() < 5.0, "greatest eclipse {off:+.1} s from NASA's");
    // NASA: umbral magnitude 1.3619, totality 82.1 minutes.
    assert!((eclipse.umbral_magnitude - 1.3619).abs() < 0.001);
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
    assert_eq!(eclipse.kind, SolarKind::Total);
    let off = seconds_from(eclipse.greatest, SOLAR_2024_TT);
    assert!(off.abs() < 5.0, "greatest eclipse {off:+.1} s from NASA's");
    // NASA: magnitude 1.0566, greatest at 25.3°N 104.1°W.
    assert!((eclipse.magnitude - 1.0566).abs() < 0.001);
    assert!((eclipse.point.latitude.get() - 25.3).abs() < 0.5);
    assert!((eclipse.point.longitude.get() + 104.1).abs() < 0.5);
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
    assert!(eclipses.lunar_between(from, to).unwrap().is_empty());
    assert!(eclipses.solar_between(from, to).unwrap().is_empty());
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
    assert_eq!(next.kind, LunarKind::Total);
    let off = seconds_from(next.greatest, march);
    assert!(off.abs() < 5.0, "greatest eclipse {off:+.1} s from NASA's");
}
