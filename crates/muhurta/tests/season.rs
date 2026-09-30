//! The season over the built-in ephemeris, held to published almanacs.
//!
//! The instants are the ones an almanac prints for New Delhi (converted
//! to UTC): the new moons of the 2026 adhika Jyeshtha, the bright
//! eleventh of Ashadha and of Kartika that bound Chaturmas, and the
//! Purnima and new moon that bound Pitru paksha. The elongation reads the
//! same in every zodiac and at every place, so these hold to minutes; the
//! tolerances are generous beside that and tight beside the month an
//! anchor on the wrong ekadashi would move them.

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
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Ayanamsha, Masa};
use teistro_core::interval::Interval;
use teistro_core::settings::{AyanamshaBasis, OverridePolicy};
use teistro_ephemeris_builtin::provider::Builtin;
use teistro_muhurta::season::{Blackout, BlackoutKind, blackouts, months};
use teistro_panchanga::limb::Zodiac;
use teistro_port_ephemeris::Frame;

/// 2025-01-01 and 2027-01-01, 00:00 UTC.
const FROM: f64 = 2_460_676.5;
const TO: f64 = 2_461_406.5;

fn zodiac() -> Zodiac {
    Zodiac::of(
        Some(Ayanamsha::Lahiri.into()),
        AyanamshaBasis::True,
        PrecessionModel::Vondrak2011,
        DeltaTModel::TableThenModel,
    )
}

/// Runs `f` over the two years' blackouts.
fn with_season<R>(f: impl FnOnce(&[Blackout]) -> R) -> R {
    let provider = Builtin::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let found = blackouts(&longitudes, zodiac(), Interval::literal(FROM, TO)).unwrap();
    f(&found)
}

fn of(found: &[Blackout], kind: BlackoutKind) -> Vec<Interval> {
    found
        .iter()
        .filter(|b| b.kind == kind)
        .map(|b| b.at)
        .collect()
}

fn near(at: f64, expected: f64, tolerance: f64, what: &str) {
    assert!(
        (at - expected).abs() < tolerance,
        "{what}: {at} against {expected}, {:.1} h off",
        (at - expected) * 24.0
    );
}

#[test]
fn the_one_adhika_month_of_the_two_years_is_2026s_jyeshtha() {
    let provider = Builtin::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let found = months(&longitudes, zodiac(), Interval::literal(FROM, TO)).unwrap();
    let adhika: Vec<_> = found
        .iter()
        .filter(|m| m.kind == MonthKind::Adhika)
        .collect();
    assert_eq!(adhika.len(), 1, "{adhika:?}");
    assert_eq!(adhika[0].masa, Masa::Jyeshtha);
    // New moons 2026-05-16 20:01 and 2026-06-15 02:54 UTC.
    near(
        adhika[0].at.from.get(),
        2_461_177.334,
        0.01,
        "opening new moon",
    );
    near(
        adhika[0].at.to.get(),
        2_461_206.621,
        0.01,
        "closing new moon",
    );
    // The months tile the span without a gap.
    for pair in found.windows(2) {
        assert_eq!(pair[0].at.to, pair[1].at.from);
    }
    with_season(|season| {
        let marked = of(season, BlackoutKind::AdhikaMasa);
        assert_eq!(marked, [adhika[0].at]);
    });
}

#[test]
fn chaturmas_runs_ashadhas_bright_eleventh_to_kartikas() {
    with_season(|season| {
        let found = of(season, BlackoutKind::Chaturmas);
        assert_eq!(found.len(), 2, "{found:?}");
        // 2025: the eleventh began 2025-07-05 13:28 UTC with the Sun still
        // in sidereal Gemini, and Kartika's 2025-11-01 03:41 UTC with the
        // Sun still in Libra.
        near(found[0].from.get(), 2_460_862.061, 0.05, "Devshayani 2025");
        near(found[0].to.get(), 2_460_980.654, 0.05, "Prabodhini 2025");
        // 2026, after the adhika Jyeshtha: Ashadha's eleventh on
        // 2026-07-25 and Kartika's on 2026-11-20 (civil dates).
        assert!(
            (2_461_245.5..2_461_247.5).contains(&found[1].from.get()),
            "{found:?}"
        );
        assert!(
            (2_461_363.5..2_461_365.5).contains(&found[1].to.get()),
            "{found:?}"
        );
    });
}

#[test]
fn pitru_paksha_runs_bhadrapadas_purnima_to_its_new_moon() {
    with_season(|season| {
        let found = of(season, BlackoutKind::PitruPaksha);
        assert_eq!(found.len(), 2, "{found:?}");
        // 2025: the Purnima began 2025-09-06 20:11 UTC; the Mahalaya new
        // moon fell 2025-09-21 19:54 UTC.
        near(found[0].from.get(), 2_460_925.341, 0.05, "Purnima 2025");
        near(found[0].to.get(), 2_460_940.329, 0.01, "Mahalaya 2025");
        // 2026: 2026-09-26 to 2026-10-10 (civil dates).
        assert!(
            (2_461_308.5..2_461_310.5).contains(&found[1].from.get()),
            "{found:?}"
        );
        assert!(
            (2_461_322.5..2_461_324.5).contains(&found[1].to.get()),
            "{found:?}"
        );
    });
}

#[test]
fn kharmas_is_the_sun_in_sagittarius_and_pisces_and_each_sankranti_is_sixteen_ghatis_either_side() {
    with_season(|season| {
        let kharmas = of(season, BlackoutKind::Kharmas);
        let sankrantis = of(season, BlackoutKind::Sankranti);
        // The range opens with the Sun in Sagittarius and closes with it
        // there again, so five spans: the first and last clipped to it.
        assert_eq!(kharmas.len(), 5, "{kharmas:?}");
        assert!(
            kharmas.iter().all(|k| (25.0..33.0).contains(&k.days())
                || k.from.get() == FROM
                || k.to.get() == TO)
        );
        // Twenty-four ingresses in two years; the window of each is 32
        // ghatis, 12 h 48 min, and a Kharmas month begins at the middle
        // of one.
        assert_eq!(sankrantis.len(), 24, "{sankrantis:?}");
        for s in &sankrantis {
            near(s.days(), 32.0 / 60.0, 1e-9, "a sankranti window");
        }
        for k in &kharmas {
            assert!(
                sankrantis
                    .iter()
                    .any(|s| (s.from.get() + 16.0 / 60.0 - k.from.get()).abs() < 1e-9
                        || k.from.get() == FROM)
            );
        }
        // Dhanu sankranti on 2025-12-16 in India's civil date (about 04:20
        // IST, so the evening of the 15th in UTC).
        assert!(
            kharmas
                .iter()
                .any(|k| (2_461_025.27..2_461_026.27).contains(&k.from.get())),
            "{kharmas:?}"
        );
    });
}

#[test]
fn every_blackout_lies_inside_the_range_in_order() {
    with_season(|season| {
        for pair in season.windows(2) {
            assert!(pair[0].at.from.get() <= pair[1].at.from.get());
        }
        for b in season {
            assert!(b.at.from.get() >= FROM && b.at.to.get() <= TO, "{b:?}");
            assert!(!b.at.is_empty(), "{b:?}");
        }
    });
}

#[test]
fn asta_holds_each_conjunction_of_2026_and_nothing_else() {
    use teistro_astro::visibility::{Criterion, Heliacal};
    use teistro_core::angle::difference_deg;
    use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Ut1};
    use teistro_muhurta::season::asta_over;
    use teistro_port_ephemeris::{Body, Horizon};

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
    let heliacal = Heliacal::new(
        &completion,
        kathmandu,
        Criterion::SURYA_SIDDHANTA,
        Horizon::CENTRE_NO_REFRACTION,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let elongation = |body: Body, at: f64| {
        use teistro_astro::events::Longitudes;
        let t = JulianDay::<Ut1>::literal(at);
        let (b, _) = longitudes.longitude_and_speed(body, t).unwrap();
        let (s, _) = longitudes.longitude_and_speed(Body::Sun, t).unwrap();
        difference_deg(b, s).abs()
    };
    let year = Interval::literal(2_461_041.5, TO);
    let guru = asta_over(&heliacal, Body::Jupiter, year).unwrap();
    let shukra = asta_over(&heliacal, Body::Venus, year).unwrap();
    // Jupiter meets the Sun once a year; Venus twice in 2026, behind it in
    // January (so the year opens with it unseen) and before it in October.
    assert_eq!(guru.len(), 1, "{guru:?}");
    assert_eq!(shukra.len(), 2, "{shukra:?}");
    assert_eq!(shukra[0].at.from, year.from);
    for (body, found) in [(Body::Jupiter, &guru), (Body::Venus, &shukra)] {
        for b in found {
            // The body is near the Sun somewhere inside, by the longitudes
            // themselves rather than the visibility reading that found it.
            let mut nearest = f64::INFINITY;
            let mut at = b.at.from.get();
            while at < b.at.to.get() {
                nearest = nearest.min(elongation(body, at));
                at += 1.0;
            }
            assert!(nearest < 2.0, "{body:?} {b:?}: {nearest}°");
            // And no asta is longer than Venus's superior one, about ten
            // weeks at most.
            assert!(b.at.days() < 90.0, "{body:?} {b:?}");
        }
    }
    // Outside every window both bodies stand well clear of the Sun.
    for (body, found) in [(Body::Jupiter, &guru), (Body::Venus, &shukra)] {
        let mut at = year.from.get() + 0.5;
        while at < year.to.get() {
            if !found.iter().any(|b| b.at.contains(JulianDay::literal(at))) {
                assert!(elongation(body, at) > 5.0, "{body:?} at {at}");
            }
            at += 1.0;
        }
    }
}

#[test]
fn asta_pairs_last_sightings_with_the_next_first_ones() {
    use teistro_astro::visibility::{HeliacalEvent, HeliacalKind, Motion, Side, Visibility};
    use teistro_core::quantity::JulianDay;
    use teistro_muhurta::season::asta;
    use teistro_port_ephemeris::Body;

    let state = |at: f64, visible: bool| Visibility {
        body: Body::Venus,
        day_start: JulianDay::literal(at.floor() - 0.5),
        instant: JulianDay::literal(at),
        side: Side::East,
        motion: Motion::Direct,
        measure_deg: 0.0,
        threshold_deg: 0.0,
        visible,
        evaluations: 0,
    };
    let event = |kind, at| HeliacalEvent {
        kind,
        day: state(
            at,
            kind == HeliacalKind::MorningFirst || kind == HeliacalKind::EveningFirst,
        ),
    };
    let range = Interval::literal(100.0, 300.0);
    let events = [
        event(HeliacalKind::EveningFirst, 110.2),
        event(HeliacalKind::EveningLast, 150.8),
        event(HeliacalKind::MorningFirst, 160.2),
        event(HeliacalKind::MorningLast, 250.3),
    ];
    // Unseen at the start: the first stretch opens at the range's start;
    // the last runs to its end.
    let found = asta(
        BlackoutKind::ShukraAsta,
        &state(100.2, false),
        &events,
        range,
    );
    let spans: Vec<(f64, f64)> = found
        .iter()
        .map(|b| (b.at.from.get(), b.at.to.get()))
        .collect();
    assert_eq!(spans, [(100.0, 110.2), (150.8, 160.2), (250.3, 300.0)]);
    // Seen at the start, the first appearance closes nothing.
    let found = asta(
        BlackoutKind::ShukraAsta,
        &state(100.2, true),
        &events,
        range,
    );
    assert_eq!(found.len(), 2);
}

/// The baseline engine anchors each end of Chaturmas on the Sun's sign
/// rather than the month's name: Devshayani the bright eleventh with the
/// Sun in Cancer, Prabodhini the one with it in Scorpio. The two readings
/// agree in 2026 and part by a month at both ends in 2025, when both
/// elevenths fell before the Sun's ingress (crux C166).
#[test]
fn the_sun_sign_anchor_moves_2025s_chaturmas_a_month_and_leaves_2026s() {
    use teistro_astro::events::Search;
    use teistro_core::catalogue::Rashi;
    use teistro_core::quantity::{JulianDay, Ut1};
    use teistro_panchanga::limb::{Sidereal, signs_within};
    use teistro_port_ephemeris::{Body, Lattice, Quantity};

    let provider = Builtin::new();
    let completion = Completion::new(
        &provider,
        OverridePolicy::SdkOnly,
        DeltaTModel::TableThenModel,
    );
    let longitudes = completion.longitudes(Frame::CANONICAL);
    let range = Interval::literal(FROM, TO);
    let source = Sidereal::over(&longitudes, zodiac());
    let elevenths: Vec<f64> = Search::new(&source, Quantity::ELONGATION, Lattice::single(120.0))
        .between(
            JulianDay::<Ut1>::literal(FROM),
            JulianDay::<Ut1>::literal(TO),
        )
        .unwrap()
        .iter()
        .map(|e| e.instant.get())
        .collect();
    let sun = signs_within(&longitudes, Body::Sun, range, zodiac(), Some(31.0)).unwrap();
    let sign_at = |at: f64| {
        sun.iter()
            .find(|s| s.whole.from.get() <= at && at < s.whole.to.get())
            .unwrap()
            .member
    };
    let with_sun_in = |sign: Rashi| -> Vec<f64> {
        elevenths
            .iter()
            .copied()
            .filter(|at| sign_at(*at) == sign)
            .collect()
    };
    let (cancer, scorpio) = (with_sun_in(Rashi::Cancer), with_sun_in(Rashi::Scorpio));
    with_season(|season| {
        let named = of(season, BlackoutKind::Chaturmas);
        // 2025: a lunar month later at both ends.
        let late = |anchor: f64, named: f64| (anchor - named - 29.5).abs() < 1.5;
        assert!(late(cancer[0], named[0].from.get()), "{cancer:?} {named:?}");
        assert!(late(scorpio[0], named[0].to.get()), "{scorpio:?} {named:?}");
        // 2026: the same instants.
        assert!(
            (cancer[1] - named[1].from.get()).abs() < 1e-6,
            "{cancer:?} {named:?}"
        );
        assert!(
            (scorpio[1] - named[1].to.get()).abs() < 1e-6,
            "{scorpio:?} {named:?}"
        );
    });
}
