//! The search over the built-in ephemeris at Kathmandu, autumn 2026.
//!
//! What is held is what the answer promises: the days the season closes
//! are named with what closed them, no window returned lies in a heeded
//! blackout, the windows are in the ranking's order, and a window is
//! constant — its instant clauses read the same a second inside either
//! end. Under the baseline engine's heeds the season reproduces its
//! regression: nothing open before Devuthani.

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index fixed lists"
)]

use teistro_astro::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::precession::PrecessionModel;
use teistro_astro::visibility::{Criterion, Heliacal};
use teistro_calendar::solar::drik::DrikSun;
use teistro_calendar::{CalendarDate, Gregorian};
use teistro_chart::foundation::Founder;
use teistro_core::catalogue::{Ayanamsha, Calendar};
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_muhurta::instant::clauses as instant_clauses;
use teistro_muhurta::season::BlackoutKind;
use teistro_muhurta::sources::Over;
use teistro_muhurta::{ActivityRules, Answer, ProviderSources, Ranking, Request, Sources, search};
use teistro_panchanga::Almanac;
use teistro_port_ephemeris::Horizon;

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

/// Runs `f` over sources at Kathmandu.
fn with_sources<R>(f: impl FnOnce(&dyn Sources) -> R) -> R {
    let provider = Builtin::new();
    let resolved = Profile::shipped(teistro_core::settings::DEFAULT_PROFILE)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let delta_t = DeltaTModel::TableThenModel;
    let precession = PrecessionModel::Vondrak2011;
    let model = DrikSun::new(
        &provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        delta_t,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let founder = Founder::new(
        &provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    let place = Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    );
    let heliacal = Heliacal::new(
        &completion,
        place,
        Criterion::SURYA_SIDDHANTA,
        Horizon::CENTRE_NO_REFRACTION,
        delta_t,
    );
    let over = Over {
        almanac: &almanac,
        founder: &founder,
        completion: &completion,
        heliacal: &heliacal,
        calendar: &Gregorian,
        clock: &clock,
        settings: &resolved.settings,
        precession,
        delta_t,
    };
    // 2026-09-01 00:00 UTC.
    let sources = ProviderSources::new(&over, place, JulianDay::literal(2_461_284.5)).unwrap();
    f(&sources)
}

fn request(rules: ActivityRules) -> Request {
    Request {
        rules,
        from: date(9, 1),
        to: date(11, 30),
        native: None,
        ranking: Ranking::Texts,
        days_with_windows: 3,
        // Every window of the days cut, so the promises are held on all.
        most: usize::MAX,
    }
}

/// The baseline engine's marriage heeds, on Raman's day rules.
fn baseline_heeds() -> ActivityRules {
    let mut rules = ActivityRules::raman_marriage();
    rules.heeds = vec![
        BlackoutKind::Chaturmas,
        BlackoutKind::AdhikaMasa,
        BlackoutKind::Kharmas,
        BlackoutKind::PitruPaksha,
        BlackoutKind::GuruAsta,
        BlackoutKind::ShukraAsta,
    ];
    rules
}

fn closed_dates(answer: &Answer) -> Vec<(u8, u8)> {
    answer
        .closed
        .iter()
        .map(|c| (c.date.month, c.date.day))
        .collect()
}

/// Holds what every answer promises: its windows are in the ranking's
/// order, none overlaps a heeded blackout, and each is constant: the sky
/// its instant clauses read reads the same a second inside either end.
fn holds_its_promises(sources: &dyn Sources, rules: &ActivityRules, answer: &Answer) {
    assert!(!answer.windows.is_empty());
    assert_eq!(answer.ranking, Ranking::Texts);
    assert_eq!(answer.unjudged, rules.unjudged);
    for pair in answer.windows.windows(2) {
        assert_ne!(
            Ranking::Texts.compare(&pair[0], &pair[1]),
            core::cmp::Ordering::Greater
        );
    }
    // 2026-08-31 to 2026-12-02, UTC: the search's own reach.
    let season = sources
        .season(Interval::literal(2_461_283.5, 2_461_376.5), &rules.heeds)
        .unwrap();
    let second = 1.0 / 86_400.0;
    for w in &answer.windows {
        assert!(!season.iter().any(|b| b.at.overlaps(w.at)), "{w:?}");
        // A window was cut at every clause's edge, so each clause it holds
        // holds over all of it.
        for c in &w.clauses {
            assert!(
                c.at.from.get() <= w.at.from.get() && c.at.to.get() >= w.at.to.get(),
                "{c:?} holds over part of {:?}",
                w.at
            );
        }
        // What the cuts promise: no graha changes sign and neither the
        // lagna nor the Moon changes navamsa inside a window. Every
        // instant clause is a function of these, so they hold still too.
        let read = |at: f64| {
            let sky = sources.sky_at(JulianDay::literal(at)).unwrap();
            let signs = sky.grahas.map(|g| (g / 30.0).floor());
            let navamsa = |deg: f64| (deg / (30.0 / 9.0)).floor();
            let mut kinds: Vec<_> = instant_clauses(&sky, None, None, w.at)
                .into_iter()
                .chain(rules.instant_clauses(&sky, w.at))
                .map(|c| format!("{:?}", c.kind))
                .collect();
            kinds.sort();
            (signs, navamsa(sky.lagna_deg), navamsa(sky.grahas[1]), kinds)
        };
        assert_eq!(
            read(w.at.from.get() + second),
            read(w.at.to.get() - second),
            "{w:?}"
        );
    }
}

#[test]
fn under_the_baseline_heeds_the_season_closes_everything_before_devuthani() {
    with_sources(|sources| {
        let rules = baseline_heeds();
        let answer = search(sources, &request(rules.clone())).unwrap();
        let closed = closed_dates(&answer);
        for day in [(9, 21), (10, 19), (11, 5)] {
            assert!(closed.contains(&day), "{day:?} is open");
        }
        // Chaturmas closes every day up to Devuthani, 2026-11-20, each
        // closed day says so, and every day from it on is judged.
        assert_eq!(closed.len(), 80, "{closed:?}");
        assert!(closed.iter().all(|day| *day < (11, 20)), "{closed:?}");
        assert!(
            answer
                .closed
                .iter()
                .all(|c| c.by.contains(&BlackoutKind::Chaturmas))
        );
        assert_eq!(answer.days_judged, 11);
        assert_eq!(answer.days_cut, 3);
        // Asta and Kharmas run into the open days, so some windows fall
        // inside a blackout and are left out.
        assert!(answer.windows_blacked_out > 0);
        holds_its_promises(sources, &rules, &answer);
    });
}

#[test]
fn under_ramans_rules_the_windows_hold_their_promises() {
    with_sources(|sources| {
        let rules = ActivityRules::raman_marriage();
        let answer = search(sources, &request(rules.clone())).unwrap();
        assert_eq!(answer.days_judged + answer.closed.len(), 91);
        holds_its_promises(sources, &rules, &answer);
    });
}
