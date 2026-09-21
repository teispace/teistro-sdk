//! The pass over the state readings a locale carries
//! (`03-design/state-readings.md`).
//!
//! A rule reading answers *this yoga is present, and here is what it
//! means*. These answer what a chart **is** without triggering anything:
//! Jupiter in the first house, the Moon in Ashwini, an Aries lagna, the
//! hour a dosha acts in. They reach the same records by a different door,
//! as **forms**, because 38 of the corpus's keys are shared between two of
//! its categories and a record per category would split one subject.
//!
//! What this pass is for is the part that cannot be asserted: that two
//! corpora describing one subject both survive. It builds the readings
//! pack and the states pack, loads them into an engine that never read
//! either source tree, and asks the merged record for both.
//!
//! `cargo xtask state-readings` writes the page and `check-state-readings` regenerates it
//! in memory and fails on any difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_intl::migrate::{STATE_CATEGORIES, StateCategory};
use teistro_intl::source::{BASE_LOCALE, ENTITY_NAMESPACE, Entry, Tree};
use teistro_rules::Rule;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};
use crate::rules_corpus::read_json;

const PAGE: &str = "docs/03-design/state-readings-measured.md";
/// The state readings' own root, loaded rather than embedded.
const ROOT: &str = "packs/states";
/// The rule readings' root, loaded beside it.
const READINGS: &str = "packs/readings";
/// The rule packs that decide which keys name a rule.
const PACKS: [(&str, &str); 2] = [
    ("yogas", "fixtures/baseline/yogas/rules.json"),
    ("doshas", "fixtures/baseline/doshas/rules.json"),
];
/// The prefix a rule's record carries inside the entity namespace.
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

/// One locale's records: the catalogue key, and the forms it carries.
type Records = BTreeMap<String, BTreeSet<String>>;

/// The records each locale of a root carries, by catalogue key.
fn records(tree: &Tree) -> BTreeMap<String, Records> {
    let mut out = BTreeMap::new();
    for (tag, locale) in &tree.locales {
        let mut here: Records = BTreeMap::new();
        if let Some(namespace) = locale.namespaces.get(ENTITY_NAMESPACE) {
            for (key, entry) in &namespace.entries {
                if let Entry::Entity(entity) = entry {
                    let mut forms: BTreeSet<String> = entity.forms.keys().cloned().collect();
                    if entity.glyph.is_some() {
                        forms.insert(String::from("glyph"));
                    }
                    here.insert(key.clone(), forms);
                }
            }
        }
        out.insert(tag.clone(), here);
    }
    out
}

/// The kind a full key names.
fn kind_of(key: &str) -> &str {
    key.split_once('.').map_or(key, |(kind, _)| kind)
}

/// Where the corpus landed: the categories mapped, and what each became.
fn where_it_landed(out: &mut String, base: &Records, mapped: &BTreeMap<String, usize>) {
    out.push_str("## Where the corpus landed\n\n");
    let keys: BTreeSet<&str> = base.keys().map(|key| kind_of(key)).collect();
    let _ = write!(
        out,
        "{} of the engine's 38 state categories map onto a subject this \
         SDK has, and they become {} under {} kinds. The mapping is a \
         written table and not a resemblance: a category with no subject \
         here is skipped and named below rather than guessed at.\n\n\
         | category | kind | form | records |\n|---|---|---|---:|\n",
        count(mapped.len()),
        plural(base.len(), "record"),
        count(keys.len()),
    );
    for StateCategory {
        category,
        kind,
        form,
    } in STATE_CATEGORIES
    {
        let Some(records) = mapped.get(category) else {
            continue;
        };
        let form = if form.is_empty() {
            String::from("a record of its own")
        } else {
            format!("`{form}`")
        };
        let _ = writeln!(
            out,
            "| `{category}` | `{kind}` | {form} | {} |",
            count(*records)
        );
    }
    out.push('\n');
}

/// The categories no subject here answers to, by name.
fn what_it_did_not_map(out: &mut String, unmapped: &BTreeMap<String, usize>) {
    out.push_str("## What it did not map\n\n");
    if unmapped.is_empty() {
        out.push_str("Every category of the corpus has a subject here.\n\n");
        return;
    }
    let records: usize = unmapped.values().sum();
    let _ = write!(
        out,
        "{} categories, {}. The list is exhaustive rather than counted, \
         because a category that gains a subject and a corpus that gains a \
         category both have to change this page. Each is a decision rather \
         than a task, and `state-readings.md` §8 says which kind of one.\n\n\
         | category | records |\n|---|---:|\n",
        count(unmapped.len()),
        plural(records, "record"),
    );
    for (category, records) in unmapped {
        let _ = writeln!(out, "| `{category}` | {} |", count(*records));
    }
    out.push('\n');
}

/// Every category whose reading a record carries.
fn categories_of(key: &str, forms: &BTreeSet<String>) -> Vec<&'static str> {
    STATE_CATEGORIES
        .iter()
        .filter(|state| {
            kind_of(key) == state.kind
                && forms.contains(if state.form.is_empty() {
                    "name"
                } else {
                    state.form
                })
        })
        .map(|state| state.category)
        .collect()
}

/// The records two categories describe, and what each subject ended with.
fn where_two_corpora_meet(out: &mut String, base: &Records) {
    let mut shared: BTreeMap<&str, Vec<&String>> = BTreeMap::new();
    let mut categories_per_kind: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (key, forms) in base {
        let here = categories_of(key, forms);
        if here.len() > 1 {
            shared.entry(kind_of(key)).or_default().push(key);
            categories_per_kind
                .entry(kind_of(key))
                .or_default()
                .extend(here);
        }
    }
    let subjects: usize = shared.values().map(Vec::len).sum();
    out.push_str("## Where two corpora meet\n\n");
    let _ = write!(
        out,
        "{} carry forms from more than one category. They are why a \
         reading is a form on a record rather than a record of its own: a \
         migration that replaced would keep whichever category it read \
         last, and the subject would lose the other reading silently.\n\n\
         | kind | subjects described more than once | the categories describing them |\n|---|---:|---|\n",
        plural(subjects, "subject"),
    );
    for (kind, keys) in &shared {
        let categories: Vec<String> = categories_per_kind
            .get(kind)
            .into_iter()
            .flatten()
            .map(|category| format!("`{category}`"))
            .collect();
        let _ = writeln!(
            out,
            "| `{kind}` | {} | {} |",
            count(keys.len()),
            categories.join(", ")
        );
    }
    out.push('\n');
}

/// What the states corpus adds to the rule readings, which is the gap the
/// rule readings' own page lists by name.
fn what_it_adds_to_the_readings(
    out: &mut String,
    rules: &BTreeMap<String, &'static str>,
    readings: &Records,
    base: &Records,
) {
    let timed: BTreeSet<&str> = base
        .keys()
        .filter_map(|key| key.strip_prefix(RULE_PREFIX))
        .collect();
    let read: BTreeSet<&str> = readings
        .keys()
        .filter_map(|key| key.strip_prefix(RULE_PREFIX))
        .collect();
    let only_timing: Vec<&&str> = timed
        .iter()
        .filter(|key| rules.contains_key(**key) && !read.contains(**key))
        .collect();
    let both: usize = timed
        .iter()
        .filter(|key| rules.contains_key(**key) && read.contains(**key))
        .count();
    let waiting: Vec<&&str> = timed
        .iter()
        .filter(|key| !rules.contains_key(**key))
        .collect();
    let silent: Vec<&String> = rules
        .keys()
        .filter(|key| !read.contains(key.as_str()) && !timed.contains(key.as_str()))
        .collect();
    out.push_str("## What it adds to the rule readings\n\n");
    let _ = write!(
        out,
        "`dosha-timing` keys onto rules, so the two corpora meet on the \
         same records. Against the {} the shipped packs hold:\n\n\
         | | |\n|---|---:|\n| rules that had a reading and gain a timing | \
         {} |\n| rules that had no reading and gain a timing | {} |\n| \
         rules with neither | {} |\n| timings naming no shipped rule | {} \
         |\n\n",
        plural(rules.len(), "rule"),
        count(both),
        count(only_timing.len()),
        count(silent.len()),
        count(waiting.len()),
    );
    let named = |keys: &[&&str]| -> String {
        keys.iter()
            .map(|key| format!("`{key}`"))
            .collect::<Vec<_>>()
            .join(", ")
    };
    if only_timing.is_empty() {
        out.push_str("No rule was waiting for one.\n\n");
    } else {
        let _ = write!(
            out,
            "**The rules that gain their only text** are {}. They are \
             exactly the list `interpretation-records-measured.md` carries \
             as rules with no reading, and what they gain is a `timing` \
             rather than a reading: it says when the dosha acts, not what \
             it means, and calling it a reading would be a claim the corpus \
             does not make.\n\n",
            named(&only_timing),
        );
    }
    if !waiting.is_empty() {
        let _ = write!(
            out,
            "**The timings naming no shipped rule** are {}. They are the \
             same ten the readings' page lists: the milan rules, waiting on \
             Phase 8's `matching` rather than on anything here.\n\n",
            named(&waiting),
        );
    }
}

/// The packs this pass built, so the claims can load them.
#[derive(Default)]
struct Built {
    /// Source and pack bytes per locale, for this root.
    packs: BTreeMap<String, (usize, usize)>,
    /// The state pack bytes per locale.
    states: Vec<(String, Vec<u8>)>,
    /// The readings pack bytes per locale.
    readings: BTreeMap<String, Vec<u8>>,
}

/// Builds every locale's pack of both roots and prints what this one costs.
fn what_it_costs(
    out: &mut String,
    root: &Path,
    tree: &Tree,
    readings: &Tree,
) -> Result<Built, String> {
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
        built.states.push((tag.clone(), bytes));
    }
    for (tag, locale) in &readings.locales {
        let bytes = teistro_intl::pack::build(locale, ENTITY_NAMESPACE)
            .map_err(|why| format!("{READINGS}/{tag}: {why}"))?;
        built.readings.insert(tag.clone(), bytes);
    }
    out.push_str("## What the state readings cost\n\n");
    let source: usize = built.packs.values().map(|(source, _)| source).sum();
    let packed: usize = built.packs.values().map(|(_, packed)| packed).sum();
    let beside: usize = built.readings.values().map(Vec::len).sum();
    let _ = write!(
        out,
        "**Loaded, not embedded**, for the reason the rule readings are \
         (`interpretation-records.md` §3): `crates/sdk`'s build script \
         compiles `i18n/` into every artefact, and a consumer computing a \
         Julian day should not carry a Sanskrit passage on Jupiter in the \
         first house to do it. A consumer loads one pack a locale from each \
         root it wants.\n\n| locale | source | pack |\n|---|---:|---:|\n",
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
        "| **all** | **{} KB** | **{} KB** |\n\nBeside the rule readings' \
         {} KB, which is the other pack a consumer that wants both would \
         load: {} KB in all for four languages, and {} KB for one.\n\n",
        count(source / 1024),
        count(packed / 1024),
        count(beside / 1024),
        count((packed + beside) / 1024),
        count(
            (built
                .packs
                .get("ne-Deva-NP")
                .map_or(0, |(_, packed)| *packed)
                + built.readings.get("ne-Deva-NP").map_or(0, Vec::len))
                / 1024
        ),
    );
    Ok(built)
}

/// The claims the packs and the records decide.
fn what_the_packs_decide(
    out: &mut String,
    per_locale: &BTreeMap<String, Records>,
    base: &Records,
    readings: &Records,
    built: &Built,
) -> Result<(), String> {
    let total: usize = per_locale.values().map(BTreeMap::len).sum();
    let mut claims = vec![Claim::counted(
        "every record names a catalogue member, or a well-formed key of an open kind",
        base.keys()
            .filter(|key| {
                !teistro_intl::source::is_open_kind_key(key)
                    && teistro_core::key::resolve(key).is_err()
            })
            .count(),
        base.len(),
    )];
    claims.push(Claim::counted(
        "every locale carries every record the base locale carries",
        per_locale
            .values()
            .map(|records| {
                base.keys()
                    .filter(|key| !records.contains_key(*key))
                    .count()
            })
            .sum(),
        base.len() * per_locale.len(),
    ));
    claims.push(Claim::counted(
        "every record carries a form to be read by",
        per_locale
            .values()
            .flat_map(BTreeMap::values)
            .filter(|forms| forms.is_empty())
            .count(),
        total,
    ));

    // The artefact, exercised rather than assumed, and the overlay with
    // it: the readings pack is loaded first, the states pack over it, and
    // a subject described by both is asked for both afterwards.
    let mut unloadable = 0usize;
    let mut unanswered = 0usize;
    let mut lost = 0usize;
    let mut shared = 0usize;
    for (tag, bytes) in &built.states {
        let mut engine = empty_engine(tag).map_err(|why| format!("{tag}: {why}"))?;
        if let Some(first) = built.readings.get(tag)
            && engine.load_pack(first).is_err()
        {
            unloadable += 1;
            continue;
        }
        if engine.load_pack(bytes).is_err() {
            unloadable += 1;
            continue;
        }
        for (key, forms) in base {
            let Some(record) = engine.entity_from(tag, key) else {
                unanswered += forms.len();
                continue;
            };
            for form in forms {
                if record.form(form).is_none() {
                    unanswered += 1;
                }
            }
            // What the readings pack put here first must still be here.
            for form in readings.get(key).into_iter().flatten() {
                shared += 1;
                if record.form(form).is_none() {
                    lost += 1;
                }
            }
        }
    }
    claims.push(Claim::counted(
        "every locale's state readings build into a pack an engine can load",
        unloadable,
        built.states.len(),
    ));
    claims.push(Claim::counted(
        "every form answers from the loaded packs, with no source tree behind them",
        unanswered,
        base.values().map(BTreeSet::len).sum::<usize>() * built.states.len(),
    ));
    claims.push(Claim::counted(
        "a rule's reading still answers after the state readings are loaded over it",
        lost,
        shared,
    ));
    out.push_str("## What the packs decide\n\n");
    out.push_str(&table(&claims));
    out.push('\n');
    Ok(())
}

/// An engine carrying one empty locale, so a pack is loaded into something
/// that never read either source tree — which is the consumer's situation.
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

/// What the migration mapped and what it did not, read from the corpus's
/// own landing rather than from the exporter's document, which this
/// repository does not carry.
fn mapped_and_unmapped(base: &Records) -> (BTreeMap<String, usize>, BTreeMap<String, usize>) {
    let mut mapped: BTreeMap<String, usize> = BTreeMap::new();
    for (key, forms) in base {
        for category in categories_of(key, forms) {
            *mapped.entry(String::from(category)).or_default() += 1;
        }
    }
    let unmapped = UNMAPPED
        .iter()
        .map(|(category, records)| ((*category).to_string(), *records))
        .collect();
    (mapped, unmapped)
}

/// The categories the SDK has no subject for, with the records each holds.
///
/// They are written here rather than counted from the corpus because this
/// repository carries the migrated packs and not the exporter's document:
/// a category that is skipped leaves nothing behind to count. The list is
/// held to the migration's own table by a claim below — every category
/// named here must be one `STATE_CATEGORIES` does not map — so it cannot
/// quietly disagree with the code (`03-design/state-readings.md` §8).
const UNMAPPED: [(&str, usize); 14] = [
    ("auspicious-kaal", 5),
    ("ayurdaya-balarishta", 4),
    ("ayurdaya-classical-rule", 5),
    ("ayurdaya-harana", 4),
    ("ayurdaya-maraka", 9),
    ("ayurdaya-maraka-trigger", 3),
    ("ayurdaya-method", 3),
    ("ayurdaya-tier", 4),
    ("ayurdaya-vulnerability", 9),
    ("inauspicious-kaal", 5),
    ("muhurta-factor", 47),
    ("planet-condition", 8),
    ("sade-sati-phala", 5),
    ("shadbala-strength", 28),
];

fn main_page(root: &Path) -> Result<String, String> {
    let tree = Tree::load(&root.join(ROOT)).map_err(|why| format!("{ROOT}: {why}"))?;
    let readings_tree =
        Tree::load(&root.join(READINGS)).map_err(|why| format!("{READINGS}: {why}"))?;
    let rules = shipped(root)?;
    let per_locale = records(&tree);
    let readings = records(&readings_tree);
    let base = per_locale
        .get(BASE_LOCALE)
        .ok_or_else(|| format!("{ROOT}: no {BASE_LOCALE}"))?;
    let base_readings = readings
        .get(BASE_LOCALE)
        .ok_or_else(|| format!("{READINGS}: no {BASE_LOCALE}"))?;
    let (mapped, unmapped) = mapped_and_unmapped(base);

    let mut out = String::new();
    out.push_str("# The state readings, measured\n\n");
    let _ = write!(
        out,
        "Status: `generated` by `cargo xtask state-readings` over `{ROOT}`, \
         `{READINGS}` and the shipped rule packs, 2026-09-21. Do not edit: \
         `check-state-readings` regenerates this page and fails on any difference. \
         The design it measures is \
         [`state-readings.md`](state-readings.md).\n\n",
    );

    where_it_landed(&mut out, base, &mapped);
    what_it_did_not_map(&mut out, &unmapped);
    where_two_corpora_meet(&mut out, base);
    what_it_adds_to_the_readings(&mut out, &rules, base_readings, base);
    let built = what_it_costs(&mut out, root, &tree, &readings_tree)?;
    what_the_packs_decide(&mut out, &per_locale, base, base_readings, &built)?;
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
        Ok(out) => check(root, &out, "cargo xtask state-readings"),
        Err(why) => {
            eprintln!("{why}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{STATE_CATEGORIES, UNMAPPED};

    /// The written list of what was not mapped cannot disagree with the
    /// table that does the mapping.
    #[test]
    fn nothing_unmapped_is_also_mapped() {
        for (category, _) in UNMAPPED {
            assert!(
                !STATE_CATEGORIES
                    .iter()
                    .any(|state| state.category == category),
                "`{category}` is both mapped and listed as unmapped"
            );
        }
        assert_eq!(
            STATE_CATEGORIES.len() + UNMAPPED.len(),
            38,
            "the corpus has 38 categories and every one is mapped or named"
        );
    }
}
