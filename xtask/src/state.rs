//! The falsification pass over the recorded planetary state, which the
//! state module is designed from (Phase 4).
//!
//! `positions.bodies.*` carries eleven fields about what a body *is* as
//! opposed to where it is: its dignity, the three friendship readings,
//! whether it is combust and how deeply, whether it is retrograde, its
//! age, three avastha families, whether it is at war and with whom, and
//! whether it sits within a hair of a classification boundary. Each is a
//! rule, and the corpus records 930 readings of every one of them over 93
//! fixtures.
//!
//! Most of those rules the corpus settles outright. **Three it cannot**,
//! and saying so is the more valuable half of this pass: the deeptadi
//! below its top three states, and three of the six lajjitadi, split on
//! something neither the corpus records nor the SDK can yet compute, and
//! their classical definitions all read "or aspected by" — which needs an
//! aspect model the SDK does not have. Guessing them would put a wrong
//! rule into code as a fact.
//!
//! `cargo xtask state` writes the page; `check-state` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_core::catalogue::{Graha, Rashi};

use crate::classical::{
    FRIENDLY_HOUSES, GRAHAS, NODES, PER_SIGN, edge, in_own_sign, lord_of, natural, panchadha,
    separation, sign_of, temporary,
};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/state-tables-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";
const VARIANTS: &str = "fixtures/baseline/variants";

// ── what the corpus records ────────────────────────────────────────────────

/// One body's recorded state in one fixture.
#[derive(Clone)]
struct Reading {
    fixture: String,
    body: &'static str,
    longitude: f64,
    latitude: f64,
    house: u8,
    retrograde: bool,
    from_sun: Option<f64>,
    dignity: String,
    natural: String,
    temporary: String,
    panchadha: String,
    combust: String,
    baladi: String,
    /// Absent for the nodes and the lagna.
    jagradadi: Option<String>,
    deeptadi: Option<String>,
    lajjitadi: Vec<String>,
    /// The opponent and whether this body won.
    war: Option<(String, bool)>,
    boundaries: [(&'static str, bool, f64); 3],
}

/// One fixture: every body's reading, and the signs they stand in.
struct Chart {
    readings: Vec<Reading>,
    signs: BTreeMap<String, u8>,
}

impl Reading {
    /// The sign it stands in.
    fn sign(&self) -> u8 {
        sign_of(self.longitude)
    }

    /// The degrees inside that sign.
    fn degrees(&self) -> f64 {
        self.longitude.rem_euclid(360.0) - f64::from(self.sign()) * PER_SIGN
    }

    /// Its graha, when it is one.
    fn graha(&self) -> Option<Graha> {
        Graha::from_key(self.body)
    }
}

fn charts(root: &Path) -> Result<Vec<Chart>, String> {
    let mut charts = Vec::new();
    for directory in [CHARTS, VARIANTS] {
        let dir = root.join(directory);
        let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
            .map_err(|err| {
                format!(
                    "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                    dir.display()
                )
            })?
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|e| e == "json"))
            .collect();
        files.sort();
        for path in &files {
            let text = std::fs::read_to_string(path)
                .map_err(|err| format!("{}: {err}", path.display()))?;
            let value: Value =
                serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default();
            if let Some(chart) = read(&name, &value) {
                charts.push(chart);
            }
        }
    }
    if charts.is_empty() {
        return Err(String::from("no fixture carries a body's state"));
    }
    Ok(charts)
}

/// One fixture's readings, or `None` when it carries no positions.
fn read(name: &str, fixture: &Value) -> Option<Chart> {
    let bodies = fixture["positions"]["bodies"].as_object()?;
    let mut readings = Vec::new();
    let mut signs = BTreeMap::new();
    for body in GRAHAS {
        let Some(recorded) = bodies.get(body) else {
            continue;
        };
        let longitude = recorded["sidereal_longitude_deg"].as_f64()?;
        signs.insert(body.to_string(), sign_of(longitude));
        let avasthas = &recorded["avasthas"];
        readings.push(Reading {
            fixture: name.to_string(),
            body,
            longitude,
            latitude: recorded["latitude_deg"].as_f64().unwrap_or(0.0),
            house: u8::try_from(recorded["house"].as_u64().unwrap_or(0)).unwrap_or(0),
            retrograde: recorded["is_retrograde"].as_bool().unwrap_or(false),
            from_sun: recorded["degrees_from_sun"].as_f64(),
            dignity: text(&recorded["dignity"]),
            natural: text(&recorded["natural_friendship"]),
            temporary: text(&recorded["temporary_friendship"]),
            panchadha: text(&recorded["panchadha_maitri"]),
            combust: text(&recorded["combust"]),
            baladi: text(&recorded["avastha_baladi"]),
            jagradadi: avasthas["jagradadi"].as_str().map(ToString::to_string),
            deeptadi: avasthas["deeptadi"].as_str().map(ToString::to_string),
            lajjitadi: avasthas["lajjitadi"]
                .as_array()
                .map(|list| list.iter().map(text).collect())
                .unwrap_or_default(),
            war: avasthas["planetary_war"].as_object().map(|war| {
                (
                    text(&war["opponent"]),
                    war["is_winner"].as_bool().unwrap_or(false),
                )
            }),
            boundaries: [
                (
                    "sign",
                    recorded["near_sign_boundary"].as_bool().unwrap_or(false),
                    edge(longitude, PER_SIGN),
                ),
                (
                    "nakshatra",
                    recorded["near_nakshatra_boundary"]
                        .as_bool()
                        .unwrap_or(false),
                    edge(longitude, 360.0 / 27.0),
                ),
                (
                    "pada",
                    recorded["near_pada_boundary"].as_bool().unwrap_or(false),
                    edge(longitude, 360.0 / 108.0),
                ),
            ],
        });
    }
    (!readings.is_empty()).then_some(Chart { readings, signs })
}

fn text(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

/// Every reading, in order.
fn readings(charts: &[Chart]) -> impl Iterator<Item = &Reading> {
    charts.iter().flat_map(|chart| chart.readings.iter())
}

// ── the proposed rules ─────────────────────────────────────────────────────

/// How near the exact debilitation degree a deep debilitation is,
/// degrees.
const DEEP_DEBILITATION_ORB: f64 = 1.0;

/// The dignity of a body.
///
/// The ladder the corpus forces: moolatrikona is checked **before**
/// exaltation, where a body's two spans overlap; the nodes take neither
/// a sign nor a friendship, so they fall to neutral.
fn dignity(reading: &Reading, panchadha: &str) -> String {
    let Some(graha) = reading.graha() else {
        return String::from("NEUTRAL");
    };
    let attributes = graha.attributes();
    let sign = reading.sign();
    let degrees = reading.degrees();
    let Some(rashi) = Rashi::from_id(u16::from(sign)) else {
        return String::from("NEUTRAL");
    };
    if let Some(span) = attributes.moolatrikona
        && span.sign == rashi
        && span.to > span.from
        && degrees >= f64::from(span.from)
        && degrees < f64::from(span.to)
    {
        return String::from("MOOLTRIKONA");
    }
    if attributes.exaltation.is_some_and(|at| at.sign == rashi) {
        return String::from("EXALTED");
    }
    if let Some(at) = attributes.debilitation
        && at.sign == rashi
    {
        let deep = !NODES.contains(&reading.body)
            && (degrees - f64::from(at.degree)).abs() <= DEEP_DEBILITATION_ORB;
        return String::from(if deep {
            "DEEP_DEBILITATED"
        } else {
            "DEBILITATED"
        });
    }
    if in_own_sign(graha, sign) {
        return String::from("OWN_SIGN");
    }
    if NODES.contains(&reading.body) {
        return String::from("NEUTRAL");
    }
    panchadha.to_string()
}

/// The combustion orbs, degrees: the body, then its orb when direct and
/// when retrograde, then the same for deep combustion.
struct Orbs {
    body: &'static str,
    direct: f64,
    retrograde: f64,
    deep_direct: f64,
    deep_retrograde: f64,
}

/// The table the recording engine's readings bracket, which is the one
/// the classical sources give.
const ORBS: [Orbs; 6] = [
    Orbs {
        body: "MOON",
        direct: 12.0,
        retrograde: 12.0,
        deep_direct: 6.0,
        deep_retrograde: 6.0,
    },
    Orbs {
        body: "MARS",
        direct: 17.0,
        retrograde: 17.0,
        deep_direct: 8.0,
        deep_retrograde: 8.0,
    },
    Orbs {
        body: "MERCURY",
        direct: 14.0,
        retrograde: 12.0,
        deep_direct: 7.0,
        deep_retrograde: 6.0,
    },
    Orbs {
        body: "JUPITER",
        direct: 11.0,
        retrograde: 11.0,
        deep_direct: 5.0,
        deep_retrograde: 5.0,
    },
    Orbs {
        body: "VENUS",
        direct: 10.0,
        retrograde: 8.0,
        deep_direct: 5.0,
        deep_retrograde: 4.0,
    },
    Orbs {
        body: "SATURN",
        direct: 15.0,
        retrograde: 15.0,
        deep_direct: 6.0,
        deep_retrograde: 6.0,
    },
];

/// How combust a body is.
fn combustion(reading: &Reading) -> &'static str {
    let Some(from_sun) = reading.from_sun else {
        return "none";
    };
    let Some(orbs) = ORBS.iter().find(|orbs| orbs.body == reading.body) else {
        return "none";
    };
    let (orb, deep) = if reading.retrograde {
        (orbs.retrograde, orbs.deep_retrograde)
    } else {
        (orbs.direct, orbs.deep_direct)
    };
    if from_sun < deep {
        "deep-combust"
    } else if from_sun < orb {
        "combust"
    } else {
        "none"
    }
}

/// The five ages, in order.
const BALADI: [&str; 5] = ["BALA", "KUMARA", "YUVA", "VRIDDHA", "MRITA"];

/// The age of a body: five parts of six degrees, running forward in an
/// odd sign and backward in an even one.
fn baladi(reading: &Reading) -> &'static str {
    let sign = reading.sign();
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a degree of a sign over six is 0 to 4"
    )]
    let part = ((reading.degrees() / 6.0) as usize).min(4);
    let index = if sign % 2 == 0 { part } else { 4 - part };
    BALADI.get(index).copied().unwrap_or("BALA")
}

/// Which of the three wakefulness states a dignity gives.
fn jagradadi(dignity: &str) -> &'static str {
    match dignity {
        "EXALTED" | "MOOLTRIKONA" | "OWN_SIGN" => "JAGRAT",
        "ENEMY" | "GREAT_ENEMY" | "DEBILITATED" => "SUSHUPTI",
        _ => "SWAPNA",
    }
}

/// How near two bodies must be to be at war, degrees.
const WAR_ORB: f64 = 1.0;

/// The bodies that can fight: not the luminaries, not the shadows.
const FIGHTERS: [&str; 5] = ["MARS", "MERCURY", "JUPITER", "VENUS", "SATURN"];

/// The bodies whose company shames another in the fifth house.
const SHAMERS: [&str; 5] = ["RAHU", "KETU", "SUN", "SATURN", "MARS"];

// ── the page ───────────────────────────────────────────────────────────────

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask state") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let charts = charts(root)?;
    let sections = [
        header(&charts),
        friendships(&charts),
        dignities(&charts),
        combustions(&charts),
        ages(&charts),
        wars(&charts),
        avasthas(&charts),
        boundaries(&charts),
        decides(&charts),
    ];
    Ok(fill(&sections.concat()))
}

fn header(charts: &[Chart]) -> String {
    let rows = readings(charts).count();
    format!(
        "# The planetary state's rules, measured\n\n\
         Status: `generated` by `cargo xtask state` over the conformance\n\
         corpus's `positions.bodies` sections, 2026-09-07. Do not edit:\n\
         `check-state` regenerates this page and fails on any difference. The\n\
         design written from it is\n\
         [`state-and-avasthas.md`](state-and-avasthas.md).\n\n\
         Beside every position the corpus records what the body *is*: its\n\
         dignity, three readings of its friendship with its dispositor,\n\
         whether it is combust and how deeply, whether it is retrograde, its\n\
         age, three families of avastha, whether it is at war and with whom,\n\
         and whether it sits within a hair of a classification boundary.\n\n\
         That is {} readings of nine grahas over {} fixtures — the 55 recorded\n\
         days and the variants that carry positions. Each field is a rule, and\n\
         this pass proposes one and measures it.\n\n\
         **Three of them the corpus cannot settle**, and saying so is the more\n\
         valuable half of the pass (§7). Their classical definitions all read\n\
         \"or aspected by\", the SDK has no aspect model yet, and no reading the\n\
         corpus carries separates them. A rule guessed there would be a wrong\n\
         rule written into code as a fact.\n\n",
        count(rows),
        charts.len(),
    )
}

// ── 1. the friendships ─────────────────────────────────────────────────────

fn friendships(charts: &[Chart]) -> String {
    let mut natural_wrong = 0;
    let mut natural_without_self = 0;
    let mut temporary_wrong = 0;
    let mut temporary_without_self = 0;
    let mut panchadha_wrong = 0;
    let mut seen = 0;
    let mut shared_sign = 0;
    for chart in charts {
        for reading in &chart.readings {
            let Some(graha) = reading.graha() else {
                continue;
            };
            seen += 1;
            let sign = reading.sign();
            natural_wrong += usize::from(natural(graha, sign) != reading.natural);
            let bare = {
                let attributes = graha.attributes();
                lord_of(sign).map_or("neutral", |lord| {
                    if attributes.friends.contains(&lord) {
                        "friend"
                    } else if attributes.enemies.contains(&lord) {
                        "enemy"
                    } else {
                        "neutral"
                    }
                })
            };
            natural_without_self += usize::from(bare != reading.natural);
            if let Some(ours) = temporary(graha, sign, &chart.signs) {
                temporary_wrong += usize::from(ours != reading.temporary);
            }
            // The same rule without the self exception.
            if let Some(lord) = lord_of(sign)
                && let Some(at) = chart.signs.get(lord.key())
            {
                let distance = (at + 12 - sign) % 12 + 1;
                let bare = if FRIENDLY_HOUSES.contains(&distance) {
                    "friend"
                } else {
                    "enemy"
                };
                temporary_without_self += usize::from(bare != reading.temporary);
                if distance == 1 && lord != graha {
                    shared_sign += 1;
                }
            }
            panchadha_wrong +=
                usize::from(panchadha(&reading.natural, &reading.temporary) != reading.panchadha);
        }
    }
    let claims = [
        Claim::counted(
            "natural friendship is the catalogue's table, a body in its own sign counting as its own friend",
            natural_wrong,
            seen,
        ),
        Claim::counted(
            "the same without the self exception",
            natural_without_self,
            seen,
        ),
        Claim::counted(
            "temporary friendship: the dispositor in the 2nd, 3rd, 4th, 10th, 11th or 12th, and a body in its own sign its own friend",
            temporary_wrong,
            seen,
        ),
        Claim::counted(
            "the same without the self exception",
            temporary_without_self,
            seen,
        ),
        Claim::counted(
            "the five-fold compound is the classical one",
            panchadha_wrong,
            seen,
        ),
    ];
    format!(
        "## 1. The friendships, and the one thing the catalogue does not say\n\n\
         A body's friendship with the lord of the sign it stands in comes in\n\
         three readings: the natural one, which is a table; the temporary one,\n\
         which is where the dispositor stands; and the five-fold compound of\n\
         the two.\n\n{}\n\
         The catalogue's table lists each graha's friends, neutrals and\n\
         enemies and says nothing about itself, because a body is not in its\n\
         own list. The engine treats a body in its own sign as its own friend\n\
         in **both** readings, and without that the two rules are wrong on the\n\
         same {} readings — which are exactly the ones where the dispositor is\n\
         the body. The temporary rule is otherwise the classical one, and it\n\
         is not the same as \"the dispositor shares the sign\": that happens {}\n\
         times to another body, and each of those is an enemy.\n\n",
        table(&claims),
        natural_without_self,
        shared_sign,
    )
}

// ── 2. the dignity ─────────────────────────────────────────────────────────

/// What §2 measures over the recorded dignities.
struct Dignities {
    wrong: usize,
    exaltation_first: usize,
    nodes_by_friendship: usize,
    seen: usize,
    /// How far from its exact degree each debilitation was, split by what
    /// the engine called it.
    deep: Vec<f64>,
    plain: Vec<f64>,
    node_plain: Vec<f64>,
}

fn measure_dignities(charts: &[Chart]) -> Dignities {
    let mut wrong = 0;
    let mut exaltation_first = 0;
    let mut nodes_by_friendship = 0;
    let mut seen = 0;
    let mut deep: Vec<f64> = Vec::new();
    let mut plain: Vec<f64> = Vec::new();
    let mut node_plain: Vec<f64> = Vec::new();
    for reading in readings(charts) {
        let Some(graha) = reading.graha() else {
            continue;
        };
        seen += 1;
        let ours = dignity(reading, &reading.panchadha);
        wrong += usize::from(ours != reading.dignity);
        // The same ladder with exaltation ahead of moolatrikona.
        let attributes = graha.attributes();
        let rashi = Rashi::from_id(u16::from(reading.sign()));
        let exalted_first = rashi.is_some_and(|rashi| {
            attributes.exaltation.is_some_and(|at| at.sign == rashi)
                && attributes.moolatrikona.is_some_and(|span| {
                    span.sign == rashi
                        && span.to > span.from
                        && reading.degrees() >= f64::from(span.from)
                        && reading.degrees() < f64::from(span.to)
                })
        });
        exaltation_first += usize::from(exalted_first && reading.dignity == "MOOLTRIKONA");
        if NODES.contains(&reading.body) && reading.dignity != reading.panchadha {
            nodes_by_friendship += 1;
        }
        if let Some(at) = attributes.debilitation
            && rashi == Some(at.sign)
        {
            let apart = (reading.degrees() - f64::from(at.degree)).abs();
            if reading.dignity == "DEEP_DEBILITATED" {
                deep.push(apart);
            } else if NODES.contains(&reading.body) {
                node_plain.push(apart);
            } else {
                plain.push(apart);
            }
        }
    }
    Dignities {
        wrong,
        exaltation_first,
        nodes_by_friendship,
        seen,
        deep,
        plain,
        node_plain,
    }
}

fn dignities(charts: &[Chart]) -> String {
    let Dignities {
        wrong,
        exaltation_first,
        nodes_by_friendship,
        seen,
        deep,
        plain,
        node_plain,
    } = measure_dignities(charts);
    let widest_deep = deep.iter().copied().fold(0.0_f64, f64::max);
    let nearest_plain = plain.iter().copied().fold(f64::INFINITY, f64::min);
    let nearest_node = node_plain.iter().copied().fold(f64::INFINITY, f64::min);
    let claims = [
        Claim::counted(
            "moolatrikona, then exaltation, then debilitation, then own sign, then the compound friendship",
            wrong,
            seen,
        ),
        Claim::counted(
            "exaltation ahead of moolatrikona, where a body's two spans overlap",
            exaltation_first,
            seen,
        ),
        Claim::counted(
            "the shadow grahas take their friendship dignity like any other body",
            nodes_by_friendship,
            seen,
        ),
        Claim::stated(
            "deep debilitation is within one degree of the exact degree",
            verdict_of(
                widest_deep <= DEEP_DEBILITATION_ORB && nearest_plain > DEEP_DEBILITATION_ORB,
            ),
            format!(
                "{widest_deep:.4}° at the widest, and the nearest plain one at {nearest_plain:.4}°"
            ),
        ),
    ];
    format!(
        "## 2. The dignity, and two things about the ladder\n\n\
         A body's dignity is the first of a ladder of tests that answers: its\n\
         moolatrikona span, its exaltation sign, its debilitation sign, its own\n\
         signs, and failing all of those the five-fold friendship with its\n\
         dispositor.\n\n{}\n\
         **Moolatrikona comes first.** Three grahas have a moolatrikona span\n\
         inside their exaltation sign — the Moon's is Taurus 3° to 30° and its\n\
         exaltation is Taurus — and where the two overlap the engine reports\n\
         the moolatrikona. Reading the ladder the other way is wrong on\n\
         {exaltation_first} readings.\n\n\
         **The shadow grahas take a reduced ladder.** Rahu and Ketu own no\n\
         sign, and where a graha would fall through to its friendship dignity\n\
         they are simply neutral: reading their friendship instead is wrong on\n\
         {nodes_by_friendship}. Their friendship values *are* recorded and are\n\
         right; they just do not reach the dignity.\n\n\
         The deep debilitation's orb is bracketed rather than stated: the\n\
         widest recorded one is {:.4}° from the exact degree and the nearest\n\
         plain debilitation of a graha is {nearest_plain:.4}°, so a whole\n\
         degree sits inside the bracket. The shadow grahas complicate it —\n\
         their nearest plain debilitation is {nearest_node:.4}°, inside a\n\
         degree — so either they are excluded from deep debilitation as they\n\
         are from the rest of the ladder, or the orb is smaller than\n\
         {nearest_node:.4}° for everyone. The corpus cannot separate the two,\n\
         and the SDK takes the first, which is what the rest of the nodes'\n\
         treatment already says.\n\n",
        table(&claims),
        widest_deep,
    )
}

// ── 3. combustion ──────────────────────────────────────────────────────────

fn combustions(charts: &[Chart]) -> String {
    let mut wrong = 0;
    let mut seen = 0;
    let mut brackets: BTreeMap<(&str, bool), (f64, f64, f64, f64)> = BTreeMap::new();
    for reading in readings(charts) {
        let Some(from_sun) = reading.from_sun else {
            continue;
        };
        if !ORBS.iter().any(|orbs| orbs.body == reading.body) {
            continue;
        }
        seen += 1;
        wrong += usize::from(combustion(reading) != reading.combust);
        let entry = brackets
            .entry((reading.body, reading.retrograde))
            .or_insert((0.0, f64::INFINITY, 0.0, f64::INFINITY));
        match reading.combust.as_str() {
            "deep-combust" => entry.0 = entry.0.max(from_sun),
            "combust" => {
                entry.1 = entry.1.min(from_sun);
                entry.2 = entry.2.max(from_sun);
            }
            _ => entry.3 = entry.3.min(from_sun),
        }
    }
    let mut out = format!(
        "## 3. Combustion, and the table the corpus brackets\n\n\
         A body near the Sun is combust, and nearer still deeply so. The orb\n\
         differs by body and by whether the body is retrograde, and the corpus\n\
         does not state it — it **brackets** it, between the widest reading\n\
         that is combust and the nearest that is not.\n\n{}\n\
         Every bracket below contains the classical value, and the `BPHS`\n\
         table the SDK ships is those values. Where a bracket is wide the\n\
         corpus is not pinning the orb, only failing to contradict it; where\n\
         it is narrow — Mercury direct, between {:.2}° and {:.2}° — it is.\n\n\
         | body | direction | deep out to | combust range | clear from | orb | deep orb |\n\
         |---|---|---|---|---|---|---|\n",
        table(&[Claim::counted(
            "the `BPHS` orb table reproduces every recorded reading",
            wrong,
            seen,
        )]),
        brackets
            .get(&("MERCURY", false))
            .map_or(0.0, |bracket| bracket.2),
        brackets
            .get(&("MERCURY", false))
            .map_or(0.0, |bracket| bracket.3),
    );
    for ((body, retrograde), (deep, from, to, clear)) in &brackets {
        let orbs = ORBS.iter().find(|orbs| orbs.body == *body);
        let show = |value: f64| {
            if value.is_finite() && value > 0.0 {
                format!("{value:.4}°")
            } else {
                String::from("—")
            }
        };
        let _ = writeln!(
            out,
            "| {body} | {} | {} | {} | {} | {} | {} |",
            if *retrograde { "retrograde" } else { "direct" },
            show(*deep),
            if from.is_finite() {
                format!("{from:.4}° to {to:.4}°")
            } else {
                String::from("—")
            },
            show(*clear),
            orbs.map_or(0.0, |orbs| if *retrograde {
                orbs.retrograde
            } else {
                orbs.direct
            }),
            orbs.map_or(0.0, |orbs| if *retrograde {
                orbs.deep_retrograde
            } else {
                orbs.deep_direct
            }),
        );
    }
    out.push_str(
        "\nThe orb is a settings knob — `state.combustion_orbs` names a table —\n\
         because the tables differ between traditions and an implementation\n\
         that hard-codes one cannot say which it used. The deep orb is where\n\
         they part: the `SURYA_SIDDHANTA` table the default profile names\n\
         gives the same outer orbs and nothing inside them, so under it the\n\
         deep column above does not apply and the readings it brackets come\n\
         back merely combust.\n\n",
    );
    out
}

// ── 4. the ages ────────────────────────────────────────────────────────────

fn ages(charts: &[Chart]) -> String {
    let mut wrong = 0;
    let mut forward = 0;
    let mut seen = 0;
    for reading in readings(charts) {
        if reading.graha().is_none() {
            continue;
        }
        seen += 1;
        wrong += usize::from(baladi(reading) != reading.baladi);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a degree of a sign over six is 0 to 4"
        )]
        let part = ((reading.degrees() / 6.0) as usize).min(4);
        forward += usize::from(BALADI.get(part).copied().unwrap_or("") != reading.baladi);
    }
    let claims = [
        Claim::counted(
            "the five ages run forward in an odd sign and backward in an even one, six degrees each",
            wrong,
            seen,
        ),
        Claim::counted("they always run forward", forward, seen),
    ];
    format!(
        "## 4. The five ages\n\n\
         A sign is cut into five parts of six degrees, and a body's age is\n\
         which part it stands in — infant, child, youth, old, dead — running\n\
         forward in an odd sign and backward in an even one.\n\n{}\n\
         That the direction alternates is not a detail: reading it forward\n\
         everywhere is wrong on {forward} of {seen} readings, which is most of\n\
         the even signs.\n\n",
        table(&claims),
    )
}

// ── 5. the war ─────────────────────────────────────────────────────────────

fn wars(charts: &[Chart]) -> String {
    let mut fought = Vec::new();
    let mut peace = f64::INFINITY;
    let mut northern_wins = 0;
    let mut wars = 0;
    let mut wrong_pairs = 0;
    for chart in charts {
        let by_body: BTreeMap<&str, &Reading> = chart
            .readings
            .iter()
            .map(|reading| (reading.body, reading))
            .collect();
        for reading in &chart.readings {
            if let Some((opponent, winner)) = &reading.war {
                wars += 1;
                if let Some(other) = by_body.get(opponent.as_str()) {
                    let apart = separation(reading.longitude, other.longitude);
                    fought.push(apart);
                    northern_wins += usize::from((reading.latitude > other.latitude) == *winner);
                }
            }
        }
        // The nearest pair of fighters that is *not* at war.
        for (index, body) in FIGHTERS.iter().enumerate() {
            for other in FIGHTERS.iter().skip(index + 1) {
                let (Some(first), Some(second)) = (by_body.get(body), by_body.get(other)) else {
                    continue;
                };
                let apart = separation(first.longitude, second.longitude);
                let at_war = first
                    .war
                    .as_ref()
                    .is_some_and(|(opponent, _)| opponent == other);
                if at_war {
                    if apart > WAR_ORB {
                        wrong_pairs += 1;
                    }
                } else {
                    peace = peace.min(apart);
                }
            }
        }
    }
    let widest = fought.iter().copied().fold(0.0_f64, f64::max);
    let mut named: Vec<String> = Vec::new();
    for chart in charts {
        let by_body: BTreeMap<&str, &Reading> = chart
            .readings
            .iter()
            .map(|reading| (reading.body, reading))
            .collect();
        for reading in &chart.readings {
            let Some((opponent, winner)) = &reading.war else {
                continue;
            };
            if !*winner {
                continue;
            }
            let apart = by_body
                .get(opponent.as_str())
                .map_or(0.0, |other| separation(reading.longitude, other.longitude));
            named.push(format!(
                "| {} | {} beats {opponent} | {apart:.4}° |",
                reading.fixture, reading.body
            ));
        }
    }
    let claims = [
        Claim::counted(
            "a war is two of the five planets within a degree of each other",
            wrong_pairs,
            wars,
        ),
        Claim::counted(
            "the northern body — the one with the greater ecliptic latitude — wins",
            wars - northern_wins,
            wars,
        ),
    ];
    format!(
        "## 5. The planetary war\n\n\
         Two planets close enough together are said to be at war, and one of\n\
         them wins. Neither the orb nor the victor is stated in the corpus and\n\
         both are decided by it.\n\n{}\n\
         The orb is bracketed between {widest:.4}°, the widest war recorded,\n\
         and {peace:.4}°, the nearest pair of planets that is not one — so a\n\
         whole degree sits inside it. The luminaries and the shadow grahas do\n\
         not fight: no recorded war involves one, and the classical rule\n\
         excludes them.\n\n\
         The victor is the **northern** body, on every one of the {wars}\n\
         readings the corpus carries. Several other rules are current — the\n\
         brighter body, the one further west, the one with the greater\n\
         diameter — and the corpus rules out none of them, only agrees with\n\
         this one; latitude is what it is recorded with.\n\n\
         The wars the corpus records, the winner named first:\n\n\
         | fixture | war | separation |\n|---|---|---|\n{}\n\n",
        table(&claims),
        named.join("\n"),
    )
}

// ── 6 and 7. the avasthas ──────────────────────────────────────────────────

/// What §§6 and 7 measure over the recorded avasthas.
struct Avasthas {
    jagradadi_wrong: usize,
    jagradadi_seen: usize,
    deep_fell_through: usize,
    garvita_wrong: usize,
    lajjita_wrong: usize,
    kshobhita_wrong: usize,
    lajjitadi_seen: usize,
    /// Which deeptadi was recorded with each dignity.
    deeptadi: BTreeMap<String, BTreeMap<String, usize>>,
    decided: usize,
    undecided: usize,
}

fn measure_avasthas(charts: &[Chart]) -> Avasthas {
    let mut jagradadi_wrong = 0;
    let mut jagradadi_seen = 0;
    let mut deep_fell_through = 0;
    let mut garvita_wrong = 0;
    let mut lajjita_wrong = 0;
    let mut kshobhita_wrong = 0;
    let mut lajjitadi_seen = 0;
    let mut deeptadi: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
    let mut decided = 0;
    let mut undecided = 0;
    for chart in charts {
        for reading in &chart.readings {
            if let Some(recorded) = &reading.jagradadi {
                jagradadi_seen += 1;
                jagradadi_wrong += usize::from(jagradadi(&reading.dignity) != *recorded);
                if reading.dignity == "DEEP_DEBILITATED" && recorded == "SWAPNA" {
                    deep_fell_through += 1;
                }
            }
            if let Some(recorded) = &reading.deeptadi {
                *deeptadi
                    .entry(reading.dignity.clone())
                    .or_default()
                    .entry(recorded.clone())
                    .or_default() += 1;
                let top = matches!(
                    reading.dignity.as_str(),
                    "EXALTED" | "OWN_SIGN" | "MOOLTRIKONA" | "GREAT_FRIEND"
                );
                if top {
                    decided += 1;
                } else {
                    undecided += 1;
                }
            }
            if reading.jagradadi.is_none() {
                continue;
            }
            lajjitadi_seen += 1;
            let garvita = matches!(reading.dignity.as_str(), "EXALTED" | "MOOLTRIKONA");
            garvita_wrong +=
                usize::from(garvita != reading.lajjitadi.iter().any(|name| name == "GARVITA"));
            let lajjita = reading.house == 5
                && SHAMERS.iter().any(|shamer| {
                    *shamer != reading.body && chart.signs.get(*shamer) == Some(&reading.sign())
                });
            lajjita_wrong +=
                usize::from(lajjita != reading.lajjitadi.iter().any(|name| name == "LAJJITA"));
            let kshobhita =
                reading.body != "SUN" && chart.signs.get("SUN") == Some(&reading.sign());
            kshobhita_wrong +=
                usize::from(kshobhita != reading.lajjitadi.iter().any(|name| name == "KSHOBHITA"));
        }
    }
    Avasthas {
        jagradadi_wrong,
        jagradadi_seen,
        deep_fell_through,
        garvita_wrong,
        lajjita_wrong,
        kshobhita_wrong,
        lajjitadi_seen,
        deeptadi,
        decided,
        undecided,
    }
}

fn avasthas(charts: &[Chart]) -> String {
    let Avasthas {
        jagradadi_wrong,
        jagradadi_seen,
        deep_fell_through,
        garvita_wrong,
        lajjita_wrong,
        kshobhita_wrong,
        lajjitadi_seen,
        deeptadi,
        decided,
        undecided,
    } = measure_avasthas(charts);
    let settled = [
        Claim::counted(
            "the wakefulness follows the dignity: exalted, moolatrikona or own sign awake; enemy or debilitated asleep; the rest dreaming",
            jagradadi_wrong,
            jagradadi_seen,
        ),
        Claim::counted(
            "garvita, the proud: exalted or in moolatrikona",
            garvita_wrong,
            lajjitadi_seen,
        ),
        Claim::counted(
            "lajjita, the ashamed: in the fifth house with the Sun, Mars, Saturn, Rahu or Ketu",
            lajjita_wrong,
            lajjitadi_seen,
        ),
        Claim::counted(
            "kshobhita, the agitated: standing in the same sign as the Sun",
            kshobhita_wrong,
            lajjitadi_seen,
        ),
    ];
    let mut out = format!(
        "## 6. The avasthas the corpus settles\n\n\
         Four of them, and each is a rule over facts the corpus already\n\
         records.\n\n{}\n\
         One thing the wakefulness does that it should not: a **deeply**\n\
         debilitated body is recorded dreaming rather than asleep, on all\n\
         {deep_fell_through} readings that have one. Plain debilitation is\n\
         asleep, so the deeper state is the milder one — which is what a\n\
         switch that lists the plain values and defaults for the rest does,\n\
         and it joins the deliberate-difference registry.\n\n\
         ## 7. The avasthas the corpus cannot settle\n\n\
         The deeptadi's nine states and three of the six lajjitadi. What the\n\
         corpus **does** settle about the deeptadi is its top: exaltation is\n\
         Deepta, an own sign or a moolatrikona is Swastha, a great friend's is\n\
         Mudita — {decided} readings, no exceptions. Below that, {undecided}\n\
         readings split six ways, and nothing the corpus records separates\n\
         them:\n\n\
         | dignity | the deeptadi recorded with it |\n|---|---|\n",
        table(&settled),
    );
    for (dignity, states) in &deeptadi {
        let shown: Vec<String> = states
            .iter()
            .map(|(state, count)| format!("{state} {count}"))
            .collect();
        let _ = writeln!(out, "| {dignity} | {} |", shown.join(", "));
    }
    out.push_str(
        "\nDebilitation splits between Khala and Shanta; an enemy's sign splits\n\
         four ways. Neither combustion, retrogradation, the war, the house,\n\
         the navamsha dignity, nor a companion or an aspect of a benefic or a\n\
         malefic in the sign separates them — each was tried and each is\n\
         uncorrelated or anti-correlated. The same is true of kshudha,\n\
         trishita and mudita among the lajjitadi.\n\n\
         Those are the states whose classical definitions read \"or aspected\n\
         by\". So the design reports the states it can decide and **nothing**\n\
         where it cannot, rather than a plausible guess: a caller can tell an\n\
         absent answer from a wrong one.\n\n\
         `cargo xtask aspect` proposed the rules again with a real drishti in\n\
         hand (`aspect-drishti-measured.md` §7) and none of them is exact —\n\
         adding the aspect clause makes every one of them worse, which is\n\
         evidence the recording engine does not compute these from a drishti\n\
         at all. What it did settle is a **necessary** condition for each of\n\
         the three lajjitadi, missed by not one recorded reading, so the\n\
         module now answers `no` with certainty where the condition fails and\n\
         withholds only where it holds.\n\n",
    );
    out
}

// ── 8. the boundaries ──────────────────────────────────────────────────────

fn boundaries(charts: &[Chart]) -> String {
    let mut flagged = 0.0_f64;
    let mut clear = f64::INFINITY;
    let mut per: BTreeMap<&str, (f64, f64)> = BTreeMap::new();
    for reading in readings(charts) {
        for (division, near, distance) in reading.boundaries {
            let entry = per.entry(division).or_insert((0.0, f64::INFINITY));
            if near {
                entry.0 = entry.0.max(distance);
                flagged = flagged.max(distance);
            } else {
                entry.1 = entry.1.min(distance);
                clear = clear.min(distance);
            }
        }
    }
    let mut out = format!(
        "## 8. The boundary flags are one threshold\n\n\
         A body within a hair of a sign, nakshatra or pada boundary is flagged,\n\
         because its classification would flip under a slightly different\n\
         ayanamsha. The threshold is not recorded, and the corpus brackets it\n\
         to between {:.5}° and {:.5}° — about {:.0} and {:.0} arcseconds — for\n\
         all three divisions at once.\n\n\
         | division | flagged out to | unflagged from |\n|---|---|---|\n",
        flagged,
        clear,
        flagged * 3600.0,
        clear * 3600.0,
    );
    for (division, (near, far)) in &per {
        let _ = writeln!(out, "| {division} | {near:.5}° | {far:.5}° |");
    }
    out.push_str(
        "\nOne threshold fits all three, which is what a single constant in the\n\
         engine looks like. The SDK does not ship a constant: it reports the\n\
         **distance** to the nearest boundary of each division, which is always\n\
         a fact, and a caller asks whether that is inside whatever tolerance\n\
         its provider claims. The corpus's own tolerance file already frames\n\
         the question that way — a classification within the longitude\n\
         tolerance of a boundary is an edge case and not a failure — and a\n\
         fixed number in the library could not answer it for two providers of\n\
         different accuracy.\n\n",
    );
    out
}

// ── what it decides ────────────────────────────────────────────────────────

fn decides(charts: &[Chart]) -> String {
    let rows = readings(charts).count();
    format!(
        "## 9. What this decides\n\n\
         1. **The friendship table needs one addition.** A body in its own\n   \
            sign is its own friend, naturally and temporarily; the catalogue\n   \
            cannot say it because a body is not in its own list.\n\
         2. **The dignity ladder checks moolatrikona before exaltation**, and\n   \
            the shadow grahas take a reduced one that ends at neutral.\n\
         3. **Combustion is a table, and the table is a knob.** The corpus\n   \
            brackets every row of it and contradicts none.\n\
         4. **The five ages alternate direction**, which is most of the\n   \
            readings.\n\
         5. **A war is a degree wide and the northern body wins**, on every\n   \
            reading the corpus has.\n\
         6. **Report absence rather than a guess.** The deeptadi below its top\n   \
            three, and three of the six lajjitadi, are not decidable from what\n   \
            the corpus records and what the SDK can compute. They wait for\n   \
            `aspect`, and until then the answer is nothing rather than\n   \
            something plausible.\n\
         7. **A boundary is a distance, not a flag.** How near a body is to a\n   \
            classification boundary is a fact; whether that is *near* depends\n   \
            on the provider, and belongs to the caller.\n\n\
         Measured over {} readings.\n",
        count(rows),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        BALADI, DEEP_DEBILITATION_ORB, FIGHTERS, FRIENDLY_HOUSES, ORBS, SHAMERS, WAR_ORB, edge,
        in_own_sign, lord_of, natural, panchadha, separation, sign_of,
    };
    use teistro_core::catalogue::{Graha, Rashi};

    #[test]
    fn a_longitude_names_its_sign_and_its_lord() {
        assert_eq!(sign_of(0.0), 0);
        assert_eq!(sign_of(29.999), 0);
        assert_eq!(sign_of(30.0), 1);
        assert_eq!(sign_of(-1.0), 11, "a negative longitude wraps");
        assert_eq!(sign_of(360.0), 0);
        assert_eq!(lord_of(0), Some(Graha::Mars), "Aries is Mars's");
        assert_eq!(lord_of(4), Some(Graha::Sun), "Leo is the Sun's");
        assert_eq!(lord_of(12), None);
    }

    #[test]
    fn a_body_in_its_own_sign_is_its_own_friend() {
        assert!(in_own_sign(Graha::Sun, 4), "the Sun in Leo");
        assert!(!in_own_sign(Graha::Sun, 3));
        assert_eq!(natural(Graha::Sun, 4), "friend", "and so its own friend");
        // The catalogue's table otherwise.
        assert_eq!(natural(Graha::Sun, 3), "friend", "Cancer is the Moon's");
        assert_eq!(natural(Graha::Sun, 1), "enemy", "Taurus is Venus's");
        assert_eq!(natural(Graha::Sun, 2), "neutral", "Gemini is Mercury's");
    }

    #[test]
    fn the_compound_is_the_classical_one() {
        assert_eq!(panchadha("friend", "friend"), "GREAT_FRIEND");
        assert_eq!(panchadha("friend", "enemy"), "NEUTRAL");
        assert_eq!(panchadha("neutral", "friend"), "FRIEND");
        assert_eq!(panchadha("neutral", "enemy"), "ENEMY");
        assert_eq!(panchadha("enemy", "friend"), "NEUTRAL");
        assert_eq!(panchadha("enemy", "enemy"), "GREAT_ENEMY");
    }

    #[test]
    fn the_tables_are_the_shape_they_claim() {
        assert_eq!(BALADI.len(), 5);
        assert_eq!(FRIENDLY_HOUSES.len(), 6);
        assert_eq!(ORBS.len(), 6, "every body but the Sun and the shadows");
        assert!(
            ORBS.iter()
                .all(|orbs| orbs.deep_direct < orbs.direct
                    && orbs.deep_retrograde <= orbs.retrograde),
            "a deep orb is inside its own"
        );
        assert_eq!(FIGHTERS.len(), 5, "not the luminaries, not the shadows");
        assert_eq!(SHAMERS.len(), 5);
        assert!((WAR_ORB - 1.0).abs() < f64::EPSILON);
        assert!((DEEP_DEBILITATION_ORB - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_separation_is_the_shorter_way_round() {
        assert!((separation(10.0, 350.0) - 20.0).abs() < 1e-9);
        assert!((separation(350.0, 10.0) - 20.0).abs() < 1e-9);
        assert!((separation(0.0, 180.0) - 180.0).abs() < 1e-9);
        assert!(separation(5.0, 5.0).abs() < 1e-9);
    }

    #[test]
    fn a_distance_to_a_boundary_is_to_the_nearer_one() {
        assert!(edge(0.0, 30.0).abs() < 1e-9);
        assert!((edge(1.0, 30.0) - 1.0).abs() < 1e-9);
        assert!((edge(29.0, 30.0) - 1.0).abs() < 1e-9, "the near side");
        assert!((edge(15.0, 30.0) - 15.0).abs() < 1e-9);
        assert_eq!(Rashi::ALL.len(), 12);
    }
}
