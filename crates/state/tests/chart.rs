//! Every graha of a founded chart, end to end over the analytic test
//! provider.
//!
//! The corpus decides the rules (`baseline.rs`); what only the assembled
//! value can show is that the parts agree with each other — that the
//! state of a graha is the state of the position the foundation gave it,
//! that a war is symmetric across the whole chart, and that the entry
//! point refuses what it cannot answer.

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
use teistro_core::catalogue::{Ayanamsha, ChartKind, Dignity, Graha};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, Settings, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_port_ephemeris::test_provider::TestProvider;
use teistro_state::{avastha, state};

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

fn founded() -> (ChartFoundation, Settings) {
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
    )
    .unwrap_or_else(|e| panic!("{e}"))
    .value;
    (foundation, resolved.settings)
}

#[test]
fn every_graha_of_the_foundation_gets_a_state() {
    let (foundation, settings) = founded();
    let states = state(&foundation, &settings).expect("a founded chart");
    assert_eq!(states.len(), foundation.grahas.len());
    for (found, position) in states.iter().zip(&foundation.grahas) {
        assert_eq!(found.graha, position.graha);
        assert_eq!(found.sign as u8, position.sign_index());
        assert_eq!(found.house, position.house.bhava);
        assert_eq!(found.motion.retrograde, position.is_retrograde());
        assert!((found.motion.speed_deg_per_day - position.speed_deg_per_day).abs() < 1e-12);
        // A boundary distance is a fact whatever the body.
        assert!(found.boundaries.nearest_deg() >= 0.0);
        // And the undecided lajjitadi are named on every one of them.
        assert_eq!(found.lajjitadi.undecided.len(), 3);
    }
    println!(
        "{} grahas, {} at home, {} combust",
        states.len(),
        states.iter().filter(|found| found.is_at_home()).count(),
        states.iter().filter(|found| found.is_combust()).count()
    );
}

#[test]
fn the_sun_burns_nothing_of_itself_and_the_shadows_burn_nobody() {
    let (foundation, settings) = founded();
    let states = state(&foundation, &settings).expect("a founded chart");
    for found in &states {
        match found.graha {
            Graha::Sun | Graha::Rahu | Graha::Ketu => {
                assert!(!found.is_combust(), "{:?}", found.graha);
                assert_eq!(found.combustion.orbs, None, "{:?}", found.graha);
            }
            _ => assert!(
                found.combustion.from_sun_deg.is_some(),
                "{:?} knows how far the Sun is",
                found.graha
            ),
        }
        // A shadow never reaches a friendship dignity.
        if matches!(found.graha, Graha::Rahu | Graha::Ketu) {
            assert!(
                !matches!(
                    found.dignity,
                    Dignity::GreatFriend | Dignity::Friend | Dignity::Enemy | Dignity::GreatEnemy
                ),
                "{:?}: {:?}",
                found.graha,
                found.dignity
            );
        }
    }
}

#[test]
fn a_war_is_symmetric_across_the_whole_chart() {
    let (foundation, settings) = founded();
    let states = state(&foundation, &settings).expect("a founded chart");
    for found in &states {
        let Some(war) = found.war else {
            continue;
        };
        let other = states
            .iter()
            .find(|state| state.graha == war.opponent)
            .expect("the opponent is in the chart");
        let back = other.war.expect("and is at war too");
        assert_eq!(back.opponent, found.graha);
        assert_ne!(back.is_winner, war.is_winner, "exactly one side wins");
        assert!((back.apart_deg - war.apart_deg).abs() < 1e-12);
        assert!(war.apart_deg < avastha::WAR_ORB_DEG);
    }
}

#[test]
fn the_deeptadi_is_absent_exactly_where_the_dignity_does_not_decide_it() {
    let (foundation, settings) = founded();
    for found in state(&foundation, &settings).expect("a founded chart") {
        let decided = matches!(
            found.dignity,
            Dignity::Exalted
                | Dignity::DeepExalted
                | Dignity::OwnSign
                | Dignity::Mooltrikona
                | Dignity::GreatFriend
        );
        assert_eq!(
            found.deeptadi.is_some(),
            decided,
            "{:?} at {:?}",
            found.graha,
            found.dignity
        );
    }
}

#[test]
fn a_combustion_table_the_sdk_does_not_ship_is_refused_by_name() {
    let (foundation, mut settings) = founded();
    settings.state.combustion_orbs = String::from("SOMEONE_ELSES");
    let error = state(&foundation, &settings).expect_err("no such table");
    assert!(error.message.contains("SOMEONE_ELSES"), "{error}");
    assert_eq!(error.field(), Some("state.combustion_orbs"));
}

#[test]
fn two_runs_of_the_same_chart_are_the_same_value() {
    let (foundation, settings) = founded();
    let once = state(&foundation, &settings).expect("a founded chart");
    let twice = state(&foundation, &settings).expect("a founded chart");
    assert_eq!(once, twice, "the determinism contract");
}
