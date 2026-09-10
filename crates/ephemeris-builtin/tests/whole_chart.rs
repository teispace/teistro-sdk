//! **Phase 3's exit, demonstrated: a full chart from nothing but the
//! SDK.**
//!
//! Everything else in this crate tests the provider against the port's
//! contract. This tests the thing the phase exists for — that a consumer
//! who installs the SDK and nothing else can cast a chart. No data files,
//! no network, no engine, and no licence beyond Apache-2.0.
//!
//! It goes through the layers a real caller goes through rather than
//! round the side of them: the built-in provider, the astronomy layer's
//! completion for precession, the apparent-place corrections and the
//! sidereal frame, the drik solar model for sunrise, the Gregorian
//! calendar, and the chart foundation. A test that called the provider
//! directly would prove the provider works and say nothing about whether
//! the SDK does.
//!
//! Pluto is absent and that is not a gap here: a Vedic chart has nine
//! grahas and Pluto is none of them. The two that would have been missing
//! until recently are Rahu and Ketu, which is why the nodes were built
//! before the outer planets.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::print_stdout,
    reason = "a test fails by panicking, and a chart is worth printing once"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::{ChartFoundation, Founder};
use teistro_core::catalogue::{Ayanamsha, ChartKind, Graha};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, Resolved, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_ephemeris_builtin::tables::TIER_NAME;

/// Kathmandu, where the conformance corpus's own charts are cast.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn resolved() -> Resolved {
    Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap_or_else(|| panic!("the default profile"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|error| panic!("{error}"))
}

/// A chart founded on the built-in ephemeris and nothing else.
fn chart(instant: f64) -> ChartFoundation {
    let provider = Builtin::new();
    let resolved = resolved();
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let founder = Founder::new(
        &provider,
        &resolved,
        &model,
        &Gregorian,
        &clock,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    );
    founder
        .found_one(
            JulianDay::<Utc>::literal(instant),
            &place(),
            ChartKind::Natal,
        )
        .unwrap_or_else(|error| panic!("the SDK could not cast a chart alone: {error}"))
        .value
}

/// The nine grahas of a Vedic chart. Ketu is derived from Rahu by the
/// chart layer rather than asked of the ephemeris, which is why the
/// provider offers no body for it.
const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

#[test]
fn the_sdk_casts_a_whole_chart_with_nothing_installed() {
    let founded = chart(2_451_545.0);
    println!("\nJ2000 at Kathmandu, on the `{TIER_NAME}` tier, no provider installed:\n");
    for graha in GRAHAS {
        let position = founded
            .graha(graha)
            .unwrap_or_else(|| panic!("{graha:?} is missing from a chart that claims nine"));
        println!(
            "  {graha:<9?} {:>10.4}°  {:>8.4}°/day",
            position.longitude_deg, position.speed_deg_per_day
        );
        assert!(
            (0.0..360.0).contains(&position.longitude_deg),
            "{graha:?} is at {}",
            position.longitude_deg
        );
        assert!(
            position.longitude_deg.is_finite() && position.speed_deg_per_day.is_finite(),
            "{graha:?} is not a number"
        );
    }
}

/// Ketu is Rahu's opposite point, exactly. It is the one graha with no
/// body behind it, so this is worth asserting from outside the chart
/// layer as well as inside it.
#[test]
fn ketu_is_opposite_rahu_to_the_last_digit() {
    for instant in [2_400_000.5, 2_451_545.0, 2_500_000.5] {
        let founded = chart(instant);
        let rahu = founded.graha(Graha::Rahu).expect("Rahu").longitude_deg;
        let ketu = founded.graha(Graha::Ketu).expect("Ketu").longitude_deg;
        let apart = (ketu - rahu).rem_euclid(360.0);
        assert!(
            (apart - 180.0).abs() < 1e-9,
            "at {instant} Rahu and Ketu are {apart} apart"
        );
    }
}

/// The same instant must give the same chart, bit for bit, or a cached
/// chart cannot be trusted to reproduce (ADR-0022).
#[test]
fn the_same_instant_gives_the_same_chart() {
    let first = chart(2_451_545.0);
    let second = chart(2_451_545.0);
    for graha in GRAHAS {
        let a = first.graha(graha).expect("a graha");
        let b = second.graha(graha).expect("a graha");
        assert!(
            a.longitude_deg.to_bits() == b.longitude_deg.to_bits(),
            "{graha:?} moved between two identical requests"
        );
    }
}

/// Across the span the tier claims, every graha must stay a graha: in
/// range, finite, and moving at a rate its name allows. The coarse sweep
/// that would catch a table falling apart at one end of its range rather
/// than at the convenient middle.
#[test]
fn every_graha_holds_across_the_whole_claimed_span() {
    for instant in [
        2_378_500.0,
        2_415_020.5,
        2_451_545.0,
        2_500_000.5,
        2_597_000.0,
    ] {
        let founded = chart(instant);
        for graha in GRAHAS {
            let position = founded
                .graha(graha)
                .unwrap_or_else(|| panic!("{graha:?} missing at {instant}"));
            assert!(
                (0.0..360.0).contains(&position.longitude_deg),
                "{graha:?} at {instant} is {}",
                position.longitude_deg
            );
            assert!(
                position.speed_deg_per_day.abs() < 16.0,
                "{graha:?} at {instant} moves {} degrees a day",
                position.speed_deg_per_day
            );
        }
    }
}
