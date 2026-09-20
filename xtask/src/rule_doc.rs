//! The pass over the prose a rule renders to (`03-design/rule-doc.md`).
//!
//! No text records the English of a rule, so nothing here is derived from a
//! recording. What the shipped packs and the corpus's own rules can falsify is
//! **collision**: two structurally different conditions that read alike, which
//! is a change a reviewer cannot see. The pass renders every rule the kernel
//! holds, groups the renderings, and names any two that read the same; zero is
//! the gate.
//!
//! Two spellings of one meaning are not a collision: the language says a
//! chara karaka's house in two ways, and a combinator with one condition in
//! it says what that condition says. Those identities are named in
//! [`plainly`] and the renderings they share are reported rather than failed.
//!
//! Beside the collisions it reports what the reviewer argues about — one
//! rendering for each of the language's kinds, the two kinds no pack holds,
//! and where the prose is longest and deepest.
//!
//! `cargo xtask rule-doc` writes the page and `check-rule-doc` regenerates it
//! in memory and fails on any difference. `cargo xtask rule-doc <what>` prints
//! the passages themselves: a pack, a category or one rule's key.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_rules::reference::{BodyRef, SignRef, Subject};
use teistro_rules::{Condition, Rule, shipped};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};

const PAGE: &str = "docs/03-design/rule-doc-measured.md";
const CORPUS: [&str; 2] = [
    "fixtures/baseline/yogas/rules.json",
    "fixtures/baseline/doshas/rules.json",
];

/// The shipped packs, in the order the page reports them.
fn packs() -> Vec<(&'static str, &'static [Rule])> {
    vec![
        ("nabhasas", shipped::nabhasas()),
        ("arishtas", shipped::arishtas()),
        ("gandantas", shipped::gandantas()),
        ("readings", shipped::readings()),
        ("doshas", shipped::computed_doshas()),
        ("yogas", shipped::computed_yogas()),
    ]
}

/// The corpus's own rules, which exercise eleven predicates no shipped pack
/// uses.
fn corpus(root: &Path) -> Result<Vec<Rule>, String> {
    #[derive(serde::Deserialize)]
    struct File {
        rules: Vec<Rule>,
    }
    let mut out = Vec::new();
    for path in CORPUS {
        let text =
            std::fs::read_to_string(root.join(path)).map_err(|err| format!("{path}: {err}"))?;
        let file: File = serde_json::from_str(&text).map_err(|err| format!("{path}: {err}"))?;
        out.extend(file.rules);
    }
    Ok(out)
}

/// The same condition with the language's two redundancies spelled one way:
/// a combinator holding one condition **is** that condition, and
/// `chara-karaka-in-house` **is** `planet-in-house` of that karaka — the
/// evaluator resolves the karaka and reads the same house for both. Prose
/// renders meaning, so two spellings of one meaning should read alike; it is
/// two *meanings* reading alike that would hide a change.
fn plainly(condition: &Condition) -> Condition {
    match condition {
        Condition::And { conditions } | Condition::Or { conditions } if conditions.len() == 1 => {
            conditions
                .first()
                .map_or_else(|| condition.clone(), plainly)
        }
        Condition::And { conditions } => Condition::And {
            conditions: conditions.iter().map(plainly).collect(),
        },
        Condition::Or { conditions } => Condition::Or {
            conditions: conditions.iter().map(plainly).collect(),
        },
        Condition::Not { condition } => Condition::Not {
            condition: Box::new(plainly(condition)),
        },
        Condition::ForAny { planets, then } => Condition::ForAny {
            planets: planets.clone(),
            then: Box::new(plainly(then)),
        },
        Condition::CountOf {
            planets,
            then,
            at_least,
            at_most,
        } => Condition::CountOf {
            planets: planets.clone(),
            then: Box::new(plainly(then)),
            at_least: *at_least,
            at_most: *at_most,
        },
        Condition::InVarga { varga, condition } => Condition::InVarga {
            varga: *varga,
            condition: Box::new(plainly(condition)),
        },
        Condition::CharaKarakaInHouse {
            karaka,
            houses,
            karaka_scheme,
        } => Condition::PlanetInHouse {
            planet: Subject::Ref(SignRef::Of(BodyRef::Karaka {
                karaka: *karaka,
                scheme: *karaka_scheme,
            })),
            houses: houses.clone(),
        },
        other => other.clone(),
    }
}

/// Every condition of a rule, its groups and its cancellations, and every
/// condition inside those.
fn conditions(rule: &Rule) -> impl Iterator<Item = &Condition> {
    rule.conditions
        .iter()
        .chain(rule.groups.iter().flat_map(|group| group.conditions.iter()))
        .chain(
            rule.cancellations
                .iter()
                .map(|cancellation| &cancellation.condition),
        )
        .flat_map(Condition::walk)
}

/// How deep a condition nests, a predicate being one.
fn depth(condition: &Condition) -> usize {
    1 + condition.children().iter().map(depth).max().unwrap_or(0)
}

/// A condition written as the JSON a rule carries, which is what "the same
/// condition" means here.
fn written(condition: &Condition) -> String {
    serde_json::to_string(condition).unwrap_or_else(|err| format!("unwritable: {err}"))
}

/// What the pass measured.
struct Measured {
    /// Every rule rendered, by pack.
    rules: Vec<(String, usize)>,
    /// How many conditions were rendered, how many spellings they have, how
    /// many meanings those spellings carry, and how many sentences.
    conditions: usize,
    spellings: usize,
    sentences: usize,
    /// Renderings shared by conditions that differ in meaning: the gate.
    collisions: Vec<(String, Vec<String>)>,
    /// Renderings shared by two spellings of one meaning: a finding about the
    /// language, not about the prose.
    synonyms: Vec<(String, Vec<String>)>,
    /// How many conditions differ once the two identities are spelled one way.
    meanings: usize,
    /// Each kind: how often it occurs, and the shortest rendering of it.
    kinds: BTreeMap<&'static str, (usize, String)>,
    /// The rules whose passage is longest and shortest, and the deepest nest.
    longest: (String, usize),
    shortest: (String, usize),
    deepest: (String, usize),
}

fn measure(root: &Path) -> Result<Measured, String> {
    let corpus = corpus(root)?;
    let mut sets: Vec<(String, Vec<&Rule>)> = packs()
        .into_iter()
        .map(|(name, rules)| (String::from(name), rules.iter().collect()))
        .collect();
    sets.push((String::from("the corpus's own"), corpus.iter().collect()));

    let mut by_rendering: BTreeMap<String, (Vec<String>, Vec<String>)> = BTreeMap::new();
    let mut meanings: BTreeSet<String> = BTreeSet::new();
    let mut spellings: BTreeSet<String> = BTreeSet::new();
    let mut kinds: BTreeMap<&'static str, (usize, String)> = BTreeMap::new();
    let mut conditions_seen = 0_usize;
    let mut longest = (String::new(), 0);
    let mut shortest = (String::new(), usize::MAX);
    let mut deepest = (String::new(), 0);

    for (_, rules) in &sets {
        for rule in rules {
            let passage = rule.to_string();
            let lines = passage.lines().count();
            if lines > longest.1 {
                longest = (rule.key.clone(), lines);
            }
            if lines < shortest.1 {
                shortest = (rule.key.clone(), lines);
            }
            for condition in conditions(rule) {
                conditions_seen += 1;
                let rendering = condition.to_string();
                let entry = by_rendering.entry(rendering.clone()).or_default();
                let spelling = written(condition);
                if !entry.0.contains(&spelling) {
                    entry.0.push(spelling.clone());
                }
                spellings.insert(spelling);
                let meaning = written(&plainly(condition));
                if !entry.1.contains(&meaning) {
                    entry.1.push(meaning.clone());
                }
                meanings.insert(meaning);
                let kind = kinds
                    .entry(condition.kind())
                    .or_insert_with(|| (0, rendering.clone()));
                kind.0 += 1;
                if (rendering.len(), &rendering) < (kind.1.len(), &kind.1) {
                    kind.1 = rendering;
                }
                let nest = depth(condition);
                if nest > deepest.1 {
                    deepest = (rule.key.clone(), nest);
                }
            }
        }
    }

    let collisions: Vec<(String, Vec<String>)> = by_rendering
        .iter()
        .filter(|(_, (_, meanings))| meanings.len() > 1)
        .map(|(rendering, (_, meanings))| (rendering.clone(), meanings.clone()))
        .collect();
    let synonyms: Vec<(String, Vec<String>)> = by_rendering
        .iter()
        .filter(|(_, (spellings, meanings))| spellings.len() > 1 && meanings.len() == 1)
        .map(|(rendering, (spellings, _))| (rendering.clone(), spellings.clone()))
        .collect();

    Ok(Measured {
        rules: sets
            .iter()
            .map(|(name, rules)| (name.clone(), rules.len()))
            .collect(),
        conditions: conditions_seen,
        spellings: spellings.len(),
        sentences: by_rendering.len(),
        collisions,
        synonyms,
        meanings: meanings.len(),
        kinds,
        longest,
        shortest,
        deepest,
    })
}

/// The one rule the page prints whole, so the shape of a passage is reviewed
/// and not only its sentences.
const WHOLE: [&str; 2] = ["MAHAPURUSHA_RUCHAKA", "BADHAKA_DOSHA"];

fn whole(measured_rules: &[&Rule], key: &str) -> String {
    measured_rules
        .iter()
        .find(|rule| rule.key == key)
        .map_or_else(
            || format!("{key} is not shipped any more\n"),
            std::string::ToString::to_string,
        )
}

/// A listing of the renderings two written conditions share.
fn shared(out: &mut String, sharing: &[(String, Vec<String>)]) {
    for (rendering, written) in sharing {
        let _ = writeln!(out, "- `{rendering}`");
        for one in written {
            let _ = writeln!(out, "  - `{one}`");
        }
    }
    out.push('\n');
}

/// The table a reviewer argues about: every kind of the language, how often
/// the packs use it, and the shortest sentence it renders to.
fn kinds(out: &mut String, measured: &Measured) {
    out.push_str("| kind | occurrences | as it reads |\n|---|---|---|\n");
    for kind in teistro_rules::language::KINDS {
        let (at, rendering) = measured
            .kinds
            .get(kind)
            .map_or((0, String::from("—")), |(at, rendering)| {
                (*at, rendering.clone())
            });
        let _ = writeln!(
            out,
            "| `{kind}` | {at} | {} |",
            if at == 0 {
                String::from("in no pack")
            } else {
                rendering.replace('|', "\\|")
            }
        );
    }
    out.push('\n');
}

/// The page's opening: the status line and what was rendered.
fn opening(out: &mut String, measured: &Measured) {
    let shipped_rules: usize = measured
        .rules
        .iter()
        .filter(|(name, _)| name != "the corpus's own")
        .map(|(_, at)| at)
        .sum();
    let total: usize = measured.rules.iter().map(|(_, at)| at).sum();
    out.push_str("# A rule in prose, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask rule-doc` over the shipped rule \
         packs and the conformance corpus's own rules, 2026-09-20. Do not \
         edit: `check-rule-doc` regenerates this page and fails on any \
         difference. The design it measures is \
         [`rule-doc.md`](rule-doc.md).\n\n"
    );
    out.push_str("## What was rendered\n\n");
    let counts: Vec<String> = measured
        .rules
        .iter()
        .map(|(name, at)| format!("{} in `{name}`", count(*at)))
        .collect();
    let _ = write!(
        out,
        "{}: {}. Their {} are written {} ways, which say {} things, and the \
         renderer gives those {} sentences.\n\n",
        plural(total, "rule"),
        counts.join(", "),
        plural(measured.conditions, "condition"),
        count(measured.spellings),
        count(measured.meanings),
        count(measured.sentences)
    );
    let _ = write!(
        out,
        "The {} shipped rules are what a consumer evaluates; the corpus's own \
         are rendered beside them because they exercise predicates no shipped \
         pack uses.\n\n",
        count(shipped_rules)
    );
}

fn page(root: &Path) -> Result<String, String> {
    let measured = measure(root)?;
    let every: Vec<&Rule> = packs()
        .into_iter()
        .flat_map(|(_, rules)| rules.iter())
        .collect();
    let unread: Vec<&&str> = teistro_rules::language::KINDS
        .iter()
        .filter(|kind| !measured.kinds.contains_key(**kind))
        .collect();

    let mut out = String::new();
    opening(&mut out, &measured);

    out.push_str("## What the corpus decides\n\n");
    let claims = vec![
        Claim::counted(
            "no two conditions that differ in meaning read alike",
            measured.collisions.len(),
            measured.meanings,
        ),
        Claim::counted(
            "every condition of every rule renders",
            0,
            measured.conditions,
        ),
    ];
    out.push_str(&table(&claims));
    out.push('\n');
    if measured.collisions.is_empty() {
        out.push_str(
            "No rendering is shared by two conditions that mean different \
             things, so a change to what a rule asks changes its prose.\n\n",
        );
    } else {
        out.push_str("These renderings are shared, each by the conditions beneath it:\n\n");
        shared(&mut out, &measured.collisions);
    }

    out.push_str("## Two spellings, one sentence\n\n");
    if measured.synonyms.is_empty() {
        out.push_str(
            "Every meaning in the packs is spelled one way, so no two \
             conditions share a sentence at all.\n\n",
        );
    } else {
        let _ = write!(
            out,
            "{} read alike because they say one thing twice. This is a \
             finding about the language rather than about the prose, and the \
             three the shipped packs held — a combinator with one condition \
             in it — were simplified when this pass first found them; what \
             remains is the corpus's own rules, which are a recording and are \
             not edited.\n\n",
            plural(measured.synonyms.len(), "rendering")
        );
        shared(&mut out, &measured.synonyms);
    }

    out.push_str("## The language, kind by kind\n\n");
    kinds(&mut out, &measured);
    if unread.is_empty() {
        out.push_str("Every kind of the language occurs in some pack.\n\n");
    } else {
        let names: Vec<String> = unread.iter().map(|kind| format!("`{kind}`")).collect();
        let _ = write!(
            out,
            "{} of the language occur in no pack at all: {}. The corpus cannot \
             falsify their prose, so the golden test in `crates/rules/src/prose.rs` \
             is their only reader.\n\n",
            plural(unread.len(), "kind"),
            names.join(", ")
        );
    }

    out.push_str("## Where the prose is at its weakest\n\n");
    let _ = write!(
        out,
        "The longest passage is `{}` at {}; the shortest is `{}` at {}; the \
         deepest nest is in `{}`, {} deep. A reader arguing about the wording \
         starts at the first and the last of those.\n\n",
        measured.longest.0,
        plural(measured.longest.1, "line"),
        measured.shortest.0,
        plural(measured.shortest.1, "line"),
        measured.deepest.0,
        plural(measured.deepest.1, "condition"),
    );

    out.push_str("## Two passages, whole\n\n");
    out.push_str(
        "`cargo xtask rule-doc <pack|category|key>` prints the rest; the full \
         text of every shipped rule is derived words and is not checked in.\n\n",
    );
    // The passages are filled by the renderer and not by the page: a line
    // wrapped here would be a line the reviewer does not see in a terminal.
    let mut passages = String::new();
    for key in WHOLE {
        let _ = write!(passages, "```text\n{}```\n\n", whole(&every, key));
    }
    Ok(fill(&out) + passages.trim_end() + "\n")
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
        Ok(outputs) => check(root, &outputs, "cargo xtask rule-doc"),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// Prints the passages a consumer asked for: a pack, a category, or one key.
#[allow(clippy::print_stdout, reason = "the command's whole purpose")]
pub(crate) fn print(root: &Path, what: Option<&str>) -> i32 {
    let mut rules: Vec<&Rule> = packs()
        .into_iter()
        .flat_map(|(_, rules)| rules.iter())
        .collect();
    let corpus = corpus(root).unwrap_or_default();
    if let Some(name) = what {
        if let Some((_, pack)) = packs().into_iter().find(|(pack, _)| *pack == name) {
            rules = pack.iter().collect();
        } else if name == "corpus" {
            rules = corpus.iter().collect();
        } else {
            rules.extend(corpus.iter());
            rules.retain(|rule| rule.key == name || rule.category == name);
        }
    }
    if rules.is_empty() {
        println!(
            "no rule, pack or category is called `{}`",
            what.unwrap_or("")
        );
        return 1;
    }
    for rule in &rules {
        println!("{rule}");
    }
    println!("{}", plural(rules.len(), "rule"));
    0
}
