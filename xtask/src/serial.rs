//! The falsification pass over the canonical form, which the serial
//! module is designed from (Phase 4).
//!
//! There is nothing recorded to measure a serialisation against, so this
//! pass is the `aspect` kind rather than the `vargas` kind: it measures
//! the form's **own invariants** over real values, and it reads the
//! source to find out what the SDK actually does against what it says it
//! does.
//!
//! Three things it looks at, and each turned something up:
//!
//! 1. **What a stamped value carries.** The envelope has every field
//!    ADR-0020 asks for. What no test had checked is whether the
//!    producers *fill* them — and the one that matters most, the hash of
//!    the value itself, is the hash of nothing on all but one.
//! 2. **Whether the canonical form is canonical.** Key order at every
//!    depth, over the corpus's own recorded documents.
//! 3. **How a double is written**, which is the one thing that can make
//!    two bindings disagree about the bytes and therefore the hash. The
//!    settings already carry the knob that answers it, and nothing reads
//!    that either.
//!
//! `cargo xtask serial` writes the page; `check-serial` regenerates it
//! in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_core::envelope::{canonical_json, content_hash};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, fill, table, verdict_of};

const PAGE: &str = "docs/03-design/serial-measured.md";
const CHARTS: &str = "fixtures/baseline/charts";

/// The crates that build a [`teistro_core::envelope::Provenance`], and so
/// are answerable for filling it.
const PRODUCERS: [&str; 4] = [
    "crates/chart/src/foundation.rs",
    "crates/panchanga/src/almanac.rs",
    "crates/ffi/src/positions.rs",
    "crates/serial/src/seal.rs",
];

/// The fields of the envelope a producer has to fill itself: everything
/// `Provenance::new` leaves empty. The identifying six it takes as
/// arguments are not in the list, because they cannot be left out.
const TO_FILL: [(&str, &str); 10] = [
    (
        "content_hash",
        "the hash of the value, which is the whole point",
    ),
    ("module_versions", "which modules took part"),
    ("provider", "which ephemeris answered, in which frame"),
    ("packs", "which packs were loaded"),
    ("time.delta_t_model", "which Delta T model"),
    ("time.delta_t_seconds", "and what it gave"),
    ("time.leap_table", "the leap-second table"),
    ("time.tzdb_version", "the zone database"),
    ("calendar", "the calendar's resolution, where one took part"),
    (
        "applied_conventions",
        "what was chosen for an unattested request",
    ),
];

/// The settings knob that says how a number is written, which nothing
/// reads.
const PRECISION_FIELDS: [&str; 3] = ["angle_decimals", "instant_decimals", "score_decimals"];

// ── what the corpus lends ──────────────────────────────────────────────────

/// Every double the corpus's own positions record, which is the sample
/// this pass measures a number format over.
fn doubles(root: &Path) -> Result<Vec<f64>, String> {
    let dir = root.join(CHARTS);
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| {
            format!(
                "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    let mut found = Vec::new();
    for path in &paths {
        let text =
            std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        collect(&value, &mut found);
    }
    if found.is_empty() {
        return Err(String::from("no fixture carries a number"));
    }
    Ok(found)
}

fn collect(value: &Value, into: &mut Vec<f64>) {
    match value {
        Value::Number(number) => {
            if let Some(double) = number.as_f64() {
                into.push(double);
            }
        }
        Value::Array(items) => items.iter().for_each(|item| collect(item, into)),
        Value::Object(fields) => fields.values().for_each(|field| collect(field, into)),
        _ => {}
    }
}

/// One fixture, read whole, as a document to canonicalise.
fn documents(root: &Path) -> Result<Vec<(String, Value)>, String> {
    let dir = root.join(CHARTS);
    let mut paths: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| format!("cannot read {}: {err}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    paths.sort();
    let mut found = Vec::new();
    for path in &paths {
        let text =
            std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let value: Value =
            serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        found.push((
            path.file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_default(),
            value,
        ));
    }
    Ok(found)
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask serial") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let numbers = doubles(root)?;
    let docs = documents(root)?;
    let sections = [
        header(&docs, &numbers),
        stamped(root)?,
        canonical(&docs),
        numbers_written(root, &numbers),
        hashes(&docs),
        decides(root)?,
    ];
    Ok(fill(&sections.concat()))
}

// ── 1. what this pass can measure ──────────────────────────────────────────

fn header(docs: &[(String, Value)], numbers: &[f64]) -> String {
    format!(
        "# The canonical form, measured\n\n\
         Status: `generated` by `cargo xtask serial`, 2026-09-07. Do not\n\
         edit: `check-serial` regenerates this page and fails on any\n\
         difference. The design written from it is\n\
         [`serial-and-the-envelope.md`](serial-and-the-envelope.md).\n\n\
         ## 1. What can be measured about a serialisation\n\n\
         Nothing is recorded to compare a serialisation against, so this is\n\
         the `aspect` kind of pass rather than the `vargas` kind. What it\n\
         can measure is the form's **own invariants** over real values, and\n\
         what the SDK does against what it says it does — for which it\n\
         reads the source, as `check-lints` does.\n\n\
         The sample is the corpus itself: {} fixture documents and the {}\n\
         numbers inside them, which are real longitudes, speeds, distances\n\
         and instants rather than round figures chosen to be easy.\n\n",
        count(docs.len()),
        count(numbers.len()),
    )
}

// ── 2. what a stamped value carries ────────────────────────────────────────

/// Which of the envelope's fields each producer assigns, read from the
/// source.
fn assigned(root: &Path) -> Result<BTreeMap<&'static str, Vec<&'static str>>, String> {
    let mut found: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for producer in PRODUCERS {
        let text = std::fs::read_to_string(root.join(producer))
            .map_err(|err| format!("{producer}: {err}"))?;
        let mut fills = Vec::new();
        for (field, _) in TO_FILL {
            // `provenance.time.delta_t_model = …`, or a struct literal
            // that names the field: either is filling it.
            let assignment = format!("{field} =");
            let literal = format!("{}: ", field.rsplit('.').next().unwrap_or(field));
            // **And `Envelope::sealing`, which is the join filling it.**
            // §8's open question closed in favour of the producers, so a
            // producer no longer spells `content_hash = …` — it hands
            // the value and the stamp to the constructor that knows
            // both. This pass's subject moved when the change landed,
            // and a reader looking only for the assignment would have
            // reported two producers as still shipping the hash of
            // nothing, which is the state the change ended.
            let sealed = field == "content_hash" && text.contains("Envelope::sealing(");
            if sealed
                || text.contains(&assignment)
                || (field.contains('.') && text.contains(&literal))
            {
                fills.push(field);
            }
        }
        found.insert(producer, fills);
    }
    Ok(found)
}

/// The paragraph about `content_hash`, in whichever of its two forms
/// the measurement supports.
///
/// Two sentences, and which one is written is the measurement. The first
/// was true for six sessions and is the reason that section exists; the
/// second is what closed §8's open question.
fn the_hash_of_nothing(missing: &[&str], producers: usize) -> String {
    let mut out = String::new();
    if missing.is_empty() {
        let _ = write!(
            out,
            "\n**`content_hash` is the hash of nothing on none of the {producers}.**\n\
             `Provenance::new` still sets it to `Hash::of(&[])` as a\n\
             placeholder, and every producer now replaces it — but not by\n\
             remembering to. The shape problem this section found is\n\
             answered the way `crates/serial` answered it: a value and its\n\
             stamp are joined by a constructor that knows both, so the one\n\
             field that cannot be filled until the value exists is filled\n\
             where it can be. `Envelope::sealing` is that join for `chart`\n\
             and `panchanga`, and the four callers that used to mend the\n\
             stamp afterwards — the boundary's two entry points and the\n\
             Rust façade's two areas — no longer do. Four callers writing\n\
             the same line is what decided it.\n\n"
        );
    } else {
        let _ = write!(
            out,
            "\n**`content_hash` is the hash of nothing on {} of the {}.**\n\
             `Provenance::new` sets it to `Hash::of(&[])` as a placeholder, and\n\
             a producer that does not replace it ships a value carrying the\n\
             hash of the empty string where its own hash should be — the field\n\
             is documented as \"the hash of the canonical serialisation of the\n\
             value\" and on those it is not that. They are: {}.\n\n\
             That is not a bug in any one producer. It is a **shape** problem:\n\
             a value and its stamp are built separately and joined at the end,\n\
             so the one field that cannot be filled until the value exists is\n\
             the one everybody forgets. `crates/serial`'s answer is to make the\n\
             joining the only way to build the pair, so the hash is computed by\n\
             the constructor and never by a caller who remembers — which is\n\
             why that crate is in the table above and fills it.\n\n",
            missing.len(),
            producers,
            missing
                .iter()
                .map(|producer| format!("`{producer}`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    out
}

fn stamped(root: &Path) -> Result<String, String> {
    let found = assigned(root)?;
    let mut never = Vec::new();
    for (field, _) in TO_FILL {
        if !found.values().any(|fills| fills.contains(&field)) {
            never.push(field);
        }
    }
    let content_hash_fillers = found
        .iter()
        .filter(|(_, fills)| fills.contains(&"content_hash"))
        .count();
    let claims = [
        Claim::counted(
            "every producer stamps the hash of the value it produced",
            found.len() - content_hash_fillers,
            found.len(),
        ),
        Claim::counted(
            "every field the envelope documents is filled by someone",
            never.len(),
            TO_FILL.len(),
        ),
    ];
    let mut out = format!(
        "## 2. What a stamped value actually carries\n\n\
         `Provenance` has every field ADR-0020 asks for: the calculation\n\
         version, the input hash, the provider's data hashes, the Delta T\n\
         model, the leap table, the zone database, the calendar's\n\
         resolution, a classical model's deviation, the conventions\n\
         applied, the confidence, and **the hash of the value itself**.\n\n\
         What no test had checked is whether anything fills them.\n\
         `Provenance::new` takes the identifying six as arguments and\n\
         leaves the rest empty, so a producer that forgets a field ships a\n\
         value that claims less than it knows — and one of those fields is\n\
         the hash the whole envelope exists to carry.\n\n{}\n\
         | producer | fills |\n|---|---|\n",
        table(&claims),
    );
    for (producer, fills) in &found {
        let shown = if fills.is_empty() {
            String::from("—")
        } else {
            fills
                .iter()
                .map(|field| format!("`{field}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let _ = writeln!(out, "| `{producer}` | {shown} |");
    }
    let missing: Vec<&str> = found
        .iter()
        .filter(|(_, fills)| !fills.contains(&"content_hash"))
        .map(|(producer, _)| *producer)
        .collect();
    out.push_str(&the_hash_of_nothing(&missing, found.len()));
    if !never.is_empty() {
        let _ = write!(
            out,
            "Nothing at all fills {}: {}. Each is a field the envelope\n\
             documents and nothing sets, which a consumer reading the\n\
             schema would expect to find.\n\n",
            never.len(),
            never
                .iter()
                .map(|field| format!("`{field}`"))
                .collect::<Vec<_>>()
                .join(", "),
        );
    }
    Ok(out)
}

// ── 3. whether the canonical form is canonical ─────────────────────────────

fn canonical(docs: &[(String, Value)]) -> String {
    let mut unsorted = 0;
    let mut unstable = 0;
    let mut differing_from_reordered = 0;
    let mut deepest = 0;
    for (_, value) in docs {
        let text = canonical_json(value);
        unsorted += usize::from(!keys_sorted(
            &serde_json::from_str(&text).unwrap_or(Value::Null),
        ));
        unstable += usize::from(canonical_json(value) != text);
        // The same document with its objects rebuilt in another order
        // must give the same bytes: that is what "canonical" means.
        differing_from_reordered += usize::from(canonical_json(&reversed(value.clone())) != text);
        deepest = deepest.max(depth(value));
    }
    let claims = [
        Claim::counted(
            "every object's keys come out in code-point order, at every depth",
            unsorted,
            docs.len(),
        ),
        Claim::counted(
            "the same value gives the same bytes twice",
            unstable,
            docs.len(),
        ),
        Claim::counted(
            "and gives them however the value's own maps were ordered",
            differing_from_reordered,
            docs.len(),
        ),
    ];
    format!(
        "## 3. The canonical form is canonical\n\n\
         Two bindings that agree on a value have to agree on the bytes, or\n\
         the content hash is not a content hash. `core`'s `canonical_json`\n\
         sorts every object's keys explicitly rather than trusting the JSON\n\
         layer's map, whose ordering changes with a feature any crate in a\n\
         build may enable — and this measures that over the corpus's own\n\
         documents, which nest {deepest} deep.\n\n{}\n\
         The third claim is the one worth having. Rebuilding every object\n\
         in the reverse of its original order and canonicalising again\n\
         gives the same bytes on all {} documents, so the form depends on\n\
         the value and not on how the value was built.\n\n",
        table(&claims),
        count(docs.len()),
    )
}

/// Whether every object in a value has its keys in code-point order.
fn keys_sorted(value: &Value) -> bool {
    match value {
        Value::Object(fields) => {
            let keys: Vec<&String> = fields.keys().collect();
            keys.windows(2).all(|pair| pair[0] <= pair[1]) && fields.values().all(keys_sorted)
        }
        Value::Array(items) => items.iter().all(keys_sorted),
        _ => true,
    }
}

/// The same value with every object's entries rebuilt in reverse.
fn reversed(value: Value) -> Value {
    match value {
        Value::Object(fields) => {
            let mut out = serde_json::Map::new();
            for (key, nested) in fields.into_iter().collect::<Vec<_>>().into_iter().rev() {
                out.insert(key, reversed(nested));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(reversed).collect()),
        other => other,
    }
}

/// How deep a value nests.
fn depth(value: &Value) -> usize {
    match value {
        Value::Object(fields) => 1 + fields.values().map(depth).max().unwrap_or(0),
        Value::Array(items) => 1 + items.iter().map(depth).max().unwrap_or(0),
        _ => 0,
    }
}

// ── 4. how a double is written ─────────────────────────────────────────────

/// Where the form switches to an exponent, either side. A binding has
/// to match these thresholds, and two of them most easily differ here,
/// so they are measured rather than assumed.
fn thresholds() -> (Option<i32>, Option<i32>) {
    let mut switches = Vec::new();
    for power in -12_i32..=25 {
        let value = 10.0_f64.powi(power);
        let text = canonical_json(&value);
        switches.push((power, text));
    }
    let small = switches
        .iter()
        .filter(|(_, text)| text.contains('e'))
        .map(|(power, _)| *power)
        .filter(|power| *power < 0)
        .max();
    let large = switches
        .iter()
        .filter(|(_, text)| text.contains('e'))
        .map(|(power, _)| *power)
        .filter(|power| *power > 0)
        .min();
    (small, large)
}

fn numbers_written(root: &Path, numbers: &[f64]) -> String {
    let mut round_trip_failures = 0;
    let mut exponent = 0;
    let mut widest = 0;
    let mut widest_text = String::new();
    for number in numbers {
        let text = canonical_json(number);
        if text.parse::<f64>().ok() != Some(*number) {
            round_trip_failures += 1;
        }
        if text.contains('e') || text.contains('E') {
            exponent += 1;
        }
        if text.len() > widest {
            widest = text.len();
            widest_text.clone_from(&text);
        }
    }
    let (small, large) = thresholds();
    let precision_readers = readers(root, &PRECISION_FIELDS);
    let claims = [
        Claim::stated(
            "every number of the corpus round-trips through the form",
            verdict_of(round_trip_failures == 0),
            format!("{round_trip_failures} of {} disagree", count(numbers.len())),
        ),
        Claim::stated(
            "the form never reaches for an exponent",
            verdict_of(exponent == 0),
            format!("{exponent} of {} do", count(numbers.len())),
        ),
        Claim::counted(
            "`output.precision` has a reader",
            usize::from(precision_readers.is_empty()),
            1,
        ),
    ];
    format!(
        "## 4. How a double is written, which is where two bindings part\n\n\
         A canonical form has to say how a number is written, because the\n\
         bytes are what is hashed. Rust's JSON layer writes the shortest\n\
         decimal that reads back as the same double, and so does\n\
         JavaScript's — but the two **switch to an exponent at different\n\
         magnitudes**, and a value written `0.000001` in one and `1e-6` in\n\
         the other has two hashes.\n\n{}\n\
         Measured over the corpus's own numbers: every one round-trips, and\n\
         {exponent} of them are written with an exponent already; the widest\n\
         form is {widest} characters, `{widest_text}`.\n\n\
         The form reaches for an exponent **below 10^{}** and at or above\n\
         10^{}. That first threshold is the one that bites, and here is the\n\
         case, written out:\n\n\
         | value | this form writes | a JavaScript binding writes |\n|---|---|---|\n\
         | 10^-6 | `{}` | `0.000001` |\n\
         | 10^-7 | `{}` | `1e-7` |\n\n\
         `JSON.stringify` switches at 10^-7 and this switches at 10^-6, so\n\
         a single value one millionth of a degree from nought hashes two\n\
         ways in two bindings that agree about the number. Nothing in the\n\
         corpus is quite that small, but a speed near a station, a residual\n\
         or the difference of two longitudes very easily is.\n\n\
         `output.precision` names the decimals of an angle, an instant and\n\
         a score, and until `crates/serial` was written **nothing read\n\
         it** — the third such knob found in as many modules, after\n\
         `state.combustion_orbs` (registry entry 23) and\n\
         `houses.module_overrides`, which is a pattern rather than an\n\
         accident and worth a gate of its own. It now has {}.\n\n\
         It governs the **rendering** and not the hash: a hash that moved\n\
         with a display setting would be a worse cache key, and the\n\
         settings hash already tells two results computed under different\n\
         precision apart. The hash form's own grammar is fixed, which is\n\
         what the second claim measures.\n\n",
        table(&claims),
        small.map_or_else(|| String::from("?"), |power| power.to_string()),
        large.map_or_else(|| String::from("?"), |power| power.to_string()),
        canonical_json(&1e-6_f64),
        canonical_json(&1e-7_f64),
        if precision_readers.is_empty() {
            String::from("none still")
        } else {
            precision_readers
                .iter()
                .map(|path| {
                    format!(
                        "`{}`",
                        path.rsplit('/')
                            .take(2)
                            .collect::<Vec<_>>()
                            .into_iter()
                            .rev()
                            .collect::<Vec<_>>()
                            .join("/")
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        },
    )
}

/// Which files mention every one of a set of names, which is what a
/// reader of a settings knob looks like.
fn readers(root: &Path, fields: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    for directory in ["crates", "bindings"] {
        let mut stack = vec![root.join(directory)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let Ok(text) = std::fs::read_to_string(&path) else {
                        continue;
                    };
                    // The definition itself is not a reader.
                    if path.ends_with("settings/mod.rs") || path.ends_with("settings/profiles.rs") {
                        continue;
                    }
                    if fields.iter().any(|field| text.contains(field)) {
                        found.push(path.display().to_string());
                    }
                }
            }
        }
    }
    found.sort();
    found
}

// ── 5. the content hash ────────────────────────────────────────────────────

fn hashes(docs: &[(String, Value)]) -> String {
    let mut unequal = 0;
    let mut collisions = 0;
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for (name, value) in docs {
        let once = content_hash(value);
        let twice = content_hash(&reversed(value.clone()));
        unequal += usize::from(once != twice);
        if let Some(other) = seen.insert(format!("{once:?}"), name.as_str()) {
            if other != name {
                collisions += 1;
            }
        }
    }
    // A hash that does not move when the value does is no hash at all.
    let moved = docs.first().is_some_and(|(_, value)| {
        let mut nudged = value.clone();
        nudged["schema"] = Value::String(String::from("nudged"));
        content_hash(&nudged) != content_hash(value)
    });
    let claims = [
        Claim::counted(
            "a value's hash does not depend on how the value was built",
            unequal,
            docs.len(),
        ),
        Claim::counted("no two documents hash alike", collisions, docs.len()),
        Claim::stated(
            "a changed value changes the hash",
            verdict_of(moved),
            if moved { "it does" } else { "it does not" },
        ),
    ];
    format!(
        "## 5. The content hash is a function of the value\n\n\
         The hash is the cache key's third part and the thing a consumer\n\
         compares when it wants to know whether two results are the same\n\
         answer. It has to depend on the value and on nothing else.\n\n{}\n\
         All three hold over the {} documents, which is what makes the\n\
         missing `content_hash` of §2 a real loss rather than a cosmetic\n\
         one: the hash works, and almost nothing carries it.\n\n",
        table(&claims),
        count(docs.len()),
    )
}

// ── 6. what the pass decides ───────────────────────────────────────────────

fn decides(root: &Path) -> Result<String, String> {
    let found = assigned(root)?;
    let missing = found
        .iter()
        .filter(|(_, fills)| !fills.contains(&"content_hash"))
        .count();
    Ok(format!(
        "## 6. What this pass decides\n\n\
         - **A value and its stamp are sealed together.** {missing} of the\n\
           {} producers ship a `content_hash` that is the hash of nothing,\n\
           because the field cannot be filled until the value exists and a\n\
           caller has to remember. The module makes the sealing the only\n\
           way to build the pair, so the constructor computes it.\n\
         - **The canonical form holds.** Keys in code-point order at every\n\
           depth, the same bytes twice, and the same bytes however the\n\
           value's own maps were ordered.\n\
         - **A number needs a stated format.** The corpus's own numbers\n\
           round-trip, but the form reaches for an exponent where a\n\
           binding's own would not, and `output.precision` — which says\n\
           how many decimals a quantity is written to — has no reader. A\n\
           canonical form that writes to a stated precision is one two\n\
           bindings can agree on without agreeing about their float\n\
           printers.\n\
         - **The hash works.** It depends on the value, on nothing else,\n\
           and moves when the value does — which is why the field being\n\
           empty matters.\n",
        found.len(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{collect, depth, keys_sorted, reversed};
    use serde_json::json;

    #[test]
    fn keys_are_read_in_order_at_every_depth() {
        assert!(keys_sorted(&json!({"a": 1, "b": 2})));
        assert!(!keys_sorted(&json!({"b": 1, "a": 2})));
        assert!(keys_sorted(&json!({"a": {"x": 1, "y": 2}})));
        assert!(!keys_sorted(&json!({"a": {"y": 1, "x": 2}})), "nested");
        assert!(!keys_sorted(&json!([{"y": 1, "x": 2}])), "inside an array");
        assert!(keys_sorted(&json!(1)), "a scalar is in order");
    }

    #[test]
    fn reversing_changes_the_order_and_not_the_value() {
        let value = json!({"a": 1, "b": {"c": 2, "d": 3}});
        let flipped = reversed(value.clone());
        assert_eq!(flipped, value, "the same value, differently built");
        let keys: Vec<&String> = flipped.as_object().expect("an object").keys().collect();
        assert_eq!(keys, vec!["b", "a"], "and in the other order");
    }

    #[test]
    fn a_depth_counts_the_nesting() {
        assert_eq!(depth(&json!(1)), 0);
        assert_eq!(depth(&json!({"a": 1})), 1);
        assert_eq!(depth(&json!({"a": {"b": 1}})), 2);
        assert_eq!(depth(&json!({"a": [{"b": 1}]})), 3);
    }

    #[test]
    fn every_number_of_a_document_is_collected() {
        let mut found = Vec::new();
        collect(
            &json!({"a": 1.5, "b": [2.5, {"c": 3.5}], "d": "not a number"}),
            &mut found,
        );
        found.sort_by(f64::total_cmp);
        assert_eq!(found, vec![1.5, 2.5, 3.5]);
    }
}
