//! The falsification pass over the yogas (Phase 6), before the rules kernel.
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

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, plural, table};

const PAGE: &str = "docs/03-design/yogas-measured.md";
const ROOT: &str = "fixtures/baseline/yogas";
const BODIES: [&str; 10] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU", "LAGNA",
];
const SEVEN: [&str; 7] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN",
];
const SIGNS: [&str; 12] = [
    "ARIES",
    "TAURUS",
    "GEMINI",
    "CANCER",
    "LEO",
    "VIRGO",
    "LIBRA",
    "SCORPIO",
    "SAGITTARIUS",
    "CAPRICORN",
    "AQUARIUS",
    "PISCES",
];
/// Each sign's lord, Aries to Pisces.
const LORDS: [&str; 12] = [
    "MARS", "VENUS", "MERCURY", "MOON", "SUN", "MERCURY", "VENUS", "MARS", "JUPITER", "SATURN",
    "SATURN", "JUPITER",
];
const KENDRA: [u64; 4] = [1, 4, 7, 10];
const TRIKONA: [u64; 3] = [1, 5, 9];
/// The orb a degree-based reading of an unqualified conjunction takes.
const ORB_DEG: f64 = 10.0;

/// One recorded body.
struct Body {
    key: &'static str,
    sign: u64,
    house: u64,
    dignity: String,
    longitude: f64,
    retrograde: bool,
    combust: bool,
    karaka7: Option<String>,
    karaka8: Option<String>,
}

/// One recorded chart and what the engine answered for it.
struct Record {
    bodies: Vec<Body>,
    /// Each present rule's involved planets, and its fired cancellations.
    present: BTreeMap<String, (Vec<String>, Vec<String>)>,
}

/// One rule of `rules.json`.
struct Rule {
    key: String,
    category: String,
    conditions: Vec<Value>,
    cancellations: Vec<Value>,
}

/// A place the condition language leaves a meaning open, named by the choice
/// the engine does not make there.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Fork {
    /// Benefics by the natural lists alone, the Moon and Mercury never turned
    /// malefic.
    NaturalBenefics,
    /// A rule asking for exaltation or debilitation met by that dignity alone.
    PlainDignity,
    /// All seven between the nodes counted from Rahu to Ketu only.
    RahuToKetuOnly,
    /// Houses counted whole-sign from the lagna rather than recorded.
    WholeSignHouses,
    /// Rahu and Ketu counted retrograde where a rule asks.
    NodesRetrograde,
    /// An involved planet only from the branch that decided.
    DecidingBranchOnly,
    /// An unqualified conjunction within an orb rather than in one sign.
    OrbConjunction,
}

/// Every fork, with how the page states it.
const FORKS: [(Fork, &str); 7] = [
    (
        Fork::NaturalBenefics,
        "benefics by the natural lists alone, the Moon and Mercury never turned malefic",
    ),
    (
        Fork::PlainDignity,
        "a rule asking for exaltation or debilitation met by that dignity alone, not its deep form",
    ),
    (
        Fork::RahuToKetuOnly,
        "all seven between the nodes counted from Rahu to Ketu only",
    ),
    (
        Fork::WholeSignHouses,
        "houses counted whole-sign from the lagna rather than recorded",
    ),
    (
        Fork::NodesRetrograde,
        "Rahu and Ketu counted retrograde where a rule asks",
    ),
    (
        Fork::DecidingBranchOnly,
        "an involved planet only from the branch that decided",
    ),
    (
        Fork::OrbConjunction,
        "an unqualified conjunction within 10° rather than in one sign",
    ),
];

/// The engine's reading, or the engine's with one fork taken the other way.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Reading(Option<Fork>);

/// The reading that reproduces the recording engine.
const ENGINE: Reading = Reading(None);

impl Reading {
    /// Whether this reading takes `fork` the other way from the engine.
    fn flips(self, fork: Fork) -> bool {
        self.0 == Some(fork)
    }
}

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

fn numbers(value: &Value) -> Vec<u64> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default()
}

fn rules(root: &Path) -> Result<Vec<Rule>, String> {
    let file = read_json(&root.join(ROOT).join("rules.json"))?;
    file["rules"]
        .as_array()
        .ok_or("rules.json: rules")?
        .iter()
        .map(|rule| {
            Ok(Rule {
                key: rule["key"].as_str().ok_or("a rule key")?.to_owned(),
                category: rule["category"].as_str().unwrap_or_default().to_owned(),
                conditions: rule["conditions"].as_array().cloned().unwrap_or_default(),
                cancellations: rule["cancellations"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default(),
            })
        })
        .collect()
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
            let inputs = &file["inputs"];
            let bodies = BODIES
                .iter()
                .map(|key| {
                    let b = &inputs["bodies"][*key];
                    Body {
                        key,
                        sign: b["sign_index"].as_u64().unwrap_or_default(),
                        house: b["house"].as_u64().unwrap_or_default(),
                        dignity: b["dignity"].as_str().unwrap_or_default().to_owned(),
                        longitude: b["sidereal_longitude_deg"].as_f64().unwrap_or_default(),
                        retrograde: b["is_retrograde"].as_bool().unwrap_or_default(),
                        combust: b["combust"].as_str() != Some("none"),
                        karaka7: inputs["chara_karaka_7"][*key].as_str().map(str::to_owned),
                        karaka8: inputs["chara_karaka_8"][*key].as_str().map(str::to_owned),
                    }
                })
                .collect();
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
            out.push(Record { bodies, present });
        }
    }
    Ok(out)
}

/// A rule evaluated over one chart under one reading.
struct Eval<'a> {
    bodies: &'a [Body],
    reading: Reading,
    benefics: Vec<&'static str>,
    malefics: Vec<&'static str>,
    lagna: u64,
}

/// The house `target` is from `from`, 1 to 12.
const fn house_from(target: u64, from: u64) -> u64 {
    (target + 12 - from) % 12 + 1
}

impl<'a> Eval<'a> {
    fn new(bodies: &'a [Body], reading: Reading) -> Eval<'a> {
        let sign = |key: &str| bodies.iter().find(|b| b.key == key).map(|b| b.sign);
        let mut benefics = vec!["JUPITER", "VENUS", "MERCURY", "MOON"];
        let mut malefics = vec!["SUN", "MARS", "SATURN", "RAHU", "KETU"];
        if !reading.flips(Fork::NaturalBenefics) {
            let longitude = |key: &str| bodies.iter().find(|b| b.key == key).map(|b| b.longitude);
            if let (Some(moon), Some(sun)) = (longitude("MOON"), longitude("SUN"))
                && (moon - sun).rem_euclid(360.0) > 180.0
            {
                benefics.retain(|k| *k != "MOON");
                malefics.push("MOON");
            }
            if let Some(mercury) = sign("MERCURY") {
                let with = |set: &[&str]| {
                    set.iter()
                        .any(|k| *k != "MERCURY" && sign(k) == Some(mercury))
                };
                if with(&malefics) && !with(&benefics) {
                    benefics.retain(|k| *k != "MERCURY");
                    malefics.push("MERCURY");
                }
            }
        }
        Eval {
            bodies,
            reading,
            benefics,
            malefics,
            lagna: sign("LAGNA").unwrap_or_default(),
        }
    }

    fn body(&self, key: &str) -> Option<&Body> {
        self.bodies.iter().find(|b| b.key == key)
    }

    fn house(&self, body: &Body) -> u64 {
        if self.reading.flips(Fork::WholeSignHouses) {
            house_from(body.sign, self.lagna)
        } else {
            body.house
        }
    }

    fn house_of(&self, key: &str) -> Option<u64> {
        self.body(key).map(|b| self.house(b))
    }

    fn sign_of(&self, key: &str) -> Option<u64> {
        self.body(key).map(|b| b.sign)
    }

    fn lord_of(&self, house: u64) -> &'static str {
        let sign = usize::try_from((self.lagna + house.saturating_sub(1)) % 12).unwrap_or(0);
        LORDS.get(sign).copied().unwrap_or("MARS")
    }

    fn dignity_meets(&self, dignity: &str, wanted: &[String]) -> bool {
        wanted.iter().any(|w| w == dignity)
            || (!self.reading.flips(Fork::PlainDignity)
                && match dignity {
                    "DEEP_EXALTED" => wanted.iter().any(|w| w == "EXALTED"),
                    "DEEP_DEBILITATED" => wanted.iter().any(|w| w == "DEBILITATED"),
                    _ => false,
                })
    }

    /// Whether a planet in `from` aspects `target` by graha drishti.
    fn aspects(planet: &str, from: u64, target: u64) -> bool {
        let house = house_from(target, from);
        house == 7
            || match planet {
                "MARS" => house == 4 || house == 8,
                "JUPITER" => house == 5 || house == 9,
                "SATURN" => house == 3 || house == 10,
                _ => false,
            }
    }

    /// Whether a condition holds, adding the planets it consulted.
    #[allow(clippy::too_many_lines, reason = "one arm a predicate of the language")]
    fn holds(&self, c: &Value, involved: &mut Vec<String>) -> Result<bool, String> {
        let planet = c["planet"].as_str().unwrap_or_default();
        let push = |key: &str, involved: &mut Vec<String>| involved.push(key.to_owned());
        let held = match c["type"].as_str().unwrap_or_default() {
            "and" => {
                let mut all = true;
                for sub in c["conditions"].as_array().into_iter().flatten() {
                    if !self.branch(sub, involved)? {
                        all = false;
                        break;
                    }
                }
                all
            }
            "or" => {
                let mut any = false;
                for sub in c["conditions"].as_array().into_iter().flatten() {
                    if self.branch(sub, involved)? {
                        any = true;
                        break;
                    }
                }
                any
            }
            "not" => {
                let mut inner = Vec::new();
                let held = self.holds(&c["condition"], &mut inner)?;
                if !self.reading.flips(Fork::DecidingBranchOnly) {
                    involved.extend(inner);
                }
                !held
            }
            "planet-in-house" | "planet-in-house-from" => {
                let houses = numbers(&c["houses"]);
                let from = match c["reference"].as_str() {
                    Some(reference) => match self.sign_of(reference) {
                        Some(sign) => Some(sign),
                        None => return Ok(false),
                    },
                    None => None,
                };
                let house_of = |body: &Body| match from {
                    Some(sign) => house_from(body.sign, sign),
                    None => self.house(body),
                };
                let set = match planet {
                    "any-benefic" => Some(&self.benefics),
                    "any-malefic" => Some(&self.malefics),
                    _ => None,
                };
                match set {
                    Some(set) => {
                        let found = self
                            .bodies
                            .iter()
                            .find(|b| set.contains(&b.key) && houses.contains(&house_of(b)));
                        if let Some(b) = found {
                            push(b.key, involved);
                        }
                        found.is_some()
                    }
                    None => match self.body(planet) {
                        Some(b) if houses.contains(&house_of(b)) => {
                            push(planet, involved);
                            true
                        }
                        _ => false,
                    },
                }
            }
            "planet-in-sign" => {
                let signs = strings(&c["signs"]);
                match self.sign_of(planet) {
                    Some(sign)
                        if signs.iter().any(|s| {
                            SIGNS.get(usize::try_from(sign).unwrap_or(0)) == Some(&s.as_str())
                        }) =>
                    {
                        push(planet, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-dignity" => match self.body(planet) {
                Some(b) if self.dignity_meets(&b.dignity, &strings(&c["dignities"])) => {
                    push(planet, involved);
                    true
                }
                _ => false,
            },
            kind @ ("planet-in-kendra" | "planet-in-trikona") => {
                let set: &[u64] = if kind == "planet-in-kendra" {
                    &KENDRA
                } else {
                    &TRIKONA
                };
                match self.house_of(planet) {
                    Some(h) if set.contains(&h) => {
                        push(planet, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-in-kendra-from" => {
                let reference = c["reference"].as_str().unwrap_or_default();
                match (self.sign_of(planet), self.sign_of(reference)) {
                    (Some(p), Some(r)) if KENDRA.contains(&house_from(p, r)) => {
                        push(planet, involved);
                        push(reference, involved);
                        true
                    }
                    _ => false,
                }
            }
            kind @ ("lord-of-house-in-kendra" | "lord-of-house-in-house") => {
                let lord = self.lord_of(c["houseRuled"].as_u64().unwrap_or(1));
                let wanted = c["houseOccupied"].as_u64();
                match self.house_of(lord) {
                    Some(h)
                        if (kind == "lord-of-house-in-kendra" && KENDRA.contains(&h))
                            || (kind == "lord-of-house-in-house" && Some(h) == wanted) =>
                    {
                        push(lord, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-conjunct" => {
                let planets = strings(&c["planets"]);
                let orb = c["maxOrb"]
                    .as_f64()
                    .filter(|o| *o > 0.0)
                    .or(self.reading.flips(Fork::OrbConjunction).then_some(ORB_DEG));
                let held = if let Some(orb) = orb {
                    let longitudes: Option<Vec<f64>> = planets
                        .iter()
                        .map(|p| self.body(p).map(|b| b.longitude))
                        .collect();
                    longitudes.is_some_and(|l| {
                        l.iter().enumerate().all(|(i, a)| {
                            l.iter().skip(i + 1).all(|b| {
                                let d = (a - b).rem_euclid(360.0);
                                d.min(360.0 - d) <= orb
                            })
                        })
                    })
                } else {
                    let signs: Option<Vec<u64>> = planets.iter().map(|p| self.sign_of(p)).collect();
                    signs.is_some_and(|s| s.windows(2).all(|w| w[0] == w[1]))
                };
                if held {
                    for p in &planets {
                        push(p, involved);
                    }
                }
                held
            }
            "no-planet-in-houses-from" => {
                let reference = c["reference"].as_str().unwrap_or_default();
                let houses = numbers(&c["houses"]);
                let except = strings(&c["except"]);
                match self.sign_of(reference) {
                    Some(from) => !self.bodies.iter().any(|b| {
                        b.key != reference
                            && b.key != "LAGNA"
                            && !except.iter().any(|e| e == b.key)
                            && houses.contains(&house_from(b.sign, from))
                    }),
                    None => false,
                }
            }
            "mutual-exchange" => {
                let (h1, h2) = (
                    c["house1"].as_u64().unwrap_or(1),
                    c["house2"].as_u64().unwrap_or(1),
                );
                let (l1, l2) = (self.lord_of(h1), self.lord_of(h2));
                if self.house_of(l1) == Some(h2) && self.house_of(l2) == Some(h1) {
                    push(l1, involved);
                    push(l2, involved);
                    true
                } else {
                    false
                }
            }
            "lord-conjunct-lord" => {
                let (l1, l2) = (
                    self.lord_of(c["house1"].as_u64().unwrap_or(1)),
                    self.lord_of(c["house2"].as_u64().unwrap_or(1)),
                );
                if l1 == l2 {
                    push(l1, involved);
                    true
                } else if self.sign_of(l1).is_some() && self.sign_of(l1) == self.sign_of(l2) {
                    push(l1, involved);
                    push(l2, involved);
                    true
                } else {
                    false
                }
            }
            "all-planets-between-nodes" => {
                let (rahu, ketu) = (
                    self.sign_of("RAHU").unwrap_or(0),
                    self.sign_of("KETU").unwrap_or(6),
                );
                let arc = (ketu + 12 - rahu) % 12;
                let (mut rahu_side, mut ketu_side) = (0, 0);
                let mut broken = false;
                for key in SEVEN {
                    let from = (self.sign_of(key).unwrap_or(0) + 12 - rahu) % 12;
                    if from == 0 || from == arc {
                        broken = true;
                        break;
                    }
                    if from < arc {
                        rahu_side += 1;
                    } else {
                        ketu_side += 1;
                    }
                }
                let held = !broken
                    && if self.reading.flips(Fork::RahuToKetuOnly) {
                        ketu_side == 0
                    } else {
                        rahu_side == 0 || ketu_side == 0
                    };
                if held {
                    for key in SEVEN {
                        push(key, involved);
                    }
                }
                held
            }
            "occupied-sign-count" => {
                let planets = strings(&c["planets"]);
                let signs: BTreeSet<u64> = planets.iter().filter_map(|p| self.sign_of(p)).collect();
                let held = u64::try_from(signs.len()).ok() == c["count"].as_u64();
                if held {
                    for p in &planets {
                        push(p, involved);
                    }
                }
                held
            }
            "all-classical-grahas-in-houses" => {
                let allowed = numbers(&c["houses"]);
                let houses: Option<BTreeSet<u64>> = SEVEN
                    .iter()
                    .map(|k| self.house_of(k).filter(|h| allowed.contains(h)))
                    .collect();
                let every = c["requireAllHousesFilled"].as_bool().unwrap_or(true);
                let distinct: BTreeSet<u64> = allowed.iter().copied().collect();
                let held = houses.is_some_and(|h| !every || h.len() == distinct.len());
                if held {
                    for key in SEVEN {
                        push(key, involved);
                    }
                }
                held
            }
            "n-grahas-conjunct-with" => {
                match self.sign_of(c["anchor"].as_str().unwrap_or_default()) {
                    Some(anchor) => {
                        let cluster: Vec<&str> = SEVEN
                            .iter()
                            .copied()
                            .filter(|k| self.sign_of(k) == Some(anchor))
                            .collect();
                        let held = u64::try_from(cluster.len()).unwrap_or(0)
                            >= c["minCount"].as_u64().unwrap_or(u64::MAX);
                        if held {
                            for k in cluster {
                                push(k, involved);
                            }
                        }
                        held
                    }
                    None => false,
                }
            }
            "chara-karaka-in-house" => {
                let karaka = c["karaka"].as_str().unwrap_or_default();
                let eight = c["karakaScheme"].as_u64() == Some(8);
                let houses = numbers(&c["houses"]);
                let holder = self.bodies.iter().find(|b| {
                    let k = if eight { &b.karaka8 } else { &b.karaka7 };
                    k.as_deref() == Some(karaka)
                });
                match holder {
                    Some(b) if houses.contains(&self.house(b)) => {
                        push(b.key, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-combust" => match self.body(planet) {
                Some(b) if b.combust => {
                    push(planet, involved);
                    true
                }
                _ => false,
            },
            "planet-retrograde" => {
                let node = planet == "RAHU" || planet == "KETU";
                match self.body(planet) {
                    Some(b)
                        if (b.retrograde && !node)
                            || (node && self.reading.flips(Fork::NodesRetrograde)) =>
                    {
                        push(planet, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-aspects-planet" => {
                let (from, target) = (
                    c["from"].as_str().unwrap_or_default(),
                    c["target"].as_str().unwrap_or_default(),
                );
                match (self.sign_of(from), self.sign_of(target)) {
                    (Some(f), Some(t)) if from != "LAGNA" && Self::aspects(from, f, t) => {
                        push(from, involved);
                        push(target, involved);
                        true
                    }
                    _ => false,
                }
            }
            "planet-aspects-house" => {
                let from = c["from"].as_str().unwrap_or_default();
                let target =
                    (self.lagna + c["houseRuled"].as_u64().unwrap_or(1).saturating_sub(1)) % 12;
                match self.sign_of(from) {
                    Some(f) if from != "LAGNA" && Self::aspects(from, f, target) => {
                        push(from, involved);
                        true
                    }
                    _ => false,
                }
            }
            other => return Err(format!("a condition this pass does not read: `{other}`")),
        };
        Ok(held)
    }

    /// A sub-condition of `and` or `or`: under the engine's reading its
    /// planets are added whether or not it held; else only when it did.
    fn branch(&self, sub: &Value, involved: &mut Vec<String>) -> Result<bool, String> {
        if !self.reading.flips(Fork::DecidingBranchOnly) {
            return self.holds(sub, involved);
        }
        let mut own = Vec::new();
        let held = self.holds(sub, &mut own)?;
        if held {
            involved.extend(own);
        }
        Ok(held)
    }
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

fn tally(rules: &[Rule], records: &[Record], reading: Reading) -> Result<Tally, String> {
    let mut t = Tally::default();
    for record in records {
        let eval = Eval::new(&record.bodies, reading);
        for rule in rules.iter().filter(|r| !r.conditions.is_empty()) {
            t.decisions += 1;
            let mut involved = Vec::new();
            let mut present = true;
            for c in &rule.conditions {
                if !eval.holds(c, &mut involved)? {
                    present = false;
                    break;
                }
            }
            let recorded = record.present.get(&rule.key);
            if present != recorded.is_some() {
                t.decisions_wrong += 1;
                t.moved.insert(rule.key.clone());
                continue;
            }
            let Some((planets, cancellations)) = recorded else {
                continue;
            };
            t.presences += 1;
            let mut unique: Vec<String> = Vec::new();
            for p in involved {
                if !unique.contains(&p) {
                    unique.push(p);
                }
            }
            if &unique != planets {
                t.planets_wrong += 1;
                t.moved.insert(rule.key.clone());
            }
            let mut fired = Vec::new();
            for c in &rule.cancellations {
                if eval.holds(c, &mut Vec::new())? {
                    fired.push(c["type"].as_str().unwrap_or_default().to_owned());
                }
            }
            if &fired != cancellations {
                t.cancellations_wrong += 1;
                t.moved.insert(rule.key.clone());
            }
        }
    }
    Ok(t)
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
fn predicate_uses(rules: &[Rule]) -> BTreeMap<String, usize> {
    fn walk(c: &Value, into: &mut BTreeMap<String, usize>) {
        *into
            .entry(c["type"].as_str().unwrap_or_default().to_owned())
            .or_default() += 1;
        for sub in c["conditions"].as_array().into_iter().flatten() {
            walk(sub, into);
        }
        if c.get("condition").is_some() {
            walk(&c["condition"], into);
        }
    }
    let mut uses = BTreeMap::new();
    for rule in rules {
        for c in rule.conditions.iter().chain(&rule.cancellations) {
            walk(c, &mut uses);
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

    let readings = core::iter::once((
        "the engine's reading, every choice below as the engine makes it",
        ENGINE,
    ))
    .chain(
        FORKS
            .iter()
            .map(|(fork, name)| (*name, Reading(Some(*fork)))),
    );
    let mut claims = Vec::new();
    let mut engine = Tally::default();
    for (name, reading) in readings {
        let t = tally(&rules, &records, reading)?;
        claims.push(claim(name, &t, reading == ENGINE));
        if reading == ENGINE {
            engine = t;
        }
    }

    let coverage = Coverage::of(&rules, &records);
    let predicates = predicate_uses(&rules);
    let mut by_use: Vec<(&String, &usize)> = predicates.iter().collect();
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
