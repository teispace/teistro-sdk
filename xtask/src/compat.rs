//! `docs/09-guidelines/baseline-map.md`: the baseline engine mapped to the
//! SDK (`03-design/compat-shim.md` §4.4), and the gate on it.
//!
//! The page is rendered from the two sources the shim itself runs on, so
//! the guide and the shim cannot disagree about a single row:
//!
//! - the call surface, `bindings/node/compat/surface.json`, one row a shim
//!   function, grouped under the design page's §3 headings;
//! - the stored settings, `teistro_core::settings::recorded::ROWS`, each
//!   row with the patch it writes as JSON, the fixed openings an empty
//!   snapshot translates to, and the listed spellings (`AYANAMSHA_KEYS`,
//!   `DASHA_SYSTEMS`);
//! - the counts: forwarded, composed and refused rows, and the baseline's
//!   entry points the A/B record counted with no row (`measure-ab`).
//!
//! `check-compat` regenerates the page in memory and fails on any
//! difference, on a refused row whose guide anchor is dead, on a crux a
//! settings refusal names that the register lacks, on a section the
//! design page does not have, and on any unmapped entry point a record
//! counted. A record that could not count them (the adapter exports no
//! `surface`) is said on the page, not passed off as none.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;
use serde_json::Value as Json;
use teistro_core::settings::recorded::{
    AYANAMSHA_KEYS, Becomes, DASHA_SYSTEMS, PROFILE, ROWS, Row, Snapshot, Stored, Value, recorded,
};
use teistro_core::settings::{Profile, SettingsPatch};

use crate::generated::{Output, check, write};

const PAGE: &str = "docs/09-guidelines/baseline-map.md";
const SURFACE: &str = "bindings/node/compat/surface.json";
const DESIGN: &str = "docs/03-design/compat-shim.md";
const GUIDE: &str = "docs/09-guidelines/baseline-migration.md";
const CRUXES: &str = "docs/01-research/feature-universe/19-verification-cruxes.md";

/// One row of `surface.json`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Surface {
    row: String,
    section: String,
    entry: String,
    kind: Kind,
    sdk: String,
    #[serde(default)]
    guide: Option<String>,
    #[serde(default)]
    instead: Option<String>,
}

/// `surface.json`: its rows (and a comment, which is not read).
#[derive(Debug, Deserialize)]
struct SurfaceFile {
    rows: Vec<Surface>,
}

/// What a shim row does (§3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Kind {
    Forward,
    Compose,
    Refuse,
}

impl Kind {
    const ALL: [Kind; 3] = [Kind::Forward, Kind::Compose, Kind::Refuse];

    const fn word(self) -> &'static str {
        match self {
            Kind::Forward => "forward",
            Kind::Compose => "compose",
            Kind::Refuse => "refuse",
        }
    }
}

/// The value a `Read` row is shown reading, by the row's field: the
/// patch a reader writes depends on the value, so the page shows it on
/// one the application stores. A `Read` row with no example here fails
/// the gate, so a new one cannot be printed without one.
fn example(field: &str) -> Option<(&'static str, Stored)> {
    match field {
        "ayanamsaId" => Some(("ayanamsaId", Stored::Text(String::from("raman")))),
        "houseSystem" => Some(("houseSystem", Stored::Text(String::from("placidus")))),
        "dashaMaxLevel" => Some(("dashaMaxLevel", Stored::Number(2))),
        "dashaDepthBySystem.*" => Some(("dashaDepthBySystem.yogini", Stored::Number(2))),
        _ => None,
    }
}

/// A value with its nulls and empty records dropped: a patch serialises
/// every knob it leaves alone, and the page prints the ones it sets.
fn pruned(value: Json) -> Option<Json> {
    match value {
        Json::Null => None,
        Json::Object(fields) => {
            let kept: serde_json::Map<String, Json> = fields
                .into_iter()
                .filter_map(|(key, value)| pruned(value).map(|value| (key, value)))
                .collect();
            (!kept.is_empty()).then_some(Json::Object(kept))
        }
        other => Some(other),
    }
}

/// A patch as the page prints it: its set knobs as one line of JSON.
fn patch_json(patch: &SettingsPatch) -> String {
    serde_json::to_value(patch)
        .ok()
        .and_then(pruned)
        .map_or_else(|| String::from("{}"), |value| value.to_string())
}

/// What `after` holds that `before` does not, field by field: the knobs a
/// read row moves off the profile, which is what it means, where its patch
/// also restates the profile's own (a depth map is written whole).
fn moved(before: &Json, after: &Json) -> Option<Json> {
    match (before, after) {
        (Json::Object(was), Json::Object(is)) => {
            let kept: serde_json::Map<String, Json> = is
                .iter()
                .filter_map(|(key, value)| {
                    let changed = was
                        .get(key)
                        .map_or_else(|| Some(value.clone()), |old| moved(old, value))?;
                    Some((key.clone(), changed))
                })
                .collect();
            (!kept.is_empty()).then_some(Json::Object(kept))
        }
        _ => (before != after).then(|| after.clone()),
    }
}

/// A stored value a row takes, as the page prints it.
fn stored_value(value: Value) -> String {
    match value {
        Value::Text(text) => format!("`{text}`"),
        Value::Flag(flag) => format!("`{flag}`"),
        Value::Number(number) => format!("`{number}`"),
        Value::Each(words) => words.to_owned(),
    }
}

/// What a row's value becomes, and the faults met saying so.
fn becomes(row: &Row, found: &mut Vec<String>) -> (&'static str, String) {
    match row.becomes {
        Becomes::Settings(write) => {
            let mut patch = SettingsPatch::default();
            write(&mut patch);
            ("knobs", format!("`{}`", patch_json(&patch)))
        }
        Becomes::Read(reader) => {
            let Some((field, stored)) = example(row.field) else {
                found.push(format!(
                    "the `Read` row `{}` has no example value in xtask/src/compat.rs",
                    row.field
                ));
                return ("read", String::from("(no example)"));
            };
            let base = Profile::shipped(PROFILE).map(|p| p.settings());
            let mut written = SettingsPatch::default();
            match base.map(|base| {
                reader(&mut written, &base, field, &stored).map(|()| {
                    let (was, is) = (
                        serde_json::to_value(&base).unwrap_or_default(),
                        serde_json::to_value(base.patched(&written)).unwrap_or_default(),
                    );
                    moved(&was, &is).map_or_else(|| String::from("{}"), |m| m.to_string())
                })
            }) {
                Some(Ok(changes)) => (
                    "read",
                    format!("read from the value; `{field}: {stored}` sets `{changes}`"),
                ),
                Some(Err(why)) => {
                    found.push(format!(
                        "the `Read` row `{}` refuses its own example `{stored}`: {why}",
                        row.field
                    ));
                    ("read", String::from("(its example is refused)"))
                }
                None => {
                    found.push(format!("the profile `{PROFILE}` is not shipped"));
                    ("read", String::new())
                }
            }
        }
        Becomes::Ephemeris(write, needs) => {
            let mut patch = SettingsPatch::default();
            write(&mut patch);
            let needs = serde_json::to_value(needs)
                .ok()
                .and_then(|value| value.as_str().map(str::to_owned))
                .unwrap_or_default();
            (
                "knobs and an ephemeris",
                format!(
                    "`{}`, with `{needs}` at the front of the natal chain",
                    patch_json(&patch)
                ),
            )
        }
        Becomes::Locale(tag) => ("the locale", format!("`{tag}`")),
        Becomes::Script => (
            "the script",
            String::from("held to the language's locale's script; any other pairing is refused"),
        ),
        Becomes::Note(why) => ("a note", why.to_owned()),
        Becomes::Refused { anchor, why } => ("refused", format!("{anchor}: {why}")),
    }
}

/// The design page's §3 headings, `3.1` to their titles.
fn sections(design: &str) -> Vec<(String, String)> {
    design
        .lines()
        .filter_map(|line| line.strip_prefix("### 3."))
        .filter_map(|rest| {
            let (number, title) = rest.split_once(' ')?;
            number
                .chars()
                .all(|c| c.is_ascii_digit())
                .then(|| (format!("3.{number}"), title.to_owned()))
        })
        .collect()
}

/// The anchor a Markdown heading gets: lower-cased, punctuation dropped,
/// spaces as hyphens (as `surface.test.mjs` reads them).
fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// Every heading anchor of a Markdown text.
fn anchors(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| {
            let hashes = line.chars().take_while(|c| *c == '#').count();
            ((1..=6).contains(&hashes))
                .then(|| line.get(hashes..)?.strip_prefix(' '))
                .flatten()
                .map(slug)
        })
        .collect()
}

/// A repository path as a link from the page, which is in `docs/09-guidelines/`.
fn link(path: &str) -> String {
    path.strip_prefix("docs/")
        .map_or_else(|| format!("../../{path}"), |inside| format!("../{inside}"))
}

/// The faults of the surface rows: each named once, in a section the
/// design page has, and a refusal and only a refusal naming a guide
/// anchor that exists and what to do instead.
fn surface_faults(root: &Path, rows: &[Surface], titles: &[(String, String)]) -> Vec<String> {
    let mut found = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        if !seen.insert(row.row.as_str()) {
            found.push(format!("{SURFACE}: `{}` is named twice", row.row));
        }
        for section in row.section.split(", ") {
            if !titles.iter().any(|(number, _)| number == section) {
                found.push(format!(
                    "{SURFACE}: `{}` is in §{section}, which {DESIGN} does not have",
                    row.row
                ));
            }
        }
        let refused = row.kind == Kind::Refuse;
        if row.guide.is_some() != refused || row.instead.is_some() != refused {
            found.push(format!(
                "{SURFACE}: `{}` must name a guide and what to do instead exactly when it is refused",
                row.row
            ));
        }
        if let Some(guide) = &row.guide {
            let (path, anchor) = guide.split_once('#').unwrap_or((guide, ""));
            match std::fs::read_to_string(root.join(path)) {
                Ok(text) if anchors(&text).contains(anchor) => {}
                Ok(_) => found.push(format!(
                    "{SURFACE}: `{}`'s guide anchor `#{anchor}` is not a heading of {path}",
                    row.row
                )),
                Err(e) => found.push(format!("{SURFACE}: `{}`'s guide {path}: {e}", row.row)),
            }
        }
    }
    found
}

/// The faults of the settings refusals: each crux a row names is in the
/// register.
fn crux_faults(cruxes: &str) -> Vec<String> {
    ROWS.iter()
        .filter_map(|row| match row.becomes {
            Becomes::Refused { anchor, .. } => Some((row.field, anchor)),
            _ => None,
        })
        .filter(|(_, anchor)| !cruxes.contains(&format!("| {anchor} |")))
        .map(|(field, anchor)| {
            format!("`{field}` is refused by `{anchor}`, which {CRUXES} has no row for")
        })
        .collect()
}

/// The baseline's entry points each record counted, as the page says them,
/// and the faults: any unmapped, or a record unread.
fn entry_points(root: &Path, found: &mut Vec<String>) -> Vec<String> {
    match crate::ab::surfaces(root) {
        Ok(records) => records
            .into_iter()
            .map(|(machine, surface)| match surface {
                Some(counted) => {
                    if counted.unmapped > 0 {
                        found.push(format!(
                            "`{machine}`: {} of the baseline's {} entry points have no row of the shim{}",
                            counted.unmapped,
                            counted.entries,
                            if counted.unknown_rows.is_empty() {
                                String::new()
                            } else {
                                format!(" (rows named and missing: {})", counted.unknown_rows.join(", "))
                            }
                        ));
                    }
                    format!(
                        "`{machine}`: {} listed, {} with no row",
                        counted.entries, counted.unmapped
                    )
                }
                None => format!(
                    "`{machine}`: not counted, since its adapter exports no `surface` (§6.4 of the design)"
                ),
            })
            .collect(),
        Err(message) => {
            found.push(message);
            Vec::new()
        }
    }
}

/// The opening an empty snapshot translates to, by context: the fixed
/// conventions every translation carries.
fn openings(found: &mut Vec<String>) -> Vec<(&'static str, String, String)> {
    match recorded(&Snapshot::default()) {
        Ok(translated) => [
            ("natal", translated.natal),
            ("kp", translated.kp),
            ("muhurta", translated.muhurta),
        ]
        .into_iter()
        .map(|(name, opening)| {
            (
                name,
                opening.profile.as_str().to_owned(),
                patch_json(&opening.settings),
            )
        })
        .collect(),
        Err(refused) => {
            found.push(format!("an empty snapshot is refused: {refused}"));
            Vec::new()
        }
    }
}

/// A settings row as the page says it: the row, its kind and what it becomes.
type Said = (&'static Row, &'static str, String);

/// §1: the shim's rows by kind, the settings rows by what they become, the
/// listed spellings, and the baseline's entry points each record counted.
fn counts(out: &mut String, rows: &[Surface], settings: &[Said], counted: &[String]) {
    let _ = writeln!(out, "## 1. Counts\n\n| | count |\n|---|---|");
    let _ = writeln!(out, "| shim rows | {} |", rows.len());
    for kind in Kind::ALL {
        let _ = writeln!(
            out,
            "| {} | {} |",
            match kind {
                Kind::Forward => "forwarded: one SDK call, its inputs mapped",
                Kind::Compose => "composed: several SDK calls, or a read across one answer",
                Kind::Refuse => "refused: throws `CompatRefused` at once, typed `never`",
            },
            rows.iter().filter(|row| row.kind == kind).count()
        );
    }
    let _ = writeln!(out, "| stored settings rows | {} |", settings.len());
    let mut seen = BTreeSet::new();
    for kind in settings.iter().map(|(_, kind, _)| *kind) {
        if seen.insert(kind) {
            let _ = writeln!(
                out,
                "| … becoming {kind} | {} |",
                settings.iter().filter(|(_, k, _)| *k == kind).count()
            );
        }
    }
    let _ = writeln!(
        out,
        "| ayanamsha keys the application stores | {} |\n| dasha systems' stored spellings | {} |\n",
        AYANAMSHA_KEYS.len(),
        DASHA_SYSTEMS.len()
    );
    let _ = writeln!(
        out,
        "The baseline engine's own entry points, counted from outside by the A/B's adapter \
         (`cargo xtask measure-ab`); the gate fails on any with no row:\n"
    );
    for line in counted {
        let _ = writeln!(out, "- {line}");
    }
    let _ = writeln!(out);
}

/// §2: the shim's rows under the design page's §3 headings that hold any.
fn surface_tables(out: &mut String, rows: &[Surface], titles: &[(String, String)]) {
    let _ = writeln!(
        out,
        "## 2. The call surface\n\n\
         Every function takes the stored birth record and the stored snapshot, \
         `(birth, snapshot, options?)`, unless its SDK call reads no birth. `ctx` is the context \
         the snapshot opens; `found` is `ctx.chart.found`.\n"
    );
    let grouped = titles.iter().filter_map(|(number, title)| {
        let held: Vec<&Surface> = rows
            .iter()
            .filter(|row| row.section.split(", ").any(|s| s == number))
            .collect();
        (!held.is_empty()).then_some((number, title, held))
    });
    for (at, (number, title, held)) in grouped.enumerate() {
        let _ = writeln!(
            out,
            "### 2.{} {title}\n\nDesign §{number}.\n\n| baseline entry point | shim | SDK | kind |\n|---|---|---|---|",
            at + 1
        );
        for row in held {
            let kind = match (&row.guide, &row.instead) {
                (Some(guide), Some(instead)) => {
                    let (path, anchor) = guide.split_once('#').unwrap_or((guide, ""));
                    format!(
                        "{}: {instead} ([why]({}#{anchor}))",
                        row.kind.word(),
                        link(path)
                    )
                }
                _ => row.kind.word().to_owned(),
            };
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {kind} |",
                row.entry, row.row, row.sdk
            );
        }
        let _ = writeln!(out);
    }
}

/// §3: the settings rows, the fixed openings and the listed spellings.
fn settings_tables(out: &mut String, settings: &[Said], found: &mut Vec<String>) {
    let _ = writeln!(
        out,
        "## 3. The stored settings\n\n\
         Each stored field and value, folded in this order over `{PROFILE}`, a later row winning \
         where two write one knob. A field or value not here is refused, never skipped. Every \
         language binding reads a snapshot through `recordedSettings` (`recorded_settings` in \
         Python, `recorded` in Rust).\n\n\
         | stored field | stored value | becomes | what |\n|---|---|---|---|"
    );
    for (row, kind, said) in settings {
        let _ = writeln!(
            out,
            "| `{}` | {} | {kind} | {said} |",
            row.field,
            stored_value(row.value)
        );
    }
    let _ = writeln!(
        out,
        "\n### 3.1 The fixed conventions\n\n\
         What a snapshot that stores nothing opens: the settings the baseline fixes whatever a \
         snapshot says. Every translation lays its rows over these.\n\n\
         | context | profile | patch |\n|---|---|---|"
    );
    for (name, profile, patch) in openings(found) {
        let _ = writeln!(out, "| `{name}` | `{profile}` | `{patch}` |");
    }
    let _ = writeln!(
        out,
        "\n### 3.2 The ayanamsha keys\n\n\
         The application stores `ayanamsaId` as a catalogued member's `swiss_mode` number or as \
         one of these keys, matched as stored.\n\n| key | member |\n|---|---|"
    );
    for (key, member) in AYANAMSHA_KEYS {
        let _ = writeln!(out, "| `{key}` | `{}` |", member.key());
    }
    let _ = writeln!(
        out,
        "\n### 3.3 The dasha systems' spellings\n\n\
         The names `dashaDepthBySystem` keys a depth by.\n\n| stored | system |\n|---|---|"
    );
    for (stored, system) in DASHA_SYSTEMS {
        let _ = writeln!(out, "| `{stored}` | `{}` |", system.key());
    }
}

/// The page, and the faults met rendering it.
fn page(root: &Path) -> Result<(String, Vec<String>), String> {
    let read =
        |path: &str| std::fs::read_to_string(root.join(path)).map_err(|e| format!("{path}: {e}"));
    let surface: SurfaceFile =
        serde_json::from_str(&read(SURFACE)?).map_err(|e| format!("{SURFACE}: {e}"))?;
    let titles = sections(&read(DESIGN)?);
    let mut found = surface_faults(root, &surface.rows, &titles);
    found.extend(crux_faults(&read(CRUXES)?));
    if !read(GUIDE)?.contains("baseline-map.md") {
        found.push(format!("{GUIDE} does not link {PAGE}"));
    }
    let counted = entry_points(root, &mut found);
    let settings: Vec<Said> = ROWS
        .iter()
        .map(|row| {
            let (kind, said) = becomes(row, &mut found);
            (row, kind, said)
        })
        .collect();

    let mut out = String::from("# The baseline engine, mapped\n\n");
    let _ = writeln!(
        out,
        "Status: `generated` by `cargo xtask compat` from `{SURFACE}` and \
         `teistro_core::settings::recorded::ROWS`, gated by `check-compat`. Do not edit. The \
         design is [`03-design/compat-shim.md`](../03-design/compat-shim.md); the move itself, \
         in prose, is the [migration guide](baseline-migration.md).\n"
    );
    let _ = writeln!(
        out,
        "Every baseline entry point an application calls, with the `@teistro/compat` function \
         that takes it and the SDK call that answers it (§2), and every stored settings field \
         with what it becomes (§3). A shim function answers exactly what its SDK call answers, \
         so dropping the shim is replacing each call by the one beside it.\n"
    );
    counts(&mut out, &surface.rows, &settings, &counted);
    surface_tables(&mut out, &surface.rows, &titles);
    settings_tables(&mut out, &settings, &mut found);
    out.truncate(out.trim_end().len());
    out.push('\n');
    Ok((out, found))
}

fn outputs(root: &Path) -> Result<(Vec<Output>, Vec<String>), i32> {
    match page(root) {
        Ok((text, found)) => Ok((vec![Output::new(PAGE, text)], found)),
        Err(message) => {
            println!("FAIL  {message}");
            Err(1)
        }
    }
}

/// Writes the page.
pub(crate) fn generate(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, found)) => {
            for line in &found {
                println!("note  {line}");
            }
            write(root, &outputs)
        }
        Err(code) => code,
    }
}

/// Regenerates the page in memory, fails on any difference, and fails a
/// dead anchor, a missing crux or section, and any unmapped entry point.
pub(crate) fn check_generated(root: &Path) -> i32 {
    match outputs(root) {
        Ok((outputs, found)) => {
            for line in &found {
                println!("FAIL  {line}");
            }
            let page = check(root, &outputs, "cargo xtask compat");
            if found.is_empty() { page } else { 1 }
        }
        Err(code) => code,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_heading_is_anchored_as_the_shim_reads_it() {
        assert_eq!(
            slug("10. What is not available yet"),
            "10-what-is-not-available-yet"
        );
        assert_eq!(
            slug("3. The name book: what ships, and what the consumer brings"),
            "3-the-name-book-what-ships-and-what-the-consumer-brings"
        );
        assert!(
            anchors("# Title\n\n## 8.3 Moving the stored charts\n")
                .contains("83-moving-the-stored-charts")
        );
        assert!(anchors("#hashtag\n").is_empty());
    }

    #[test]
    fn a_patch_prints_the_knobs_it_sets() {
        assert_eq!(
            pruned(serde_json::json!({"a": null, "b": {}, "c": {"d": 1}})),
            Some(serde_json::json!({"c": {"d": 1}}))
        );
        assert_eq!(patch_json(&SettingsPatch::default()), "{}");
    }

    #[test]
    fn a_read_row_is_shown_by_the_knobs_it_moves() {
        let was = serde_json::json!({"dasha": {"depth": {"A": 3, "B": 3}, "balance": "S"}});
        let is = serde_json::json!({"dasha": {"depth": {"A": 3, "B": 2}, "balance": "S"}});
        assert_eq!(
            moved(&was, &is),
            Some(serde_json::json!({"dasha": {"depth": {"B": 2}}}))
        );
        assert_eq!(moved(&was, &was), None);
    }

    #[test]
    fn every_read_row_has_an_example_it_reads() {
        let mut found = Vec::new();
        for row in ROWS {
            becomes(row, &mut found);
        }
        assert_eq!(found, Vec::<String>::new());
    }

    #[test]
    fn a_path_links_from_the_page() {
        assert_eq!(
            link("docs/03-design/namakarana.md"),
            "../03-design/namakarana.md"
        );
        assert_eq!(
            link("bindings/node/compat/README.md"),
            "../../bindings/node/compat/README.md"
        );
    }

    #[test]
    fn a_refusal_names_a_crux_the_register_has() {
        assert_eq!(crux_faults("| C777 |\n| C60 |\n"), Vec::<String>::new());
        assert_eq!(crux_faults("").len(), 3);
    }
}
