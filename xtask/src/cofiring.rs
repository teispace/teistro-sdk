//! Rules that hold on exactly the same charts: probable duplicates
//! (`05-testing/01-quality-bar.md`, "rule fixtures").
//!
//! Every shipped rule is evaluated over a sky the SDK computes: charts
//! spread over two centuries at six places, no two sharing an instant.
//! Rules that hold on the same charts, and on enough of them that chance
//! is not the reason, form a group, and a group has one of three
//! explanations:
//!
//! - **two texts state one condition**: its rules cite different texts,
//!   which is agreement between the sources and the reason the SDK ships
//!   both;
//! - **one aspect read from both grahas**: two rules of one text place
//!   each graha in the sign seventh from the other's, under the other's
//!   aspect, and both cast only the seventh, so each holds exactly where
//!   the other does;
//! - **one text says it twice**: listed in [`REVIEWED`] with where.
//!
//! Anything else is a rule copied where it should have differed, and the
//! gate fails naming it; so does a [`REVIEWED`] entry that no longer
//! co-fires. The pass computes each chart's whole reading, its panchanga
//! included, so it runs in verify rather than fast-check.
//!
//! `cargo xtask cofiring` writes the page; `check-cofiring` regenerates it
//! in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro::rules::{Body, BodyRef, BodySubject, Condition, House, Rule, SignRef};
use teistro::{ChartRequest, Context, Ephemeris, RuleRequest, ShippedRules, UtcOffset};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};

use crate::generated::{Output, check, write};
use crate::measure::count;

const PAGE: &str = "docs/03-design/rule-cofiring-measured.md";

/// How many places.
const PLACE_COUNT: u32 = 6;

/// The latitudes and longitudes the sky is computed at: Kathmandu, the
/// equator, the tropics, two temperate cities and the southern hemisphere.
const PLACES: [(f64, f64); PLACE_COUNT as usize] = [
    (27.7172, 85.324),
    (0.0, -78.5),
    (13.08, 80.27),
    (40.71, -74.0),
    (51.5, -0.13),
    (-33.87, 151.21),
];

/// Charts at each place.
const PER_PLACE: u32 = 1000;

/// The first instant, 1900-01-01 0h UT, and the span the charts cover.
const FIRST_JD: f64 = 2_415_020.5;
const SPAN_DAYS: f64 = 73_049.0;

/// The fewest charts a group must hold on to be listed. Below it, nested
/// conjunctions of five and six grahas (each holding on a chart or two of
/// the sky) meet by chance; every group above it is an equivalence.
const SUPPORT: usize = 5;

/// One text stating one condition twice, each with where it does.
const REVIEWED: [(&[&str], &str); 3] = [
    (
        &[
            "BPHS_ARMY_CHIEF_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA",
            "BPHS_KING_BY_MALEFICS_THIRD_AND_SIXTH_FROM_ATMAKARAKA",
        ],
        "BPHS gives malefics third and sixth from the Atmakaraka twice: ch. 39 v. 10 makes the \
         native a king, ch. 40 v. 7 an army chief",
    ),
    (
        &[
            "SARAVALI_MERCURY_AND_JUPITER_IN_THE_SEVENTH",
            "SARAVALI_MERCURY_JUPITER_IN_BHAVA_7",
        ],
        "Saravali reads Mercury and Jupiter in the seventh in ch. 34 v. 47 (one wife), and again \
         among two grahas in an angle",
    ),
    (
        &[
            "SARAVALI_MERCURY_AND_VENUS_IN_THE_SEVENTH",
            "SARAVALI_MERCURY_VENUS_IN_BHAVA_7",
        ],
        "Saravali reads Mercury and Venus in the seventh in ch. 34, and again among two grahas \
         in an angle",
    ),
];

/// The instants of the `place`-th place: evenly through the span, each
/// place offset by a fraction of a step so no two places share a sky, and
/// an hour of the day that walks the golden ratio so the lagna visits
/// every sign.
fn instants(place: u32) -> Vec<JulianDay<Utc>> {
    const GOLDEN: f64 = 0.618_033_988_749_894_9;
    let step = SPAN_DAYS / f64::from(PER_PLACE);
    let offset = f64::from(place) * step / f64::from(PLACE_COUNT);
    (0..PER_PLACE)
        .map(|at| {
            let at = f64::from(at);
            let day = (at * step + offset).floor();
            JulianDay::<Utc>::literal(FIRST_JD + day + (at * GOLDEN).fract())
        })
        .collect()
}

/// What the sky gave: the shipped rules and, for each that held at least
/// once, the charts it held on.
struct Sky {
    rules: Vec<Rule>,
    held: BTreeMap<String, Vec<u32>>,
}

fn sky() -> Result<Sky, String> {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|e| e.to_string())?;
    let set = RuleRequest::shipped(ShippedRules::ALL)
        .rule_set()
        .map_err(|e| e.to_string())?;
    let mut held: BTreeMap<String, Vec<u32>> = BTreeMap::new();
    let mut chart = 0_u32;
    for ((latitude, longitude), at) in PLACES.into_iter().zip(0..) {
        let place = Place::new(
            Latitude::try_new(latitude).map_err(|e| e.to_string())?,
            Longitude::try_new(longitude).map_err(|e| e.to_string())?,
            Altitude::try_new(0.0).map_err(|e| e.to_string())?,
        );
        let request = ChartRequest::at(place, UtcOffset::UTC);
        let read = sdk
            .chart()
            .readings_with_rules(&instants(at), &request, &set)
            .map_err(|e| e.to_string())?
            .value;
        for (_, reading) in &read {
            for present in &reading.present {
                held.entry(present.rule.key.clone())
                    .or_default()
                    .push(chart);
            }
            chart += 1;
        }
    }
    Ok(Sky {
        rules: set.rules().to_vec(),
        held,
    })
}

/// Why a group co-fires, or that nothing explains it; a group with several
/// explanations is filed under the last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Why {
    Texts,
    Mutual,
    Reviewed(usize),
    Unexplained,
}

/// `(graha, sign, aspecting graha)` when a rule is exactly "the graha in
/// the sign under that graha's aspect".
fn aspected_in_sign(rule: &Rule) -> Option<(Graha, Rashi, Graha)> {
    let [
        Condition::PlanetInSign {
            planet: SignRef::Of(BodyRef::Body(Body::Graha(graha))),
            signs,
        },
        Condition::PlanetAspectsPlanet {
            from: BodySubject::Ref(BodyRef::Body(Body::Graha(from))),
            target: SignRef::Of(BodyRef::Body(Body::Graha(target))),
        },
    ] = rule.conditions.as_slice()
    else {
        return None;
    };
    let [sign] = signs.as_slice() else {
        return None;
    };
    (graha == target).then_some((*graha, *sign, *from))
}

/// Whether two rules of one text are one aspect read from both grahas:
/// each in the sign seventh from the other's, under the other's aspect,
/// both casting only the seventh (the Sun, the Moon, Mercury and Venus).
fn mutual(one: &Rule, two: &Rule) -> bool {
    const SEVENTH_ONLY: [Graha; 4] = [Graha::Sun, Graha::Moon, Graha::Mercury, Graha::Venus];
    let (Some((a, sign_a, by_a)), Some((b, sign_b, by_b))) =
        (aspected_in_sign(one), aspected_in_sign(two))
    else {
        return false;
    };
    by_a == b
        && by_b == a
        && SEVENTH_ONLY.contains(&a)
        && SEVENTH_ONLY.contains(&b)
        && House::between(sign_a, sign_b).get() == 7
}

/// Why `keys` co-fire: within each text the rules must be one rule, a
/// mutual pair or a reviewed entry.
fn why(keys: &[&str], rules: &BTreeMap<&str, &Rule>) -> Why {
    let mut by_text: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for key in keys {
        let text = rules.get(key).map_or("", |rule| rule.source.text.as_str());
        by_text.entry(text).or_default().push(key);
    }
    let mut verdict = Why::Texts;
    for same in by_text.values().filter(|same| same.len() > 1) {
        let paired = match same.as_slice() {
            [one, two] => rules
                .get(one)
                .zip(rules.get(two))
                .is_some_and(|(a, b)| mutual(a, b)),
            _ => false,
        };
        let found = if paired {
            Why::Mutual
        } else {
            REVIEWED
                .iter()
                .position(|(listed, _)| listed.iter().copied().eq(same.iter().copied()))
                .map_or(Why::Unexplained, Why::Reviewed)
        };
        verdict = verdict.max(found);
    }
    verdict
}

/// One listed group: how many charts it holds on, its rules and why.
struct Group<'a> {
    support: usize,
    keys: Vec<&'a str>,
    why: Why,
}

/// The groups at or above [`SUPPORT`], most charts first within each
/// explanation, and how many fell below it.
fn groups(sky: &Sky) -> (Vec<Group<'_>>, usize) {
    let rules: BTreeMap<&str, &Rule> = sky.rules.iter().map(|r| (r.key.as_str(), r)).collect();
    let mut by_charts: BTreeMap<&[u32], Vec<&str>> = BTreeMap::new();
    for (key, charts) in &sky.held {
        by_charts
            .entry(charts.as_slice())
            .or_default()
            .push(key.as_str());
    }
    let mut chance = 0;
    let mut listed = Vec::new();
    for (charts, keys) in by_charts {
        if keys.len() < 2 {
            continue;
        }
        if charts.len() < SUPPORT {
            chance += 1;
            continue;
        }
        let why = why(&keys, &rules);
        listed.push(Group {
            support: charts.len(),
            keys,
            why,
        });
    }
    listed.sort_by(|a, b| (a.why, b.support, &a.keys).cmp(&(b.why, a.support, &b.keys)));
    (listed, chance)
}

/// The table rows of the groups `keep` admits, with the reviewed reason
/// when `reason` is asked.
fn rows(groups: &[Group<'_>], keep: impl Fn(Why) -> bool, reason: bool) -> String {
    let mut out = String::new();
    for group in groups.iter().filter(|g| keep(g.why)) {
        let keys: Vec<String> = group.keys.iter().map(|k| format!("`{k}`")).collect();
        let _ = write!(out, "| {} | {} |", group.support, keys.join("<br>"));
        if reason {
            let said = match group.why {
                Why::Reviewed(at) => REVIEWED.get(at).map_or("", |(_, why)| why),
                _ => "**unexplained**",
            };
            let _ = write!(out, " {said} |");
        }
        out.push('\n');
    }
    out
}

/// The rules written in the condition language that held on no chart, a
/// row to a category.
fn never(sky: &Sky) -> String {
    let mut by_category: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for rule in sky
        .rules
        .iter()
        .filter(|r| !r.conditions.is_empty() && !sky.held.contains_key(&r.key))
    {
        by_category
            .entry(rule.category.as_str())
            .or_default()
            .push(format!("`{}`", rule.key));
    }
    let mut out = String::new();
    for (category, keys) in &by_category {
        let _ = writeln!(out, "| {category} | {} |", keys.join("<br>"));
    }
    out
}

/// Refuses a reviewed entry that no longer co-fires and a group nothing
/// explains.
fn refusals(groups: &[Group<'_>]) -> Result<(), String> {
    for (at, (keys, _)) in REVIEWED.iter().enumerate() {
        if !groups.iter().any(|g| g.why == Why::Reviewed(at)) {
            return Err(format!(
                "the reviewed group of `{}` no longer co-fires; remove it from REVIEWED in \
                 xtask/src/cofiring.rs",
                keys.first().copied().unwrap_or_default()
            ));
        }
    }
    let unexplained = groups.iter().filter(|g| g.why == Why::Unexplained).count();
    if unexplained > 0 {
        return Err(format!(
            "{unexplained} group(s) of one text co-fire with nothing to explain them, probable \
             duplicates; fix the rule, or list the group in REVIEWED with where the text says \
             it twice:\n{}",
            rows(groups, |w| w == Why::Unexplained, false)
        ));
    }
    Ok(())
}

fn page() -> Result<String, String> {
    let sky = sky()?;
    let (groups, chance) = groups(&sky);
    refusals(&groups)?;
    let of = |keep: &dyn Fn(Why) -> bool| groups.iter().filter(|g| keep(g.why)).count();
    let charts = PER_PLACE * PLACE_COUNT;
    Ok(format!(
        "# Rules that hold together, measured\n\n\
         Status: `generated` by `cargo xtask cofiring`; `check-cofiring` regenerates it in verify \
         and fails on any difference.\n\n\
         Two rules that hold on exactly the same charts are a duplicate unless something says \
         why ([`01-quality-bar.md`](../05-testing/01-quality-bar.md), \"rule fixtures\"). The \
         pass evaluates every shipped rule, {rules} of them, on {charts} charts the SDK \
         computes: {per} at each of {places} places from Quito to Sydney, from 1900 to 2100, no \
         two at one instant. {held} rules hold at least once and {never} never do.\n\n\
         Rules holding on the same {support} or more charts form {total} groups, and each is \
         explained. Below that, {chance} groups are nested conjunctions of five and six grahas, \
         each held on a chart or two, which is chance.\n\n\
         - **{texts} groups: two texts state one condition.** The rules cite different texts, \
         so the group is agreement between sources, and each rule carries its own reading.\n\
         - **{mutual} groups: one aspect read from both grahas.** Saravali reads a graha in \
         each sign under each aspect, a chapter to a graha, and two grahas that cast only the \
         seventh aspect each other exactly when each stands seventh from the other.\n\
         - **{reviewed} groups: one text says it twice**, where the table below says.\n\n\
         A group of one text that is none of these fails the gate, and so does a listed group \
         that no longer holds together.\n\n\
         ## One text, twice\n\n\
         | charts | rules | where |\n|---:|---|---|\n{reviewed_rows}\n\
         ## One aspect from both grahas\n\n\
         | charts | rules |\n|---:|---|\n{mutual_rows}\n\
         ## Two texts, one condition\n\n\
         | charts | rules |\n|---:|---|\n{texts_rows}\n\
         ## Never held\n\n\
         A rule that holds on none of the charts is not a duplicate of anything, but it is \
         either rare or dead, and these are the ones to read first. {computed} of the {never} \
         are computed in code rather than written as conditions, and this evaluation does not \
         see them; the rest follow.\n\n\
         | category | rules |\n|---|---|\n{never_rows}",
        rules = count(sky.rules.len()),
        charts = count(charts as usize),
        per = count(PER_PLACE as usize),
        places = PLACE_COUNT,
        held = count(sky.held.len()),
        never = count(sky.rules.len() - sky.held.len()),
        support = SUPPORT,
        total = groups.len(),
        texts = of(&|w| w == Why::Texts),
        mutual = of(&|w| w == Why::Mutual),
        reviewed = of(&|w| matches!(w, Why::Reviewed(_))),
        reviewed_rows = rows(&groups, |w| matches!(w, Why::Reviewed(_)), true),
        mutual_rows = rows(&groups, |w| w == Why::Mutual, false),
        texts_rows = rows(&groups, |w| w == Why::Texts, false),
        never_rows = never(&sky),
        computed = sky
            .rules
            .iter()
            .filter(|r| r.conditions.is_empty() && !sky.held.contains_key(&r.key))
            .count(),
    ))
}

fn outputs() -> Result<[Output; 1], String> {
    Ok([Output::new(PAGE, page()?)])
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => write(root, &outputs),
        Err(error) => {
            eprintln!("cofiring: {error}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs() {
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask cofiring") != 0),
        Err(error) => {
            eprintln!("cofiring: {error}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    /// A rule of `text` reading `graha` in `sign` under `by`'s aspect.
    fn aspected(key: &str, text: &str, graha: &str, sign: &str, by: &str) -> Rule {
        serde_json::from_value(serde_json::json!({
            "key": key,
            "category": "test",
            "source": {"text": text},
            "conditions": [
                {"type": "planet-in-sign", "planet": graha, "signs": [sign]},
                {"type": "planet-aspects-planet", "from": by, "target": graha},
            ],
            "outcomes": [],
        }))
        .unwrap()
    }

    /// A mutual pair is recognised only for grahas casting the seventh
    /// alone, standing seventh from each other; anything else of one text
    /// is unexplained unless reviewed.
    #[test]
    fn a_group_of_one_text_is_explained_or_refused() {
        let rules = [
            aspected("A", "Saravali", "MOON", "CANCER", "MERCURY"),
            aspected("B", "Saravali", "MERCURY", "CAPRICORN", "MOON"),
            aspected("C", "Saravali", "MERCURY", "ARIES", "MOON"),
            aspected("D", "Saravali", "MARS", "CAPRICORN", "MOON"),
            aspected("E", "Phaladeepika", "MOON", "CANCER", "MERCURY"),
        ];
        let rules: BTreeMap<&str, &Rule> = rules.iter().map(|r| (r.key.as_str(), r)).collect();
        assert_eq!(why(&["A", "B"], &rules), Why::Mutual);
        assert_eq!(why(&["A", "B", "E"], &rules), Why::Mutual);
        assert_eq!(why(&["A", "E"], &rules), Why::Texts);
        assert_eq!(why(&["A", "C"], &rules), Why::Unexplained);
        assert_eq!(why(&["A", "D"], &rules), Why::Unexplained);
    }
}
