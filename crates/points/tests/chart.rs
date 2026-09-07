//! Every derived point of a founded chart, end to end over the analytic
//! test provider.
//!
//! The rules are held to the corpus in `baseline.rs`. What only the
//! assembled value can show is that the parts agree with each other —
//! that a point's sign is the sign of its own longitude, that a family
//! holds what the catalogue says it does, and that the entry point
//! refuses what it cannot answer.
//!
//! And, as `aspect` does, that **every shipped profile founds a chart
//! and computes its points**: `crates/state` shipped with the SDK's own
//! default profile naming a table nothing resolved (registry entry 23),
//! and the cheapest guard against that class of failure is to try each
//! profile rather than only the default.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    reason = "tests fail by panicking, index their own results and print counts under --nocapture"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Point, PointFamily, Rashi, Vara};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{
    OverridePolicy, Profile, SHIPPED_PROFILES, Settings, SettingsPatch, Sunrise,
};
use teistro_core::time::UtcOffset;
use teistro_points::eighth::{self, EIGHTHS};
use teistro_points::{Points, solar};
use teistro_port_ephemeris::test_provider::TestProvider;

/// Which sign a longitude falls in, 0 for Aries.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a normalised longitude over thirty is 0 to 11"
)]
fn sign_index(longitude_deg: f64) -> u16 {
    (longitude_deg.rem_euclid(360.0) / 30.0) as u16
}

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// An ascendant that turns once through the day: enough to tell one
/// instant from another, and no ephemeris.
///
/// It returns a `Result` because that is the trait's shape, and the
/// trait's shape is what a real implementation needs.
#[expect(
    clippy::unnecessary_wraps,
    reason = "the `Ascendant` trait's own signature"
)]
fn rising(at: JulianDay<Utc>) -> Result<f64, Error> {
    Ok((at.get() * 360.0).rem_euclid(360.0))
}

fn founded_on(profile: &str) -> Result<(ChartFoundation, Settings), Error> {
    let provider = TestProvider;
    let resolved = Profile::shipped(profile)
        .unwrap_or_else(|| panic!("the profile `{profile}`"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{profile}: {e}"));
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let foundation = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .found_one(
        JulianDay::<Utc>::literal(2_460_482.5),
        &place(),
        ChartKind::Natal,
    )?
    .value;
    Ok((foundation, resolved.settings))
}

fn founded() -> ChartFoundation {
    founded_on(teistro_core::settings::DEFAULT_PROFILE)
        .expect("the default profile founds")
        .0
}

#[test]
fn every_profile_the_provider_can_found_computes_its_points() {
    let mut computed = Vec::new();
    let mut deferred = Vec::new();
    for profile in SHIPPED_PROFILES {
        match founded_on(profile) {
            Ok((foundation, _)) => {
                let points = Points::from_longitudes(&foundation)
                    .unwrap_or_else(|e| panic!("{profile}: {e}"));
                assert_eq!(points.all().len(), 11, "{profile}");
                computed.push(profile);
            }
            Err(error) => {
                assert_eq!(
                    error.status,
                    Status::Unsupported,
                    "{profile} failed for a reason that is not a provider's limit: {error}"
                );
                deferred.push(profile);
            }
        }
    }
    println!("computed under {computed:?}; {deferred:?} ask more of the provider");
    assert!(computed.contains(&teistro_core::settings::DEFAULT_PROFILE));
    assert_eq!(computed.len() + deferred.len(), SHIPPED_PROFILES.len());
}

#[test]
fn every_point_of_a_founded_chart_agrees_with_itself() {
    let foundation = founded();
    let points = Points::from_longitudes(&foundation).expect("a founded chart");
    assert_eq!(points.all().len(), 11, "five cast, four driven, two yogi");
    for derived in points.all() {
        // The sign is the sign of its own longitude.
        assert_eq!(
            Some(derived.sign),
            Rashi::from_id(sign_index(derived.longitude_deg)),
            "{:?}",
            derived.point
        );
        assert!((0.0..360.0).contains(&derived.longitude_deg));
        assert!((0.0..30.0).contains(&derived.in_sign_deg()));
        // And its boundary distance is a fact about that longitude.
        assert!(derived.boundaries.sign_deg >= 0.0);
        assert!(derived.boundaries.sign_deg <= 15.0);
        assert!(!derived.is_firm(15.0));
        // Every one is findable by its own key.
        assert_eq!(points.at(derived.point), Some(derived));
    }
    println!(
        "{} points: {} cast by the Sun, {} special lagnas, {} sphutas",
        points.all().len(),
        points.family(PointFamily::UpagrahaSolar).count(),
        points.family(PointFamily::SpecialLagna).count(),
        points.family(PointFamily::Sphuta).count(),
    );
}

#[test]
fn the_families_partition_what_the_chart_holds() {
    let foundation = founded();
    let points = Points::from_longitudes(&foundation).expect("a founded chart");
    let mut counted = 0;
    for family in [
        PointFamily::UpagrahaSolar,
        PointFamily::UpagrahaDay,
        PointFamily::SpecialLagna,
        PointFamily::Sphuta,
    ] {
        for derived in points.family(family) {
            assert_eq!(derived.point.attributes().family, family);
            counted += 1;
        }
    }
    assert_eq!(counted, points.all().len(), "every point has one family");
    assert_eq!(points.family(PointFamily::UpagrahaSolar).count(), 5);
    assert_eq!(
        points.family(PointFamily::UpagrahaDay).count(),
        0,
        "the two the day divides need an ascendant"
    );
    // And a sign holds whatever falls in it.
    for sign in Rashi::ALL {
        for derived in points.in_sign(sign) {
            assert_eq!(derived.sign, sign);
        }
    }
    assert_eq!(
        Rashi::ALL
            .into_iter()
            .map(|sign| points.in_sign(sign).count())
            .sum::<usize>(),
        points.all().len()
    );
}

#[test]
fn the_two_the_day_divides_join_the_rest_when_an_ascendant_is_given() {
    let foundation = founded();
    let arc = Interval::literal(2_460_482.25, 2_460_482.75);
    let with =
        Points::of(&foundation, Vara::ALL[0], arc, true, &rising).expect("an arc with eighths");
    let without = Points::from_longitudes(&foundation).expect("a founded chart");
    assert_eq!(with.all().len(), without.all().len() + 2);
    assert_eq!(with.family(PointFamily::UpagrahaDay).count(), 2);
    let gulika = with.at(Point::Gulika).expect("Gulika");
    let mandi = with.at(Point::Mandi).expect("Mandi");
    // They are one eighth apart in time, so the ascendant has turned.
    let portion = eighth::saturns(arc, Vara::ALL[0], true).expect("a portion");
    assert!((gulika.longitude_deg - rising(portion.at.from).unwrap()).abs() < 1e-9);
    assert!((mandi.longitude_deg - rising(portion.at.to).unwrap()).abs() < 1e-9);
    assert!(
        (portion.at.days() - arc.days() / f64::from(EIGHTHS)).abs() < 1e-12,
        "one eighth of the arc"
    );
    // Everything the longitudes decide is unchanged by adding them.
    for derived in without.all() {
        assert_eq!(with.at(derived.point), Some(derived));
    }
}

#[test]
fn a_house_is_counted_from_the_lagnas_own_sign() {
    let foundation = founded();
    let points = Points::from_longitudes(&foundation).expect("a founded chart");
    for derived in points.all() {
        let house = points
            .house_of(derived.point, &foundation)
            .expect("a placed point");
        assert!((1..=12).contains(&house), "{:?}: {house}", derived.point);
        // The lagna's own sign is the first house, so a point there is
        // in the first.
        if derived.sign as u16 == sign_index(foundation.lagna_deg) {
            assert_eq!(house, 1, "{:?}", derived.point);
        }
    }
    // A point the chart does not hold has no house.
    assert_eq!(points.house_of(Point::Gulika, &foundation), None);
}

#[test]
fn a_chart_missing_a_luminary_is_refused_by_field() {
    let mut foundation = founded();
    foundation
        .grahas
        .retain(|position| position.graha != teistro_core::catalogue::Graha::Moon);
    let error = Points::from_longitudes(&foundation).expect_err("no Moon");
    assert_eq!(error.field(), Some("points.foundation"));
    assert!(error.message.contains("MOON"), "{error}");
}

#[test]
fn two_runs_of_the_same_chart_are_the_same_value() {
    let foundation = founded();
    let once = Points::from_longitudes(&foundation).expect("a founded chart");
    let twice = Points::from_longitudes(&foundation).expect("a founded chart");
    assert_eq!(once, twice, "the determinism contract");
    let json = serde_json::to_string(&once).expect("a serialisable value");
    assert!(json.contains("DHUMA"), "{json:.200}");
    // And the chain's directions are a property of the chain, not of a
    // chart: the doc's claim, checked where a reader will look.
    assert_eq!(solar::advances(Point::Vyatipata), Some(false));
    assert_eq!(solar::advances(Point::Indrachapa), Some(true));
}
