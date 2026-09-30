//! The lagna's cuts over a real founding: the plausible sky's ascendant at
//! Kathmandu, asked through the same `Founder::ascendant_at` a chart's own
//! derived points ask through.
//!
//! What is held is the promise a window makes: the lagna's navamsa is one
//! thing over each window and another over the next, so a judgement read
//! anywhere inside a window is the judgement for all of it.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_calendar::Gregorian;
use teistro_calendar::solar::drik::DrikSun;
use teistro_chart::foundation::Founder;
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::catalogue::{Ayanamsha, Rashi};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1, Utc};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_muhurta::ClauseKind;
use teistro_muhurta::window::{lagna_cuts, tyajya, windows};
use teistro_port_ephemeris::test_provider::TestProvider;

/// 2024-06-18 00:00 UTC, and the day after it.
const FROM: f64 = 2_460_479.5;

fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.3240),
        Altitude::literal(1400.0),
    )
}

/// Runs `f` with the lagna at an instant, founded at Kathmandu.
fn with_lagna<R>(f: impl FnOnce(&dyn Fn(JulianDay<Utc>) -> f64) -> R) -> R {
    let provider = TestProvider;
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
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
    let (tt, _) = tt_of(
        JulianDay::<Ut1>::literal(FROM + 0.5),
        DeltaTModel::TableThenModel,
    )
    .unwrap();
    let zodiac = ChartZodiac::of(
        &resolved.settings,
        tt,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
    .unwrap();
    let place = place();
    f(&|at| founder.ascendant_at(at, &place, &zodiac).unwrap())
}

fn navamsa(deg: f64) -> u16 {
    let n = (deg.rem_euclid(360.0) / (30.0 / 9.0)).floor();
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a floor in 0..108"
    )]
    let n = n as u16;
    n
}

#[test]
fn each_window_holds_one_navamsa_and_the_next_another() {
    let day = Interval::literal(FROM, FROM + 1.0);
    with_lagna(|lagna| {
        let cuts = lagna_cuts(day, |at| Ok(lagna(at))).unwrap();
        // The lagna turns once a sidereal day, so a solar day sees every
        // navamsa and a few of them twice.
        assert!((108..=110).contains(&cuts.len()), "{}", cuts.len());
        assert_eq!(cuts.iter().filter(|c| c.sign_changed).count(), 12);
        let pieces = windows(day, cuts.iter().map(|c| c.at));
        assert_eq!(pieces.len(), cuts.len() + 1);
        // A tenth of a second inside each edge: well above the cut's
        // millisecond and well below the shortest window.
        let inset = 0.1 / 86_400.0;
        let mut before = None;
        for w in &pieces {
            let first = navamsa(lagna(JulianDay::literal(w.from.get() + inset)));
            let last = navamsa(lagna(JulianDay::literal(w.to.get() - inset)));
            assert_eq!(first, last, "{w:?}");
            assert_ne!(before, Some(first), "{w:?}");
            before = Some(first);
        }
        // Each cut names the navamsa entered.
        for c in &cuts {
            let after = navamsa(lagna(JulianDay::literal(c.at.get() + inset)));
            assert_eq!(after, u16::from(c.navamsa), "{c:?}");
        }
    });
}

#[test]
fn the_tyajya_lies_inside_the_sign_it_names() {
    let day = Interval::literal(FROM, FROM + 1.0);
    with_lagna(|lagna| {
        let cuts = lagna_cuts(day, |at| Ok(lagna(at))).unwrap();
        let clauses = tyajya(&cuts, day);
        // Twelve entries bound eleven whole signs.
        assert_eq!(clauses.len(), 11);
        for clause in &clauses {
            let ClauseKind::LagnaTyajya { sign } = clause.kind else {
                panic!("{clause:?}")
            };
            assert!(
                (clause.at.days() * 1440.0 - 12.0).abs() < 1e-3,
                "{clause:?}"
            );
            for at in [clause.at.from.get(), clause.at.to.get()] {
                let inside = at.clamp(clause.at.from.get() + 1e-6, clause.at.to.get() - 1e-6);
                let deg = lagna(JulianDay::literal(inside));
                assert_eq!(Rashi::from_id(navamsa(deg) / 9), Some(sign), "{clause:?}");
            }
        }
    });
}
