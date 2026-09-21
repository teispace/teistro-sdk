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

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro_interpret::{KEYS, Plan, placements, readings};
use teistro_intl::source::{Completeness, Tree};
use teistro_intl::{Intl, Rendered};
use teistro_rules::{Evaluator, Readings as RuleReadings, Rule, RuleChart, shipped};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, plural, table};
use crate::rules_corpus::{chart, read_json};

const PAGE: &str = "docs/03-design/interpret-measured.md";
const ROOT: &str = "fixtures/baseline/yogas";
/// The chart the page ends with, rendered whole: the corpus's first.
const SNAPSHOT: &str = "c001-kathmandu-1990-04-14";

/// One chart of the corpus and the plan its composers write.
struct Composed {
    name: String,
    plan: Plan,
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
            plan.items
                .extend(readings(held.iter().map(|(rule, result)| (*rule, result))));
            out.push(Composed { name, plan });
        }
    }
    Ok(out)
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
/// count alone, so the page names it ([[count-then-list]]: the lists are
/// short or the gate is red).
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

    decided(&mut out, &said, items, strict.len());

    out.push_str("## What the composers say\n\n");
    out.push_str("| key | items |\n|---|---|\n");
    for (key, at) in &by_key {
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
        plural(plans.len(), "item")
    );

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
