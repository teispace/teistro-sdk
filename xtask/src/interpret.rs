//! The pass over the narrative plans the composers write
//! (`03-design/interpret-composers.md`).
//!
//! The corpus records no interpretation text — not a composed sentence, not a
//! reading — so there is nothing here to compare a plan against. What it can
//! decide is whether a plan can be **said**: every item of every chart's plan
//! is rendered in each strict locale, and a fallback or a warning is a defect
//! the gate refuses. Beside that it counts what the composers say and what
//! they cannot say yet.
//!
//! `cargo xtask interpret` writes the page and `check-interpret` regenerates
//! it in memory and fails on any difference. The rendered plan the page ends
//! with is the per-language snapshot the module checklist asks for
//! (`09-guidelines/03-adding-a-module.md` §7), held byte for byte.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_aspect::{Drishti, Strength, drishti};
use teistro_core::angle::Nas;
use teistro_core::boundary::Boundaries;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::Degrees;
use teistro_houses::chart::Bhava;
use teistro_houses::classify::{Quadrant, lord_of};
use teistro_interpret::{
    KEYS, Plan, aspects, conditions, houses, karakas, placements, positions, readings, strength,
};
use teistro_intl::source::{Completeness, Tree};
use teistro_intl::{Intl, Rendered};
use teistro_rules::{Body, Evaluator, Readings as RuleReadings, Rule, RuleChart, shipped};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};
use crate::rules_corpus::{chart, read_json};

const PAGE: &str = "docs/03-design/interpret-measured.md";
const ROOT: &str = "fixtures/baseline/yogas";
/// The recorded Shadbalas the strength composer is measured over. They are
/// the corpus's **own** numbers, not the SDK's: a composer is measured on
/// whether it can say what it is given, and `check-shadbala` is where the
/// numbers themselves are held.
const WEIGHTS: &str = "fixtures/baseline/shadbala";
/// The recorded charts the houses composer is measured over. Their
/// `houses.selected` section records which sign each of the twelve cusps
/// falls in; the lord of a sign is the SDK's own table, as the lord on a
/// `Bhava` always is.
const DIVISIONS: &str = "fixtures/baseline";
/// The chart the page ends with, rendered whole: the corpus's first.
const SNAPSHOT: &str = "c001-kathmandu-1990-04-14";

/// One chart of the corpus and the plan its composers write.
struct Composed {
    name: String,
    plan: Plan,
    chart: RuleChart,
}

/// What the corpus records of a chart's Shadbala, as the strength composer
/// needs it: each graha's total in rupas, and whether it reached the rupas
/// its text requires — which is the part **no locale can say**, counted on
/// the page rather than guessed at in a message.
struct Weighed {
    reading: teistro_strength::shadbala::ShadbalaReading,
    sufficient: usize,
}

/// Every chart the Shadbala corpus records, by its file stem.
fn weights(root: &Path) -> BTreeMap<String, Weighed> {
    let mut out = BTreeMap::new();
    for dir in ["charts", "variants"] {
        let directory = root.join(WEIGHTS).join(dir);
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Ok(file) = read_json(&path) else { continue };
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            let recorded = &file["shadbala"];
            let mut grahas = Vec::new();
            let mut sufficient = 0;
            for graha in Graha::ALL.into_iter().take(7) {
                let at = &recorded[graha.key()];
                let Some(rupas) = at["total_rupas"].as_f64() else {
                    continue;
                };
                sufficient += usize::from(at["is_sufficient"].as_bool().unwrap_or(false));
                grahas.push(teistro_strength::shadbala::GrahaShadbala {
                    graha,
                    sthana: teistro_strength::shadbala::SthanaBala::default(),
                    dig: 0.0,
                    kaala: teistro_strength::shadbala::KaalaBala::default(),
                    cheshta: 0.0,
                    naisargika: 0.0,
                    drik: 0.0,
                    virupas: at["total_shashtiamshas"].as_f64().unwrap_or_default(),
                    rupas,
                    required_rupas: at["minimum_rupas"].as_f64().unwrap_or_default(),
                    strong: at["is_sufficient"].as_bool().unwrap_or(false),
                    ishta: 0.0,
                    kashta: 0.0,
                    subha_rashmi: 0.0,
                    ashubha_rashmi: 0.0,
                });
            }
            if !grahas.is_empty() {
                out.insert(
                    name,
                    Weighed {
                        reading: teistro_strength::shadbala::ShadbalaReading {
                            rules: teistro_strength::shadbala::ShadbalaRules::BPHS,
                            grahas,
                        },
                        sufficient,
                    },
                );
            }
        }
    }
    out
}

/// What the corpus records of a chart's division, as the houses composer
/// needs it and as the page needs to say what it left out: the twelve
/// bhavas, the system they were divided under, whether the division came
/// back degenerate, and how many bodies fall in a different house under the
/// chalit — the last three being what **no locale can say**.
struct Divided {
    bhavas: [Bhava; 12],
    system: String,
    degenerate: bool,
    placed: usize,
    shifted: usize,
}

/// Every chart whose division the corpus records, by its file stem.
///
/// The bhavas are built from the recorded `cusp_sign_index` and nothing
/// else: the sign is the corpus's, the lord is the SDK's table, and the
/// fields the composer does not read keep the record's own zeroes, so a
/// field added to `Bhava` does not reach this pass with a number it would
/// have to invent.
fn divisions(root: &Path) -> BTreeMap<String, Divided> {
    let mut out = BTreeMap::new();
    for dir in ["charts", "variants"] {
        let directory = root.join(DIVISIONS).join(dir);
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            if path.extension().is_none_or(|ext| ext != "json") {
                continue;
            }
            let Ok(file) = read_json(&path) else { continue };
            let Some(name) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
                continue;
            };
            let selected = &file["houses"]["selected"];
            let Some(signs) = selected["cusp_sign_index"].as_array() else {
                continue;
            };
            let mut bhavas = Vec::with_capacity(12);
            for (index, sign) in signs.iter().enumerate() {
                let Some(sign) = sign
                    .as_u64()
                    .and_then(|at| usize::try_from(at).ok())
                    .and_then(|at| Rashi::ALL.get(at).copied())
                else {
                    break;
                };
                bhavas.push(Bhava {
                    number: u8::try_from(index + 1).unwrap_or(1),
                    sign,
                    lord: lord_of(sign),
                    madhya_deg: 0.0,
                    sandhi_deg: 0.0,
                    quadrant: Quadrant::Kendra,
                });
            }
            let Ok(bhavas): Result<[Bhava; 12], _> = bhavas.try_into() else {
                continue;
            };
            let chalit = &file["houses"]["bhava_chalit"];
            out.insert(
                name,
                Divided {
                    bhavas,
                    system: selected["system"].as_str().unwrap_or("unknown").to_owned(),
                    degenerate: selected["is_degenerate"].as_bool().unwrap_or(false),
                    placed: chalit["planet_houses"]
                        .as_object()
                        .map_or(0, serde_json::Map::len),
                    shifted: chalit["shifted"].as_array().map_or(0, Vec::len),
                },
            );
        }
    }
    out
}

/// The drishtis a recorded chart holds, computed from its **signs**.
///
/// The corpus records no aspect at all — not a relation, not a strength —
/// so unlike the strengths and the houses there is nothing recorded to
/// compose from. What it does record is where every graha stands, and a
/// drishti is a function of the looking graha and two signs
/// (`aspect::drishti::between`), so the relations are derived here and the
/// page says they were. Whether they are the right relations is
/// `aspect-drishti-measured.md`'s business; what this pass decides is
/// whether a plan made of them can be said.
/// How near a longitude stands to each division's edge.
///
/// A drishti carries these and no composer reads them; they are filled
/// honestly anyway, because a zero would say the body sits exactly on a
/// boundary, which is a claim rather than an absence — and for the same
/// reason a longitude that is not a degree is refused here rather than
/// given a zero. The corpus reader already refuses one, so this is the
/// second statement of the same rule and not a branch that fires.
fn edges(longitude: f64) -> Result<Boundaries, String> {
    Degrees::try_new(longitude.rem_euclid(360.0))
        .map(|degrees| Boundaries::of(Nas::from_degrees(degrees)))
        .map_err(|why| format!("{longitude}: {why}"))
}

fn relations(chart: &RuleChart) -> Result<Vec<Drishti>, String> {
    let mut out = Vec::new();
    for from in Graha::ALL.into_iter().take(9) {
        let Some(here) = chart.placements.get(Body::Graha(from).index()) else {
            continue;
        };
        for to in Graha::ALL.into_iter().take(9) {
            if from == to {
                continue;
            }
            let Some(there) = chart.placements.get(Body::Graha(to).index()) else {
                continue;
            };
            let strength = drishti::between(from, here.sign, there.sign);
            if strength == Strength::None {
                continue;
            }
            out.push(Drishti {
                from,
                to,
                houses: drishti::house_count(here.sign, there.sign),
                strength,
                // The composer reads neither, but the chart carries the
                // longitudes, so these are the real edges rather than a
                // placeholder zero claiming a body sits on a boundary.
                from_edge: edges(here.longitude)?,
                to_edge: edges(there.longitude)?,
            });
        }
    }
    Ok(out)
}

/// The rules the readings composer is measured over: every set the kernel
/// ships whose rules say something in words, a span, a class, a severity or
/// a cancellation.
fn rules() -> Vec<Rule> {
    let mut rules = Vec::new();
    for pack in [
        shipped::nabhasas(),
        shipped::arishtas(),
        shipped::gandantas(),
        shipped::computed_doshas(),
        shipped::computed_yogas(),
    ] {
        rules.extend(pack.iter().cloned());
    }
    rules
}

fn composed(root: &Path, rules: &[Rule]) -> Result<Vec<Composed>, String> {
    let weighed = weights(root);
    let divided = divisions(root);
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let directory = root.join(ROOT).join(dir);
        let Ok(entries) = std::fs::read_dir(&directory) else {
            continue;
        };
        let mut paths: Vec<_> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let file = read_json(&path)?;
            let chart: RuleChart = chart(&file["inputs"])?;
            let name = path
                .file_stem()
                .map(|stem| stem.to_string_lossy().into_owned())
                .unwrap_or_default();
            let evaluator = Evaluator::new(&chart, RuleReadings::TEXTS).with_rules(rules);
            let held: Vec<(&Rule, teistro_rules::RuleResult)> = rules
                .iter()
                .filter_map(|rule| {
                    let result = evaluator.evaluate(rule);
                    result.present.then_some((rule, result))
                })
                .collect();
            let mut plan = placements(&chart);
            plan.items.extend(positions(&chart));
            plan.items.extend(conditions(&chart));
            plan.items.extend(karakas(&chart));
            plan.items.extend(aspects(&relations(&chart)?));
            plan.items
                .extend(readings(held.iter().map(|(rule, result)| (*rule, result))));
            if let Some(weighed) = weighed.get(&name) {
                plan.items.extend(strength(&weighed.reading));
            }
            if let Some(divided) = divided.get(&name) {
                plan.items.extend(houses(&divided.bhavas));
            }
            out.push(Composed { name, plan, chart });
        }
    }
    Ok(out)
}

/// What a plan costs written down, which is what it costs to cross a
/// boundary: the JSON `Plan` serialises to, and how much of that is the
/// verses' own cited words rather than keys and slots.
struct Written {
    bytes: usize,
    cited: usize,
}

/// A plan's written size, and the share of it the cited verses take.
///
/// The cited text is measured from the items themselves rather than from
/// the JSON, because it is the one slot whose length is a text's and not
/// the SDK's: every other slot is a key, a number or a short list.
fn written(plan: &Plan) -> Written {
    let bytes = serde_json::to_string(plan).map_or(0, |json| json.len());
    let cited = plan
        .items
        .iter()
        .filter(|item| {
            item.key
                == <teistro_intl::messages::sdk::reading::Effect as teistro_intl::TypedMessage>::KEY
        })
        .filter_map(|item| match item.params.get("text") {
            Some(teistro_intl::Value::Str(text)) => Some(text.len()),
            _ => None,
        })
        .sum();
    Written { bytes, cited }
}

/// How a locale answered every item of every plan.
#[derive(Default)]
struct Said {
    items: usize,
    fallbacks: Vec<String>,
    warnings: Vec<String>,
    missing: Vec<String>,
}

/// Renders every item in one locale, keeping what went wrong rather than a
/// count alone, so the page names it: either the lists are short enough to
/// print or the gate is red, and a bare count would hide which.
fn say(intl: &mut Intl, tag: &str, plans: &[Composed]) -> Result<Said, String> {
    intl.set_locale(tag).map_err(|err| err.to_string())?;
    let mut said = Said::default();
    for key in KEYS {
        if !intl.has(key) {
            said.missing.push(format!("`{key}` in `{tag}`"));
        }
    }
    for composed in plans {
        for item in &composed.plan {
            said.items += 1;
            let rendered: Rendered = intl.render(&item.key, &item.params);
            if rendered.is_fallback || rendered.resolved_from.is_none() {
                said.fallbacks.push(format!("`{}` in `{tag}`", item.key));
            }
            for warning in &rendered.warnings {
                said.warnings
                    .push(format!("`{}` in `{tag}`: {warning}", item.key));
            }
        }
    }
    said.fallbacks.sort();
    said.fallbacks.dedup();
    said.warnings.sort();
    said.warnings.dedup();
    Ok(said)
}

/// The text of one plan in one locale, item by item, each line prefixed
/// with the rule it came from where the item names one. The prefix is the
/// page's, not the prose's: a reader reviewing a snapshot needs to know
/// which rule said what, and the `rule` slot is there for exactly that.
fn lines(intl: &mut Intl, tag: &str, plan: &Plan) -> Result<Vec<String>, String> {
    intl.set_locale(tag).map_err(|err| err.to_string())?;
    Ok(plan
        .items
        .iter()
        .map(|item| {
            let said = intl.render(&item.key, &item.params).text;
            match item.params.get("rule") {
                Some(teistro_intl::Value::Str(rule)) => format!("{rule}: {said}"),
                _ => said,
            }
        })
        .collect())
}

/// What the plans cost written down, which is what they cost to cross a
/// boundary or to fill a golden file
/// (`03-design/plans-at-the-boundary.md` §2).
fn costs(out: &mut String, plans: &[Composed], items: usize) {
    let sizes: Vec<Written> = plans
        .iter()
        .map(|composed| written(&composed.plan))
        .collect();
    let bytes: usize = sizes.iter().map(|size| size.bytes).sum();
    let cited: usize = sizes.iter().map(|size| size.cited).sum();
    let widest = sizes.iter().map(|size| size.bytes).max().unwrap_or(0);
    let _ = write!(
        out,
        "Written down, a plan is what it costs to cross a boundary or fill a \
         golden file: {} of JSON over the {}, {} a chart, {} for the widest \
         and {} an item. The verses' own cited words are **not** what weighs \
         it — {}, {}% — so what a plan costs is the items themselves, each \
         naming its message and its rule again. Small enough to cross whole: \
         nothing here asks to be packed.\n\n",
        plural(bytes, "byte"),
        plural(plans.len(), "chart"),
        plural(bytes.checked_div(plans.len()).unwrap_or(0), "byte"),
        plural(widest, "byte"),
        plural(bytes.checked_div(items).unwrap_or(0), "byte"),
        plural(cited, "byte"),
        cited.saturating_mul(100).checked_div(bytes).unwrap_or(0),
    );
}

/// The key table, and the two silences that belong to no section: the verse
/// a reading cites untranslated, and the lagna no placement message names.
fn what_they_say(out: &mut String, by_key: &BTreeMap<&str, usize>, items: usize, charts: usize) {
    out.push_str("## What the composers say\n\n");
    out.push_str("| key | items |\n|---|---|\n");
    for (key, at) in by_key {
        let _ = writeln!(out, "| `{key}` | {} |", count(*at));
    }
    out.push('\n');
    let effects = by_key
        .get(<teistro_intl::messages::sdk::reading::Effect as teistro_intl::TypedMessage>::KEY)
        .copied()
        .unwrap_or_default();
    let _ = write!(
        out,
        "**The verse's own statement is not translated.** {} of the {} — every \
         `sdk.reading.effect` — carry the words the rule itself cites, in the \
         language the rule was written in, and the message prints them as they \
         are. So a Nepali reading says the placements, who took part, the span, \
         the class and the cancellation in Nepali, and the verse's sentence in \
         the translator's English, until a locale carries a reading of that \
         rule written by someone who reads the text. A machine translation \
         there would be worse than the visible seam.\n\n",
        count(effects),
        plural(items, "item")
    );
    let _ = write!(
        out,
        "What they cannot say is counted too: the **lagna** stands in every \
         one of these charts and is in none of the placement items, because \
         those messages read a graha and the lagna is `point.LAGNA` — {} it \
         does not say, one a chart. It does take part in a reading, where the \
         message names no kind and the lagna is the point it is.\n\n",
        plural(charts, "item")
    );
}

/// What the strengths do **not** say, counted from the corpus's own
/// recordings rather than asserted (`03-design/interpret-composers.md` §4).
fn unsaid_strength(out: &mut String, root: &Path) {
    let weighed = weights(root);
    let charts = weighed.len();
    let grahas: usize = weighed
        .values()
        .map(|weighed| weighed.reading.grahas.len())
        .sum();
    let sufficient: usize = weighed.values().map(|weighed| weighed.sufficient).sum();
    let _ = write!(
        out,
        "And the **strengths** say what a graha weighs and not whether it is \
         strong enough. The corpus records a Shadbala for {} of these charts, \
         {} in all, and for each of them whether it reaches the rupas its \
         text requires — {} of {} do. The plan says none of that: no locale \
         carries a message for it, and a machine-translated \"strong\" would \
         be the stub the project refuses. What it says instead is the \
         ordering, strongest first, which needs no word at all.\n\n",
        count(charts),
        plural(grahas, "graha"),
        count(sufficient),
        count(grahas),
    );
}

/// What the **houses** do not say, counted from the corpus's own recordings
/// rather than asserted (`03-design/interpret-composers.md` §4).
///
/// It is the largest silence any composer carries, so it is the one worth
/// counting: the composer says a lord and the corpus records, for the same
/// charts, three further facts no locale has a sentence for.
///
/// The counts are over the charts this pass actually **reached** and not
/// over every division the corpus holds, because a page that counted the
/// second would imply the composer had been measured on it. The difference
/// between the two is itself worth a sentence: the corpus records eight
/// charts under Placidus, and none of them is in this corpus.
fn unsaid_houses(out: &mut String, root: &Path, plans: &[Composed]) {
    let divided = divisions(root);
    let reached: Vec<&Divided> = plans
        .iter()
        .filter_map(|composed| divided.get(&composed.name))
        .collect();
    let systems: BTreeSet<&str> = reached.iter().map(|read| read.system.as_str()).collect();
    let degenerate = reached.iter().filter(|read| read.degenerate).count();
    let placed: usize = reached.iter().map(|read| read.placed).sum();
    let shifted: usize = reached.iter().map(|read| read.shifted).sum();
    let named: Vec<String> = systems.iter().map(|name| format!("`{name}`")).collect();
    let _ = write!(
        out,
        "And the **houses** say who rules each bhava and nothing else, which \
         is the largest silence a composer here carries. The corpus records a \
         division for {} of these charts, every one of them under {}, of which \
         {} came back degenerate — and records for each which bodies fall in \
         a different house under the chalit: {} of {} placings do. A bhava also knows the sign it falls in, which third of the \
         wheel it stands in and whether it is a trine, a house of difficulty \
         or one that grows better with time. **No locale carries a message \
         for any of it**, so the plan claims none of it. Each is a sentence a \
         locale would have to be given before a composer could say it, which \
         is a translator's decision and not a composer's.\n\n",
        count(reached.len()),
        named.join(" and "),
        count(degenerate),
        count(shifted),
        count(placed),
    );
    let all = divided.len();
    let unequal = divided
        .values()
        .filter(|read| read.system != "whole-sign")
        .count();
    let _ = write!(
        out,
        "That every one of them is whole-sign is a fact about **this** corpus \
         and not about the recordings: the conformance repository holds {} \
         divisions in all, {} of them under an unequal system, and the \
         composer reaches none of those. It matters because a bhava's sign is \
         the sign its *middle* falls in, which is the same as its cusp's only \
         where the division is equal — so the branch that tells the two apart \
         is the houses service's to hold, and this page does not claim to \
         have tried it.\n\n",
        count(all),
        count(unequal),
    );
}

/// What the corpus holds of the six facts a placement carries beyond its
/// sign, its house and its longitude — the ones `conditions` and `karakas`
/// say (`03-design/interpret-composers.md` §4).
///
/// A composer that says a condition only where it holds says nothing at all
/// on a corpus that holds none, so the page counts rather than assumes.
#[derive(Default)]
struct Rest {
    placed: usize,
    retrograde: usize,
    nodes_retrograde: usize,
    combust: usize,
    vargottama: usize,
    dignities: BTreeMap<&'static str, usize>,
    seven: usize,
    eight: usize,
    agree: usize,
    differ: usize,
    only_eight: usize,
    /// Charts recording at least one node **direct**, and how many of those
    /// are true-node variants: the measurement that decides whether saying
    /// "Rahu is retrograde" carries information or repeats a definition.
    charts_direct: usize,
    charts_direct_true: usize,
    charts_true_node: usize,
}

fn rest_of(plans: &[Composed]) -> Rest {
    let mut rest = Rest::default();
    for composed in plans {
        let is_true_node = composed.name.contains("true-node");
        rest.charts_true_node += usize::from(is_true_node);
        let mut any_direct = false;
        for graha in Graha::ALL.into_iter().take(9) {
            let Some(at) = composed.chart.placements.get(Body::Graha(graha).index()) else {
                continue;
            };
            rest.placed += 1;
            let is_node = matches!(graha, Graha::Rahu | Graha::Ketu);
            if at.retrograde {
                rest.retrograde += 1;
                rest.nodes_retrograde += usize::from(is_node);
            } else if is_node {
                any_direct = true;
            }
            rest.combust += usize::from(at.combust);
            rest.vargottama += usize::from(at.navamsha == at.sign);
            *rest.dignities.entry(at.dignity.key()).or_default() += 1;
            rest.seven += usize::from(at.karaka7.is_some());
            rest.eight += usize::from(at.karaka8.is_some());
            match (at.karaka7, at.karaka8) {
                (Some(of_seven), Some(of_eight)) if of_seven == of_eight => rest.agree += 1,
                (Some(_), Some(_)) => rest.differ += 1,
                (None, Some(_)) => rest.only_eight += 1,
                _ => {}
            }
        }
        if any_direct {
            rest.charts_direct += 1;
            rest.charts_direct_true += usize::from(is_true_node);
        }
    }
    rest
}

/// What the two karaka schemes disagree about, and what that decides.
fn the_karaka_schemes(out: &mut String, rest: &Rest) {
    let _ = write!(
        out,
        "**The two karaka schemes are not a formality.** Where both name a \
         graha they name the same karaka {} times and a different one {}, \
         and the eight reach {} the seven do not rank at all. A composer \
         emitting one of them would be choosing for the consumer in about \
         half of all cases, so `karakas` emits both and the key says which \
         — `ofSeven` or `ofEight` — so that filtering by key gives one \
         scheme whole. Which order the eight are ranked in is the chart's \
         and not the composer's: `rule_chart` follows BPHS ch. 32, the \
         recording engine puts the Pitrikaraka last, and these are the \
         corpus's **recorded** karakas, as the rupas above are its recorded \
         rupas.\n\n",
        count(rest.agree),
        count(rest.differ),
        plural(rest.only_eight, "graha"),
    );
}

/// Whether the nodes' retrogression is a fact or a definition, and which
/// dignities the corpus actually holds.
fn the_conditions_counted(out: &mut String, rest: &Rest, charts: usize) {
    let _ = write!(
        out,
        "**Saying a node is retrograde carries information**, which is a \
         measurement and not an assumption. {} of the {} retrogressions are \
         Rahu's and Ketu's, and the nodes would be a tautology if they \
         always moved backwards — but {} of these {} charts record them \
         **direct**, and {} of those {} are `--true-node` variants, of {} \
         the corpus holds. The true node turns; the mean node does not. So \
         the condition is said of every graha that holds it, the nodes \
         included.\n\n",
        count(rest.nodes_retrograde),
        count(rest.retrograde),
        count(rest.charts_direct),
        count(charts),
        count(rest.charts_direct_true),
        count(rest.charts_direct),
        count(rest.charts_true_node),
    );
    let absent: Vec<&str> = teistro_core::catalogue::Dignity::ALL
        .into_iter()
        .map(teistro_core::catalogue::Dignity::key)
        .filter(|key| !rest.dignities.contains_key(key))
        .collect();
    let all = teistro_core::catalogue::Dignity::ALL.len();
    let _ = write!(
        out,
        "**A dignity is said of every graha, `NEUTRAL` included**, because \
         *sama* is a dignity the texts name rather than the absence of one \
         — which is the line `aspects` draws on the other side, skipping \
         `Strength::None`. It crosses as an **entity** and not as a string, \
         so the message has no arms to go stale: {} of the catalogue's {} \
         dignities occur in these charts{}, and a locale carrying nothing \
         but `sdk.entity` renders each one's own word.\n\n",
        count(all - absent.len()),
        count(all),
        if absent.is_empty() {
            String::new()
        } else {
            format!(" (no chart records {})", absent.join(", "))
        },
    );
}

/// The rest of a placement, now that every one of its nine facts is said.
fn the_rest_of_a_placement(out: &mut String, plans: &[Composed]) {
    let rest = rest_of(plans);
    let _ = write!(
        out,
        "And the **rest of a placement** is now said. A placement is nine \
         facts: `placements` says the sign and the house, `positions` the \
         longitude, and `conditions` and `karakas` the six that were left. \
         Over the {} these charts place: {} stand retrograde, {} are burnt \
         by the Sun, {} are vargottama, {} carry a chara karaka among seven \
         and {} among eight. Every one of them is an item now, where before \
         the plan said none of it.\n\n",
        plural(rest.placed, "graha"),
        count(rest.retrograde),
        count(rest.combust),
        count(rest.vargottama),
        count(rest.seven),
        count(rest.eight),
    );
    the_karaka_schemes(out, &rest);
    the_conditions_counted(out, &rest, plans.len());
}

/// Why a message the base locale carries is read by no composer.
///
/// The list is exhaustive and written here rather than in prose, because a
/// claim about which messages are spare goes stale the moment one is
/// written: this page reports any message it does not recognise as
/// **unaccounted**, which is the gate asking for a reason rather than
/// assuming there is none.
const SPARE: [(&str, &str); 7] = [
    (
        "sdk.reason.appName",
        "the pack's own name, said inside `welcome`",
    ),
    (
        "sdk.reason.welcome",
        "a greeting the packs ship as an example",
    ),
    (
        "sdk.reason.greeting",
        "the same, and the only message reading a gender",
    ),
    (
        "sdk.reason.exactLongitude",
        "a longitude alone (`222°34′35″`) — a fragment a consumer formats with, not a sentence a plan says",
    ),
    (
        "sdk.reason.strength.rank",
        "an ordinal alone (`1st`, `१लो`) — the same, and why `strength` carries the ranking in the items' order instead",
    ),
    (
        "sdk.reason.rashiNature",
        "a fact about the zodiac rather than about a chart: every chart would say the same twelve sentences",
    ),
    (
        "sdk.reason.conjunction",
        "a count of what `occupants` already names, graha by graha",
    ),
];

/// Every message the base locale carries, and which composer reads it.
///
/// This is where the composers' coverage of the packs is decided rather
/// than asserted in prose. A message that is neither emitted nor listed in
/// [`SPARE`] is printed as unaccounted, so writing one and forgetting to
/// read it shows up here instead of quietly sitting unread.
fn coverage(out: &mut String, tree: &Tree) {
    let Some(base) = tree.base() else { return };
    // The scope is **derived**, not listed: every namespace a composer
    // already emits from, split by the locale's own rule. Adding a composer
    // over a new namespace widens this by itself, and a namespace that is
    // not a reading vocabulary — `sdk.calendar`, which formats dates —
    // stays out without being named. Naming the two it started with is
    // what let `sdk.aspect` slip past this very check on its first run.
    let namespaces: BTreeSet<&str> = KEYS
        .iter()
        .filter_map(|key| base.split(key).map(|(namespace, _)| namespace))
        .collect();
    let messages: Vec<String> = base
        .keys()
        .filter(|key| {
            base.split(key)
                .is_some_and(|(namespace, _)| namespaces.contains(namespace))
        })
        .collect();
    let spare: BTreeMap<&str, &str> = SPARE.iter().copied().collect();
    let read = messages.iter().filter(|key| KEYS.contains(&key.as_str()));
    let named: Vec<String> = namespaces.iter().map(|ns| format!("`{ns}`")).collect();
    let unaccounted: Vec<&String> = messages
        .iter()
        .filter(|key| !KEYS.contains(&key.as_str()) && !spare.contains_key(key.as_str()))
        .collect();
    let _ = write!(
        out,
        "## What the packs carry, and what reads it\n\n{} of the {} the base \
         locale carries under {} are emitted by a composer. The namespaces \
         are the ones the composers already read, taken from `KEYS` rather \
         than named here, so a composer over a new one widens this by \
         itself. The rest are listed one by one with the reason no composer \
         reads them, because \"there is nothing left to compose\" is a claim \
         that goes stale the moment a message is written.\n\n",
        count(read.count()),
        plural(messages.len(), "message"),
        named.join(", "),
    );
    out.push_str("| message | why no composer reads it |\n|---|---|\n");
    for (key, why) in SPARE {
        let known = messages.iter().any(|carried| carried == key);
        let why = if known {
            why
        } else {
            "**not in the packs any more**"
        };
        let _ = writeln!(out, "| `{key}` | {why} |");
    }
    out.push('\n');
    if unaccounted.is_empty() {
        out.push_str(
            "No message is unaccounted for: every one either has a composer \
             or has a reason. **So a further composer needs a key that does \
             not exist yet**, as `aspects` did for the drishti and as \
             `conditions` and `karakas` did for the rest of a placement. \
             The second time cost less than the first: four of those seven \
             messages say a value the **entity** namespace already names in \
             all five locales — a dignity, a rashi, two chara karakas — so \
             what had to be written was the frame and not the vocabulary. \
             What remains unsaid is counted above rather than guessed at \
             here.\n\n",
        );
    } else {
        for key in unaccounted {
            let _ = writeln!(
                out,
                "- `{key}` is carried, emitted by nothing, and unexplained"
            );
        }
        out.push('\n');
    }
}

/// The claims the packs and the corpus decide, and whatever went wrong.
fn decided(out: &mut String, said: &[(String, Said)], items: usize, locales: usize) {
    out.push_str("## What the corpus decides\n\n");
    let mut claims = vec![Claim::counted(
        "every key a composer can emit is carried by every strict locale",
        said.iter().map(|(_, said)| said.missing.len()).sum(),
        KEYS.len() * locales,
    )];
    for (tag, said) in said {
        claims.push(Claim::counted(
            format!("every item renders from `{tag}`'s own message, not a fallback"),
            said.fallbacks.len(),
            said.items,
        ));
        claims.push(Claim::counted(
            format!("every item renders in `{tag}` with nothing to warn about"),
            said.warnings.len(),
            said.items,
        ));
    }
    out.push_str(&table(&claims));
    out.push('\n');
    let mut wrong = 0;
    for (_, said) in said {
        for line in said
            .missing
            .iter()
            .chain(&said.fallbacks)
            .chain(&said.warnings)
        {
            let _ = writeln!(out, "- {line}");
            wrong += 1;
        }
    }
    if wrong == 0 {
        let _ = write!(
            out,
            "Every one of the {} renderings — {} in each of {} — answered from \
             the locale's own message with nothing to warn about.\n\n",
            count(items * locales),
            count(items),
            plural(locales, "strict locale"),
        );
    } else {
        out.push('\n');
    }
}

fn page(root: &Path) -> Result<String, String> {
    let tree = Tree::load(&root.join("i18n")).map_err(|err| err.to_string())?;
    let strict: Vec<String> = tree
        .locales
        .values()
        .filter(|locale| locale.meta.completeness == Completeness::Strict)
        .map(|locale| locale.tag.clone())
        .collect();
    let mut intl = Intl::from_tree(&tree).map_err(|err| err.to_string())?;
    let shipped = rules();
    let plans = composed(root, &shipped)?;
    let mut by_key: BTreeMap<&str, usize> = KEYS.iter().map(|key| (*key, 0)).collect();
    for composed in &plans {
        for item in &composed.plan {
            if let Some(at) = by_key.get_mut(item.key.as_str()) {
                *at += 1;
            }
        }
    }
    let items: usize = plans.iter().map(|composed| composed.plan.len()).sum();
    let said: Vec<(String, Said)> = strict
        .iter()
        .map(|tag| say(&mut intl, tag, &plans).map(|said| (tag.clone(), said)))
        .collect::<Result<_, _>>()?;

    let mut out = String::new();
    out.push_str("# The composers, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask interpret` over the conformance \
         corpus's recorded charts and the `i18n/` sources, 2026-09-21. Do not \
         edit: `check-interpret` regenerates this page and fails on any \
         difference. The design it measures is \
         [`interpret-composers.md`](interpret-composers.md).\n\n"
    );
    out.push_str("## What was composed\n\n");
    let _ = write!(
        out,
        "{} composed to {}, {} a chart. The corpus records no interpretation \
         text of any kind, so nothing here is compared against a recording: \
         what is measured is whether a plan can be **said** in every locale \
         that must carry it.\n\n",
        plural(plans.len(), "recorded chart"),
        plural(items, "item"),
        plural(items.checked_div(plans.len()).unwrap_or(0), "item"),
    );

    costs(&mut out, &plans, items);

    decided(&mut out, &said, items, strict.len());

    what_they_say(&mut out, &by_key, items, plans.len());

    unsaid_strength(&mut out, root);

    unsaid_houses(&mut out, root, &plans);

    the_rest_of_a_placement(&mut out, &plans);

    coverage(&mut out, &tree);

    let snapshot = plans
        .iter()
        .find(|composed| composed.name == SNAPSHOT)
        .ok_or_else(|| format!("{SNAPSHOT} is not in the corpus any more"))?;
    out.push_str("## One chart, said\n\n");
    let _ = write!(
        out,
        "`{SNAPSHOT}`, every item of its plan, in each strict locale. This is \
         the per-language snapshot the module checklist asks for, and a \
         change to a composer or to a message moves it.\n\n"
    );
    let mut rendered = String::new();
    for tag in &strict {
        let _ = writeln!(rendered, "**{tag}**\n");
        let _ = writeln!(rendered, "```text");
        for line in lines(&mut intl, tag, &snapshot.plan)? {
            let _ = writeln!(rendered, "{line}");
        }
        let _ = writeln!(rendered, "```\n");
    }
    Ok(fill(&out) + rendered.trim_end() + "\n")
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    Ok(vec![Output::new(PAGE, page(root)?)])
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// Regenerates the page in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => check(root, &outputs, "cargo xtask interpret"),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
