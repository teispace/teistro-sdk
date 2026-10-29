//! KP against the corpus's `baseline/kp`: from each fixture's recorded
//! sidereal longitudes and Placidus cusps, every cusp's and every graha's
//! KP number and lords, each graha's house, the four significator levels of
//! every house, the houses each graha signifies, and the ruling planets'
//! seven slots, as the recording engine read them
//! (`05-testing/01-golden-vectors.md`, the second export's KP).
//!
//! Two of the engine's readings are the SDK's own knobs and are taken from
//! `conformance-baseline`: seven ruling planets with the sub lords (C150) and
//! the civil weekday's lord (C151). Two are not, and each miss they make is
//! counted against its entry: the engine's equal houses where its Placidus
//! degenerates, and an intercepted sign's lord as a significator (C154).

#![allow(
    clippy::panic,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::print_stdout,
    clippy::too_many_lines,
    reason = "tests fail by panicking, index JSON by key, print what they skip and walk a file's every comparison in one test"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[path = "../../core/tests/support/conformance.rs"]
mod conformance;

use conformance::{Tally, carried};

use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Graha, HouseSystem, Vara};
use teistro_core::quantity::Degrees;
use teistro_core::settings::{DayLordDay, Profile, RulingCount, SettingsPatch};
use teistro_kp::{
    KpChart, KpNumber, Lords, Position, Reason, RulingPlanets, RulingRules, Significators,
};

const SECTION: &str = "baseline/kp";

/// The engine's cusps where its Placidus degenerates: equal houses from the
/// ascendant, where the SDK casts Placidus as the sphere gives it.
const FALLBACK: &str = "the engine's equal houses where its Placidus degenerates (kp.md)";

/// An intercepted sign's lord, which the engine counts a significator of its
/// house and the SDK reports beside the levels.
const INTERCEPTED: &str = "an intercepted sign's lord as a significator (kp.md, C154)";

const GRAHAS: [&str; 9] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU",
];

fn files() -> Vec<(String, Value)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/baseline/kp");
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .unwrap_or_else(|err| panic!("{dir}: {err}; the corpus lists {SECTION}"))
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let file = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
            out.push((
                format!("{dir}/{}", path.file_name().unwrap().to_string_lossy()),
                file,
            ));
        }
    }
    out
}

fn nas(value: &Value) -> Nas {
    Nas::from_degrees(Degrees::try_new(value.as_f64().unwrap().rem_euclid(360.0)).unwrap())
}

fn graha(value: &Value) -> Graha {
    Graha::from_key(value.as_str().unwrap()).unwrap()
}

fn grahas(value: &Value) -> BTreeSet<Graha> {
    value.as_array().unwrap().iter().map(graha).collect()
}

/// The recorded lords of a cusp or a graha, as the SDK's are compared.
fn recorded_lords(value: &Value) -> [Graha; 4] {
    ["sign_lord", "star_lord", "sub_lord", "sub_sub_lord"].map(|key| graha(&value[key]))
}

fn our_lords(lords: &Lords) -> [Graha; 4] {
    [
        lords.sign,
        lords.star.lord,
        lords.sub.lord,
        lords.sub_sub.lord,
    ]
}

/// The lord of the weekday on the birth's own clock, which is what the
/// SDK's `kp.day_lord_day = CIVIL` reads: Julian day 0.5 began a Monday.
fn civil_day_lord(jd_ut: f64, offset_min: f64) -> Graha {
    let weekday = (jd_ut + offset_min / 1440.0 + 1.5).floor().rem_euclid(7.0);
    Vara::ALL
        .into_iter()
        // Both are whole days, so half a day apart is apart.
        .find(|vara| (f64::from(vara.attributes().weekday) - weekday).abs() < 0.5)
        .map(|vara| vara.attributes().lord)
        .unwrap()
}

/// The comparisons of one file, each agreeing, explained by an entry, or
/// kept as an unexplained miss, so that a run reports every miss before it
/// fails.
struct Compared<'a> {
    tally: &'a mut Tally,
    unexplained: &'a mut Vec<String>,
    /// Whether the engine replaced this chart's degenerate Placidus.
    fallback: bool,
}

impl Compared<'_> {
    /// One comparison: on a chart whose cusps the engine replaced, a miss
    /// is the fallback's; elsewhere it is `explained`'s when that names an
    /// entry, and unexplained otherwise.
    fn check<T: PartialEq + std::fmt::Debug>(
        &mut self,
        ours: &T,
        recorded: &T,
        explained: Option<&'static str>,
        what: impl Fn() -> String,
    ) {
        if ours == recorded {
            self.tally.agreed(1);
        } else if self.fallback {
            self.tally.explained(FALLBACK, 1);
        } else if let Some(entry) = explained {
            self.tally.explained(entry, 1);
        } else {
            self.unexplained
                .push(format!("{}: ours {ours:?}, recorded {recorded:?}", what()));
        }
    }
}

#[test]
fn every_recorded_kp_reading_is_the_sdk_s_under_the_engine_s_reading() {
    if let Err(skipped) = carried(SECTION) {
        println!("{skipped}");
        return;
    }
    let settings = Profile::shipped("conformance-baseline")
        .unwrap()
        .resolve(&SettingsPatch::default())
        .unwrap()
        .settings;
    assert_eq!(settings.kp.ruling_count, RulingCount::WithSubs);
    assert_eq!(settings.kp.day_lord_day, DayLordDay::Civil);
    let files = files();
    let mut tally = Tally::new(SECTION, "the engine's reading");
    let mut unexplained = Vec::new();
    let mut replaced = 0;
    for (name, file) in &files {
        let inputs = &file["inputs"];
        let cusps: [Nas; 12] = std::array::from_fn(|i| nas(&inputs["placidus"]["cusps"][i]));
        // The engine replaces a degenerate Placidus with equal houses; the
        // cusps it used are the file's.
        let fallback = (0..12).any(|i| {
            (file["cusps"][i]["longitude_deg"].as_f64().unwrap()
                - inputs["placidus"]["cusps"][i].as_f64().unwrap())
            .abs()
                > 1e-9
        });
        replaced += usize::from(fallback);
        let positions = GRAHAS.map(|key| {
            Position::new(
                Graha::from_key(key).unwrap(),
                nas(&inputs["graha_longitude_deg"][key]),
            )
            .retrograde(inputs["graha_retrograde"][key].as_bool().unwrap())
        });
        let chart = KpChart::new(HouseSystem::Placidus, cusps, positions);
        let mut c = Compared {
            tally: &mut tally,
            unexplained: &mut unexplained,
            fallback,
        };

        for (cusp, recorded) in chart.cusps.iter().zip(file["cusps"].as_array().unwrap()) {
            let house = cusp.house;
            c.check(
                &u64::from(KpNumber::of(cusp.longitude).get()),
                &recorded["kp_number"].as_u64().unwrap(),
                None,
                || format!("{name} cusp {house}: KP number"),
            );
            c.check(
                &our_lords(&cusp.lords),
                &recorded_lords(recorded),
                None,
                || format!("{name} cusp {house}: lords"),
            );
        }

        for key in GRAHAS {
            let recorded = &file["grahas"][key];
            let planet = chart.planet(Graha::from_key(key).unwrap()).unwrap();
            // A graha's lords read its own longitude and no cusp, so even
            // the fallback cannot move them.
            let mut own = Compared {
                tally: &mut *c.tally,
                unexplained: &mut *c.unexplained,
                fallback: false,
            };
            own.check(
                &u64::from(KpNumber::of(planet.longitude).get()),
                &recorded["kp_number"].as_u64().unwrap(),
                None,
                || format!("{name} {key}: KP number"),
            );
            own.check(
                &our_lords(&planet.lords),
                &recorded_lords(recorded),
                None,
                || format!("{name} {key}: lords"),
            );
            c.check(
                &u64::from(planet.house),
                &recorded["house"].as_u64().unwrap(),
                None,
                || format!("{name} {key}: house"),
            );
        }

        // The four levels of every house. The engine adds to the fourth the
        // lords of the signs a house holds whole, and so to the third the
        // planets in their stars (C154).
        let significators = Significators::of(&chart, settings.aspect.node_aspects);
        let star_of = |g: Graha| chart.planet(g).unwrap().lords.star.lord;
        let mut signified: BTreeMap<Graha, (BTreeSet<u64>, BTreeSet<u64>)> = BTreeMap::new();
        for (ours, recorded) in significators
            .houses
            .iter()
            .zip(file["significators"].as_array().unwrap())
        {
            let house = ours.house;
            let intercepted: BTreeSet<Graha> = ours
                .intercepted
                .iter()
                .map(|sign| sign.attributes().lord)
                .collect();
            let levels: [BTreeSet<Graha>; 4] =
                ours.levels().map(|level| level.iter().copied().collect());
            let mut with_intercepted = levels.clone();
            with_intercepted[3].extend(intercepted.iter().copied());
            with_intercepted[2].extend(
                GRAHAS
                    .map(|k| Graha::from_key(k).unwrap())
                    .into_iter()
                    .filter(|g| intercepted.contains(&star_of(*g))),
            );
            for (level, key) in ["level1", "level2", "level3", "level4"]
                .into_iter()
                .enumerate()
            {
                let theirs = grahas(&recorded[key]);
                let explained = (with_intercepted[level] == theirs).then_some(INTERCEPTED);
                c.check(&levels[level], &theirs, explained, || {
                    format!("{name} house {house}: {key}")
                });
            }
            for g in GRAHAS.map(|k| Graha::from_key(k).unwrap()) {
                let entry = signified.entry(g).or_default();
                if levels.iter().any(|level| level.contains(&g)) {
                    entry.0.insert(u64::from(house));
                }
                if with_intercepted.iter().any(|level| level.contains(&g)) {
                    entry.1.insert(u64::from(house));
                }
            }
        }
        for key in GRAHAS {
            let (mine, with_intercepted) = &signified[&Graha::from_key(key).unwrap()];
            let theirs: BTreeSet<u64> = file["grahas"][key]["signifies"]
                .as_array()
                .unwrap()
                .iter()
                .map(|h| h.as_u64().unwrap())
                .collect();
            let explained = (*with_intercepted == theirs).then_some(INTERCEPTED);
            c.check(mine, &theirs, explained, || {
                format!("{name} {key}: houses signified")
            });
        }

        // The ruling planets: the lagna's three lords, the Moon's three and
        // the day's, slot by slot; then the rulers as a set, the engine
        // naming no node as an agent.
        let day_lord = civil_day_lord(
            inputs["jd_ut"].as_f64().unwrap(),
            inputs["tz_offset_min"].as_f64().unwrap(),
        );
        let ruling = RulingPlanets::of(&chart, day_lord, RulingRules::of(&settings));
        let recorded = &file["ruling_planets"];
        let lagna = chart.cusps[0].lords;
        c.check(
            &[lagna.sign, lagna.star.lord, lagna.sub.lord],
            &[0, 1, 2].map(|i| graha(&recorded["ascendant"][i])),
            None,
            || format!("{name}: the lagna's ruling lords"),
        );
        let moon = chart.planet(Graha::Moon).unwrap().lords;
        c.check(
            &[moon.sign, moon.star.lord, moon.sub.lord],
            &[0, 1, 2].map(|i| graha(&recorded["moon"][i])),
            None,
            || format!("{name}: the Moon's ruling lords"),
        );
        c.check(&day_lord, &graha(&recorded["day_lord"]), None, || {
            format!("{name}: the day lord")
        });
        let rulers: BTreeSet<Graha> = ruling
            .rulers
            .iter()
            .filter(|ruler| !matches!(ruler.reasons.first(), Some(Reason::Agent { .. })))
            .map(|ruler| ruler.graha)
            .collect();
        c.check(&rulers, &grahas(&recorded["ranked"]), None, || {
            format!("{name}: the rulers")
        });
    }
    for miss in &unexplained {
        println!("unexplained: {miss}");
    }
    println!(
        "{} files, {replaced} with the engine's equal houses; {} unexplained",
        files.len(),
        unexplained.len()
    );
    assert!(
        unexplained.is_empty(),
        "{} misses no entry explains",
        unexplained.len()
    );
    assert_eq!(files.len(), 75);
    tally.record();
}
