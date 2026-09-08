//! The falsification pass over the chart document's shape, which a JSON
//! Schema for it is designed from (`serial-and-the-envelope.md` §8).
//!
//! A schema is a set of **claims** about a document: which keys are
//! required, what type each value has, which strings are drawn from a
//! fixed list, what may be null. The project's rule is to derive such
//! claims from real values rather than propose them
//! (`03-design/varga-tables-measured.md`), so this pass builds the
//! documents the chart layer really produces and measures them.
//!
//! The one question it exists to answer is **where the schema should
//! come from**: the documents, the Rust types, or the API description.
//! The measurement decides it, and the answer is not the documents.
//!
//! `cargo xtask schema` writes the page; `check-schema` regenerates it in
//! memory and fails on any difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde_json::Value;
use teistro_core::envelope::CANONICAL_DECIMALS;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/schema-measured.md";
/// Where the sample documents are written, and read back from.
const DOCUMENTS: &str = "target/documents";

/// The crates whose types the document is made of. Every one of them
/// answers the question "can a consumer read this back".
const LAYER: [&str; 8] = [
    "serial",
    "chart",
    "panchanga",
    "vargas",
    "state",
    "aspect",
    "points",
    "houses",
];

/// The samples the example writes, in the order the page reports them:
/// widest first, because the widest is what a schema must cover.
const SAMPLES: [(&str, &str); 3] = [
    ("whole", "every section the layer can produce"),
    ("day", "a foundation and the almanac of its day"),
    ("bare", "a foundation alone, the smallest document there is"),
];

/// One sample: its name, the document parsed, the canonical bytes the
/// layer wrote, and the canonical bytes of what those parse to. The last
/// two are written by the example, because only `teistro-serial` can
/// write the form and this crate does not depend on it.
type Sample = (&'static str, Value, String, String);

// ── the sample documents ───────────────────────────────────────────────────

/// Builds the sample documents and reads them back.
///
/// They are built rather than recorded, because a schema has to describe
/// what the layer produces *now*: a recorded sample would go stale the
/// first time a section gained a field, and the pass would not notice.
fn documents(root: &Path) -> Result<Vec<Sample>, String> {
    let into = root.join(DOCUMENTS);
    let built = Command::new(crate::binding::cargo())
        .args([
            "run",
            "--quiet",
            "-p",
            "teistro-serial",
            "--example",
            "documents",
            "--",
        ])
        .arg(&into)
        .current_dir(root)
        .status()
        .map_err(|err| format!("the sample documents could not be built: {err}"))?;
    if !built.success() {
        return Err(String::from("the sample documents could not be built"));
    }
    let mut found = Vec::new();
    for (name, _) in SAMPLES {
        let read = |file: String| -> Result<String, String> {
            let path = into.join(file);
            std::fs::read_to_string(&path).map_err(|err| format!("{}: {err}", path.display()))
        };
        let text = read(format!("{name}.json"))?;
        let again = read(format!("{name}.again.json"))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|err| format!("{name}.json: {err}"))?;
        found.push((name, value, text, again));
    }
    Ok(found)
}

/// What a JSON value is, in the words a schema uses.
fn kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(number) if number.is_f64() => "number",
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Every path in a document, with the types and the string values seen at
/// it. An array index is collapsed to `[]`, because a schema describes
/// the items of an array and not the third of them.
fn paths(
    value: &Value,
    at: &str,
    into: &mut BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>,
) {
    match value {
        Value::Object(fields) => {
            for (key, inner) in fields {
                paths(inner, &format!("{at}.{key}"), into);
            }
        }
        Value::Array(items) => {
            let at = format!("{at}[]");
            for inner in items {
                paths(inner, &at, into);
            }
        }
        _ => {
            let entry = into.entry(at.to_string()).or_default();
            entry.0.insert(kind(value));
            if let Value::String(text) = value {
                entry.1.insert(text.clone());
            }
        }
    }
}

/// Every path of every sample, merged.
fn merged(docs: &[Sample]) -> BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)> {
    let mut all = BTreeMap::new();
    for (_, value, _, _) in docs {
        paths(value, "", &mut all);
    }
    all
}

// ── what the source says ───────────────────────────────────────────────────

/// How many types of a crate derive each half of the round trip.
fn derives(root: &Path, crate_name: &str) -> (usize, usize) {
    let mut serialise = 0;
    let mut read_back = 0;
    let mut files = Vec::new();
    collect_rust(
        &root.join("crates").join(crate_name).join("src"),
        &mut files,
    );
    for file in files {
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        for line in text.lines() {
            let line = line.trim();
            if !line.contains("derive(") {
                continue;
            }
            if line.contains("Serialize") {
                serialise += 1;
            }
            if line.contains("Deserialize") {
                read_back += 1;
            }
        }
    }
    (serialise, read_back)
}

/// Every `.rs` file under a directory.
fn collect_rust(directory: &Path, into: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rust(&path, into);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            into.push(path);
        }
    }
}

/// The casing conventions the layer's own types serialise under, counted
/// from the source: a schema's `enum` has to spell a member the way the
/// document really writes it.
fn casings(root: &Path) -> BTreeMap<String, usize> {
    let mut found: BTreeMap<String, usize> = BTreeMap::new();
    let mut files = Vec::new();
    for crate_name in LAYER.iter().chain(["core", "calendar", "astro"].iter()) {
        collect_rust(
            &root.join("crates").join(crate_name).join("src"),
            &mut files,
        );
    }
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        for line in text.lines() {
            let Some(rest) = line.trim().split("rename_all = \"").nth(1) else {
                continue;
            };
            if let Some(name) = rest.split('"').next() {
                *found.entry(name.to_string()).or_default() += 1;
            }
        }
    }
    found
}

// ── the page ───────────────────────────────────────────────────────────────

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask schema") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let docs = documents(root)?;
    let all = merged(&docs);
    let sections = [
        header(&docs, &all),
        shape(&docs, &all),
        numbers(&docs, &all),
        strings(&all),
        optional(&docs),
        nullable(&all),
        round_trip(root),
        spelling(root),
        fixed_point(&docs),
        decides(root, &all, &docs),
    ];
    Ok(fill(&sections.concat()))
}

/// The largest magnitude a number reaches in a document, and the
/// decimals an `f64` can actually carry there.
///
/// The canonical grammar writes a fixed twelve. That is inside an
/// `f64`'s resolution for a longitude and outside it for a Julian day,
/// which is four orders of magnitude larger.
fn magnitudes(value: &Value, worst: &mut f64, past: &mut usize) {
    match value {
        Value::Object(fields) => {
            for inner in fields.values() {
                magnitudes(inner, worst, past);
            }
        }
        Value::Array(items) => {
            for inner in items {
                magnitudes(inner, worst, past);
            }
        }
        Value::Number(number) => {
            if let Some(as_f64) = number.as_f64() {
                *worst = worst.max(as_f64.abs());
                if resolved_decimals(as_f64) < f64::from(CANONICAL_DECIMALS) {
                    *past += 1;
                }
            }
        }
        _ => {}
    }
}

/// The decimals an `f64` still resolves at a value's magnitude: the
/// exponent of one unit in the last place.
fn resolved_decimals(value: f64) -> f64 {
    let magnitude = value.abs();
    if magnitude == 0.0 {
        return f64::from(CANONICAL_DECIMALS);
    }
    -(f64::EPSILON * magnitude).log10().floor()
}

/// Whether writing what the form wrote gives the same bytes.
///
/// This is the invariant the content hash rests on: a consumer that
/// reads a stored document and hashes it again must get the producer's
/// hash. It is measured here rather than in `serial`'s own pass because
/// only a document assembled from the whole layer carries a Julian day.
fn fixed_point(docs: &[Sample]) -> String {
    let mut rows = String::new();
    for (name, value, text, again) in docs {
        let mut worst = 0.0_f64;
        let mut past = 0;
        magnitudes(value, &mut worst, &mut past);
        let _ = writeln!(
            rows,
            "| `{name}` | {worst:.0} | {:.0} | {past} | {} |",
            resolved_decimals(worst),
            if again == text {
                "holds"
            } else {
                "**falsified**"
            }
        );
    }
    format!(
        "## 9. The form is not a fixed point where the numbers are large

         The content hash rests on one invariant: a consumer that reads a
         stored document and hashes it again gets the producer's hash. The
         grammar writes every number to {CANONICAL_DECIMALS} decimals,
         which is inside an `f64`'s resolution for a longitude and outside
         it for a Julian day — four orders of magnitude larger, where one
         unit in the last place is already about 5e-10.

| sample | largest number | decimals resolved there | numbers written past it | writing it twice |
|---|---|---|---|---|
{rows}
         The three digits past the resolution are the decimal expansion of
         a binary value, not information, and they do not survive a parse:
         `2460483.108666389249` is written, read, and written again as
         `2460483.108666389715`.

         Whether a given value survives is a coin toss, which is why the
         smallest sample holding does not make it safe: a foundation alone
         carries a handful of instants and happens to win every toss, and
         a document with an almanac in it carries two hundred and loses.
         The finding is not that a large document fails but that any
         document may, and one that does is one whose stored hash a reader
         cannot reproduce.

         `serial-measured.md` asserted this invariant and found it held,
         over the corpus's recorded documents — whose numbers are
         longitudes and speeds, all under 360. A chart document carries the
         instant it was cast for, and that is where the grammar runs out.

         The fix is a decision rather than a patch, because it moves the
         hash of every document: a shortest-round-trip decimal, which Rust
         and JavaScript already agree on, rendered without an exponent as
         this grammar already renders one. It belongs to the design page.

"
    )
}

fn header(
    docs: &[Sample],
    all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>,
) -> String {
    format!(
        "# The chart document's shape, measured\n\n\
         Status: `generated` by `cargo xtask schema`. Do not edit:\n\
         `check-schema` regenerates this page and fails on any difference.\n\
         The design it is written for is\n\
         [`serial-and-the-envelope.md`](serial-and-the-envelope.md) §8.\n\n\
         ## 1. What can be measured about a schema\n\n\
         A JSON Schema is a set of claims about a document: which keys are\n\
         required, what type each value has, which strings come from a\n\
         fixed list, what may be null. Nothing is recorded that a schema\n\
         could be compared against, so this is the `aspect` kind of pass\n\
         rather than the `vargas` kind — it measures the document's own\n\
         shape over real values, and reads the source for what the values\n\
         cannot say about themselves.\n\n\
         The sample is built rather than recorded, by\n\
         `cargo run -p teistro-serial --example documents`: {} documents\n\
         over the analytic test provider, {} distinct paths between them.\n\
         A recorded sample would go stale the first time a section gained a\n\
         field and the pass would not notice.\n\n\
         The question the pass exists to answer is **where the schema\n\
         should come from** — the documents, the Rust types, or the API\n\
         description. Sections 3 and 4 decide it.\n\n",
        count(docs.len()),
        count(all.len()),
    )
}

fn shape(
    docs: &[Sample],
    all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>,
) -> String {
    let mut out = String::from(
        "## 2. The documents\n\n| sample | what it holds | sections | paths |\n|---|---|---|---|\n",
    );
    for (name, description) in SAMPLES {
        let value = docs.iter().find(|(n, ..)| *n == name).map(|(_, v, ..)| v);
        let sections = value
            .and_then(Value::as_object)
            .map_or(0, serde_json::Map::len);
        let mut own = BTreeMap::new();
        if let Some(value) = value {
            paths(value, "", &mut own);
        }
        let _ = writeln!(
            out,
            "| `{name}` | {description} | {sections} | {} |",
            own.len()
        );
    }
    let mut kinds: BTreeMap<&str, usize> = BTreeMap::new();
    for (types, _) in all.values() {
        for kind in types {
            *kinds.entry(kind).or_default() += 1;
        }
    }
    let spread = kinds
        .iter()
        .map(|(kind, n)| format!("{n} {kind}"))
        .collect::<Vec<_>>()
        .join(", ");
    let _ = write!(
        out,
        "\nAcross all three, by the type a schema would give the value:\n{spread}.\n\n"
    );
    out
}

fn numbers(
    docs: &[Sample],
    all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>,
) -> String {
    let numeric: Vec<(&String, &BTreeSet<&'static str>)> = all
        .iter()
        .filter(|(_, (types, _))| types.contains("number") || types.contains("integer"))
        .map(|(path, (types, _))| (path, types))
        .collect();
    let integer_only: Vec<&String> = numeric
        .iter()
        .filter(|(_, types)| types.len() == 1 && types.contains("integer"))
        .map(|(path, _)| *path)
        .collect();
    let both: Vec<&String> = numeric
        .iter()
        .filter(|(_, types)| types.contains("integer") && types.contains("number"))
        .map(|(path, _)| *path)
        .collect();
    let _ = docs;
    let mut out = String::from(
        "## 3. A whole double is written as an integer\n\n\
         The canonical form removes a trailing zero on purpose —\n\
         `to_hash_form(&2.0)` is `\"2\"` — because the hash has to be stable\n\
         and `2` and `2.0` are the same number. The consequence is that a\n\
         field the SDK holds as an `f64` appears in the document as a JSON\n\
         **integer** whenever its value is whole, and is then\n\
         indistinguishable from a field that really is a count.\n\n",
    );
    let _ = write!(
        out,
        "| numeric paths | integer in every sample | decimal somewhere | both, across samples |\n\
         |---|---|---|---|\n\
         | {} | {} | {} | {} |\n\n",
        numeric.len(),
        integer_only.len(),
        numeric.len() - integer_only.len() - both.len(),
        both.len()
    );
    if both.is_empty() {
        out.push_str(
            "No path is written both ways in this sample, which does not\n\
             mean none can be: it means the sample happens not to contain a\n\
             whole value where it also contains a fractional one.\n\n",
        );
    } else {
        let _ = write!(
            out,
            "The ambiguity is not theoretical. {} written both ways within\n\
             the same sample set:\n\n",
            crate::measure::plural(both.len(), "path is")
        );
        for path in &both {
            let _ = writeln!(out, "- `{path}`");
        }
        out.push('\n');
    }
    let _ = write!(
        out,
        "So a schema derived from the documents alone would type {} paths on\n\
         the evidence of a sample that cannot tell a count from a round\n\
         number. Some of them really are counts — a day of the month, a\n\
         bhava — and some are doubles that happened to land on a whole\n\
         value. Nothing in the JSON separates them.\n\n\
         **Only the Rust types know.** This is the pass's first answer to\n\
         where the schema comes from.\n\n",
        integer_only.len()
    );
    out
}

fn strings(all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>) -> String {
    let string_paths: Vec<(&String, &BTreeSet<String>)> = all
        .iter()
        .filter(|(_, (types, _))| types.contains("string"))
        .map(|(path, (_, values))| (path, values))
        .collect();
    let catalogued: Vec<&(&String, &BTreeSet<String>)> = string_paths
        .iter()
        .filter(|(_, values)| {
            !values.is_empty()
                && values.iter().all(|value| {
                    !value.is_empty()
                        && value
                            .chars()
                            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
                })
        })
        .collect();
    let widest = catalogued
        .iter()
        .map(|(_, values)| values.len())
        .max()
        .unwrap_or(0);
    format!(
        "## 4. Almost every string is a catalogue member\n\n\
         | string paths | drawn from the catalogue | free text |\n|---|---|---|\n| {} | {} | {} |\n\n\
         A schema would constrain each of those {} with an `enum`, and it\n\
         cannot get the members from the documents: the widest of them\n\
         shows {} values, where the catalogue's own list is longer for\n\
         every one. A sample proves a member exists; it never proves a\n\
         member does not.\n\n\
         **Only the description knows the full list.** This is the pass's\n\
         second answer, and it points at the same place as the first: the\n\
         schema is generated from what the SDK already describes, beside\n\
         the C header, the TypeScript surface, the Dart classes and the\n\
         Python declarations, and not from a sample.\n\n",
        string_paths.len(),
        catalogued.len(),
        string_paths.len() - catalogued.len(),
        catalogued.len(),
        widest,
    )
}

fn optional(docs: &[Sample]) -> String {
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for (_, value, _, _) in docs {
        let mut own = BTreeMap::new();
        paths(value, "", &mut own);
        for path in own.keys() {
            *seen.entry(path.clone()).or_default() += 1;
        }
    }
    let always = seen.values().filter(|n| **n == docs.len()).count();
    let sometimes = seen.len() - always;
    let top: Vec<String> = docs
        .iter()
        .find(|(name, ..)| *name == "whole")
        .and_then(|(_, value, ..)| value.as_object())
        .map(|fields| fields.keys().cloned().collect())
        .unwrap_or_default();
    format!(
        "## 5. What is required, measured\n\n\
         Every section but the foundation carries a `skip_serializing_if`,\n\
         so a Rust `Option` and an absent JSON key do not line up one to\n\
         one and a schema's `required` cannot be read off the struct. Over\n\
         the three samples:\n\n\
         | paths in every sample | paths in some | top-level sections |\n|---|---|---|\n| {always} | {sometimes} | {} |\n\n\
         The top-level sections of the widest document are {}. Only\n\
         `foundation` is in all three, which is what the module says it\n\
         intends; the measurement agrees with the intention here rather\n\
         than contradicting it.\n\n",
        top.len(),
        top.iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", "),
    )
}

fn nullable(all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>) -> String {
    let nulls: Vec<&String> = all
        .iter()
        .filter(|(_, (types, _))| types.contains("null"))
        .map(|(path, _)| path)
        .collect();
    let mut out = format!(
        "## 6. What may be null\n\n\
         {} carry a null in the sample, so a schema has to admit one\n\
         there and a consumer has to expect it:\n\n",
        crate::measure::plural(nulls.len(), "path")
    );
    for path in &nulls {
        let _ = writeln!(out, "- `{path}`");
    }
    out.push_str(
        "\nA null here is a real answer and not a missing one — no war, no\n\
         sankranti that day, no combustion orbs for a body that cannot be\n\
         combust — which is why the field is written rather than skipped.\n\n",
    );
    out
}

fn round_trip(root: &Path) -> String {
    let mut rows = String::new();
    let mut serialise_total = 0;
    let mut read_back_total = 0;
    for crate_name in LAYER {
        let (serialise, read_back) = derives(root, crate_name);
        serialise_total += serialise;
        read_back_total += read_back;
        let _ = writeln!(rows, "| `{crate_name}` | {serialise} | {read_back} |");
    }
    format!(
        "## 7. Nothing reads back\n\n\
         A schema's most valuable consumer is the SDK itself: a stored\n\
         document is worth validating precisely because something will try\n\
         to read it later. Counting the derives over the layer's own\n\
         source:\n\n\
         | crate | types that serialise | types that read back |\n|---|---|---|\n{rows}\
         | **total** | **{serialise_total}** | **{read_back_total}** |\n\n\
         This is the same shape the previous pass found and the opposite\n\
         end of it. `serial-measured.md` found that `ChartFoundation`\n\
         derived no `Serialize`, so the SDK could not publish a chart; that\n\
         was fixed, and the SDK can now publish one it cannot read.\n\n\
         It matters for the schema rather than merely being untidy. The\n\
         gate a schema wants is *every sample validates, and every sample\n\
         reads back equal* — the second half of which cannot be written at\n\
         all today, so a schema shipped now would be a claim nothing checks\n\
         from the inside.\n\n",
    )
}

fn spelling(root: &Path) -> String {
    let found = casings(root);
    let total: usize = found.values().sum();
    let mut rows = String::new();
    for (convention, n) in &found {
        let _ = writeln!(rows, "| `{convention}` | {n} |");
    }
    format!(
        "## 8. Two spellings, in one document\n\n\
         A schema's `enum` has to spell a member the way the document\n\
         really writes it. Counting `rename_all` over the layer and the\n\
         crates it holds values from, {total} types declare one:\n\n\
         | convention | types |\n|---|---|\n{rows}\n\
         The minority is not unreached: `CalendarResolution` is one of\n\
         them, and it appears in every document there is — a chart's\n\
         foundation carries the resolution of its own date. So a consumer\n\
         reading one document meets both conventions, and a generated\n\
         schema must take the spelling from each type rather than assume\n\
         the majority's.\n\n",
    )
}

fn decides(
    root: &Path,
    all: &BTreeMap<String, (BTreeSet<&'static str>, BTreeSet<String>)>,
    docs: &[Sample],
) -> String {
    let (_, read_back) = LAYER
        .iter()
        .map(|name| derives(root, name))
        .fold((0, 0), |(a, b), (c, d)| (a + c, b + d));
    let moved = docs
        .iter()
        .filter(|(_, _, text, again)| text != again)
        .count();
    let numeric = all
        .values()
        .filter(|(types, _)| types.contains("number") || types.contains("integer"))
        .count();
    let ambiguous = all
        .values()
        .filter(|(types, _)| types.len() == 1 && types.contains("integer"))
        .count();
    let claims = [
        Claim {
            rule: String::from("the schema can be derived from the documents"),
            verdict: verdict_of(ambiguous == 0),
            measured: format!("{ambiguous} of {numeric} numeric paths are ambiguous"),
        },
        Claim {
            rule: String::from("a sample gives a string field its full member list"),
            verdict: Verdict::Falsified,
            measured: String::from("a sample proves a member exists, never that one does not"),
        },
        Claim {
            rule: String::from("the layer's types read back, so a round trip can gate the schema"),
            verdict: verdict_of(read_back > 0),
            measured: format!("{read_back} types derive `Deserialize`"),
        },
        Claim {
            rule: String::from("one casing convention covers every enum in a document"),
            verdict: verdict_of(casings(root).len() <= 1),
            measured: format!("{} conventions declared", casings(root).len()),
        },
        Claim {
            rule: String::from("the canonical form of a document is a fixed point"),
            verdict: verdict_of(moved == 0),
            measured: format!("{moved} of {} samples move when written twice", docs.len()),
        },
    ];
    let falsified = claims
        .iter()
        .filter(|claim| matches!(claim.verdict, Verdict::Falsified))
        .count();
    format!(
        "## 10. What this decides\n\n{}\n\n\
         The measurement falsifies {} of the {} proposed rules. The first\n\
         four say the same thing about **where** a schema comes from: the\n\
         description, beside the other four surfaces, and not a sample nor\n\
         a derive macro over the Rust types. The description is the only\n\
         place that has the member lists, and the only place that cannot\n\
         disagree with what the bindings already say.\n\n\
         The fifth says something about **when**. A schema describes a\n\
         document a consumer will store and read back, and two of the\n\
         three samples do not survive being read back and written again,\n\
         so the bytes a schema would describe are not yet stable. The\n\
         round trip is the other half of that: until the layer's values\n\
         derive `Deserialize`, the schema's natural gate — every sample\n\
         validates and reads back equal — cannot be written at all.\n\n\
         Both are the design page's questions rather than this pass's, and\n\
         both come before an emitter.\n\n",
        table(&claims),
        falsified,
        claims.len(),
    )
}
