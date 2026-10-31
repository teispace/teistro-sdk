//! What each locale says (P2-5 of the consumer brief): for every locale
//! under `i18n/`, the share of the catalogue members a document can carry
//! that it names, kind by kind, and the share of the base locale's
//! messages it translates.
//!
//! A strict locale names everything a document carries but the members
//! `intl.rs`'s `UNNAMED` lists, and is gated by `check-intl`. A base
//! locale falls back to English for whatever it leaves out, so nothing
//! fails when it says less: this page is what makes a loss visible.
//! `check-locale-coverage` regenerates the page, and fails when any
//! locale's count falls below its floor in
//! `docs/03-design/locale-coverage-floor.json`. The floor is raised by
//! hand when a pack grows, as a size budget is, so a pack can only grow
//! unless someone says in a commit that it shrank.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro_core::catalogue::{Kind, key_of};
use teistro_core::key::KeyId;
use teistro_intl::source::{Completeness, Entry, LocaleSource, Tree};

use crate::generated::{Output, check, write};
use crate::intl::document_kinds;

const PAGE: &str = "docs/03-design/locale-coverage-measured.md";
const FLOOR: &str = "docs/03-design/locale-coverage-floor.json";

/// One locale's counts.
struct Coverage {
    /// The locale's tag.
    tag: String,
    /// Whether every base key is gated.
    completeness: Completeness,
    /// Members named, kind by kind: (named, of).
    kinds: BTreeMap<String, (usize, usize)>,
    /// Base messages translated.
    messages: usize,
    /// The namespaces no reviewer has signed off.
    draft: Vec<String>,
}

impl Coverage {
    fn members(&self) -> (usize, usize) {
        self.kinds
            .values()
            .fold((0, 0), |(named, of), (n, o)| (named + n, of + o))
    }
}

/// Every key of every kind a document can carry, by kind.
fn members() -> BTreeMap<String, Vec<String>> {
    document_kinds()
        .into_iter()
        .filter_map(|name| {
            let kind = Kind::from_name(&name)?;
            let keys = (0..u16::try_from(kind.count()).unwrap_or(u16::MAX))
                .filter_map(|id| key_of(KeyId::new(kind, id)))
                .map(str::to_owned)
                .collect();
            Some((name, keys))
        })
        .collect()
}

/// The base locale's message keys: every entry that is not an entity.
fn messages(base: &LocaleSource) -> BTreeSet<String> {
    base.namespaces
        .iter()
        .flat_map(|(ns, namespace)| {
            namespace
                .entries
                .iter()
                .filter(|(_, entry)| matches!(entry, Entry::Message(_)))
                .map(move |(key, _)| format!("{ns}.{key}"))
        })
        .collect()
}

fn coverage(tree: &Tree) -> (Vec<Coverage>, usize, usize) {
    let members = members();
    let base = tree.base().map(messages).unwrap_or_default();
    let total: usize = members.values().map(Vec::len).sum();
    let locales = tree
        .locales
        .values()
        .map(|locale| Coverage {
            tag: locale.tag.clone(),
            completeness: locale.meta.completeness,
            draft: locale.meta.draft.clone(),
            kinds: members
                .iter()
                .map(|(kind, keys)| {
                    let named = keys
                        .iter()
                        .filter(|key| locale.entity(&format!("{kind}.{key}")).is_some())
                        .count();
                    (kind.clone(), (named, keys.len()))
                })
                .collect(),
            messages: base
                .iter()
                .filter(|key| matches!(locale.entry(key), Some(Entry::Message(_))))
                .count(),
        })
        .collect();
    (locales, total, base.len())
}

#[expect(clippy::cast_precision_loss, reason = "counts are thousands at most")]
fn share(part: usize, whole: usize) -> String {
    if whole == 0 {
        return "—".to_owned();
    }
    format!("{:.1}%", 100.0 * part as f64 / whole as f64)
}

fn page(tree: &Tree) -> String {
    let (locales, total, base) = coverage(tree);
    let mut out = String::from(
        "# What each locale says\n\n\
         Status: `generated` by `cargo xtask locale-coverage`. Do not edit: \
         `check-locale-coverage` regenerates this page, and fails when a \
         locale's count falls below its floor in \
         `docs/03-design/locale-coverage-floor.json`, which is raised by \
         hand as a pack grows.\n\n\
         A **member** is a catalogue key a document can carry (its schema \
         names the kind), and a locale names it when its `sdk.entity` \
         namespace holds the record. A **message** is one of the base \
         locale's messages, and a locale translates it when it holds the \
         same key. A strict locale is complete but for the members \
         `xtask/src/intl.rs` lists as unnamed, and `check-intl` gates it; a \
         base locale falls back along its chain for the rest, so this page \
         is where what it leaves out shows.\n\n",
    );
    let _ = writeln!(
        out,
        "| locale | completeness | members named | | messages translated | |\n\
         |---|---|---:|---:|---:|---:|"
    );
    for locale in &locales {
        let (named, of) = locale.members();
        let _ = writeln!(
            out,
            "| `{}` | {} | {named} of {of} | {} | {} of {base} | {} |",
            locale.tag,
            match locale.completeness {
                Completeness::Strict => "strict",
                Completeness::Base => "base",
            },
            share(named, of),
            locale.messages,
            share(locale.messages, base),
        );
    }
    let _ = writeln!(
        out,
        "\n{total} members of {} kinds, {base} messages.\n",
        locales.first().map_or(0, |locale| locale.kinds.len())
    );
    // A count says a text is there, and not that anyone has read it.
    for locale in locales.iter().filter(|locale| !locale.draft.is_empty()) {
        let _ = writeln!(
            out,
            "`{}` ships unreviewed, marked draft on every render (QUESTIONS.md Q42): {}.\n",
            locale.tag,
            locale
                .draft
                .iter()
                .map(|namespace| format!("`{namespace}`"))
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    let _ = writeln!(out, "## By kind\n");
    let mut header = String::from("| kind | members |");
    let mut rule = String::from("|---|---:|");
    for locale in &locales {
        let _ = write!(header, " `{}` |", locale.tag);
        rule.push_str("---:|");
    }
    let _ = writeln!(out, "{header}\n{rule}");
    if let Some(first) = locales.first() {
        for (kind, (_, of)) in &first.kinds {
            let _ = write!(out, "| `{kind}` | {of} |");
            for locale in &locales {
                let named = locale.kinds.get(kind).map_or(0, |(named, _)| *named);
                let _ = write!(out, " {named} |");
            }
            out.push('\n');
        }
    }
    out
}

/// The floors, by locale: members named and messages translated.
fn floors(root: &Path) -> Result<BTreeMap<String, (usize, usize)>, String> {
    let text = std::fs::read_to_string(root.join(FLOOR)).map_err(|e| format!("{FLOOR}: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| format!("{FLOOR}: {e}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| format!("{FLOOR}: an object of locales"))?;
    object
        .iter()
        .map(|(tag, floor)| {
            let count = |field: &str| {
                floor
                    .get(field)
                    .and_then(serde_json::Value::as_u64)
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| format!("{FLOOR}: `{tag}` needs a whole `{field}`"))
            };
            Ok((tag.clone(), (count("members")?, count("messages")?)))
        })
        .collect()
}

/// Every locale held to its floor, and every floor to a locale; the
/// number of failures.
fn hold(tree: &Tree, floors: &BTreeMap<String, (usize, usize)>) -> usize {
    let (locales, _, _) = coverage(tree);
    let mut failures = 0;
    for locale in &locales {
        let (named, _) = locale.members();
        let Some(&(members, messages)) = floors.get(&locale.tag) else {
            println!(
                "FAIL  `{}` has no floor in {FLOOR}: add {{\"members\": {named}, \"messages\": {}}}",
                locale.tag, locale.messages
            );
            failures += 1;
            continue;
        };
        for (what, has, floor) in [
            ("members named", named, members),
            ("messages translated", locale.messages, messages),
        ] {
            if has < floor {
                println!(
                    "FAIL  `{}` has {has} {what}, below its floor of {floor}: restore them, \
                     or lower the floor in {FLOOR} saying why",
                    locale.tag
                );
                failures += 1;
            } else if has > floor {
                println!(
                    "ok    `{}` has {has} {what}, above its floor of {floor}: raise the floor",
                    locale.tag
                );
            }
        }
    }
    for tag in floors.keys() {
        if !tree.locales.contains_key(tag) {
            println!("FAIL  {FLOOR} holds a floor for `{tag}`, which is no locale");
            failures += 1;
        }
    }
    if failures == 0 {
        println!("ok    every locale holds its floor");
    }
    failures
}

fn load(root: &Path) -> Result<Tree, String> {
    Tree::load(&root.join("i18n")).map_err(|e| format!("the i18n/ sources: {e}"))
}

pub(crate) fn generate(root: &Path) -> i32 {
    match load(root) {
        Ok(tree) => write(root, &[Output::new(PAGE, page(&tree))]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let (tree, floors) = match (load(root), floors(root)) {
        (Ok(tree), Ok(floors)) => (tree, floors),
        (Err(err), _) | (_, Err(err)) => {
            println!("FAIL  {err}");
            return 1;
        }
    };
    let page = check(
        root,
        &[Output::new(PAGE, page(&tree))],
        "cargo xtask locale-coverage",
    );
    i32::from(page != 0 || hold(&tree, &floors) != 0)
}
