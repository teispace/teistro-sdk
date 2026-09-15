//! The falsification pass over the yogas (Phase 6), run on the rules kernel.
//!
//! The corpus's `baseline/yogas` records the recording engine's yoga
//! evaluator run on every fixture with its positions: the rules it ships, in
//! its own condition language (`rules.json`), and for each fixture the rules
//! that are present, with the planets and houses involved and the
//! cancellations that fired, computed from the recorded chart it repeats.
//!
//! An input and an output are both recorded, so the pass derives the rules'
//! semantics rather than proposing them. It evaluates every rule over every
//! recorded chart under a *reading* — one choice at each place the condition
//! language leaves a meaning open — and counts the decisions, the involved
//! planets and the cancellations that disagree. The engine's reading is the
//! one that reproduces everything; every other reading is the engine's with
//! one choice flipped, so each row says how much that one choice moves.
//!
//! `cargo xtask yogas` writes the page; `check-yogas` regenerates it in memory
//! and fails on any difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize as _;
use serde_json::Value;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{CharaKaraka, Dignity, Rashi, Varga};
use teistro_core::quantity::Degrees;
use teistro_rules::{
    Benefics, Body, Conjunction, DignityMatch, Evaluator, Gathering, House, Houses, Karaka,
    NodeMotion, NodeSides, Placement, Readings, Rule, RuleChart,
};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, table};

const PAGE: &str = "docs/03-design/yogas-measured.md";
const ROOT: &str = "fixtures/baseline/yogas";

/// One recorded chart and what the engine answered for it.
struct Record {
    chart: RuleChart,
    /// Each present rule's involved planets, and its fired cancellations.
    present: BTreeMap<String, (Vec<String>, Vec<String>)>,
}

/// Every place the condition language leaves a meaning open, as the reading
/// that takes it the other way from the engine, and how the page states it.
const FORKS: [(&str, Readings); 7] = [
    (
        "benefics by the natural lists alone, the Moon and Mercury never turned malefic",
        Readings {
            benefics: Benefics::Natural,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "a rule asking for exaltation or debilitation met by that dignity alone, not its deep form",
        Readings {
            dignity: DignityMatch::Exact,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "all seven between the nodes counted from Rahu to Ketu only",
        Readings {
            node_sides: NodeSides::RahuToKetu,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "houses counted whole-sign from the lagna rather than recorded",
        Readings {
            houses: Houses::WholeSign,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "Rahu and Ketu counted retrograde where a rule asks",
        Readings {
            node_motion: NodeMotion::AlwaysRetrograde,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "an involved planet only from the branch that decided",
        Readings {
            gathering: Gathering::DecidingBranch,
            ..Readings::RECORDING_ENGINE
        },
    ),
    (
        "an unqualified conjunction within 10° rather than in one sign",
        Readings {
            conjunction: Conjunction::Orb(10.0),
            ..Readings::RECORDING_ENGINE
        },
    ),
];

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?,
    )
    .map_err(|e| format!("{}: {e}", path.display()))
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn rules(root: &Path) -> Result<Vec<Rule>, String> {
    let file = read_json(&root.join(ROOT).join("rules.json"))?;
    serde_json::from_value(file["rules"].clone()).map_err(|e| format!("rules.json: {e}"))
}

/// The chara karaka a recorded abbreviation names, if the body holds one.
fn karaka(value: &Value) -> Option<CharaKaraka> {
    Karaka::deserialize(value).ok().map(|Karaka(karaka)| karaka)
}

/// The chart a file repeats.
fn chart(inputs: &Value) -> Result<RuleChart, String> {
    let mut placements = Vec::with_capacity(Body::ALL.len());
    for body in Body::ALL {
        let b = &inputs["bodies"][body.key()];
        let sign = b["sign_index"]
            .as_u64()
            .and_then(|n| u16::try_from(n).ok())
            .and_then(Rashi::from_id)
            .ok_or_else(|| format!("{}: sign", body.key()))?;
        let house = b["house"]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
            .and_then(|n| House::try_new(n).ok())
            .ok_or_else(|| format!("{}: house", body.key()))?;
        let longitude = b["sidereal_longitude_deg"]
            .as_f64()
            .ok_or_else(|| format!("{}: longitude", body.key()))?;
        placements.push(Placement {
            longitude,
            sign,
            house,
            dignity: b["dignity"]
                .as_str()
                .and_then(Dignity::from_key)
                .ok_or_else(|| format!("{}: dignity", body.key()))?,
            retrograde: b["is_retrograde"].as_bool().unwrap_or_default(),
            combust: b["combust"].as_str() != Some("none"),
            karaka7: karaka(&inputs["chara_karaka_7"][body.key()]),
            karaka8: karaka(&inputs["chara_karaka_8"][body.key()]),
            navamsha: teistro_vargas::sign(
                &teistro_vargas::Scheme::of(Varga::D9),
                Nas::from_degrees(
                    Degrees::try_new(longitude.rem_euclid(360.0)).map_err(|e| e.to_string())?,
                ),
            ),
        });
    }
    let placements = placements
        .try_into()
        .map_err(|_| String::from("ten placements"))?;
    // The yogas' inputs carry no tithi, and no yoga reads one.
    Ok(RuleChart {
        placements,
        panchanga: None,
    })
}

fn records(root: &Path) -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(ROOT).join(dir))
            .map_err(|e| format!("{dir}: {e}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let file = read_json(&path)?;
            let present = file["present"]
                .as_object()
                .ok_or("present")?
                .iter()
                .map(|(key, found)| {
                    (
                        key.clone(),
                        (strings(&found["planets"]), strings(&found["cancellations"])),
                    )
                })
                .collect();
            out.push(Record {
                chart: chart(&file["inputs"]).map_err(|e| format!("{}: {e}", path.display()))?,
                present,
            });
        }
    }
    Ok(out)
}

/// What one reading reproduces, over every chart and every rule it reads.
#[derive(Default)]
struct Tally {
    decisions: usize,
    decisions_wrong: usize,
    presences: usize,
    planets_wrong: usize,
    cancellations_wrong: usize,
    moved: BTreeSet<String>,
}

fn tally(rules: &[Rule], records: &[Record], readings: Readings) -> Tally {
    let mut t = Tally::default();
    for record in records {
        let evaluator = Evaluator::new(&record.chart, readings);
        for rule in rules.iter().filter(|r| r.is_evaluable()) {
            t.decisions += 1;
            let result = evaluator.evaluate(rule);
            let recorded = record.present.get(&rule.key);
            if result.present != recorded.is_some() {
                t.decisions_wrong += 1;
                t.moved.insert(rule.key.clone());
                continue;
            }
            let Some((planets, cancellations)) = recorded else {
                continue;
            };
            t.presences += 1;
            let participants: Vec<String> = result
                .participants
                .iter()
                .map(|b| b.key().to_owned())
                .collect();
            if &participants != planets {
                t.planets_wrong += 1;
                t.moved.insert(rule.key.clone());
            }
            let fired: Vec<String> = result
                .cancellations
                .iter()
                .filter_map(|i| rule.cancellations.get(*i))
                .map(|c| c.condition.kind().to_owned())
                .collect();
            if &fired != cancellations {
                t.cancellations_wrong += 1;
                t.moved.insert(rule.key.clone());
            }
        }
    }
    t
}

fn claim(name: &str, t: &Tally, engine: bool) -> Claim {
    let wrong = t.decisions_wrong + t.planets_wrong + t.cancellations_wrong;
    if !engine && wrong == 0 {
        // Flipping the choice moves nothing, so the corpus cannot tell the two
        // readings apart: untested rather than held.
        return Claim::stated(
            name,
            Verdict::Untested,
            format!(
                "moves none of the {} decisions or {} presences",
                count(t.decisions),
                count(t.presences)
            ),
        );
    }
    let mut moved: Vec<&str> = t.moved.iter().map(String::as_str).take(4).collect();
    if t.moved.len() > moved.len() {
        moved.push("…");
    }
    let claim = Claim::counted(name, wrong, t.decisions).with_note(format!(
        "{} decisions wrong, {} of {} presences' planets, {} cancellations",
        count(t.decisions_wrong),
        count(t.planets_wrong),
        count(t.presences),
        count(t.cancellations_wrong)
    ));
    if t.moved.is_empty() {
        claim
    } else {
        claim.with_note(format!(
            "{} rules move ({})",
            count(t.moved.len()),
            moved.join(", ")
        ))
    }
}

/// Which rules the corpus shows present, and how often.
struct Coverage {
    /// The rules written in the condition language.
    evaluated: usize,
    /// Those never present on any recorded chart.
    never: Vec<String>,
    /// How many are present on every one.
    always: usize,
    /// Each category's rules and presences, as the page writes them.
    categories: String,
}

impl Coverage {
    fn of(rules: &[Rule], records: &[Record]) -> Coverage {
        let mut presence: BTreeMap<&str, usize> = BTreeMap::new();
        for record in records {
            for key in record.present.keys() {
                *presence.entry(key.as_str()).or_default() += 1;
            }
        }
        let evaluated: Vec<&Rule> = rules.iter().filter(|r| !r.conditions.is_empty()).collect();
        let mut categories: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
        for rule in rules {
            let entry = categories.entry(rule.category.as_str()).or_default();
            entry.0 += 1;
            entry.1 += presence.get(rule.key.as_str()).copied().unwrap_or(0);
        }
        Coverage {
            evaluated: evaluated.len(),
            never: evaluated
                .iter()
                .filter(|r| !presence.contains_key(r.key.as_str()))
                .map(|r| r.key.clone())
                .collect(),
            always: evaluated
                .iter()
                .filter(|r| presence.get(r.key.as_str()) == Some(&records.len()))
                .count(),
            categories: categories
                .iter()
                .map(|(k, (n, p))| format!("{k} {n} ({p})"))
                .collect::<Vec<_>>()
                .join(", "),
        }
    }
}

/// How often each condition type appears, conditions and cancellations alike.
fn predicate_uses(rules: &[Rule]) -> BTreeMap<&'static str, usize> {
    let mut uses = BTreeMap::new();
    for rule in rules {
        for condition in rule.every_condition() {
            *uses.entry(condition.kind()).or_default() += 1;
        }
    }
    uses
}

fn page(root: &Path) -> Result<String, String> {
    let rules = rules(root)?;
    let records = records(root)?;
    let custom: Vec<&str> = rules
        .iter()
        .filter(|r| r.conditions.is_empty())
        .map(|r| r.key.as_str())
        .collect();

    let engine_reading = (
        "the engine's reading, every choice below as the engine makes it",
        Readings::RECORDING_ENGINE,
    );
    let mut claims = Vec::new();
    let mut engine = Tally::default();
    for (index, (name, readings)) in core::iter::once(engine_reading).chain(FORKS).enumerate() {
        let t = tally(&rules, &records, readings);
        claims.push(claim(name, &t, index == 0));
        if index == 0 {
            engine = t;
        }
    }

    let coverage = Coverage::of(&rules, &records);
    let predicates = predicate_uses(&rules);
    let mut by_use: Vec<(&&str, &usize)> = predicates.iter().collect();
    by_use.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));

    let mut out = String::new();
    let _ = write!(
        out,
        "# The yogas, measured\n\n\
         Status: `generated` by `cargo xtask yogas` over the conformance corpus's `baseline/yogas`, \
         2026-09-15. Do not edit: `check-yogas` regenerates this page and fails on any difference. The \
         design it measures is [`rules-engine.md`](rules-engine.md). The pass evaluates every rule of \
         `rules.json` over every recorded chart under a reading and counts what disagrees with what the \
         engine recorded.\n\n\
         ## What the corpus holds\n\n\
         {rules} over {charts}: {decisions} decisions of the {evaluated} rules written in the condition \
         language, {presences} of them present. The other {custom} rules carry no conditions because the \
         engine computes them in code: {custom_keys}. The rules use {kinds} condition types; by use, {uses}.\n\n\
         ## What the corpus decides\n\n{table}\n\
         ## Coverage\n\n\
         {never_count} of the {evaluated} rules are never present on any recorded chart, so the corpus \
         holds no positive case for them; {always} are present on every one, so it holds no negative case. \
         By category, rules and presences: {categories}.\n\n\
         Never present: {never}.\n\n",
        rules = plural(rules.len(), "rule"),
        charts = plural(records.len(), "chart"),
        decisions = count(engine.decisions),
        evaluated = count(coverage.evaluated),
        presences = count(engine.presences),
        custom = count(custom.len()),
        custom_keys = custom.join(", "),
        kinds = count(predicates.len()),
        uses = by_use
            .iter()
            .map(|(k, n)| format!("`{k}` {n}"))
            .collect::<Vec<_>>()
            .join(", "),
        table = table(&claims),
        never_count = count(coverage.never.len()),
        always = count(coverage.always),
        categories = coverage.categories,
        never = if coverage.never.is_empty() {
            String::from("none")
        } else {
            coverage.never.join(", ")
        },
    );
    let _ = write!(
        out,
        "## What it means for the kernel\n\n\
         **The engine's semantics are settled over the corpus**, each a choice the rows above measure: \
         conjunction in one sign unless a rule gives an orb, the Moon malefic when waning and Mercury when \
         only malefics share its sign, a deep dignity meeting its plain form, all seven between the nodes \
         on either side, the nodes never retrograde, houses as recorded, and a rule's planets gathered \
         from every condition that held, including inside a branch that went on to fail. Two of those choices \
         the corpus decides — the Moon's and Mercury's natures, and the sign against an orb — and the rest \
         it cannot see, since flipping them moves nothing; the kernel still makes each explicit, and a \
         fixture that separates them is worth adding.\n\n\
         **The involved planets are part of the answer**, so the kernel's trace has to reproduce the \
         engine's accumulation where the conformance profile asks for it, and may report the deciding \
         branch alone where it does not.\n\n\
         **Coverage is the kernel's first test debt.** A rule never present here has only negative cases, \
         and the rules-engine page requires a positive and a negative fixture before a rule is marked \
         stable; the Neecha Bhanga family needs the kernel's table lookups or its divisional-chart \
         predicate before it can be written as rules at all.\n",
    );
    Ok(fill(&out))
}

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask yogas") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
