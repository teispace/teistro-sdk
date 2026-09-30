//! The search over the built-in ephemeris at Kathmandu, autumn 2026.
//!
//! What is held is what the answer promises: the days the season closes
//! are named with what closed them, no window returned lies in a heeded
//! blackout, the windows are in the ranking's order, and a window is
//! constant — its instant clauses read the same a second inside either
//! end, or a quarter of the way in for a window shorter than four. Under the baseline engine's heeds the season reproduces its
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
use teistro_core::catalogue::{Ayanamsha, Calendar, Choghadiya, Kaala};
use teistro_core::error::Status;
use teistro_core::interval::Interval;
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro_core::settings::{OverridePolicy, Profile, SettingsPatch, Sunrise};
use teistro_core::time::UtcOffset;
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_muhurta::instant::clauses as instant_clauses;
use teistro_muhurta::season::BlackoutKind;
use teistro_muhurta::sources::Over;
use teistro_muhurta::{
    ActivityRules, Answer, ClauseKind, ProviderSources, Ranking, Request, Sources, search,
};
use teistro_panchanga::Almanac;
use teistro_port_ephemeris::{CountingProvider, Horizon};

fn date(month: u8, day: u8) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, 2026, month, day)
}

/// Kathmandu.
fn place() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

/// The instant the sources take the chart zodiac at: 2026-09-01 00:00 UTC.
fn reference() -> JulianDay<Utc> {
    JulianDay::literal(2_461_284.5)
}

/// The built-in provider, counted.
type Counted = CountingProvider<Builtin>;

/// Runs `f` over the search's collaborators at Kathmandu, and the provider
/// they ask, so a test can count what they asked it.
fn with_over<R>(f: impl FnOnce(&Over<'_, Counted>, &Counted) -> R) -> R {
    let provider = CountingProvider::new(Builtin::new());
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
    let heliacal = Heliacal::new(
        &completion,
        place(),
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
    f(&over, &provider)
}

/// Runs `f` over sources at Kathmandu.
fn with_sources<R>(f: impl FnOnce(&dyn Sources) -> R) -> R {
    with_over(|over, _| f(&ProviderSources::new(over, place(), reference()).unwrap()))
}

fn request(rules: ActivityRules, ranking: Ranking) -> Request {
    Request {
        rules,
        from: date(9, 1),
        to: date(11, 30),
        native: None,
        ranking,
        days_with_windows: 3,
        // Every window of the days cut, so the promises are held on all.
        most: usize::MAX,
    }
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
    assert_eq!(answer.unjudged, rules.unjudged);
    for pair in answer.windows.windows(2) {
        assert_ne!(
            answer.ranking.compare(&pair[0], &pair[1]),
            core::cmp::Ordering::Greater
        );
    }
    // 2026-08-31 to 2026-12-02, UTC: the search's own reach.
    let season = sources
        .season(Interval::literal(2_461_283.5, 2_461_376.5), &rules.heeds)
        .unwrap();
    let second: f64 = 1.0 / 86_400.0;
    for w in &answer.windows {
        assert!(!season.iter().any(|b| b.at.overlaps(w.at)), "{w:?}");
        // A second inside either end, or a quarter of a window shorter
        // than four seconds: a cut a second after another is real.
        let inset = second.min(w.at.days() / 4.0);
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
            read(w.at.from.get() + inset),
            read(w.at.to.get() - inset),
            "{w:?}"
        );
    }
}

#[test]
fn under_the_baseline_the_season_closes_everything_before_devuthani() {
    with_sources(|sources| {
        let rules = ActivityRules::baseline_marriage();
        // Every open day cut, so the windows a partial blackout touches
        // are among them whichever days score best.
        let every_day = Request {
            days_with_windows: 11,
            ..request(rules.clone(), Ranking::Baseline)
        };
        let answer = search(sources, &every_day).unwrap();
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
        assert_eq!(answer.days_cut, 11);
        // Asta and Kharmas run into the open days, so some windows fall
        // inside a blackout and are left out.
        assert!(answer.windows_blacked_out > 0);
        holds_its_promises(sources, &rules, &answer);
        // An open window is scored exactly when the engine would offer
        // it — in Amrita, Shubha or Labha, since a marriage forbids
        // Abhijit — and none lies in Rahu kaala, a star the rite does not
        // take or a month it is not held in.
        let open: Vec<_> = answer.windows.iter().filter(|w| w.open()).collect();
        assert!(open.iter().any(|w| w.score.is_some()));
        assert!(open.iter().any(|w| w.score.is_none()));
        // Read without the ranking's own comparison: the open windows come
        // first, the scored ahead of the rest, highest score first.
        let first_barred = answer.windows.iter().position(|w| !w.open());
        assert!(first_barred.is_none_or(|i| answer.windows[i..].iter().all(|w| !w.open())));
        let values: Vec<Option<u8>> = open
            .iter()
            .map(|w| w.score.as_ref().map(|s| s.value))
            .collect();
        assert!(values.windows(2).all(|p| p[0] >= p[1]), "{values:?}");
        for w in &open {
            let offered = w.clauses.iter().any(|c| {
                matches!(
                    c.kind,
                    ClauseKind::Choghadiya {
                        choghadiya: Choghadiya::Amrit | Choghadiya::Shubha | Choghadiya::Laabh
                    }
                )
            });
            assert_eq!(w.score.is_some(), offered, "{w:?}");
            assert!(
                w.clauses.iter().all(|c| !matches!(
                    c.kind,
                    ClauseKind::Kaala {
                        kaala: Kaala::RahuKaala
                    } | ClauseKind::Nakshatra { .. }
                        | ClauseKind::SolarMonth { .. }
                )),
                "{w:?}"
            );
        }
    });
}

#[test]
fn the_baseline_ranking_is_refused_without_its_event() {
    with_sources(|sources| {
        let err = search(
            sources,
            &request(ActivityRules::raman_marriage(), Ranking::Baseline),
        )
        .unwrap_err();
        assert_eq!(err.status, Status::InvalidArg);
        assert_eq!(err.field(), Some("rules.baseline"));
    });
}

#[test]
fn under_ramans_rules_the_windows_hold_their_promises() {
    with_sources(|sources| {
        let rules = ActivityRules::raman_marriage();
        let answer = search(sources, &request(rules.clone(), Ranking::Texts)).unwrap();
        assert_eq!(answer.days_judged + answer.closed.len(), 91);
        assert!(answer.windows.iter().all(|w| w.score.is_none()));
        holds_its_promises(sources, &rules, &answer);
    });
}

#[test]
fn days_the_caller_founded_answer_the_same_and_are_not_founded_again() {
    with_over(|over, provider| {
        let asked = request(ActivityRules::raman_marriage(), Ranking::Texts);
        let sources = || ProviderSources::new(over, place(), reference()).unwrap();
        provider.reset();
        let alone = search(&sources(), &asked).unwrap();
        let founding = provider.calls().total();

        let days = over
            .almanac
            .between(&asked.from, &asked.to, &place())
            .unwrap()
            .value;
        provider.reset();
        let shared = search(&sources().with_days(&days), &asked).unwrap();
        let reading = provider.calls().total();
        assert_eq!(shared, alone);
        assert!(
            reading < founding,
            "served from the days, the search asked the provider {reading} times against {founding}"
        );

        // A range the days cover only in part: the rest is founded as before.
        let (first_half, _) = days.split_at(days.len() / 2);
        let partly = search(&sources().with_days(first_half), &asked).unwrap();
        assert_eq!(partly, alone);
    });
}
