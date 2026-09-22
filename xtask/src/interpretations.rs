//! The pass over the rule readings a locale carries
//! (`03-design/interpretation-records.md`).
//!
//! `rule` is the catalogue's only **open** kind: its members are a
//! consumer's rule packs and not a table, so a reading's key cannot be
//! checked against the catalogue the way `graha.SUN` is. The authority is
//! the packs, and this pass is where they are asked — both ways. A shipped
//! rule with no reading and a reading naming no shipped rule are each
//! listed by name, because a count alone is a claim that goes stale
//! ("count, then list": the list fails in both directions).
//!
//! It also reports what the corpus is rather than what it was assumed to
//! be: the effect facets are an **open** set, a facet the base locale has
//! and another lacks is counted rather than hidden, and the bytes per
//! locale are printed so the pack's cost is a number on a page.
//!
//! `cargo xtask interpretations` writes the page and `check-interpretations`
//! regenerates it in memory and fails on any difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_intl::source::{BASE_LOCALE, ENTITY_NAMESPACE, Entry, Tree};
use teistro_rules::Rule;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/interpretation-records-measured.md";
/// The readings' own root. It is **not** `i18n/`, which `crates/sdk`'s build
/// script compiles into every artefact; these records are loaded, not
/// embedded (`03-design/interpretation-records.md` §3).
const ROOT: &str = "packs/readings";
/// The rule packs that decide which keys name a rule.
const PACKS: [(&str, &str); 2] = [
    ("yogas", "fixtures/baseline/yogas/rules.json"),
    ("doshas", "fixtures/baseline/doshas/rules.json"),
];
/// The prefix a reading's key carries inside the entity namespace.
const RULE_PREFIX: &str = "rule.";

/// Every shipped rule's key, with the pack it came from.
fn shipped(root: &Path) -> Result<BTreeMap<String, &'static str>, String> {
    let mut out = BTreeMap::new();
    for (pack, path) in PACKS {
        let file = read_json(&root.join(path))?;
        let rules: Vec<Rule> = serde_json::from_value(file["rules"].clone())
            .map_err(|why| format!("{path}: {why}"))?;
        for rule in rules {
            out.insert(rule.key, pack);
        }
    }
    Ok(out)
}

/// One locale's readings: the rule key, and the forms the record carries.
type Readings = BTreeMap<String, BTreeSet<String>>;

/// The readings each locale of the root carries.
fn readings(tree: &Tree) -> BTreeMap<String, Readings> {
    let mut out = BTreeMap::new();
    for (tag, locale) in &tree.locales {
        let mut here: Readings = BTreeMap::new();
        if let Some(namespace) = locale.namespaces.get(ENTITY_NAMESPACE) {
            for (key, entry) in &namespace.entries {
                let Entry::Entity(entity) = entry else {
                    continue;
                };
                let Some(rule) = key.strip_prefix(RULE_PREFIX) else {
                    continue;
                };
                here.insert(rule.to_string(), entity.forms.keys().cloned().collect());
            }
        }
        out.insert(tag.clone(), here);
    }
    out
}

/// What the packs and the readings say of each other, both ways.
fn coverage(
    out: &mut String,
    rules: &BTreeMap<String, &'static str>,
    base: &Readings,
    locales: usize,
) {
    let read: Vec<&String> = rules.keys().filter(|key| base.contains_key(*key)).collect();
    let unread: Vec<&String> = rules
        .keys()
        .filter(|key| !base.contains_key(*key))
        .collect();
    let orphan: Vec<&String> = base
        .keys()
        .filter(|key| !rules.contains_key(*key))
        .collect();
    out.push_str("## What the packs and the readings say of each other\n\n");
    let _ = write!(
        out,
        "{} of the {} the shipped packs hold carry a reading, in {} \
         locales. The two lists below are exhaustive rather than counted, \
         because a rule that gains a reading and a reading that loses its \
         rule both have to change this page.\n\n",
        count(read.len()),
        plural(rules.len(), "rule"),
        count(locales),
    );
    let _ = write!(
        out,
        "| | |\n|---|---:|\n| rules with a reading | {} |\n| rules without \
         | {} |\n| readings naming no shipped rule | {} |\n\n",
        count(read.len()),
        count(unread.len()),
        count(orphan.len()),
    );
    if unread.is_empty() {
        out.push_str("Every shipped rule has a reading.\n\n");
    } else {
        let mut by_pack: BTreeMap<&str, Vec<&String>> = BTreeMap::new();
        for key in &unread {
            by_pack
                .entry(rules.get(*key).copied().unwrap_or("?"))
                .or_default()
                .push(key);
        }
        out.push_str("**The rules with no reading**, by pack:\n\n");
        for (pack, keys) in &by_pack {
            let named: Vec<String> = keys.iter().map(|key| format!("`{key}`")).collect();
            let _ = writeln!(out, "- `{pack}`: {}", named.join(", "));
        }
        out.push('\n');
    }
    if orphan.is_empty() {
        out.push_str("Every reading names a shipped rule.\n\n");
    } else {
        let named: Vec<String> = orphan.iter().map(|key| format!("`{key}`")).collect();
        let _ = write!(
            out,
            "**The readings naming no shipped rule** are {}. They are not \
             errors: each is a reading waiting for a module the SDK has not \
             built, and a pass that deleted them would lose work already \
             done.\n\n",
            named.join(", "),
        );
    }
}

/// The facets a reading carries, which are an open set.
fn facets_carried(out: &mut String, per_locale: &BTreeMap<String, Readings>) {
    let mut facets: BTreeMap<String, usize> = BTreeMap::new();
    let mut records = 0usize;
    for readings in per_locale.values() {
        for forms in readings.values() {
            records += 1;
            for form in forms {
                if form != "name" && form != "prose" {
                    *facets.entry(form.clone()).or_default() += 1;
                }
            }
        }
    }
    out.push_str("## The facets a reading carries\n\n");
    let _ = write!(
        out,
        "A record's `name` is its summary and its `prose` is the passage; \
         the rest are the reading's named facets, and they are an **open** \
         set rather than a fixed record — a reading carries the facets it \
         has. Over {}:\n\n| facet | records |\n|---|---:|\n",
        plural(records, "language-record"),
    );
    for (facet, seen) in &facets {
        let _ = writeln!(out, "| `{facet}` | {} |", count(*seen));
    }
    out.push('\n');
}

/// Where a locale lacks a facet the base locale has.
fn facet_gaps(out: &mut String, per_locale: &BTreeMap<String, Readings>, base: &Readings) {
    let mut gaps: BTreeMap<String, usize> = BTreeMap::new();
    for (tag, readings) in per_locale {
        if tag == BASE_LOCALE {
            continue;
        }
        let mut missing = 0usize;
        for (key, forms) in base {
            let Some(here) = readings.get(key) else {
                continue;
            };
            missing += forms.difference(here).count();
        }
        gaps.insert(tag.clone(), missing);
    }
    let total_gaps: usize = gaps.values().sum();
    let _ = write!(
        out,
        "**A facet the base locale has is not always a facet another has.** \
         {} in all, counted rather than hidden: where a locale lacks a \
         facet its record's summary stands in, which is a visible \
         shortfall and not a wrong answer.\n\n| locale | facets the base \
         has and it lacks |\n|---|---:|\n",
        plural(total_gaps, "facet"),
    );
    for (tag, missing) in &gaps {
        let _ = writeln!(out, "| `{tag}` | {} |", count(*missing));
    }
    out.push('\n');
}

/// What the corpus costs **built**, which is the number a consumer pays,
/// and that the built artefact loads and answers.
///
/// The source JSON is what this repository carries; the pack is what ships.
/// Both are printed because they are different questions, and the pack is
/// built here rather than trusted: an artefact nothing exercises is one
/// that has already stopped working (`unbuilt-configuration-is-broken`).
/// The bytes of every message file the build script embeds, which is the
/// other half of the comparison the decision rests on.
///
/// Measured rather than written down: the figure appeared in three
/// documents as three different numbers, none of them current, because a
/// size is a measurement and prose is where a measurement goes stale.
fn embedded_bytes(root: &Path) -> usize {
    fn under(dir: &Path, total: &mut usize) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                under(&path, total);
            } else if path.extension().is_some_and(|it| it == "json")
                && let Ok(meta) = std::fs::metadata(&path)
            {
                *total += usize::try_from(meta.len()).unwrap_or(0);
            }
        }
    }
    let mut total = 0;
    under(&root.join("i18n"), &mut total);
    total
}

fn what_it_costs(out: &mut String, root: &Path, tree: &Tree) -> Result<Built, String> {
    let mut built = Built::default();
    for (tag, locale) in &tree.locales {
        let source = root
            .join(ROOT)
            .join(tag)
            .join(format!("{ENTITY_NAMESPACE}.json"));
        let source_bytes = std::fs::metadata(&source)
            .map_or(0, |meta| usize::try_from(meta.len()).unwrap_or(usize::MAX));
        let bytes = teistro_intl::pack::build(locale, ENTITY_NAMESPACE)
            .map_err(|why| format!("{ROOT}/{tag}: {why}"))?;
        built.packs.insert(tag.clone(), (source_bytes, bytes.len()));
        built.bytes.push((tag.clone(), bytes));
    }
    out.push_str("## What the readings cost\n\n");
    let source: usize = built.packs.values().map(|(source, _)| source).sum();
    let packed: usize = built.packs.values().map(|(_, packed)| packed).sum();
    let _ = write!(
        out,
        "They are **loaded, not embedded**: `crates/sdk`'s build script \
         compiles `i18n/` into every artefact the SDK produces, and this \
         corpus is several times that root's size, so a consumer computing \
         a Julian day would carry every Nepali yoga reading to do it. The \
         numbers are why the decision is a decision — and the pack is not \
         much smaller than its source, so nothing is being deferred to \
         compression.\n\n| locale | source | pack |\n|---|---:|---:|\n",
    );
    for (tag, (source, packed)) in &built.packs {
        let _ = writeln!(
            out,
            "| `{tag}` | {} KB | {} KB |",
            count(source / 1024),
            count(packed / 1024)
        );
    }
    let _ = write!(
        out,
        "| **all** | **{} KB** | **{} KB** |\n\nOne pack a locale, because \
         a Nepali application wants Nepali and its fallback rather than \
         every language's worth of prose: {} KB of the {} KB, and the \
         consumer chooses.\n\n",
        count(source / 1024),
        count(packed / 1024),
        count(
            built
                .packs
                .get("ne-Deva-NP")
                .map_or(0, |(_, packed)| *packed)
                / 1024
        ),
        count(packed / 1024),
    );
    let embedded = embedded_bytes(root);
    let times = (source + embedded / 2).checked_div(embedded).unwrap_or(0);
    let _ = write!(
        out,
        "Against the root the build script *does* embed: `i18n/` is **{} \
         KB** of messages across every locale and namespace, so the corpus \
         is about **{} times** it. Both halves of that comparison are \
         measured here, because the figure was written into three \
         documents as three different numbers and none of them was \
         current.\n\n",
        count(embedded / 1024),
        times,
    );

    Ok(built)
}

/// The packs this pass built, so the claims can load them.
#[derive(Default)]
struct Built {
    /// Source and pack bytes per locale.
    packs: BTreeMap<String, (usize, usize)>,
    /// The pack bytes themselves.
    bytes: Vec<(String, Vec<u8>)>,
}

/// The claims the packs and the records decide.
fn what_the_packs_decide(
    out: &mut String,
    per_locale: &BTreeMap<String, Readings>,
    base: &Readings,
    built: &Built,
) -> Result<(), String> {
    let records: usize = per_locale.values().map(BTreeMap::len).sum();
    let mut claims = vec![Claim::counted(
        "every reading names a key a rule pack could name",
        base.keys()
            .filter(|key| !teistro_intl::source::is_member_key(key))
            .count(),
        base.len(),
    )];
    claims.push(Claim::counted(
        "every locale carries a reading for every rule the base locale reads",
        per_locale
            .values()
            .map(|readings| {
                base.keys()
                    .filter(|key| !readings.contains_key(*key))
                    .count()
            })
            .sum(),
        base.len() * per_locale.len(),
    ));
    claims.push(Claim::counted(
        "every reading carries a summary to be named by",
        per_locale
            .values()
            .flat_map(BTreeMap::values)
            .filter(|forms| !forms.contains("name"))
            .count(),
        records,
    ));
    // The artefact, exercised rather than assumed: every pack loads into
    // an engine that did not have it, and every reading answers from the
    // locale's own record afterwards.
    let mut unloadable = 0usize;
    let mut unanswered = 0usize;
    for (tag, bytes) in &built.bytes {
        let mut engine = teistro_intl::Intl::new(BTreeMap::new())
            .or_else(|_| empty_engine(tag))
            .map_err(|why| format!("{tag}: {why}"))?;
        if engine.load_pack(bytes).is_err() {
            unloadable += 1;
            continue;
        }
        for key in base.keys() {
            if engine
                .entity_from(tag, &format!("{RULE_PREFIX}{key}"))
                .is_none()
            {
                unanswered += 1;
            }
        }
    }
    claims.push(Claim::counted(
        "every locale's readings build into a pack an engine can load",
        unloadable,
        built.bytes.len(),
    ));
    claims.push(Claim::counted(
        "every reading answers from the loaded pack, with no source tree behind it",
        unanswered,
        base.len() * built.bytes.len(),
    ));
    out.push_str("## What the packs decide\n\n");
    out.push_str(&table(&claims));
    out.push('\n');
    Ok(())
}

/// An engine carrying one empty locale, so a pack is loaded into something
/// that never read the source tree — which is the consumer's situation and
/// the only way this pass can prove the artefact stands on its own.
fn empty_engine(tag: &str) -> Result<teistro_intl::Intl, teistro_intl::render::IntlError> {
    let mut locales = BTreeMap::new();
    locales.insert(
        tag.to_string(),
        teistro_intl::source::LocaleSource {
            tag: tag.to_string(),
            meta: serde_json::from_value(serde_json::json!({ "locale": tag }))
                .unwrap_or_else(|_| unreachable!("a tag is a locale")),
            namespaces: BTreeMap::new(),
        },
    );
    teistro_intl::Intl::new(locales)
}

fn main_page(root: &Path) -> Result<String, String> {
    let tree = Tree::load(&root.join(ROOT)).map_err(|why| format!("{ROOT}: {why}"))?;
    let rules = shipped(root)?;
    let per_locale = readings(&tree);
    let base = per_locale
        .get(BASE_LOCALE)
        .ok_or_else(|| format!("{ROOT}: no {BASE_LOCALE}"))?;

    let mut out = String::new();
    out.push_str("# The rule readings, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask interpretations` over \
         `{ROOT}` and the shipped rule packs, 2026-09-21. Do not edit: \
         `check-interpretations` regenerates this page and fails on any \
         difference. The design it measures is \
         [`interpretation-records.md`](interpretation-records.md).\n\n",
    );

    coverage(&mut out, &rules, base, per_locale.len());
    facets_carried(&mut out, &per_locale);
    facet_gaps(&mut out, &per_locale, base);
    let built = what_it_costs(&mut out, root, &tree)?;
    what_the_packs_decide(&mut out, &per_locale, base, &built)?;
    Ok(fill(&out))
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    Ok(vec![Output::new(PAGE, main_page(root)?)])
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(out) => write(root, &out),
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}

/// Regenerates the page in memory and fails on any difference.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(out) => check(root, &out, "cargo xtask interpretations"),
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}
