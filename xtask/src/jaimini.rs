//! The measurement pass over **Jaimini's significators**: the karakamsha
//! and the Brahma graha, over every recorded birth
//! (`03-design/jaimini-significators.md`).
//!
//! The corpus records the chara karakas and neither significator, so the
//! pass can hold the one input it records — the Atmakaraka the karakamsha
//! is read from — and **count** the rest: how often the karakamsha's houses
//! differ between the rasi chart and the navamsha (crux C130), and on how
//! many births each rule for Brahma finds none (C128), under every reading
//! of Scorpio's and Aquarius's lords (C126). It calls the shipped path,
//! `sdk.chart().jaimini`, so it turns over when the module does.
//!
//! `cargo xtask jaimini` writes the page; `check-jaimini` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Graha;
use teistro::dasha::jaimini::{JaiminiReading, NoBrahma};
use teistro::{Context, Ephemeris};

use crate::births::{Birth, CHARTS, births};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, capitalised, fill, spelled, table};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/jaimini-measured.md";

/// The profile the corpus was recorded under, which the births are
/// founded in.
const PROFILE: &str = "conformance-baseline";

/// How near a sign's edge an Atmakaraka the SDK and the recording part on
/// must stand for the difference to be the ephemerides' and not the
/// ranking's: the built-in tier's worst Sun is under an arc-second.
const EDGE_ARCSEC: f64 = 1.0;

/// The rules for Brahma, as the settings spell them.
const RULES: [&str; 2] = ["VERSES", "TRANSLATORS_NOTE"];

/// The readings of Scorpio's and Aquarius's lords, as the settings spell
/// them.
const CO_LORDSHIPS: [&str; 3] = ["NONE", "STRONGER_LORD", "BOTH"];

/// The nine grahas, in the catalogue's order.
const GRAHAS: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

pub(crate) fn generate(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page(root) {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask jaimini") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// A context under the recording's profile with a settings patch.
fn context(patch: &str) -> Result<Context, String> {
    Context::builder()
        .profile(PROFILE)
        .settings_json(patch)
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("{PROFILE} with {patch}: {why}"))
}

/// The recorded Atmakaraka of a birth under a scheme (`system7` or
/// `system8`).
fn recorded_atmakaraka(root: &Path, birth: &Birth, scheme: &str) -> Result<Graha, String> {
    let file = read_json(&root.join(CHARTS).join(format!("{}.json", birth.name)))?;
    let karakas = file["houses"]["chara_karakas"][scheme]
        .as_object()
        .ok_or_else(|| format!("{}: no recorded {scheme} karakas", birth.name))?;
    let key = karakas
        .iter()
        .find(|(_, karaka)| karaka.as_str() == Some("AK"))
        .map(|(graha, _)| graha.clone())
        .ok_or_else(|| format!("{}: no recorded Atmakaraka under {scheme}", birth.name))?;
    GRAHAS
        .into_iter()
        .find(|graha| graha.key() == key)
        .ok_or_else(|| format!("{}: a recorded Atmakaraka `{key}` is no graha", birth.name))
}

/// Arc-seconds from a longitude to the nearest sign boundary.
fn to_boundary(longitude_deg: f64) -> f64 {
    let into = longitude_deg.rem_euclid(30.0);
    into.min(30.0 - into) * 3600.0
}

/// How close to a sign boundary a disagreement lies: the recorded
/// Atmakaraka's distance from its sign's edge in the recording and in the
/// SDK's founding. The Atmakaraka is the graha furthest through its sign,
/// so a graha at a sign's very end ranks first in one and last in the other
/// when the two ephemerides put it on either side of the edge.
struct Boundary {
    text: String,
    arcsec: f64,
}

fn boundary_case(
    root: &Path,
    birth: &Birth,
    scheme: &str,
    recorded: Graha,
    ours: Graha,
) -> Result<Boundary, String> {
    let file = read_json(&root.join(CHARTS).join(format!("{}.json", birth.name)))?;
    let recorded_deg = file["positions"]["bodies"][recorded.key()]["sidereal_longitude_deg"]
        .as_f64()
        .ok_or_else(|| format!("{}: no recorded {}", birth.name, recorded.key()))?;
    let founded_deg = birth
        .document
        .foundation
        .graha(recorded)
        .map(|at| at.longitude_deg)
        .ok_or_else(|| format!("{}: no founded {}", birth.name, recorded.key()))?;
    let arcsec = to_boundary(recorded_deg).max(to_boundary(founded_deg));
    Ok(Boundary {
        text: format!(
            "`{}` ({scheme}): the recording ranks {} at {recorded_deg:.6}° and the SDK \
             {} with {} at {founded_deg:.6}°, {:.2}″ and {:.2}″ from a sign's edge",
            birth.name,
            recorded.key(),
            ours.key(),
            recorded.key(),
            to_boundary(recorded_deg),
            to_boundary(founded_deg),
        ),
        arcsec,
    })
}

/// One cell of the Brahma table: a rule under a co-lordship, over every
/// birth.
#[derive(Default)]
struct Tally {
    found: usize,
    none: BTreeMap<&'static str, usize>,
    passed: usize,
}

impl Tally {
    fn add(&mut self, reading: &JaiminiReading) {
        let brahma = &reading.brahma;
        match (brahma.graha, brahma.none) {
            (Some(_), _) => self.found += 1,
            (None, Some(why)) => {
                let reason = match why {
                    NoBrahma::NoLordQualifies => "no lord qualifies",
                    NoBrahma::NoPlanetInTheSixth => "no planet in the 6th",
                    NoBrahma::NoPlanetQualifies => "no planet qualifies",
                };
                *self.none.entry(reason).or_default() += 1;
            }
            (None, None) => {}
        }
        if brahma.passed_from.is_some() {
            self.passed += 1;
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "a page reads top to bottom, and splitting it would scatter its prose"
)]
fn page(root: &Path) -> Result<String, String> {
    let founding = context("{}")?;
    let born = births(root, &founding)?;

    // The Atmakaraka, against the recording under both schemes.
    let mut atmakaraka_agree = [0usize; 2];
    let mut atmakaraka_differ: Vec<Boundary> = Vec::new();
    for (at, (scheme, patch)) in [
        ("system7", r#"{"jaimini": {"chara_karakas": "SEVEN"}}"#),
        ("system8", r#"{"jaimini": {"chara_karakas": "EIGHT"}}"#),
    ]
    .into_iter()
    .enumerate()
    {
        let sdk = context(patch)?;
        for birth in &born {
            let ours = sdk
                .chart()
                .jaimini(&birth.document)
                .map_err(|why| format!("{}: {why}", birth.name))?
                .karakamsha
                .atmakaraka;
            let recorded = recorded_atmakaraka(root, birth, scheme)?;
            if ours == recorded {
                if let Some(count) = atmakaraka_agree.get_mut(at) {
                    *count += 1;
                }
            } else {
                atmakaraka_differ.push(boundary_case(root, birth, scheme, recorded, ours)?);
            }
        }
    }

    // The karakamsha's houses in the two charts (C130), under the
    // profile's own scheme.
    let mut houses = 0usize;
    let mut houses_differ = 0usize;
    let mut births_all_differ = 0usize;
    for birth in &born {
        let reading = founding
            .chart()
            .jaimini(&birth.document)
            .map_err(|why| format!("{}: {why}", birth.name))?;
        let pairs = reading
            .karakamsha
            .in_rasi
            .iter()
            .zip(reading.karakamsha.in_navamsha.iter());
        let differ = pairs.filter(|(rasi, navamsha)| rasi != navamsha).count();
        houses += GRAHAS.len();
        houses_differ += differ;
        if differ == GRAHAS.len() {
            births_all_differ += 1;
        }
    }

    // Brahma under every rule and co-lordship.
    let mut tallies: Vec<(&str, &str, Tally)> = Vec::new();
    for rule in RULES {
        for co in CO_LORDSHIPS {
            let sdk = context(&format!(
                r#"{{"jaimini": {{"brahma": "{rule}", "node_co_lordship": "{co}"}}}}"#
            ))?;
            let mut tally = Tally::default();
            for birth in &born {
                tally.add(
                    &sdk.chart()
                        .jaimini(&birth.document)
                        .map_err(|why| format!("{}: {why}", birth.name))?,
                );
            }
            tallies.push((rule, co, tally));
        }
    }

    let n = born.len();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Jaimini's significators, measured\n\n\
         Status: `generated` by `cargo xtask jaimini` over the recorded\n\
         births. Do not edit: `check-jaimini` regenerates this page and\n\
         fails on any difference.\n\n\
         The design this measures is `jaimini-significators.md`. The corpus\n\
         records the chara karakas and neither the karakamsha nor the Brahma\n\
         graha, so the one input it records is held, and the rest counted.\n"
    );
    let _ = writeln!(
        out,
        "## 1. The Atmakaraka the karakamsha is read from\n\n\
         Every recorded birth ({} of them), founded under `{PROFILE}` and\n\
         asked for its significators through `sdk.chart().jaimini`, against\n\
         the Atmakaraka the recording ranks: {} of {n} agree under seven\n\
         karakas and {} of {n} under eight{}\n",
        spelled(n),
        atmakaraka_agree[0],
        atmakaraka_agree[1],
        if atmakaraka_differ.is_empty() {
            String::from(".")
        } else {
            format!(
                ". Where they part, the recorded Atmakaraka stands on a sign's edge, \
                 where the two ephemerides put it on either side:\n\n{}",
                atmakaraka_differ
                    .iter()
                    .map(|case| format!("- {}", case.text))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        },
    );
    let _ = writeln!(
        out,
        "## 2. The karakamsha's houses, in two charts (C130)\n\n\
         A graha's house counted from the karakamsha in the rasi chart and\n\
         in the navamsha: {houses_differ} of {houses} differ, and on\n\
         {births_all_differ} of {n} births every graha's does. The schools\n\
         that read one chart and those that read the other answer\n\
         differently almost always, which is why both are reported.\n"
    );
    let mut rows = String::from(
        "| rule | Scorpio's and Aquarius's lords | found | none, and why | Saturn or a node passed it on |\n|---|---|---:|---|---:|\n",
    );
    for (rule, co, tally) in &tallies {
        let none = if tally.none.is_empty() {
            String::from("none")
        } else {
            tally
                .none
                .iter()
                .map(|(why, count)| format!("{count} {why}"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(
            rows,
            "| `{rule}` | `{co}` | {} | {none} | {} |",
            tally.found, tally.passed
        );
    }
    let _ = writeln!(
        out,
        "## 3. The Brahma graha, under each rule\n\n\
         Over the {} births, each rule under each reading of the dual\n\
         lords:\n\n{rows}",
        spelled(n)
    );
    let verses_none: usize = tallies
        .iter()
        .filter(|(rule, co, _)| *rule == "VERSES" && *co == "NONE")
        .map(|(_, _, tally)| tally.none.values().sum::<usize>())
        .sum();
    let note_none: usize = tallies
        .iter()
        .filter(|(rule, co, _)| *rule == "TRANSLATORS_NOTE" && *co == "NONE")
        .map(|(_, _, tally)| tally.none.values().sum::<usize>())
        .sum();

    let off_edge = atmakaraka_differ
        .iter()
        .filter(|case| case.arcsec > EDGE_ARCSEC)
        .count();
    let claims = vec![
        Claim::counted(
            "the SDK ranks the recording's Atmakaraka under seven karakas",
            n - atmakaraka_agree[0],
            n,
        ),
        Claim::counted(
            "the SDK ranks the recording's Atmakaraka under eight karakas",
            n - atmakaraka_agree[1],
            n,
        ),
        Claim::counted(
            "every Atmakaraka the SDK and the recording part on stands within an arc-second of a sign's edge",
            off_edge,
            atmakaraka_differ.len(),
        ),
        Claim::counted(
            "the karakamsha's houses are the same in the rasi chart and the navamsha",
            houses_differ,
            houses,
        ),
        Claim::counted(
            "the verses find a Brahma for every birth (C128)",
            verses_none,
            n,
        ),
        Claim::counted(
            "the translator's note finds a Brahma for every birth",
            note_none,
            n,
        ),
    ];
    let _ = writeln!(out, "## 4. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} of the {} claims {} falsified. A chart the verses find no Brahma\n\
         for has no Sthira dasa under them, and says so by name; the note's\n\
         rule is the knob that supplies one (`jaimini.brahma`).\n",
        capitalised(&spelled(falsified)),
        spelled(claims.len()),
        if falsified == 1 { "is" } else { "are" }
    );
    Ok(fill(&out))
}
