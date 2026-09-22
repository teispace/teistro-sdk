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
use teistro_chart::foundation::GrahaPosition;
use teistro_core::angle::Nas;
use teistro_core::boundary::Boundaries;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::Degrees;
use teistro_houses::chart::Bhava;
use teistro_houses::classify::{Quadrant, lord_of};
use teistro_interpret::{
    KEYS, Plan, aspects, chalit, conditions, houses, karakas, phala, placements, positions,
    readings, strength,
};
use teistro_intl::source::{BASE_LOCALE, Completeness, Tree};
use teistro_intl::{Intl, OutPart, Rendered};
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
/// The rule readings a locale carries, loaded rather than embedded.
const READINGS: &str = "packs/readings";
/// The state readings, loaded beside them: what a chart *is* rather than
/// what it triggers (`03-design/state-readings.md`).
const STATES: &str = "packs/states";

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
    /// Each graha under **both** house readings, built from the recorded
    /// `planet_houses` (the chalit's) and `shifted` (which names the
    /// placement system's where the two differ). The fields the composer
    /// does not read keep the record's own zeroes, as the bhavas do.
    positions: Vec<GrahaPosition>,
}

/// The grahas of a recorded chalit, under both readings.
fn positions_of(chalit: &serde_json::Value) -> Vec<GrahaPosition> {
    let shifted: BTreeMap<&str, u8> = chalit["shifted"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let planet = row["planet"].as_str()?;
            let whole = u8::try_from(row["whole_sign_house"].as_u64()?).ok()?;
            Some((planet, whole))
        })
        .collect();
    let mut out = Vec::new();
    for graha in Graha::ALL.into_iter().take(9) {
        let Some(bhava) = chalit["planet_houses"][graha.key()]
            .as_u64()
            .and_then(|n| u8::try_from(n).ok())
        else {
            continue;
        };
        // Where the record names no shift, the two readings agree.
        let house = shifted.get(graha.key()).copied().unwrap_or(bhava);
        out.push(GrahaPosition {
            graha,
            longitude_deg: 0.0,
            tropical_deg: 0.0,
            latitude_deg: 0.0,
            distance_au: 0.0,
            speed_deg_per_day: 0.0,
            placement: placed_at(bhava),
            house: placed_at(house),
        });
    }
    out
}

/// A placement in a bhava and nothing else the composer reads.
fn placed_at(bhava: u8) -> teistro_chart::bhava::Placement {
    teistro_chart::bhava::Placement {
        bhava,
        method: teistro_core::catalogue::HouseSystem::WholeSign,
        through: 0.0,
        from_madhya_deg: 0.0,
    }
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
                    positions: positions_of(chalit),
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

fn composed(
    root: &Path,
    rules: &[Rule],
    vocabulary: &dyn teistro_interpret::Vocabulary,
) -> Result<Vec<Composed>, String> {
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
            plan.items.extend(conditions(&chart, vocabulary));
            plan.items.extend(karakas(&chart));
            plan.items.extend(phala(&chart, vocabulary));
            plan.items.extend(aspects(&relations(&chart)?));
            plan.items.extend(readings(
                held.iter().map(|(rule, result)| (*rule, result)),
                vocabulary,
            ));
            if let Some(weighed) = weighed.get(&name) {
                plan.items.extend(strength(&weighed.reading));
            }
            if let Some(divided) = divided.get(&name) {
                plan.items.extend(houses(&divided.bhavas));
                plan.items.extend(chalit(&divided.positions));
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

/// The key table, the verse a reading cites untranslated, and the lagna —
/// a silence until a message that reads a point closed it.
fn what_they_say(
    out: &mut String,
    by_key: &BTreeMap<&str, usize>,
    items: usize,
    charts: usize,
    composed_rules: usize,
    carried: usize,
) {
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
    the_readings_seam(out, by_key, composed_rules, carried);
    let _ = write!(
        out,
        "**The lagna is said now, and it took a message that reads a \
         point.** It stands in every one of these charts and was in none of \
         the placement items, because those messages read a graha and the \
         lagna is `point.LAGNA`: {} it did not say, one a chart. \
         `sdk.reason.pointInRashi` and `sdk.reason.pointAt` read a **point** \
         instead, and both strict locales already named eight members of \
         that kind — the ascendant, the five upagrahas, Gulika and Mandi — \
         so the vocabulary was bought before the frame was written. It is \
         said first, because it is what the rest is read against, and by its \
         sign alone: its bhava is the first by definition.\n\n",
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
        "And the **strengths** say what a graha weighs *and* whether that is \
         enough. The corpus records a Shadbala for {} of these charts, {} in \
         all, and for each of them the rupas its text requires and whether it \
         reaches them — {} of {} do. Both are items now, score then \
         sufficiency, where for four composers the second crossed in the \
         document and was absent from the plan. The message names the \
         **requirement** and not a verdict: it says the rupas the text asks \
         for and whether the graha reaches them, and never \"strong\", which \
         is a word no locale here has been given and a machine translation of \
         it would be the stub the project refuses.\n\n",
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
    let said_shifts: usize = plans
        .iter()
        .flat_map(|composed| composed.plan.iter())
        .filter(|item| {
            item.key
                == <teistro_intl::messages::sdk::reason::ChalitShift as teistro_intl::TypedMessage>::KEY
        })
        .count();
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
        "And the **houses** say the sign each bhava falls in and the graha \
         that rules it. The corpus records a division for {} of these charts, \
         every one of them under {}, of which {} came back degenerate — and \
         records for each which bodies fall in a different house under the \
         chalit: {} of {} placings do. The sign cost no new vocabulary, \
         because a `Rashi` is catalogued and every locale names one; what a \
         bhava knows **besides** is which third of the wheel it stands in and \
         whether it is a trine, a house of difficulty or one that grows \
         better with time, and none of those is a catalogue member — a \
         `Quadrant` is a Rust enum and the rest are predicates. Saying them \
         needs words no locale here has been given, which is the shape the \
         state readings' §8 names, not a composer's decision.\n\n\
         **The shift itself is said**, by a composer of its own: {} of the \
         plan's items are `sdk.reason.chalitShift`, one for each placing the \
         corpus records as moving, and none for a placing it does not — the \
         two counts are the same number above and below, so a composer that \
         said one shift too many or too few would change this page.\n\n",
        count(reached.len()),
        named.join(" and "),
        count(degenerate),
        count(shifted),
        count(placed),
        count(said_shifts),
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
         where the division is equal — so the composer says the sign the \
         **record** carries, which is its middle's, and this page cannot tell \
         the two apart on a corpus where they never differ. The branch is the \
         houses service's to hold; nothing here claims to have tried it.\n\n",
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

/// Where the seam closes and where it does not, which is a measurement and
/// not a promise (`03-design/interpretation-records.md` §5).
///
/// `readings` says a rule's own reading where the base locale carries one
/// and the verse's cited words where it does not. How often that helps is a
/// fact about **two rule sets** rather than about the mechanism: the
/// readings were written against the recording engine's keys, and the
/// kernel ships packs of its own.
fn the_readings_seam(
    out: &mut String,
    by_key: &BTreeMap<&str, usize>,
    composed_rules: usize,
    carried: usize,
) {
    let says = by_key
        .get(<teistro_intl::messages::sdk::reading::Says as teistro_intl::TypedMessage>::KEY)
        .copied()
        .unwrap_or_default();
    let _ = write!(
        out,
        "**Where a locale has been given a reading, the seam closes**, and \
         the measurement says how far. A readings pack is loaded here the \
         way a consumer loads one, and `readings` asks the **base** locale \
         for each rule: {} of the {} this pass composes carry a reading, and \
         they produced {}, said in each locale's own words instead of the \
         verse's English.\n\n",
        count(carried),
        plural(composed_rules, "rule"),
        plural(says, "`sdk.reading.says` item"),
    );
    let _ = write!(
        out,
        "**That ratio is a fact about two rule sets and not about the \
         mechanism.** The readings were written against the recording \
         engine's rule keys, where they cover all but eighteen \
         ([`interpretation-records-measured.md`](interpretation-records-measured.md)); \
         the kernel ships packs written independently, and their keys are \
         not the same keys. They are not two spellings of one set either — \
         dropping the kernel's leading segment matches 44 of its 263 \
         nabhasas and none of its 73 arishtas — so nothing is mapped across \
         by resemblance, and the page reports the gap rather than closing \
         it with a guess. Every rule that matched states **no** effect of \
         its own, which is why a reading is said for a rule rather than for \
         one of its statements: those rules were the silent ones.\n\n",
    );
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

/// Where the chart document's own list of sections is declared, which this
/// page reads rather than restates.
const SECTIONS_SOURCE: &str = "crates/sdk/src/reading.rs";

/// Every section a chart document can carry, and either the composer that
/// says it or the reason none does.
///
/// **This is the queue the composers work through, and it was prose
/// before.** The page used to say that the silences left were all of one
/// kind — a computed value with no catalogue member — and that was a claim
/// written once and believed after. It is not the whole truth: five of the
/// eleven sections a document can carry have **no composer at all**, and
/// the reason differs for each. A section here with no composer and no
/// reason fails, a reason for a section that has gained a composer fails,
/// and a section this list does not name fails, so the queue cannot rot
/// into a sentence again.
///
/// A row may carry **both**, and three do: a section can have a composer
/// and still not be finished. `STATE` is the case that forced the third
/// column to mean *what is left* rather than *why nobody says it* — a
/// `GrahaState` carries a dozen facts and `conditions` says the four that
/// reach it through a `Placement`, so the row was reading as answered.
///
/// The composer named must be a member of `PlanRequest`, which is the
/// other half: a composer renamed would otherwise leave a section looking
/// answered.
///
/// The fourth element names the kinds a row's reason cites as **unnamed**,
/// and each must still be on `intl`'s own list of members no strict locale
/// names. That is the check this list needed most: two of these reasons
/// said a section was cheap *because its values are catalogue members*,
/// and being a catalogue member is not being named — `vaiseshikamsa` and
/// `nature` are both catalogued and neither is named anywhere. A kind that
/// gains a vetted table now fails here, so the reason cannot outlive the
/// blocker.
const SECTION_SAYS: [(&str, &[&str], &str, &[&str]); 11] = [
    (
        "PANCHANGA",
        &["panchanga", "phala"],
        "\
        the four **spans** each limb covered, the sankranti and the \
        eclipse. `panchanga` says the five limbs running at the chart's \
        instant, the Moon's pada and whether the birth fell by day; \
        `phala` says what a loaded pack reads of them. A span is a pair of \
        ghatikas a consumer formats with, which is the line \
        `sdk.reason.exactLongitude` is already on, and a sankranti and an \
        eclipse are **conditions** of the day rather than limbs of it. The \
        kaalas, the choghadiya, the horas and the thirty muhurtas are on \
        the same record and belong to a reader of **times** rather than of \
        a chart",
        &[],
    ),
    (
        "STATE",
        &["conditions", "states"],
        "\
        the three friendships and the four avasthas are said now, which \
        is what this row asked for: `conditions` says every fact a \
        `Placement` carries and the section's own `GrahaState` carries a \
        dozen. What is still unsaid is the **Sayanadi**, whose members no \
        strict locale names because the vetted tables stop at the four \
        avasthas, the Cheshta for the same reason, the war a graha is in \
        and how near it stands to a classification boundary — the last \
        two being records whose own shape is undecided rather than \
        unnamed",
        &["avastha_sayanadi", "avastha_cheshta"],
    ),
    ("ASPECTS", &["aspects"], "", &[]),
    (
        "POINTS",
        &[],
        "\
        the five upagrahas and the special lagnas are points with \
        longitudes, and `positions` says a **graha's** degree in the same \
        sentence a point's would need — but the vetted table names the \
        upagrahas only, so a special lagna has no name for the sentence to \
        put in. The name comes before the sentence, and five of the state \
        corpus's `special-lagna` readings are waiting on the same table",
        &["point"],
    ),
    ("HOUSES", &["houses"], "", &[]),
    (
        "ASHTAKAVARGA",
        &["ashtakavarga"],
        "\
        the reductions and the three pindas. This row was the last of the \
        four called **decisions** and the only one that stayed one, and \
        what it asked was never whether the numbers cross — they have \
        crossed as `Document.ashtakavarga` since the module landed — but \
        which of them a sentence is worth. The answer taken is the two a \
        text quotes: each graha's bindus in the **sign it stands in**, \
        which is why this composer takes the chart beside the section, \
        and each sign's **sarvashtakavarga**, said once rather than \
        repeated inside every graha's sentence. Nineteen items. No band \
        and no verdict, which is `bhava_bala`'s rule and not \
        `strength`'s: a `GrahaShadbala` carries `required_rupas` and an \
        `AshtakavargaReading` carries no threshold, so *weak* and \
        *strong* would be a rule invented rather than said — the \
        objection that keeps `shadbala-strength`'s four bands out of the \
        state corpus. What is left is left because it is \
        **ruleset-dependent**: the measured page falsifies the text's \
        reading of the trine reduction, the Ekadhipatya and all three \
        pindas against the corpus on every chart, so a sentence saying \
        one would have to name the ruleset it was reduced under. And the \
        sarva is said by sign, which is what the reading is indexed by; \
        by bhava would fold in the house system, which is `houses`'",
        &[],
    ),
    ("VIMSHOPAKA", &["vimshopaka"], "", &[]),
    (
        "SHADBALA",
        &["strength"],
        "\
        the **total** and the requirement, which is the whole of it: a \
        `GrahaShadbala` carries the six strengths it is the sum of and \
        none of the six is an item, so a consumer reading a plan learns \
        what a graha weighs and not what makes it weigh that. Four of the \
        six — `dig`, `cheshta`, `naisargika`, `drik` — are **bare \
        numbers** and would say themselves exactly as the Vimshopaka \
        does, a message matching on the component's key. The other two \
        are not: `SthanaBala` is five parts and `KaalaBala` is seven, so \
        the question is how deep to go, and adding 42 items to a composer \
        a consumer already asks for by name is a change to something that \
        works rather than an addition. **That** is the decision — where \
        the breakdown belongs, not whether the words exist",
        &[],
    ),
    ("BHAVA_BALA", &["bhavaBala"], "", &[]),
    (
        "VAISESHIKAMSA",
        &[],
        "\
        **no strict locale names its designations.** Kimshuka, Parijata, \
        Gopura and the rest are catalogue members, and being a catalogue \
        member is not being named: `vaiseshikamsa` is on the unnamed list \
        with no vetted source, exactly as the special lagnas are, so a \
        composer saying them would print nothing a locale carries. This \
        row called it the cheapest of the six on the strength of *being \
        catalogued*, which is the conflation this page had been making in \
        prose. It also names a graha under four schemes at once, which is \
        `VIMSHOPAKA`'s knob, and carries an `impaired` flag that decides \
        whether the name it earned is auspicious",
        &["vaiseshikamsa"],
    ),
    ("DASHA_PHALA", &["dashaPhala"], "", &[]),
];

/// Every section the document declares, and what says it.
///
/// The list of sections is read from the source that declares them, so a
/// twelfth added to `Sections` and left out of [`SECTION_SAYS`] fails here
/// rather than going unnoticed.
/// Keys no single chart can emit, and why each needs a different one.
///
/// A **zero** on the table above is a question, and the day this list was
/// written one of them had been sitting unasked for a week:
/// `sdk.phala.tithi` read zero because `RuleInputs::of` never gave a
/// chart its limbs, which also kept 11 shipped rules from ever holding.
/// So every key a composer can emit is now emitted by **one founded
/// chart** with every section asked for and both corpora loaded, and a
/// key that chart cannot reach is listed here with the reason rather than
/// printed as a zero for someone to wonder about.
///
/// The list fails **three** ways: a key this chart does not emit and is
/// not here fails; a key here that the chart does emit fails; and an
/// excuse naming something no composer emits at all fails, so a key
/// renamed cannot leave a reason behind pointing at nothing.
///
/// It is not a hypothetical. **Three of the four excuses first written
/// here were wrong** — the undecided lajjitadi, the dasha's favour and
/// the birth by day were all guesses about what a chart would contain,
/// and the check refused them one at a time until only the burnt graha
/// was left.
const UNREACHED: [(&str, &str); 1] = [(
    "sdk.condition.combust",
    "no graha of this chart is burnt; a chart with one is a different chart",
)];

/// One chart the SDK founds itself, with every section and both corpora,
/// so that every key a composer can emit is emitted somewhere.
///
/// The recorded corpus cannot do this: it records bodies and chara karakas
/// and no graha states, no almanac and no strengths, so nine of the
/// thirteen composers run over it and four do not. That is a fact about
/// the fixtures rather than about the composers, and reading it off the
/// table above as thirty-three zeros is how a dead end hid in plain sight.
/// One context with both corpora loaded, as a consumer's would be.
fn context_with_the_corpora(root: &Path) -> Result<teistro::Context, String> {
    let sdk = teistro::Context::builder()
        .profile("conformance-baseline")
        .ephemeris([teistro::Ephemeris::Builtin])
        .build()
        .map_err(|why| format!("the conformance profile: {why}"))?;
    for corpus in [READINGS, STATES] {
        let tree = Tree::load(&root.join(corpus)).map_err(|err| err.to_string())?;
        for locale in tree.locales.values() {
            let bytes = teistro_intl::pack::build(locale, teistro_intl::source::ENTITY_NAMESPACE)
                .map_err(|err| format!("{corpus}/{}: {err}", locale.tag))?;
            sdk.intl()
                .load_pack(&bytes)
                .map_err(|err| format!("{corpus}/{}: {err}", locale.tag))?;
        }
    }
    Ok(sdk)
}

/// Every composer's plan for one chart the SDK founds itself, with every
/// section asked for.
fn the_founded_plan(sdk: &teistro::Context) -> Result<Plan, String> {
    let place = teistro::quantity::Place::new(
        teistro::quantity::Latitude::try_new(27.7172).map_err(|e| e.to_string())?,
        teistro::quantity::Longitude::try_new(85.324).map_err(|e| e.to_string())?,
        teistro::quantity::Altitude::try_new(1400.0).map_err(|e| e.to_string())?,
    );
    let offset = teistro::UtcOffset::try_from_seconds(20_700).map_err(|e| e.to_string())?;
    let every = rules();
    let request = teistro::ChartRequest::at(place, offset)
        .with_rule_inputs(&every)
        .with_panchanga()
        .with_aspects()
        .with_houses()
        .with_shadbala()
        .with_dasha_phala()
        .with_bhava_bala()
        .with_vimshopaka()
        .with_ashtakavarga()
        .with_points();
    let instant =
        teistro::quantity::JulianDay::try_new(2_448_000.5).map_err(|why| why.to_string())?;
    let read = sdk
        .chart()
        .readings(&[instant], &request)
        .map_err(|why| format!("founding one chart: {why}"))?;
    let document = read
        .value
        .first()
        .ok_or_else(|| String::from("one chart was asked for"))?;

    let mut plan = Plan::default();
    let interpret = || sdk.interpret();
    for said in [
        interpret().placements(document),
        interpret().positions(document),
        interpret().conditions(document),
        interpret().karakas(document),
        interpret().phala(document),
        interpret().states(document),
        interpret().panchanga(document),
        interpret().dasha_phala(document),
        interpret().houses(document),
        interpret().aspects(document),
        interpret().strength(document),
        interpret().bhava_bala(document),
        interpret().vimshopaka(document),
        interpret().ashtakavarga(document),
    ] {
        plan.items.extend(said.map_err(|why| why.to_string())?);
    }
    plan.items.extend(interpret().chalit(document));
    let set = teistro::RuleRequest::shipped([])
        .with_rules(every.clone())
        .rule_set()
        .map_err(|why| why.to_string())?;
    let with_rules = sdk
        .chart()
        .readings_with_rules(&[instant], &request, &set)
        .map_err(|why| format!("reading the rules: {why}"))?;
    if let Some((_, reading)) = with_rules.value.first() {
        plan.items.extend(interpret().readings(reading));
    }

    Ok(plan)
}

/// What a reader of **each shipped locale** gets of one composed plan.
///
/// `strict` completeness is gated and the other locales are not, which is
/// a fact the roster states — `hi-Deva-IN`, `sa-Deva` and `sa-Latn` are at
/// `base` "until their messages are translated" — but until now it was
/// only a flag. What it *costs a reader* is measurable, and it is not
/// all-or-nothing: an item is a message frame around **entity slots**, and
/// those slots are named in every shipped locale, so a reader at `base`
/// gets the grahas, the rashis and the nakshatras in their own script
/// inside an English sentence.
///
/// This table is what a consumer would want before choosing a locale, and
/// what the translation work would be measured against.
fn what_each_reader_gets(
    out: &mut String,
    sdk: &teistro::Context,
    plan: &Plan,
    root: &Path,
) -> Result<(), String> {
    let tree = Tree::load(&root.join("i18n")).map_err(|err| err.to_string())?;
    let mut rows: Vec<(String, &'static str, usize, usize, usize)> = Vec::new();
    // The base locale's rendering of each item, to compare the rest with.
    sdk.intl()
        .set_locale(BASE_LOCALE)
        .map_err(|why| format!("{BASE_LOCALE}: {why}"))?;
    let english: Vec<String> = plan
        .items
        .iter()
        .map(|item| sdk.intl().render(&item.key, &item.params).text)
        .collect();
    for locale in tree.locales.values() {
        let completeness = if locale.meta.completeness == Completeness::Strict {
            "strict"
        } else {
            "base"
        };
        sdk.intl()
            .set_locale(&locale.tag)
            .map_err(|why| format!("{}: {why}", locale.tag))?;
        let mut own = 0usize;
        let mut theirs = 0usize;
        for (item, base) in plan.items.iter().zip(&english) {
            let said = sdk.intl().render(&item.key, &item.params);
            if said.resolved_from.as_deref() == Some(locale.tag.as_str()) {
                own += 1;
            }
            // What the reader actually sees: an item whose frame falls
            // back can still carry its **slots** in the reader's own
            // script, and for a rule's reading the slot is the whole
            // sentence. Comparing the text against the base locale's is
            // what says whether anything reached them.
            if said.text != *base {
                theirs += 1;
            }
        }
        rows.push((
            locale.tag.clone(),
            completeness,
            own,
            theirs,
            plan.items.len(),
        ));
    }
    out.push_str("## What a reader of each locale gets\n\n");
    out.push_str(
        "Of the founded chart's plan, two questions a consumer choosing a \
         locale would ask. **Its own messages** is how many items the \
         locale answers without falling back; **anything of its own** is \
         how many render differently from the base locale, which is the \
         one that says whether a reader sees their own language at all. \
         The two differ because an item is a frame around **slots**, and a \
         slot is named in every shipped locale — for a rule's reading the \
         slot is the whole sentence, so a locale at `base` can carry the \
         substance of an item whose frame is English.\n\n\
         | locale | completeness | its own messages | anything of its own |\n|---|---|---:|---:|\n",
    );
    for (tag, completeness, own, theirs, total) in &rows {
        let _ = writeln!(
            out,
            "| `{tag}` | `{completeness}` | {} of {} | {} of {} |",
            count(*own),
            count(*total),
            count(*theirs),
            count(*total)
        );
    }
    out.push('\n');
    Ok(())
}

/// What a consumer's **rich** renderer gets from the plan, which until
/// the parts crossed the boundary was nothing at all.
///
/// The two facts that matter to a reader of this page are how much of a
/// plan carries markup and whether ignoring it is safe. The second is a
/// property rather than a number, so it is checked over every item in
/// every strict locale and the pass fails rather than the page reporting
/// a near miss.
fn what_a_rich_renderer_gets(
    out: &mut String,
    sdk: &teistro::Context,
    plan: &Plan,
    strict: &[String],
) -> Result<(), String> {
    let (rows, by_key) = markup_of(sdk, plan, strict)?;
    said_richly(out, plan, strict, &rows, &by_key)
}

/// One row a locale: its tag, the items of the plan that carry markup,
/// how many markup parts those are, and the tag names.
type MarkupRow = (String, usize, usize, String);

/// Every item of the plan rendered in every strict locale, counting the
/// markup and refusing a rendering whose text parts are not its text.
fn markup_of<'a>(
    sdk: &teistro::Context,
    plan: &'a Plan,
    strict: &[String],
) -> Result<(Vec<MarkupRow>, BTreeMap<&'a str, usize>), String> {
    let mut rows: Vec<MarkupRow> = Vec::new();
    let mut by_key: BTreeMap<&str, usize> = BTreeMap::new();
    for tag in strict {
        sdk.intl()
            .set_locale(tag)
            .map_err(|why| format!("{tag}: {why}"))?;
        let mut with_markup = 0usize;
        let mut tags: BTreeSet<String> = BTreeSet::new();
        let mut parts = 0usize;
        for item in &plan.items {
            let said = sdk.intl().render(&item.key, &item.params);
            let marked: Vec<&str> = said
                .parts
                .iter()
                .filter_map(|part| match part {
                    OutPart::Markup { name, .. } => Some(name.as_str()),
                    OutPart::Text(_) => None,
                })
                .collect();
            // Ignoring the markup has to be safe, or a renderer that does
            // not know a tag silently drops text with it.
            let joined: String = said
                .parts
                .iter()
                .filter_map(|part| match part {
                    OutPart::Text(text) => Some(text.as_str()),
                    OutPart::Markup { .. } => None,
                })
                .collect();
            if joined != said.text {
                return Err(format!(
                    "`{}` in {tag}: the text parts joined are not the text",
                    item.key
                ));
            }
            if !marked.is_empty() {
                with_markup += 1;
                parts += marked.len();
                tags.extend(marked.iter().map(|name| (*name).to_string()));
                // By key from the base locale alone, so the table counts
                // items and not items times locales.
                if tag == BASE_LOCALE {
                    *by_key.entry(item.key.as_str()).or_default() += 1;
                }
            }
        }
        let tags = tags
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        rows.push((
            tag.clone(),
            with_markup,
            parts,
            if tags.is_empty() {
                String::from("none")
            } else {
                tags
            },
        ));
    }
    Ok((rows, by_key))
}

/// The section itself, from what `markup_of` counted.
fn said_richly(
    out: &mut String,
    plan: &Plan,
    strict: &[String],
    rows: &[MarkupRow],
    by_key: &BTreeMap<&str, usize>,
) -> Result<(), String> {
    out.push_str("## What a rich renderer gets\n\n");
    out.push_str(
        "MF2 markup (`{#b}…{/b}`) is how a message says that part of it is a \
         name, emphasis or a link **without saying what that looks like**: \
         the message stays free of any markup language and the consumer's \
         renderer decides. Until a rendered message's parts crossed the C \
         boundary only a Rust caller could see one, so the numbers below \
         are what the other three bindings had no way to reach — the \
         messages carried the tags and the text arrived with them \
         stripped.\n\n\
         | locale | items with markup | markup parts | tags |\n|---|---:|---:|---|\n",
    );
    for (tag, with_markup, parts, tags) in rows {
        let _ = writeln!(
            out,
            "| `{tag}` | {} of {} | {} | {tags} |",
            count(*with_markup),
            count(plan.items.len()),
            count(*parts)
        );
    }
    out.push('\n');
    if let Some((first, rest)) = rows.split_first()
        && let Some(other) = rest.iter().find(|row| row.1 != first.1)
    {
        return Err(format!(
            "{} marks up {} item(s) where {} marks up {}: a translation has dropped or added \
             markup",
            other.0, other.1, first.0, first.1
        ));
    }
    out.push_str("| key | items of the plan |\n|---|---:|\n");
    for (key, items) in by_key {
        let _ = writeln!(out, "| `{key}` | {} |", count(*items));
    }
    out.push('\n');
    out.push_str(
        "The share is small and the property beside it is what makes the \
         boundary safe to use: **the text parts joined are the text**, for \
         every item in every strict locale, so a renderer that knows none \
         of the tags can drop every markup part and lose no words. That is \
         what lets a binding ship the parts before anyone has written a \
         renderer for them.\n\n\
         The key table is the **base locale's**, and every strict locale \
         marks up the same items, which the pass fails on rather than \
         reports: a translation that drops the base's markup reads \
         perfectly as text and tells a renderer nothing, so the loss is \
         invisible in exactly the place a reviewer looks. The engine's own \
         parity check refuses it at the source, and this is that rule read \
         back off a rendered plan.\n\n",
    );
    out.push_str(&table(&[
        Claim::counted(
            "a rendered item's text parts, joined, are its text",
            0,
            plan.items.len() * strict.len(),
        ),
        Claim::counted(
            "every strict locale marks up the same items of the plan",
            0,
            strict.len(),
        ),
    ]));
    out.push('\n');
    Ok(())
}

/// That every key a composer can emit is emitted by that one chart, and
/// that every item of it is said in each strict locale.
fn every_key_is_emitted(out: &mut String, root: &Path, strict: &[String]) -> Result<(), String> {
    let sdk = context_with_the_corpora(root)?;
    let plan = the_founded_plan(&sdk)?;
    let emitted: BTreeSet<&str> = plan
        .items
        .iter()
        .filter_map(|item| KEYS.iter().find(|key| **key == item.key).copied())
        .collect();
    let excused: BTreeMap<&str, &str> = UNREACHED.iter().copied().collect();
    for key in excused.keys() {
        if !KEYS.contains(key) {
            return Err(format!(
                "UNREACHED excuses `{key}`, which no composer can emit at all"
            ));
        }
    }
    for key in KEYS {
        match (emitted.contains(key), excused.contains_key(key)) {
            (false, false) => {
                return Err(format!(
                    "`{key}` is emitted by no composer over the founded chart and UNREACHED does \
                     not say why"
                ));
            }
            (true, true) => {
                return Err(format!(
                    "`{key}` is emitted over the founded chart and UNREACHED still excuses it"
                ));
            }
            _ => {}
        }
    }

    // The other half: a key emitted is not a key said. Every item of the
    // founded plan is rendered in each strict locale, which is what the
    // recorded corpus can only do for nine composers' worth.
    let mut rendered = 0usize;
    for tag in strict {
        sdk.intl()
            .set_locale(tag)
            .map_err(|why| format!("{tag}: {why}"))?;
        for item in &plan.items {
            let said = sdk.intl().render(&item.key, &item.params);
            if said.is_fallback || !said.warnings.is_empty() || said.text.is_empty() {
                return Err(format!(
                    "`{}` in {tag}: fallback {}, {} warning(s), {} characters",
                    item.key,
                    said.is_fallback,
                    said.warnings.len(),
                    said.text.len()
                ));
            }
            rendered += 1;
        }
    }

    what_each_reader_gets(out, &sdk, &plan, root)?;

    what_a_rich_renderer_gets(out, &sdk, &plan, strict)?;

    out.push_str("## Every key, emitted at least once\n\n");
    let _ = write!(
        out,
        "The table above counts what the **recorded corpus** exercises, and \
         it records bodies and chara karakas: no graha states, no almanac, \
         no strengths. So nine of the thirteen composers run over it and \
         four do not, and the zeros that leaves are a fact about the \
         fixtures rather than about the composers. Reading them as \
         nothing-to-see is how a dead end hid for a week — \
         `sdk.phala.tithi` read zero because `RuleInputs::of` never gave a \
         chart its limbs, which also kept **11 shipped rules** from ever \
         holding.\n\n\
         So every key is emitted here by **one chart the SDK founds \
         itself**, with every section asked for and both corpora loaded — \
         {} of {} — and every item of it renders in each strict locale \
         from that locale's own message, with no fallback and nothing to \
         warn about: {}. What this chart cannot reach is named below, \
         with the reason it needs another. A key missing from both fails, \
         and so does a key excused that this chart emits — which is not a \
         hypothetical: **three of the four excuses first written here were \
         wrong**, and the check said so one at a time.\n\n\
         | key | why this chart cannot reach it |\n|---|---|\n",
        count(emitted.len()),
        count(KEYS.len()),
        plural(rendered, "rendering"),
    );
    for (key, why) in UNREACHED {
        let _ = writeln!(out, "| `{key}` | {why} |");
    }
    out.push('\n');
    out.push_str(&table(&[Claim::counted(
        "every key a composer can emit is emitted by one founded chart, or listed with the reason \
         it needs another",
        0,
        KEYS.len(),
    )]));
    out.push('\n');
    Ok(())
}

/// The section names `Sections` declares, read from its own source.
///
/// `const fn has(self, one: Sections)` splits the same way a constant
/// does, so a section is recognised by its name's own shape.
fn sections_declared(text: &str) -> BTreeSet<&str> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("pub(crate) const "))
        .filter_map(|rest| rest.split_once(": Sections"))
        .map(|(name, _)| name)
        .filter(|name| {
            !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
        })
        .collect()
}

/// That [`SECTION_SAYS`] and `Sections` agree, and that every row says
/// either who says it or why nobody does.
fn sections_agree(declared: &BTreeSet<&str>) -> Result<(), String> {
    let listed: BTreeSet<&str> = SECTION_SAYS.iter().map(|(name, ..)| *name).collect();
    let unnamed: BTreeSet<&str> = crate::intl::UNNAMED
        .iter()
        .map(|(kind, ..)| *kind)
        .collect();
    for name in declared {
        if !listed.contains(name) {
            return Err(format!(
                "`Sections::{name}` is declared in {SECTIONS_SOURCE} and SECTION_SAYS does not name it"
            ));
        }
    }
    for (name, composers, why, kinds) in SECTION_SAYS {
        if !declared.contains(name) {
            return Err(format!(
                "SECTION_SAYS names `{name}` and {SECTIONS_SOURCE} declares no such section"
            ));
        }
        if composers.is_empty() && why.is_empty() {
            return Err(format!(
                "`{name}` names neither a composer nor what is left to say of it"
            ));
        }
        for composer in composers {
            if !teistro::PlanRequest::MEMBERS.contains(composer) {
                return Err(format!(
                    "`{name}` is said by `{composer}`, which is not a member of PlanRequest"
                ));
            }
        }
        for kind in kinds {
            if !unnamed.contains(kind) {
                return Err(format!(
                    "`{name}` gives `{kind}` as a reason it cannot be said, and no strict \
                     locale is short of a name for that kind any more"
                ));
            }
        }
    }
    Ok(())
}

/// The sections `SECTION_SAYS` gives no composer, named.
///
/// **Named and not counted**, and read from the table rather than
/// written beside it: two paragraphs of this page each counted them in
/// prose, and each said a number that a composer built the same week
/// made wrong.
fn sections_with_no_composer() -> Vec<String> {
    SECTION_SAYS
        .iter()
        .filter(|(_, composers, _, _)| composers.is_empty())
        .map(|(name, _, _, _)| format!("`{name}`"))
        .collect()
}

/// A list of names as a sentence reads them, or a phrase for none.
fn named_or_none(names: &[String], none: &str) -> String {
    if names.is_empty() {
        String::from(none)
    } else {
        names.join(" and ")
    }
}

fn every_section(out: &mut String, root: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(root.join(SECTIONS_SOURCE))
        .map_err(|why| format!("{SECTIONS_SOURCE}: {why}"))?;
    sections_agree(&sections_declared(&text))?;

    let said = SECTION_SAYS
        .iter()
        .filter(|(_, composers, ..)| !composers.is_empty())
        .count();
    let partly = SECTION_SAYS
        .iter()
        .filter(|(_, composers, why, _)| !composers.is_empty() && !why.is_empty())
        .count();
    let sourcing = SECTION_SAYS
        .iter()
        .filter(|(_, _, _, kinds)| !kinds.is_empty())
        .count();
    let unsaid = sections_with_no_composer();
    out.push_str("## Every section, and what says it\n\n");
    let _ = write!(
        out,
        "A composer says a **section** or it says a placement, and the \
         sections are what a chart request asks for by name. {} of the {} \
         a document can carry have a composer, with {} of those \
         unfinished, and the rest carrying the reason nobody says them — \
         because \"the silences left are all of one kind\" is exactly the \
         sentence this page had and exactly the sentence that was not \
         true. The third column means *what is left*, which `STATE` is \
         why: a `GrahaState` carries a dozen facts and `conditions` says \
         the four that reach it through a `Placement`, so the row read as \
         answered. The list is read from the source that declares the \
         sections, so a twelfth fails here rather than being forgotten, \
         and a composer named must be a member of `PlanRequest`. The rows \
         nothing says are **named** rather than described — {} — because \
         the sentence that described them named `VIMSHOPAKA` as one of \
         them for as long as it took to build it, which is the rot this \
         page keeps catching in its own prose. {} rows are short a \
         **name** rather than a sentence, because being a catalogue \
         member is not being named. The kinds those rows cite are on \
         `intl`'s own list of members no strict locale names, checked \
         here so a reason cannot outlive its blocker.\n\n\
         | section | said by | what is left |\n|---|---|---|\n",
        count(said),
        count(SECTION_SAYS.len()),
        count(partly),
        named_or_none(&unsaid, "there are none"),
        count(sourcing),
    );
    for (name, composers, why, _) in SECTION_SAYS {
        let by = if composers.is_empty() {
            String::from("—")
        } else {
            composers
                .iter()
                .map(|composer| format!("`{composer}`"))
                .collect::<Vec<_>>()
                .join(" and ")
        };
        let reason = if why.is_empty() { "—" } else { why };
        let _ = writeln!(out, "| `{name}` | {by} | {reason} |");
    }
    out.push('\n');
    out.push_str(&table(&[Claim::counted(
        "every section a chart document can carry has a composer, or a reason here that it has none",
        0,
        SECTION_SAYS.len(),
    )]));
    out.push('\n');
    Ok(())
}

/// Whether a message says any words of its own, as against rendering a
/// value some other table names.
///
/// This is the whole of the native-review question. `{$graha :entity
/// kind=graha}` carries no prose: it prints whatever `sdk.entity` holds,
/// so a reviewer looking at it is looking at the entity table. `{$graha}
/// is burnt by the Sun` carries four words that someone wrote, and those
/// are what a native speaker has to read.
///
/// The parser answers it rather than a regular expression: literal text is
/// `Part::Text`, and a pattern's other parts are expressions and markup.
fn says_words_of_its_own(source: &str) -> bool {
    fn in_pattern(pattern: &teistro_intl::mf2::ast::Pattern) -> bool {
        pattern.0.iter().any(|part| match part {
            teistro_intl::mf2::ast::Part::Text(text) => !text.trim().is_empty(),
            teistro_intl::mf2::ast::Part::Expression(_)
            | teistro_intl::mf2::ast::Part::Markup(_) => false,
        })
    }
    let Ok(message) = teistro_intl::mf2::parser::parse(source) else {
        // An unparseable message fails `check-intl` long before this page
        // runs; counting it as prose is the cautious reading.
        return true;
    };
    match message {
        teistro_intl::mf2::ast::Message::Simple(pattern) => in_pattern(&pattern),
        teistro_intl::mf2::ast::Message::Complex(complex) => match complex.body {
            teistro_intl::mf2::ast::Body::Pattern(pattern) => in_pattern(&pattern),
            teistro_intl::mf2::ast::Body::Matcher(matcher) => matcher
                .variants
                .iter()
                .any(|variant| in_pattern(&variant.pattern)),
        },
    }
}

/// What a native reviewer has to read, in each strict locale that is not
/// the base.
///
/// **The flag was prose, and prose is what rots.** The roadmap's exit
/// criterion asks for `ne` and `hi` sign-off, and which messages that
/// covers was recorded in two design-page sentences — "flagged for the
/// native review", "the Nepali of `sdk.reading` awaits it" — written when
/// two namespaces existed. Ten messages were written on 2026-09-22 alone.
/// So the set is measured here instead: every message whose translation
/// says words of its own, by namespace, with the ones that only render a
/// catalogue value left out because a reviewer reading those is reading
/// `sdk.entity`.
fn what_a_reviewer_reads(out: &mut String, tree: &Tree, strict: &[String]) {
    out.push_str("## What a native reviewer has to read\n\n");
    let base = tree.base().map(|base| base.tag.clone()).unwrap_or_default();
    let _ = write!(
        out,
        "The roadmap's exit criterion asks for **`ne` and `hi` sign-off**, \
         and this is what that covers. A message that only renders a \
         catalogue value — `{{$graha :entity kind=graha}}` — carries no \
         prose of its own, and a reviewer reading it is reading \
         `sdk.entity`; a message with words in it is what someone has to \
         read. The parser decides which, so a message rewritten from one \
         into the other moves this list by itself. `{base}` is left out: \
         it is the base and its words are the source.\n\n",
    );
    let mut any = false;
    for tag in strict {
        if *tag == base {
            continue;
        }
        let Some(locale) = tree.locales.get(tag) else {
            continue;
        };
        let mut per_namespace: BTreeMap<&str, usize> = BTreeMap::new();
        let mut prose = 0usize;
        let mut rendered = 0usize;
        for (name, namespace) in &locale.namespaces {
            if !name.starts_with("sdk.") || name == "sdk.entity" {
                continue;
            }
            for entry in namespace.entries.values() {
                if let teistro_intl::source::Entry::Message(source) = entry {
                    if says_words_of_its_own(source) {
                        *per_namespace.entry(name.as_str()).or_default() += 1;
                        prose += 1;
                    } else {
                        rendered += 1;
                    }
                }
            }
        }
        any = true;
        let _ = write!(
            out,
            "**`{tag}`** — {} to read, {} that render a value and need no \
             reading.\n\n| namespace | messages with words of their own |\
             \n|---|---:|\n",
            plural(prose, "message"),
            count(rendered),
        );
        for (name, carried) in &per_namespace {
            let _ = writeln!(out, "| `{name}` | {} |", count(*carried));
        }
        out.push('\n');
    }
    if !any {
        out.push_str("No strict locale but the base carries a message.\n\n");
    }
    // The other locales carry `sdk.entity` and no messages at all, which
    // is what `base` completeness means, so their share of the review is
    // the names and not the sentences. Naming them here rather than
    // leaving the omission to be read as "nothing to review".
    let quiet: Vec<String> = tree
        .locales
        .keys()
        .filter(|tag| !strict.contains(tag))
        .map(|tag| format!("`{tag}`"))
        .collect();
    if !quiet.is_empty() {
        let _ = write!(
            out,
            "{} carry no `sdk.*` message at all — `base` completeness is \
             the entity names and the packs' own records — so their share \
             of the sign-off is the **names**, held by \
             [`entity-names.md`](entity-names.md) §4 and its list of \
             members with no vetted source. `hi-Deva-IN` is one of them, \
             which is worth saying plainly: the roadmap asks for `hi` \
             sign-off and there is not a sentence in it to sign off.\n\n",
            quiet.join(", "),
        );
    }
}

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
        let _ = write!(
            out,
            "No message is unaccounted for: every one either has a composer \
             or has a reason. **So a further composer needs a key that does \
             not exist yet**, as every one since the fifth has: the drishti, \
             the rest of a placement, a reading a pack carries, the lagna, \
             the rupas a graha's text requires, a bhava's sign, the chalit's \
             disagreement.\n\n\
             **What each cost was decided by the catalogue and not by the \
             sentence.** Where the thing being said is a catalogue member, \
             `sdk.entity` already names it in all five locales and only the \
             frame had to be written — a dignity, a rashi, a point, two \
             chara karakas. Where it is not, the words are the whole cost: \
             a bhava's quadrant, a longevity tier, a strength band are \
             computed and are not catalogue members, so a message would \
             have to name each in words no locale here has been given \
             (`state-readings.md` §8). The question a new composer asks \
             is often not whether the SDK knows the fact but whether the \
             model reached the catalogue — and [Q38](../QUESTIONS.md) \
             sharpened even that: a kind is a **key space**, so where a \
             corpus is what would supply the words, the question is \
             whether it keys by the vocabulary the SDK computes.\n\n\
             **It is not the only question, and this page used to say it \
             was.** The sentence here read \"the silences that remain are \
             all of that kind\", which was a claim about a set the \
             repository owns, written once and believed after. The \
             sections with no composer are named in the table above with \
             a reason each — {} — and they are not of one kind: a name, \
             a decision about *which* number deserves a sentence, a knob, \
             or nothing but the work. Naming them rather than counting \
             them is the point, because the sentence that counted them \
             said **six** for as long as it took to build four of \
             them.\n\n",
            named_or_none(&sections_with_no_composer(), "there are none left"),
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

/// Loads the rule readings into an engine, as a consumer would.
///
/// The records live in a root of their own because `i18n/` is compiled into
/// every artefact and this corpus is several times its size; building each
/// locale's pack and loading it is what a consumer does, so it is what this
/// pass does.
fn load_readings(root: &Path, intl: &mut Intl) -> Result<(), String> {
    for corpus in [READINGS, STATES] {
        let tree = Tree::load(&root.join(corpus)).map_err(|err| err.to_string())?;
        for locale in tree.locales.values() {
            let bytes = teistro_intl::pack::build(locale, teistro_intl::source::ENTITY_NAMESPACE)
                .map_err(|err| format!("{corpus}/{}: {err}", locale.tag))?;
            intl.load_pack(&bytes)
                .map_err(|err| format!("{corpus}/{}: {err}", locale.tag))?;
        }
    }
    Ok(())
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
    // The readings are **loaded**, as a consumer loads them: the pack is
    // built from `packs/readings` and handed to the engine through the same
    // `load_pack` the boundary offers, so this pass measures the path that
    // ships rather than a merge only it can do
    // (`03-design/interpretation-records.md` §3).
    load_readings(root, &mut intl)?;
    let shipped = rules();
    // How many of the rules this pass composes the base locale carries a
    // reading for — asked of the same vocabulary the composer asks, so the
    // page cannot claim a coverage the plan did not get.
    let carried = shipped
        .iter()
        .filter(|rule| teistro_interpret::Vocabulary::has_reading(&intl, &rule.key))
        .count();
    let plans = composed(root, &shipped, &intl)?;
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

    what_they_say(
        &mut out,
        &by_key,
        items,
        plans.len(),
        shipped.len(),
        carried,
    );

    unsaid_strength(&mut out, root);

    unsaid_houses(&mut out, root, &plans);

    the_rest_of_a_placement(&mut out, &plans);

    every_key_is_emitted(&mut out, root, &strict)?;

    every_section(&mut out, root)?;

    coverage(&mut out, &tree);

    what_a_reviewer_reads(&mut out, &tree, &strict);

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
