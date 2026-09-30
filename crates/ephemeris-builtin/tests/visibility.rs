//! Visibility over a real sky, where the test provider's analytic one
//! cannot reach: a superior planet at opposition.

#![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

use teistro_astro::visibility::{Criterion, Heliacal};
use teistro_astro::{Completion, delta_t::DeltaTModel};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
use teistro_core::settings::OverridePolicy;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_port_ephemeris::{Body, Horizon};

/// Jupiter stood opposite the Sun on 2026-01-10 and was up all night for
/// weeks either side. Every criterion must read it seen; two of the
/// three read it unseen for about four weeks while the body's own rising
/// was searched for half a day either side of the sunrise and the one
/// after it was taken.
#[test]
fn a_superior_planet_is_seen_all_about_its_opposition() {
    let provider = Builtin::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let kathmandu = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    for criterion in [
        Criterion::SURYA_SIDDHANTA,
        Criterion::COMBUSTION_ORB,
        Criterion::PTOLEMY,
    ] {
        let heliacal = Heliacal::new(
            &completion,
            kathmandu,
            criterion,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        for offset in -30..=30 {
            let day =
                heliacal.day_start(JulianDay::<Ut1>::literal(2_461_050.5 + f64::from(offset)));
            let state = heliacal.state(Body::Jupiter, day).unwrap();
            assert!(
                state.visible && state.measure_deg > 60.0,
                "{criterion} {offset:+} days: {state:?}"
            );
        }
    }
}
