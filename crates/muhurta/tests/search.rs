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
use teistro_port_ephemeris::{CountingProvider, EphemerisProvider, Horizon, TestProvider};

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
    over_on(&provider, |over| f(over, &provider))
}

/// Runs `f` over the search's collaborators at Kathmandu, asking
/// `provider`.
fn over_on<P: EphemerisProvider, R>(provider: &P, f: impl FnOnce(&Over<'_, P>) -> R) -> R {
    over_under(provider, teistro_core::settings::DEFAULT_PROFILE, f)
}

/// Runs `f` over the search's collaborators at Kathmandu under a shipped
/// profile, asking `provider`.
fn over_under<P: EphemerisProvider, R>(
    provider: &P,
    profile: &str,
    f: impl FnOnce(&Over<'_, P>) -> R,
) -> R {
    let resolved = Profile::shipped(profile)
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap();
    let delta_t = DeltaTModel::TableThenModel;
    let precession = PrecessionModel::Vondrak2011;
    let model = DrikSun::new(
        provider,
        Ayanamsha::Lahiri,
        Sunrise::CentreNoRefraction.into(),
        OverridePolicy::PreferNative,
        delta_t,
    );
    let clock = UtcOffset::literal(5, 45, 0);
    let almanac = Almanac::new(
        provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let founder = Founder::new(
        provider, &resolved, &model, &Gregorian, &clock, precession, delta_t,
    );
    let completion = Completion::new(provider, resolved.settings.provider.overrides, delta_t);
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
    f(&over)
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
    assert!(!answer.windows.is_empty(), "`answer.windows` is empty");
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
        .unwrap()
        .blackouts;
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

/// The eclipse blackouts at Kathmandu over September and October 2025
/// (`muhurta.md` §4.1.1): the total lunar eclipse of 7 September, seen
/// whole near midnight, gives a vedha and bars its star; the partial solar
/// eclipse of the 21st, not seen in Nepal, gives neither.
#[test]
fn a_seen_eclipse_bars_its_star_and_its_vedha_and_an_unseen_one_nothing() {
    use teistro_astro::eclipse::Eclipses;
    use teistro_core::quantity::Ut1;
    with_over(|over, _| {
        let sources = ProviderSources::new(over, place(), reference()).unwrap();
        let range = Interval::literal(2_460_919.5, 2_460_979.5);
        let kinds = [BlackoutKind::EclipseStar, BlackoutKind::EclipseVedha];
        let season = sources.season(range, &kinds).unwrap();
        assert_eq!(season.unjudged, Vec::new());
        let season = season.blackouts;
        let found = Eclipses::new(over.completion, over.delta_t)
            .here_between(
                JulianDay::<Ut1>::literal(range.from.get()),
                JulianDay::<Ut1>::literal(range.to.get()),
                place(),
                &Horizon::from_convention(over.settings.day.sunrise),
            )
            .unwrap();
        let lunar = &found.lunar[0];
        let umbral = lunar.here.umbral_seen.expect("seen from Kathmandu");
        assert!(
            found
                .solar
                .iter()
                .all(|e| e.here.is_none_or(|h| h.seen.is_none()))
        );

        // One vedha, closing as the umbra leaves the Moon, which is still
        // up, and opening on a prahara of the eclipse's day: a quarter of
        // its daylight or of its night.
        let vedhas: Vec<Interval> = season
            .iter()
            .filter(|b| b.kind == BlackoutKind::EclipseVedha)
            .map(|b| b.at)
            .collect();
        let [vedha] = vedhas.as_slice() else {
            panic!("one vedha, found {vedhas:?}");
        };
        // Exactly: the vedha closes at the instant the stretch seen does.
        assert_eq!(vedha.to.get().to_bits(), umbral.to.get().to_bits());
        let day = sources
            .day(&CalendarDate::defined(Calendar::Gregorian, 2025, 9, 7))
            .unwrap();
        let (rise, set, next) = (
            day.day.sunrise.get(),
            day.day.sunset.get(),
            day.day.next_sunrise.get(),
        );
        let edges: Vec<f64> = (0..4)
            .map(|q| rise + (set - rise) * f64::from(q) / 4.0)
            .chain((0..4).map(|q| set + (next - set) * f64::from(q) / 4.0))
            .collect();
        let at = edges
            .iter()
            .position(|edge| (edge - vedha.from.get()).abs() < 1e-9)
            .expect("the vedha opens on a prahara");
        let holding = edges
            .iter()
            .rposition(|edge| *edge <= umbral.from.get())
            .unwrap();
        assert_eq!(holding - at, 3, "three praharas before a lunar eclipse");

        // The star: the passage the eclipse falls in from the eclipse on,
        // then one a sidereal month later, each about a day long.
        let stars: Vec<Interval> = season
            .iter()
            .filter(|b| b.kind == BlackoutKind::EclipseStar)
            .map(|b| b.at)
            .collect();
        assert_eq!(
            stars[0].from.get().to_bits(),
            lunar.eclipse.greatest.get().to_bits()
        );
        assert!(stars.len() >= 2, "{stars:?}");
        for pair in stars.windows(2) {
            let apart = pair[1].from.get() - pair[0].from.get();
            assert!((26.0..29.0).contains(&apart), "{apart} days apart");
        }
        for passage in &stars[1..] {
            let days = passage.to.get() - passage.from.get();
            assert!(
                (0.8..1.3).contains(&days) || passage.to == range.to,
                "{days}"
            );
        }
    });
}

/// A provider whose frame the SDK cannot complete to an apparent Sun and
/// Moon cannot say what a place saw: the eclipse kinds are reported as
/// unjudged, with the provider's refusal, rather than failing the search
/// for every activity that heeds them.
#[test]
fn a_sky_that_cannot_see_an_eclipse_leaves_its_kinds_unjudged() {
    over_on(&TestProvider::new(), |over| {
        let sources = ProviderSources::new(over, place(), reference()).unwrap();
        // 2024-11-24 to 2024-11-28, UTC.
        let range = Interval::literal(2_460_638.5, 2_460_642.5);
        let kinds = [
            BlackoutKind::AdhikaMasa,
            BlackoutKind::EclipseStar,
            BlackoutKind::EclipseVedha,
        ];
        let season = sources.season(range, &kinds).unwrap();
        assert!(season.blackouts.iter().all(|b| !matches!(
            b.kind,
            BlackoutKind::EclipseStar | BlackoutKind::EclipseVedha
        )));
        let what: Vec<&str> = season.unjudged.iter().map(|u| u.what.as_str()).collect();
        assert_eq!(
            what,
            [
                "the eclipse's star (grahanotpatha)",
                "the eclipse's vedha (sutak)"
            ]
        );
        for unjudged in &season.unjudged {
            assert!(unjudged.why.contains("refused them"), "{}", unjudged.why);
        }
    });
}

/// Nepal's committee prints the vedha as fixed three-hour praharas back
/// from the first moment Nepal sees, and `nepali-default` reads it so
/// (`FIXED_HOURS`). Three eclipses, as the committee gave them (UTC
/// here, Nepal's clock in the comments):
///
/// - 2025-09-07, lunar: touch 22:11, no food from 13:11, release 01:41;
/// - 2026-03-03, lunar, the Moon rising eclipsed at 18:03: no food from
///   09:03, release 19:02;
/// - 2022-10-25, solar, the Sun setting eclipsed: touch 16:52, no food
///   from 04:52 until the next sunrise.
///
/// The printed minutes are rounded, and a touch is the umbra's, so the
/// tolerance is three minutes.
#[test]
fn the_nepali_vedha_is_nine_hours_before_a_lunar_eclipse_seen_and_twelve_before_a_solar_one() {
    const MINUTES: f64 = 3.0 / 1440.0;
    over_under(&Builtin::new(), "nepali-default", |over| {
        assert_eq!(
            over.settings.panchanga.eclipse_vedha,
            teistro_core::settings::EclipseVedha::FixedHours
        );
        let sources = ProviderSources::new(over, place(), reference()).unwrap();
        let vedha = |from: f64, to: f64| {
            let season = sources
                .season(Interval::literal(from, to), &[BlackoutKind::EclipseVedha])
                .unwrap();
            assert_eq!(season.unjudged, Vec::new());
            let found: Vec<Interval> = season.blackouts.iter().map(|b| b.at).collect();
            let [one] = found.as_slice() else {
                panic!("one vedha, found {found:?}");
            };
            *one
        };
        let near = |at: f64, printed: f64, what: &str| {
            assert!(
                (at - printed).abs() < MINUTES,
                "{what}: {at} against {printed}, {:.1} min off",
                (at - printed) * 1440.0
            );
        };

        let september = vedha(2_460_924.5, 2_460_927.5);
        near(september.from.get(), 2_460_925.809_7, "2025-09-07 opens");
        near(september.to.get(), 2_460_926.330_6, "2025-09-07 closes");

        let march = vedha(2_461_101.5, 2_461_104.5);
        near(march.from.get(), 2_461_102.637_5, "2026-03-03 opens");
        near(march.to.get(), 2_461_103.053_5, "2026-03-03 closes");

        let october = vedha(2_459_876.5, 2_459_879.5);
        near(october.from.get(), 2_459_877.463_2, "2022-10-25 opens");
        let next_day = sources
            .day(&CalendarDate::defined(Calendar::Gregorian, 2022, 10, 26))
            .unwrap();
        assert!((october.to.get() - next_day.day.sunrise.get()).abs() < 1e-6);
    });
}
