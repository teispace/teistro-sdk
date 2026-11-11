//! Each record's brief and full answers, measured (`03-design/result-detail.md`,
//! A2-2): every record's example request (`crates/sdk/tests/requests.json`)
//! answered at both details over the built-in ephemeris, each written as
//! the envelope an agent reads, and counted with the dependency-free
//! token approximation of `tokens.rs` (C1165); and every brief's paths
//! held to its record's answer schema.
//!
//! `cargo xtask detail-tokens` writes the page; `check-detail-tokens`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro::records::{Brief, Detail, Record};
use teistro::{Context, Ephemeris};
use teistro_core::envelope::canonical_json;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, share, table};
use crate::tokens::approximate;

const PAGE: &str = "docs/03-design/result-detail-measured.md";

/// The fixture holding one request of each record's.
const EXAMPLES: &str = "crates/sdk/tests/requests.json";
/// The geo table `geo.resolve`'s example is answered from.
const GEO_TABLE: &str = "crates/sdk/tests/geo-table.json";

/// Records whose example is not answered here, each with why. Refused
/// both ways: a listed record that answers, or is gone, fails the page.
const SET_ASIDE: [(&str, &str); 2] = [
    (
        "engine.call",
        "asks an engine's own operation, which the built-in ephemeris has none of; its value is the engine's JSON, unread by the SDK, so its brief is the whole",
    ),
    (
        "engine.manifest",
        "describes an engine's own operations, which the built-in ephemeris has none of; its brief is the whole",
    ),
];

/// A full answer of at least this many tokens has its fields listed.
const LISTED_FROM: usize = 400;

struct Row {
    name: &'static str,
    brief: Brief,
    full: usize,
    short: usize,
    full_value: usize,
    short_value: usize,
    provenance: (usize, usize),
    /// Each top-level field of the full value with its tokens, and
    /// whether the brief keeps any of it.
    fields: Vec<(String, usize, bool)>,
}

struct Measured {
    rows: Vec<Row>,
    set_aside: Vec<(&'static str, String)>,
    paths: usize,
    unknown: Vec<String>,
    untagged: Vec<String>,
    open: Vec<String>,
    /// Each record keeping fields, with what its schema states beside
    /// them that the brief leaves out.
    left_out: Vec<(&'static str, Vec<String>)>,
    untyped: Vec<&'static str>,
    stale: Vec<&'static str>,
}

/// The tokens of a value as canonical JSON.
fn tokens(value: &Value) -> usize {
    approximate(&canonical_json(value))
}

/// The top-level fields of a value (of each item, for a list), each with
/// the tokens of `"name":value`, summed over the items.
fn fields(value: &Value, brief: Brief) -> Vec<(String, usize, bool)> {
    let items: Vec<&Value> = match value {
        Value::Array(items) => items.iter().collect(),
        other => vec![other],
    };
    let mut out: Vec<(String, usize, bool)> = Vec::new();
    for item in items {
        let Value::Object(object) = item else {
            continue;
        };
        for (key, inner) in object {
            let spent = approximate(&canonical_json(key)) + 1 + tokens(inner);
            let kept = match brief {
                Brief::Whole => true,
                Brief::Keeps(paths) => paths
                    .iter()
                    .any(|path| path.split('.').next() == Some(key.as_str())),
            };
            match out.iter_mut().find(|(name, _, _)| name == key) {
                Some(row) => row.1 += spent,
                None => out.push((key.clone(), spent, kept)),
            }
        }
    }
    out.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    out
}

fn provenance_tokens(envelope: &Value) -> usize {
    envelope.get("provenance").map_or(0, tokens)
}

fn answered(record: &Record, sdk: &Context, request: &str) -> Result<Row, teistro::Error> {
    let full = record.answer_at(sdk, request, Detail::Full)?;
    let short = record.brief_of(full.clone());
    let (full_envelope, short_envelope) = (full.envelope()?, short.envelope()?);
    Ok(Row {
        name: record.name,
        brief: record.brief(),
        full: tokens(&full_envelope),
        short: tokens(&short_envelope),
        full_value: tokens(&full.value),
        short_value: tokens(&short.value),
        provenance: (
            provenance_tokens(&full_envelope),
            provenance_tokens(&short_envelope),
        ),
        fields: fields(&full.value, record.brief()),
    })
}

fn measure(root: &Path) -> Result<Measured, String> {
    let text =
        std::fs::read_to_string(root.join(EXAMPLES)).map_err(|why| format!("{EXAMPLES}: {why}"))?;
    let examples: Vec<Value> =
        serde_json::from_str(&text).map_err(|why| format!("{EXAMPLES}: {why}"))?;
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| why.to_string())?;
    // `geo.resolve` answers from a provider, which no context has unless
    // given one: the test table every binding's tests load.
    let table = std::fs::read_to_string(root.join(GEO_TABLE))
        .map_err(|why| format!("{GEO_TABLE}: {why}"))?;
    sdk.geo().load(&table).map_err(|why| why.to_string())?;
    let mut measured = Measured {
        rows: Vec::new(),
        set_aside: Vec::new(),
        paths: 0,
        unknown: Vec::new(),
        untagged: Vec::new(),
        open: Vec::new(),
        left_out: Vec::new(),
        untyped: Vec::new(),
        stale: Vec::new(),
    };
    let records = teistro::records::records();
    for record in &records {
        let mut generator = schemars::generate::SchemaSettings::draft2020_12().into_generator();
        measured.paths += record.brief().paths().len();
        match record.answer_schema_at(&mut generator, Detail::Brief) {
            Some((_, narrowed)) => {
                let name = record.name;
                measured.unknown.extend(
                    narrowed
                        .unknown
                        .iter()
                        .map(|path| format!("`{name}` `{path}`")),
                );
                measured.untagged.extend(
                    narrowed
                        .untagged
                        .iter()
                        .map(|tag| format!("`{name}` `{tag}`")),
                );
                if record.brief() != Brief::Whole {
                    measured.left_out.push((name, narrowed.left_out.clone()));
                }
                measured.open.extend(
                    narrowed
                        .open
                        .iter()
                        .map(|path| format!("`{name}` `{path}`")),
                );
            }
            None if record.brief() != Brief::Whole => measured.untyped.push(record.name),
            None => {}
        }
        let request = examples
            .iter()
            .find(|example| example["record"] == record.name)
            .and_then(|example| example.get("request"))
            .ok_or_else(|| format!("{EXAMPLES} holds no request of `{}`", record.name))?
            .to_string();
        let aside = SET_ASIDE.iter().find(|(name, _)| *name == record.name);
        match (answered(record, &sdk, &request), aside) {
            (Ok(row), None) => measured.rows.push(row),
            (Ok(_), Some((name, _))) => measured.stale.push(name),
            (Err(why), Some((name, reason))) => measured.set_aside.push((
                name,
                format!("{reason} (refused here: {})", why.status.name()),
            )),
            (Err(why), None) => return Err(format!("`{}`: {why}", record.name)),
        }
    }
    measured.stale.extend(
        SET_ASIDE
            .iter()
            .filter(|(name, _)| records.iter().all(|record| record.name != *name))
            .map(|(name, _)| *name),
    );
    Ok(measured)
}

fn kept(brief: Brief) -> String {
    match brief {
        Brief::Whole => String::from("the whole value"),
        Brief::Keeps(paths) => paths
            .iter()
            .map(|path| format!("`{path}`"))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

/// The page's claims, or a refusal naming each one falsified: every one is
/// a rule the declarations must keep, so the page is not written past it.
fn claims(measured: &Measured, keeps: &[&Row]) -> Result<Vec<Claim>, String> {
    let saves_nothing: Vec<String> = keeps
        .iter()
        .filter(|row| row.short_value >= row.full_value)
        .map(|row| format!("`{}`", row.name))
        .collect();
    let quoted =
        |names: &[&str]| -> Vec<String> { names.iter().map(|name| format!("`{name}`")).collect() };
    let (untyped, stale) = (quoted(&measured.untyped), quoted(&measured.stale));
    let rules: [(&str, &[String], usize); 5] = [
        (
            "every path a brief keeps is a property of its record's answer schema, or runs \
             into a value the schema leaves open",
            &measured.unknown,
            measured.paths,
        ),
        (
            "no brief leaves out the tag a `oneOf` branch is told apart by",
            &measured.untagged,
            measured.paths,
        ),
        (
            "every brief that keeps fields answers its example's value in fewer tokens than \
             the full answer",
            &saves_nothing,
            keeps.len(),
        ),
        (
            "every brief that keeps fields belongs to a record whose answer states a schema",
            &untyped,
            keeps.len(),
        ),
        (
            "every record set aside here is a record, and refuses its example over the \
             built-in ephemeris",
            &stale,
            SET_ASIDE.len(),
        ),
    ];
    let claims: Vec<Claim> = rules
        .iter()
        .map(|(rule, wrong, of)| {
            let claim = Claim::counted(*rule, wrong.len(), *of);
            if wrong.is_empty() {
                claim
            } else {
                claim.with_note(wrong.join(", "))
            }
        })
        .collect();
    let falsified: Vec<&str> = claims
        .iter()
        .filter(|claim| claim.verdict == Verdict::Falsified)
        .map(|claim| claim.measured.as_str())
        .collect();
    if falsified.is_empty() {
        Ok(claims)
    } else {
        Err(format!(
            "a brief breaks its rules: {}",
            falsified.join(" | ")
        ))
    }
}

fn page(measured: &Measured) -> Result<String, String> {
    let rows = &measured.rows;
    let sum = |pick: fn(&Row) -> usize| rows.iter().map(pick).sum::<usize>();
    let (full, short) = (sum(|row| row.full), sum(|row| row.short));
    let (full_value, short_value) = (sum(|row| row.full_value), sum(|row| row.short_value));
    let (full_prov, short_prov) = (sum(|row| row.provenance.0), sum(|row| row.provenance.1));
    let keeps: Vec<&Row> = rows
        .iter()
        .filter(|row| matches!(row.brief, Brief::Keeps(_)))
        .collect();
    let claims = claims(measured, &keeps)?;
    let mut out = String::from("# Brief and full answers, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask detail-tokens` and held by \
         `check-detail-tokens`; do not edit. Each record's example request \
         (`crates/sdk/tests/requests.json`) answered over the built-in ephemeris at both \
         details (`result-detail.md`), each written as the envelope an agent reads \
         (`Answered::envelope`, canonical JSON: what the agent server sends as a tool's \
         text), and counted by the dependency-free approximation of `xtask/src/tokens.rs` \
         (C1165), the ruler `dossier-tokens-measured.md` describes: read the columns \
         against each other, not as a model's bill. An agent server's result carries the \
         same JSON twice, as text and as structured content, so a host that reads both \
         pays each figure twice.\n\n",
    );
    let _ = write!(
        out,
        "## The claims\n\n{}\n## Over {} records\n\n\
         | | full | brief | brief against full |\n|---|---|---|---|\n\
         | the envelope | {} | {} | {} |\n\
         | the value | {} | {} | {} |\n\
         | the provenance | {} | {} | {} |\n\n\
         {} of the {} records answered keep fields in brief; the rest answer their whole \
         value, and every one writes the brief provenance (`Brief::PROVENANCE`).\n\n",
        table(&claims),
        rows.len(),
        count(full),
        count(short),
        share(short, full),
        count(full_value),
        count(short_value),
        share(short_value, full_value),
        count(full_prov),
        count(short_prov),
        share(short_prov, full_prov),
        keeps.len(),
        rows.len(),
    );
    if !measured.open.is_empty() {
        let _ = write!(
            out,
            "{} of the {} paths run into a value its schema leaves open, a section the \
             answer carries as any JSON; what lies below is the section's own, unchecked \
             here: {}.\n\n",
            measured.open.len(),
            measured.paths,
            measured.open.join(", "),
        );
    }
    if !measured.set_aside.is_empty() {
        out.push_str("Set aside:\n\n");
        for (name, why) in &measured.set_aside {
            let _ = writeln!(out, "- `{name}`: {why}.");
        }
        out.push('\n');
    }
    out.push_str(
        "## Each record\n\nTokens of the envelope at each detail, and what the brief keeps \
         of the value.\n\n| record | full | brief | brief against full | the brief keeps |\n\
         |---|---|---|---|---|\n",
    );
    for row in rows {
        let _ = writeln!(
            out,
            "| `{}` | {} | {} | {} | {} |",
            row.name,
            count(row.full),
            count(row.short),
            share(row.short, row.full),
            kept(row.brief),
        );
    }
    appendix(&mut out, measured);
    Ok(fill(&out))
}

/// What each brief leaves out, and the fields of the larger answers.
fn appendix(out: &mut String, measured: &Measured) {
    out.push_str(
        "\n## What each brief leaves out\n\nEvery property the record's answer schema \
         states beside a path its brief keeps and the brief does not keep, read from the \
         schema rather than from the example, so a field the example leaves empty is \
         listed too. A field added to an answer is left out of its brief until its \
         record's list names it.\n\n",
    );
    for (name, left) in &measured.left_out {
        let listed: Vec<String> = left.iter().map(|path| format!("`{path}`")).collect();
        let _ = writeln!(out, "- `{name}`: {}.", listed.join(", "));
    }
    let _ = write!(
        out,
        "\n## The fields of the larger answers\n\nEach record whose full answer is \
         {LISTED_FROM} tokens or more, its value's top-level fields (an answer that is a \
         list, its items' fields summed) with their tokens, largest first; a field the \
         brief keeps any of is marked.\n\n"
    );
    for row in measured.rows.iter().filter(|row| row.full >= LISTED_FROM) {
        let listed: Vec<String> = row
            .fields
            .iter()
            .map(|(name, spent, kept)| {
                format!(
                    "`{name}` {}{}",
                    count(*spent),
                    if *kept { " (kept)" } else { "" }
                )
            })
            .collect();
        let _ = writeln!(out, "- `{}`: {}.", row.name, listed.join(", "));
    }
}

fn outputs(root: &Path) -> Result<Vec<Output>, String> {
    Ok(vec![Output::new(PAGE, page(&measure(root)?)?)])
}

pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => write(root, &outputs),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok(outputs) => i32::from(check(root, &outputs, "cargo xtask detail-tokens") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
