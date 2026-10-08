//! The purifier against the gloss's worked figures, and the edge finder
//! against a sky whose every change is known.

#![allow(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they built"
)]

use teistro_core::catalogue::{Rashi, Vara};
use teistro_core::error::{Error, Status};
use teistro_core::quantity::{JulianDay, Utc};

use crate::purifier::Judge;
use crate::{
    Day, EDGE_TOLERANCE_DAYS, GulikaAt, GulikaExtension, Native, PranapadaRule, Purifier, PurifyAs,
    Reference, Rules, Sky, Window, gulika_instant, narrow, pranapada_deg, pranapada_working,
};

/// A ghati in days.
const GHATI: f64 = 1.0 / 60.0;

/// The gloss's Sun, 2s 4°28′1″.
const SUN: f64 = 64.0 + 28.0 / 60.0 + 1.0 / 3600.0;

/// 6;17 ghatis in hours.
const EXAMPLE_HOURS: f64 = (6.0 * 60.0 + 17.0) * 24.0 / 3600.0;

/// A sky of straight lines: sunrise at a quarter past each Julian day, a
/// twelve-hour daylight, a lagna turning once a day from 0° at sunrise, a
/// Sun and a Moon that stand still.
struct Line {
    moon: f64,
}

const BASE: f64 = 2_460_000.25;

impl Sky for Line {
    fn ascendant_deg(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(((at.get() - BASE) * 360.0).rem_euclid(360.0))
    }
    fn sun_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(SUN)
    }
    fn moon_deg(&self, _: JulianDay<Utc>) -> Result<f64, Error> {
        Ok(self.moon)
    }
    fn day(&self, at: JulianDay<Utc>) -> Result<Day, Error> {
        let sunrise = BASE + (at.get() - BASE).floor();
        Ok(Day::literal(
            sunrise,
            sunrise + 0.5,
            sunrise + 1.0,
            Vara::Budhavara,
        ))
    }
}

const SKY: Line = Line { moon: 100.0 };

fn window(from_hours: f64, to_hours: f64) -> Window {
    Window::between(
        JulianDay::literal(BASE + from_hours / 24.0),
        JulianDay::literal(BASE + to_hours / 24.0),
    )
    .expect("a window")
}

#[test]
fn the_verse_count_is_the_1899_working_figure_for_figure() {
    let working = pranapada_working(6, 17);
    assert_eq!(working.signs, 25);
    assert_eq!(working.palas_left, 2);
    assert_eq!(working.degrees, 4);
    assert_eq!(working.mod_twelve, 1);
    assert_eq!(working.mod_fifteen, 10);
}

#[test]
fn the_verse_gives_scorpio_and_the_print_gives_three_signs_four_degrees() {
    let verse = pranapada_deg(PranapadaRule::Verse, SUN, EXAMPLE_HOURS).expect("verse");
    // A dual Sun shifts to its 5th: 64°28′1″ + 120° + 34° (25 signs and 4°).
    assert_eq!(Rashi::of_longitude(verse), Rashi::Scorpio);
    assert!((verse - (218.0 + 28.0 / 60.0 + 1.0 / 3600.0)).abs() < 1e-9);

    // The differing cell: the print answers Cancer 4°, which the verse's
    // sequence does not reach.
    let printed =
        pranapada_deg(PranapadaRule::PrintedExample, SUN, EXAMPLE_HOURS).expect("printed");
    assert!((printed - 94.0).abs() < 1e-9);
    assert_eq!(Rashi::of_longitude(printed), Rashi::Cancer);

    // The other reading that reaches it: the mod-12 count added to the
    // Sun's sign with no shift, 2s + 1s 4°.
    let working = pranapada_working(6, 17);
    let unshifted = Rashi::of_longitude(SUN).start_deg()
        + 30.0 * f64::from(working.mod_twelve)
        + f64::from(working.degrees);
    assert!((unshifted - printed).abs() < 1e-9);
}

#[test]
fn the_printed_reading_counts_its_fifteenth_sign_on_the_movable_sign() {
    // Fifteen signs elapsed is zero mod 15, which counted inclusively is
    // the sign before Libra.
    let printed = pranapada_deg(PranapadaRule::PrintedExample, SUN, 1.5).expect("printed");
    assert_eq!(Rashi::of_longitude(printed), Rashi::Virgo);
    // One sign elapsed, the count's first, is the movable sign itself: a
    // Leo Sun counts from Aries, a Taurus Sun from Capricorn.
    for (sun, from) in [(125.0, Rashi::Aries), (40.0, Rashi::Capricorn)] {
        let first = pranapada_deg(PranapadaRule::PrintedExample, sun, 0.1).expect("printed");
        assert_eq!(Rashi::of_longitude(first), from, "a Sun at {sun}°");
    }
}

#[test]
fn the_sdk_point_is_a_knob_and_moves_at_its_own_rate() {
    let point = pranapada_deg(PranapadaRule::SdkPoint, SUN, 1.0).expect("point");
    let again = pranapada_deg(PranapadaRule::SdkPoint, SUN, 2.0).expect("point");
    assert!(((again - point).rem_euclid(360.0) - 60.0).abs() < 1e-9);
}

#[test]
fn gulika_ends_saturns_eighth_at_the_gloss_figure() {
    // A Wednesday of 33;14 ghatis: an eighth of 4;09;15, Saturn's the
    // fourth, its end at 16;37.
    let day = Day::literal(
        0.0,
        (33.0 + 14.0 / 60.0) * GHATI,
        60.0 * GHATI,
        Vara::Budhavara,
    );
    let end = gulika_instant(&day, JulianDay::literal(0.1), GulikaAt::End).expect("end");
    assert!((end.get() / GHATI - (16.0 + 37.0 / 60.0)).abs() < 1.0 / 120.0);
    let start = gulika_instant(&day, JulianDay::literal(0.1), GulikaAt::Start).expect("start");
    assert!(((end.get() - start.get()) / GHATI - (4.0 + 9.25 / 60.0)).abs() < 1e-9);
}

#[test]
fn a_human_lagna_is_held_in_a_trine_and_nowhere_else() {
    let rules = Rules {
        pranapada: false,
        gulika: false,
        ..Rules::default()
    };
    let judge = Judge::new(&SKY, &rules);
    // The Moon at 100°, Cancer; the lagna turns a sign every two hours.
    for sign in 0..12_u8 {
        let at = JulianDay::literal(BASE + (f64::from(sign) * 2.0 + 1.0) / 24.0);
        let verdict = judge.verdict(at).expect("verdict");
        let clause = verdict.clauses.first().expect("the Moon's clause");
        assert_eq!(clause.purifier, Purifier::Moon);
        assert_eq!(clause.sign, Rashi::Cancer);
        let house = (sign + 12 - 3) % 12 + 1;
        assert_eq!(clause.house, house);
        assert_eq!(clause.held, [1, 5, 9].contains(&house), "house {house}");
        assert_eq!(verdict.pure, clause.held);
    }
}

#[test]
fn each_species_takes_its_own_three_houses() {
    let all: Vec<u8> = [Native::Human, Native::Beast, Native::Bird, Native::Creeper]
        .iter()
        .flat_map(|native| native.houses())
        .collect();
    let mut sorted = all.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, (1..=12).collect::<Vec<u8>>());
}

#[test]
fn gulikas_extension_adds_three_clauses_to_gulika_alone() {
    let rules = Rules::default();
    let judge = Judge::new(&SKY, &rules);
    let verdict = judge
        .verdict(JulianDay::literal(BASE + 0.1))
        .expect("verdict");
    let references: Vec<(Purifier, Reference)> = verdict
        .clauses
        .iter()
        .map(|clause| (clause.purifier, clause.reference))
        .collect();
    assert_eq!(
        references,
        [
            (Purifier::Pranapada, Reference::Itself),
            (Purifier::Gulika, Reference::Itself),
            (Purifier::Gulika, Reference::Seventh),
            (Purifier::Gulika, Reference::Navamsha),
            (Purifier::Gulika, Reference::NavamshaSeventh),
            (Purifier::Moon, Reference::Itself),
        ]
    );
    let gulika = verdict.clauses.get(1).expect("Gulika");
    let seventh = verdict.clauses.get(2).expect("its 7th");
    assert_eq!(gulika.sign.opposite(), seventh.sign);
}

/// X7: "when the two are weak" read as "neither purifies" keeps exactly
/// the instants the extension always counted keeps, and says which
/// clauses counted; without it Gulika reads its own sign alone.
#[test]
fn gulikas_extension_counts_only_when_the_two_fail() {
    let rules = |gulika_extension| Rules {
        gulika_extension,
        ..Rules::default()
    };
    let (two_fail, always, never) = (
        rules(GulikaExtension::WhenTwoFail),
        rules(GulikaExtension::Always),
        rules(GulikaExtension::Never),
    );
    let (by_two_fail, by_always, by_never) = (
        Judge::new(&SKY, &two_fail),
        Judge::new(&SKY, &always),
        Judge::new(&SKY, &never),
    );
    let (mut fallback, mut withheld) = (0, 0);
    for minute in 0..1440 {
        let at = JulianDay::literal(BASE + f64::from(minute) / 1440.0);
        let verdict = by_two_fail.verdict(at).expect("verdict");
        let plain = by_always.verdict(at).expect("verdict");
        assert_eq!(
            verdict.pure, plain.pure,
            "minute {minute}: the same instants"
        );
        assert!(plain.clauses.iter().all(|c| c.counted));
        let two_hold = verdict.held().any(|c| c.purifier != Purifier::Gulika);
        for clause in &verdict.clauses {
            let extension = clause.reference != Reference::Itself;
            assert_eq!(clause.counted, !(extension && two_hold), "minute {minute}");
        }
        if !two_hold
            && verdict
                .purified_by()
                .any(|c| c.reference != Reference::Itself)
        {
            fallback += 1;
        }
        if two_hold && verdict.held().any(|c| !c.counted) {
            withheld += 1;
        }
        let alone = by_never.verdict(at).expect("verdict");
        assert_eq!(alone.clauses.len(), 3, "the three purifiers' own signs");
    }
    // Both cases happen on this sky, so neither branch is idle.
    assert!(fallback > 0 && withheld > 0, "{fallback} and {withheld}");
}

#[test]
fn the_runs_tile_the_window_and_change_at_every_edge() {
    let window = window(1.0, 7.0);
    let found = narrow(window, &SKY, &Rules::default()).expect("narrowed");
    let mut runs: Vec<_> = found.intervals.iter().chain(&found.removed).collect();
    runs.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    assert_eq!(runs.first().expect("a run").from, window.from);
    assert_eq!(runs.last().expect("a run").to, window.to);
    for pair in runs.windows(2) {
        let [a, b] = pair else { unreachable!() };
        assert_eq!(a.to, b.from, "the runs leave no gap");
        assert_ne!(a.verdict, b.verdict, "neighbours that agree are merged");
    }
    assert_eq!(found.edges.len(), runs.len() - 1);
    // Every edge is a change, pinned to the tolerance.
    let rules = Rules::default();
    let judge = Judge::new(&SKY, &rules);
    for edge in &found.edges {
        let before = judge.verdict(JulianDay::literal(edge.get() - EDGE_TOLERANCE_DAYS));
        let after = judge.verdict(JulianDay::literal(edge.get() + EDGE_TOLERANCE_DAYS));
        assert_ne!(
            before.expect("before"),
            after.expect("after"),
            "at {}",
            edge.get()
        );
    }
    assert!(found.intervals.iter().all(|run| run.verdict.pure));
    assert!(found.removed.iter().all(|run| !run.verdict.pure));
    assert_eq!(found.grid.cells, 360);
}

#[test]
fn the_verses_pranapada_moves_a_sign_every_six_minutes() {
    // Only the pranapada judges, so the runs change with it or the lagna.
    let rules = Rules {
        gulika: false,
        moon: false,
        ..Rules::default()
    };
    let found = narrow(window(1.0, 1.9), &SKY, &rules).expect("narrowed");
    let pranapada = |at: f64| {
        let hours = (at - BASE) * 24.0;
        Rashi::of_longitude(pranapada_deg(PranapadaRule::Verse, SUN, hours).expect("deg"))
    };
    for edge in &found.edges {
        let at = edge.get();
        let minutes = (at - BASE) * 24.0 * 60.0;
        assert_ne!(
            pranapada(at - 1e-6),
            pranapada(at + 1e-6),
            "at {minutes} minutes"
        );
    }
    // The lagna stays in Aries through the window, so the pranapada is
    // pure in three signs of every twelve, Aries, Leo and Sagittarius, a
    // sign at a time: no pure run outlasts six minutes.
    assert_ne!(found.intervals, []);
    for run in &found.intervals {
        let minutes = (run.to.get() - run.from.get()) * 24.0 * 60.0;
        assert!(minutes < 6.0 + 1e-3, "a pure run of {minutes} minutes");
    }
}

#[test]
fn a_sunrise_inside_the_window_is_an_edge() {
    let found = narrow(window(23.0, 25.0), &SKY, &Rules::default()).expect("narrowed");
    let sunrise = BASE + 1.0;
    assert!(
        found
            .edges
            .iter()
            .any(|edge| (edge.get() - sunrise).abs() < EDGE_TOLERANCE_DAYS),
        "{:?}",
        found.edges
    );
}

#[test]
fn a_weight_keeps_every_run_and_the_same_edges() {
    let barred = narrow(window(1.0, 4.0), &SKY, &Rules::default()).expect("barred");
    let weighed = narrow(
        window(1.0, 4.0),
        &SKY,
        &Rules {
            purify_as: PurifyAs::Weight,
            ..Rules::default()
        },
    )
    .expect("weighed");
    assert_eq!(weighed.removed, []);
    assert_eq!(barred.edges, weighed.edges);
    assert_eq!(
        weighed.intervals.len(),
        barred.intervals.len() + barred.removed.len()
    );
}

#[test]
fn a_window_and_its_rules_are_refused_by_field() {
    let at = JulianDay::literal(BASE);
    let reversed = Window::between(at, at).expect_err("empty");
    assert_eq!(reversed.status, Status::InvalidArg);
    assert_eq!(reversed.field(), Some("rectification.window.to"));
    let long = Window::between(at, JulianDay::literal(BASE + 1.6)).expect_err("too long");
    assert_eq!(long.status, Status::OutOfRange);

    let none = Rules {
        pranapada: false,
        gulika: false,
        moon: false,
        ..Rules::default()
    };
    let refused = narrow(window(1.0, 2.0), &SKY, &none).expect_err("nothing purifies");
    assert_eq!(refused.field(), Some("rectification.purifiers"));
    let coarse = Rules {
        seed_minutes: 0.0,
        ..Rules::default()
    };
    let refused = narrow(window(1.0, 2.0), &SKY, &coarse).expect_err("no step");
    assert_eq!(refused.field(), Some("rectification.seedMinutes"));
}

#[test]
fn rules_cross_in_the_knob_spellings_the_design_names() {
    let json = serde_json::to_value(Rules::default()).expect("json");
    assert_eq!(json["pranapadaRule"], "VERSE");
    assert_eq!(json["gulikaAt"], "END");
    assert_eq!(json["purifyAs"], "BAR");
    assert_eq!(json["native"], "HUMAN");
    let partial: Rules =
        serde_json::from_str(r#"{"purifyAs":"WEIGHT"}"#).expect("a partial rule set");
    assert_eq!(partial.purify_as, PurifyAs::Weight);
    assert_eq!(partial.pranapada_rule, PranapadaRule::Verse);
}
