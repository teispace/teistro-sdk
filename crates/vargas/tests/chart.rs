//! A whole divisional chart of a founded moment, over the analytic test
//! provider.
//!
//! The corpus decides the numbers (`baseline.rs`); what only the
//! assembled value can show is that the parts agree with each other. The
//! rashi chart is the foundation's own signs; every chart places every
//! body the foundation carries; the mixed axis moves the lagna without
//! moving the grahas; and vargottama is answered for the navamsha and
//! refused for anything else, so "vargottama in D10" cannot be asked by
//! accident.

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
use teistro_core::catalogue::{Ayanamsha, ChartKind, Graha, Varga};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_vargas::{Axis, Scheme, chart, every_chart, sign};

/// Kathmandu, where the corpus's own charts are.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// A founded moment of the plausible sky.
fn founded() -> ChartFoundation {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap_or_else(|| panic!("the default profile"))
        .resolve(&SettingsPatch::default())
        .unwrap_or_else(|e| panic!("{e}"));
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        DeltaTModel::TableThenModel,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    Founder::new(
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
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .value
}

#[test]
fn the_rashi_chart_is_the_foundations_own_signs() {
    let foundation = founded();
    let rashi = chart(&foundation, Axis::of(Varga::D1)).expect("a founded moment");
    assert_eq!(rashi.grahas.len(), foundation.grahas.len());
    for (placement, position) in rashi.grahas.iter().zip(&foundation.grahas) {
        assert_eq!(placement.graha, position.graha);
        assert_eq!(
            placement.at.sign as u8,
            position.sign_index(),
            "{:?}",
            position.graha
        );
        assert!(
            placement.at.keeps_its_sign(),
            "the rashi chart moves nobody"
        );
    }
    // And so is the lagna's.
    let lagna = teistro_core::angle::Nas::from_degrees(
        teistro_core::quantity::Degrees::try_new(foundation.lagna_deg.rem_euclid(360.0))
            .expect("finite"),
    );
    assert_eq!(rashi.lagna.sign, lagna.sign());
}

#[test]
fn every_catalogued_chart_places_every_body() {
    let foundation = founded();
    let charts = every_chart(&foundation).expect("a founded moment");
    assert_eq!(charts.len(), Varga::ALL.len());
    let mut placed = 0;
    for (chart, varga) in charts.iter().zip(Varga::ALL) {
        assert_eq!(chart.axis.grahas.varga, Some(varga));
        assert_eq!(chart.grahas.len(), foundation.grahas.len());
        assert_eq!(chart.signs().len(), foundation.grahas.len() + 1);
        for placement in &chart.grahas {
            assert!(chart.graha(placement.graha).is_some());
            placed += 1;
        }
    }
    println!("{placed} placements over {} charts", charts.len());
    assert_eq!(placed, Varga::ALL.len() * foundation.grahas.len());
}

#[test]
fn a_mixed_axis_moves_the_lagna_without_moving_the_grahas() {
    let foundation = founded();
    let plain = chart(&foundation, Axis::of(Varga::D9)).expect("a founded moment");
    let mixed = chart(
        &foundation,
        Axis::mixed(Scheme::of(Varga::D9), Scheme::of(Varga::D1)),
    )
    .expect("a founded moment");

    assert_eq!(mixed.axis.key(), "D9/D1");
    assert_eq!(mixed.grahas, plain.grahas, "the grahas are the same chart");
    // The lagna is the rashi one now, which is the foundation's own sign.
    let rashi = chart(&foundation, Axis::of(Varga::D1)).expect("a founded moment");
    assert_eq!(mixed.lagna, rashi.lagna);
    // Unless the navamsha happens to keep it, the two differ.
    if !plain.lagna.keeps_its_sign() {
        assert_ne!(mixed.lagna.sign, plain.lagna.sign);
    }
}

#[test]
fn vargottama_is_answered_for_the_navamsha_and_refused_elsewhere() {
    let foundation = founded();
    let navamsha = chart(&foundation, Axis::of(Varga::D9)).expect("a founded moment");
    for placement in &navamsha.grahas {
        assert_eq!(
            navamsha.vargottama(placement.graha),
            Some(placement.at.keeps_its_sign()),
            "{:?}",
            placement.graha
        );
    }
    assert!(navamsha.lagna_vargottama().is_some());
    // The same question of a dashamsha is not a question.
    let dashamsha = chart(&foundation, Axis::of(Varga::D10)).expect("a founded moment");
    assert!(dashamsha.vargottama(Graha::Sun).is_none());
    assert!(dashamsha.lagna_vargottama().is_none());
    // The general fact is still available there.
    let keeping = dashamsha.keeping_their_sign();
    assert!(keeping.len() <= dashamsha.grahas.len());
}

#[test]
fn a_chart_agrees_with_the_kernel_it_is_made_of() {
    // The assembled value must be exactly what placing each longitude
    // one at a time gives: no rounding, no reordering, no second path.
    let foundation = founded();
    for varga in Varga::ALL {
        let scheme = Scheme::of(varga);
        let built = chart(&foundation, Axis::of(varga)).expect("a founded moment");
        for (placement, position) in built.grahas.iter().zip(&foundation.grahas) {
            let alone = sign(
                &scheme,
                teistro_core::angle::Nas::from_degrees(
                    teistro_core::quantity::Degrees::try_new(
                        position.longitude_deg.rem_euclid(360.0),
                    )
                    .expect("finite"),
                ),
            );
            assert_eq!(placement.at.sign, alone, "{varga:?} {:?}", position.graha);
        }
    }
}
