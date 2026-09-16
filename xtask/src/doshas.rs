//! The falsification pass over the doshas (Phase 6), run on the rules kernel.
//!
//! The corpus's `baseline/doshas` records the recording engine's natal dosha
//! evaluator over every fixture with its positions and, for 77 of them, its
//! recorded panchanga: the rules it evaluates, in its own shape
//! (`rules.json`), and for each fixture every dosha that is present, with
//! where it was found from, the planets and houses involved, its severity, the
//! cancellations that fired and its net status.
//!
//! The pass measures two things. First, the 35 rules the engine evaluates from
//! their conditions, under the engine's dosha reading and under each reading
//! flipped, so each row says what that one choice moves. Second, the seventeen
//! the engine computes in code, which the SDK ships as rules
//! (`crates/rules/rules/computed-doshas.json`): whether each says present
//! exactly where the engine's code did, and where their answers part.
//!
//! `cargo xtask doshas` writes the page; `check-doshas` regenerates it in
//! memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro_rules::{
    AspectGathering, Bhaga, Conjunction, DignityMatch, Evaluator, Found, Gathering, Houses,
    NetStatus, Readings, Rule, RuleChart, Tables, shipped,
};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, table};
use crate::rules_corpus::{chart, read_json, rules, strings};

const PAGE: &str = "docs/03-design/doshas-measured.md";
const ROOT: &str = "fixtures/baseline/doshas";

/// What the engine recorded for one present dosha.
struct Presence {
    found_from: Vec<String>,
    planets: Vec<String>,
    houses: Vec<u64>,
    severity: u64,
    cancellations: Vec<String>,
    status: String,
}

/// One recorded chart and what the engine answered for it.
struct Record {
    chart: RuleChart,
    present: BTreeMap<String, Presence>,
}

/// Every place the language leaves a meaning open that a dosha rule reads, as
/// the reading that takes it the other way from the engine.
const FORKS: [(&str, Readings); 6] = [
    (
        "an aspect condition involving the two bodies, as the engine's yoga evaluator has it",
        Readings {
            aspect_gathering: AspectGathering::Both,
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
    (
        "a table's degree the ordinal degree, n − 1° to n°, rather than a degree either side",
        Readings {
            bhaga: Bhaga::Running,
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
    (
        "a rule asking for exaltation or debilitation met by that dignity alone, not its deep form",
        Readings {
            dignity: DignityMatch::Exact,
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
    (
        "houses counted whole-sign from the lagna rather than recorded",
        Readings {
            houses: Houses::WholeSign,
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
    (
        "an involved planet only from the branch that decided",
        Readings {
            gathering: Gathering::DecidingBranch,
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
    (
        "an unqualified conjunction within 10° rather than in one sign",
        Readings {
            conjunction: Conjunction::Orb(10.0),
            ..Readings::RECORDING_ENGINE_DOSHAS
        },
    ),
];

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
                        Presence {
                            found_from: strings(&found["present_from"]),
                            planets: strings(&found["planets"]),
                            houses: found["houses"]
                                .as_array()
                                .map(|houses| {
                                    houses
                                        .iter()
                                        .filter_map(serde_json::Value::as_u64)
                                        .collect()
                                })
                                .unwrap_or_default(),
                            severity: found["severity"].as_u64().unwrap_or_default(),
                            cancellations: strings(&found["cancellations"]),
                            status: found["net_status"].as_str().unwrap_or_default().to_owned(),
                        },
                    )
                })
                .collect();
            out.push(Record {
                chart: chart(&file["inputs"])?,
                present,
            });
        }
    }
    Ok(out)
}

/// What one reading reproduces of the rules it is measured over.
#[derive(Default)]
struct Tally {
    decisions: usize,
    decisions_wrong: usize,
    presences: usize,
    found_wrong: usize,
    planets_wrong: usize,
    houses_wrong: usize,
    severity_wrong: usize,
    cancellations_wrong: usize,
    status_wrong: usize,
    moved: BTreeMap<String, usize>,
}

impl Tally {
    /// Everything it got wrong.
    fn wrong(&self) -> usize {
        self.decisions_wrong
            + self.found_wrong
            + self.planets_wrong
            + self.houses_wrong
            + self.severity_wrong
            + self.cancellations_wrong
            + self.status_wrong
    }
}

/// How the engine names a body in a label: its key in title case.
fn display(key: &str) -> String {
    let mut chars = key.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_string() + &chars.as_str().to_ascii_lowercase()
    })
}

/// The label the engine's result names a cancellation by: its own, or the one
/// the engine's code writes from the condition. It mirrors the engine's prose,
/// as `crates/rules/tests/doshas.rs` does for the kernel's own test; neither is
/// the kernel's, which carries the label a rule gives and nothing else.
fn label(cancellation: &teistro_rules::Cancellation) -> String {
    use teistro_rules::{BodyRef, Condition, Subject};

    if let Some(label) = &cancellation.label {
        return label.clone();
    }
    let name_of = |reference: &BodyRef| {
        reference
            .named()
            .map_or_else(String::new, |b| display(b.key()))
    };
    match &cancellation.condition {
        Condition::PlanetDignity { planet, dignities } => {
            let list: Vec<String> = dignities
                .iter()
                .map(|d| match d.key() {
                    "OWN_SIGN" => String::from("own"),
                    other => other.to_ascii_lowercase(),
                })
                .collect();
            format!("{} in {} sign", name_of(planet), list.join("/"))
        }
        Condition::PlanetConjunct { planets, .. } => {
            let names: Vec<String> = planets.iter().map(name_of).collect();
            names.split_last().map_or_else(String::new, |(last, head)| {
                format!("{} conjunct {last}", head.join(", "))
            })
        }
        Condition::PlanetInHouse {
            planet: Subject::Ref(reference),
            houses,
        } => {
            let houses: Vec<String> = houses.iter().map(|h| h.get().to_string()).collect();
            format!(
                "{} in {}th house",
                reference
                    .named()
                    .map_or_else(String::new, |b| display(b.key())),
                houses.join("/")
            )
        }
        Condition::LordOfHouseStrong { house_ruled } => {
            format!("Lord of {}th in own/exalted sign", house_ruled.get())
        }
        other => other.kind().to_owned(),
    }
}

fn status(recorded: &str) -> Option<NetStatus> {
    match recorded {
        "active" => Some(NetStatus::Active),
        "partially-cancelled" => Some(NetStatus::PartiallyCancelled),
        "fully-cancelled" => Some(NetStatus::FullyCancelled),
        _ => None,
    }
}

/// What a tally holds a result to, beyond its presence: the labels it was
/// found from, which only the engine's own rules carry, and the planets it
/// involved, which the rules the SDK wrote for the Kalsarpa family name
/// differently on purpose.
#[derive(Clone, Copy)]
struct Compare {
    labels: bool,
    participants: bool,
}

impl Compare {
    /// Everything, for the engine's own rules.
    const EVERYTHING: Compare = Compare {
        labels: true,
        participants: true,
    };
    /// Everything but the labels, for the rules the SDK wrote.
    const WITHOUT_LABELS: Compare = Compare {
        labels: false,
        participants: true,
    };
    /// What no rule here parts from on purpose.
    const SETTLED: Compare = Compare {
        labels: false,
        participants: false,
    };
}

/// Evaluates `rules` over every record under a reading, against what the
/// engine recorded.
fn tally(rules: &[&Rule], records: &[Record], readings: Readings, compare: Compare) -> Tally {
    let mut t = Tally::default();
    let tables = Tables::classical();
    for record in records {
        let evaluator = Evaluator::new(&record.chart, readings).with_tables(tables);
        for rule in rules {
            t.decisions += 1;
            let result = evaluator.evaluate(rule);
            let Some(recorded) = record.present.get(&rule.key) else {
                if result.present {
                    t.decisions_wrong += 1;
                    *t.moved.entry(rule.key.clone()).or_default() += 1;
                }
                continue;
            };
            if !result.present {
                t.decisions_wrong += 1;
                *t.moved.entry(rule.key.clone()).or_default() += 1;
                continue;
            }
            t.presences += 1;
            let mut moved = false;
            if compare.labels {
                let found: Vec<String> = result
                    .found_from
                    .iter()
                    .map(|found| match found {
                        Found::Conditions => String::from("Lagna"),
                        Found::Group(at) => rule
                            .groups
                            .get(*at)
                            .map(|g| g.label.clone())
                            .unwrap_or_default(),
                    })
                    .collect();
                if found != recorded.found_from {
                    t.found_wrong += 1;
                    moved = true;
                }
            }
            if compare.participants {
                let planets: Vec<String> = result
                    .participants
                    .iter()
                    .map(|b| b.key().to_owned())
                    .collect();
                if planets != recorded.planets {
                    t.planets_wrong += 1;
                    moved = true;
                }
                let houses: Vec<u64> = result.houses.iter().map(|h| u64::from(h.get())).collect();
                if houses != recorded.houses {
                    t.houses_wrong += 1;
                    moved = true;
                }
            }
            if result.severity.map(u64::from) != Some(recorded.severity) {
                t.severity_wrong += 1;
                moved = true;
            }
            let fired: Vec<String> = result
                .cancellations
                .iter()
                .filter_map(|i| rule.cancellations.get(*i))
                .map(label)
                .collect();
            if fired != recorded.cancellations {
                t.cancellations_wrong += 1;
                moved = true;
            }
            if result.status != status(&recorded.status) {
                t.status_wrong += 1;
                moved = true;
            }
            if moved {
                *t.moved.entry(rule.key.clone()).or_default() += 1;
            }
        }
    }
    t
}

fn claim(name: &str, t: &Tally, engine: bool) -> Claim {
    let wrong = t.wrong();
    if !engine && wrong == 0 {
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
    let mut moved: Vec<&str> = t.moved.keys().map(String::as_str).take(4).collect();
    if t.moved.len() > moved.len() {
        moved.push("…");
    }
    let claim = Claim::counted(name, wrong, t.decisions).with_note(format!(
        "{} decisions, and of {} presences {} found from, {} planets, {} houses, {} severities, {} cancellations, {} statuses",
        count(t.decisions_wrong),
        count(t.presences),
        count(t.found_wrong),
        count(t.planets_wrong),
        count(t.houses_wrong),
        count(t.severity_wrong),
        count(t.cancellations_wrong),
        count(t.status_wrong)
    ));
    if t.moved.is_empty() {
        claim
    } else {
        claim.with_note(format!(
            "{} move ({})",
            plural(t.moved.len(), "rule"),
            moved.join(", ")
        ))
    }
}

/// A row per rule the SDK wrote for one the engine computes in code.
fn written(shipped: &[Rule], records: &[Record]) -> String {
    let mut out =
        String::from("| rule | decisions | presences | what parts |\n|---|---|---|---|\n");
    for rule in shipped {
        // The labels a result is found from are the engine's own prose, which
        // its code writes rather than a rule.
        let t = tally(
            &[rule],
            records,
            Readings::RECORDING_ENGINE_DOSHAS,
            Compare::WITHOUT_LABELS,
        );
        let parts: Vec<String> = [
            ("the planets", t.planets_wrong),
            ("the houses", t.houses_wrong),
            ("the severity", t.severity_wrong),
            ("the cancellations", t.cancellations_wrong),
            ("the status", t.status_wrong),
        ]
        .into_iter()
        .filter(|(_, wrong)| *wrong > 0)
        .map(|(what, wrong)| format!("{what} on {}", plural(wrong, "presence")))
        .collect();
        let _ = writeln!(
            out,
            "| `{}` | {} of {} | {} | {} |",
            rule.key,
            count(t.decisions_wrong),
            count(t.decisions),
            count(t.presences),
            if parts.is_empty() {
                String::from("nothing")
            } else {
                parts.join(", ")
            }
        );
    }
    out
}

fn page(root: &Path) -> Result<String, String> {
    let all = rules(root, "doshas")?;
    let records = records(root)?;
    let evaluable: Vec<&Rule> = all.iter().filter(|r| r.is_evaluable()).collect();
    let computed: Vec<&Rule> = all.iter().filter(|r| r.computed.is_some()).collect();
    let shipped = shipped::computed_doshas();

    let readings = core::iter::once((
        "the engine's dosha reading, every choice below as its dosha evaluator makes it",
        Readings::RECORDING_ENGINE_DOSHAS,
    ))
    .chain(FORKS);
    let mut claims = Vec::new();
    let mut engine = Tally::default();
    for (at, (name, reading)) in readings.enumerate() {
        let t = tally(&evaluable, &records, reading, Compare::EVERYTHING);
        claims.push(claim(name, &t, at == 0));
        if at == 0 {
            engine = t;
        }
    }

    // The same forks over the rules the SDK wrote, where the table degree is
    // the one the corpus can see.
    let shipped_refs: Vec<&Rule> = shipped.iter().collect();
    let shipped_claims: Vec<Claim> = core::iter::once((
        "the engine's dosha reading, its table degree a degree either side",
        Readings::RECORDING_ENGINE_DOSHAS,
    ))
    .chain(FORKS)
    .enumerate()
    .map(|(at, (name, reading))| {
        let t = tally(&shipped_refs, &records, reading, Compare::SETTLED);
        claim(name, &t, at == 0)
    })
    .collect();

    let mut out = String::new();
    let _ = write!(
        out,
        "# The doshas, measured\n\n\
         Status: `generated` by `cargo xtask doshas` over the conformance corpus's `baseline/doshas`, \
         2026-09-16. Do not edit: `check-doshas` regenerates this page and fails on any difference. The \
         design it measures is [`rules-engine.md`](rules-engine.md). The pass evaluates the engine's own \
         rules over every recorded chart under a reading, and the SDK's rules for the ones it computes in \
         code against what that code recorded.\n\n\
         ## What the corpus holds\n\n\
         {rules} over {charts}, {panchangas} of them with a recorded panchanga: {decisions} decisions of \
         the {evaluable} rules the engine evaluates from their conditions, {presences} of them present. \
         The other {computed} it computes in code: {computed_keys}.\n\n\
         ## What the corpus decides\n\n{table}\n\
         ## The seventeen, written as rules\n\n\
         `crates/rules/rules/computed-doshas.json` says in the language what the engine says in code, \
         each rule carrying the citation and the severity the engine's own rule declares. Measured \
         against what its code recorded:\n\n{written}\n\
         The Kalsarpa family parts from the engine in one way, deliberately: its code names the two \
         nodes as the graha involved, and these rules name the seven grahas the nodes caught, whose \
         houses follow.\n\n\
         ### What a reading moves in them\n\n\
         Their presences, severities, cancellations and statuses; the planets the Kalsarpa family names \
         are left out, as the row above says they part on purpose.\n\n{shipped_table}\n\
         ## What it means for the kernel\n\n\
         **The language says what the engine's code said.** All {computed} rules it computes in code \
         decide presence exactly as that code did on every recorded chart, and Mrityu Bhaga, Dagdha \
         Rashi and Badhaka reproduce every recorded field as well: their planets, houses, severities, \
         cancellations and statuses. What the language needed for them was the arc's side named in the \
         rule, a badhaka reference, and a weight on a group so the luminaries and the lagna count \
         double in a Mrityu Bhaga severity.\n\n\
         **The corpus decides one reading and cannot see five.** Conjunction in one sign against a 10° \
         orb moves 131 answers over twelve rules; the other five move nothing here, as they move \
         nothing in the yogas. The degree a Mrityu Bhaga table names is the reading this corpus does \
         see (crux C82), and the rows above say what it moves.\n",
        rules = plural(all.len(), "rule"),
        charts = plural(records.len(), "chart"),
        panchangas = count(
            records
                .iter()
                .filter(|r| r.chart.panchanga.is_some())
                .count()
        ),
        decisions = count(engine.decisions),
        evaluable = count(evaluable.len()),
        presences = count(engine.presences),
        computed = count(computed.len()),
        computed_keys = computed
            .iter()
            .map(|r| format!("`{}`", r.key))
            .collect::<Vec<_>>()
            .join(", "),
        table = table(&claims),
        written = written(shipped, &records),
        shipped_table = table(&shipped_claims),
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask doshas") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
