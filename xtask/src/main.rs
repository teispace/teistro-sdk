//! Repository tasks, in Rust, so contributors and CI need one toolchain.
//!
//! Invoked as `cargo xtask <task>` through the alias in `.cargo/config.toml`:
//!
//! - `check-docs`: the documentation gates.
//! - `check-dco BASE HEAD`: every commit in `BASE..HEAD` carries a sign-off.
//! - `check-fixtures`: the golden-vector corpus is well formed and listed.
//! - `check-catalogue` and `gen catalogue`: the entity catalogue's generated
//!   code equals its sources.
//! - `check-calendars`, `gen calendars` and `calendars bs-fit`: the Bikram
//!   Sambat table equals what the official rows and the engine produce, and
//!   the measurement that chose the engine's rule.
//! - `check-time` and `gen time`: the Delta T tables of `astro` and the
//!   leap-second table of `time` equal their data files (the IERS series,
//!   the historical table, the IANA list).
//! - `check-ffi` and `gen ffi`: the API description (`idl/api.json`) and
//!   the C header equal what the boundary crates' source describes.
//! - `check-c`: the C binding's smoke test compiles against the generated
//!   header with warnings as errors and passes (needs a C compiler; run by
//!   hand and in the nightly matrix).
//! - `check-dart`: the Dart binding's layer and decoders against the real
//!   library, the package analysed and formatted.
//! - `check-node`: the Node binding's decoders read blobs the library
//!   produced and its types pass a consumer's strict type-check (needs
//!   Node, and TypeScript for the second half).
//!
//! Each gate exists because the failure it catches is easy to make and
//! invisible to a reader; the comment on each one names that failure.

// A tooling binary, not a library: it reports through stdout and stderr and
// stops on an unreadable file, so the library lints against printing,
// panicking and indexing (workspace `Cargo.toml`) are allowed here and in
// no library crate.
#![allow(
    clippy::print_stdout,
    clippy::print_stderr,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::panic,
    clippy::indexing_slicing
)]

mod accuracy;
mod binding;
mod c_binding;
mod calendars;
mod catalogue;
mod dart_binding;
mod ffi;
mod generated;
mod intl;
mod node_binding;
mod time;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use regex::Regex;
use serde_json::Value;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("check-docs") => check_docs(),
        Some("check-dco") => match args.as_slice() {
            [_, base, head] => check_dco(base, head),
            _ => usage(),
        },
        Some("check-fixtures") => check_fixtures(),
        Some("check-catalogue") => catalogue::check(&repo_root()),
        Some("check-calendars") => calendars::check(&repo_root()),
        Some("check-time") => time::check_generated(&repo_root()),
        Some("check-accuracy") => accuracy::check_generated(&repo_root()),
        Some("check-intl") => intl::check_generated(&repo_root()),
        Some("check-ffi") => ffi::check_generated(&repo_root()),
        Some("check-c") => c_binding::check(&repo_root()),
        Some("check-node") => node_binding::check(&repo_root()),
        Some("check-dart") => dart_binding::check(&repo_root()),
        Some("accuracy") => accuracy::generate(&repo_root()),
        Some("calendars") => match args.get(1).map(String::as_str) {
            Some("bs-fit") => calendars::bs_fit(&repo_root(), args.iter().any(|a| a == "--detail")),
            _ => usage(),
        },
        Some("gen") => match args.get(1).map(String::as_str) {
            Some("catalogue") => catalogue::generate(&repo_root()),
            Some("calendars") => calendars::generate(&repo_root()),
            Some("time") => time::generate(&repo_root()),
            Some("intl") => intl::generate(&repo_root()),
            Some("ffi") => ffi::generate(&repo_root()),
            _ => usage(),
        },
        _ => usage(),
    };
    process::exit(code);
}

fn usage() -> i32 {
    eprintln!(
        "usage: cargo xtask <check-docs | check-dco BASE HEAD | check-fixtures | check-catalogue | check-calendars | check-time | check-accuracy | check-intl | check-ffi | check-c | check-node | check-dart | accuracy | calendars bs-fit | gen catalogue | gen calendars | gen time | gen intl | gen ffi>"
    );
    2
}

/// The repository root: the parent of this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives one level below the repository root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()))
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

// ── check-docs ─────────────────────────────────────────────────────────────

/// Runs the four documentation gates and reports every failure at once.
fn check_docs() -> i32 {
    let root = repo_root();
    let files = markdown_files(&root);
    let mut failures = Vec::new();
    failures.extend(check_links(&root, &files));
    failures.extend(check_status_lines(&root, &files));
    failures.extend(check_forbidden(&root, &files));
    failures.extend(check_status_tracker(&root));
    for failure in &failures {
        println!("FAIL  {failure}");
    }
    println!(
        "checked {} files: {} failure(s)",
        files.len(),
        failures.len()
    );
    i32::from(!failures.is_empty())
}

/// Every Markdown file the gates look at, plus the two root text files that
/// must obey the naming rule.
fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for dir in [
        "docs", "rfcs", ".github", "fixtures", "spikes", "adapters", "crates",
    ] {
        collect_markdown(&root.join(dir), &mut files);
    }
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }
    }
    for name in ["NOTICE", "CODEOWNERS"] {
        let path = root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    files.sort();
    files.dedup();
    files
}

/// Recursively collects Markdown files, skipping build and dependency trees.
fn collect_markdown(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if matches!(name.as_ref(), ".git" | "target" | "node_modules") {
                continue;
            }
            collect_markdown(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }
}

/// Every relative link resolves to a file that exists. A renamed page
/// silently breaks the map otherwise.
fn check_links(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let link = Regex::new(r"\]\(([^)#\s]+)(?:#[^)]*)?\)").expect("valid regex");
    let mut failures = Vec::new();
    for md in files {
        let text = read(md);
        let dir = md.parent().unwrap_or(root);
        for captures in link.captures_iter(&text) {
            let target = &captures[1];
            if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("mailto:")
            {
                continue;
            }
            if !dir.join(target).exists() {
                failures.push(format!("broken link in {}: {target}", rel(root, md)));
            }
        }
    }
    failures
}

/// Every page inside a numbered docs directory states its status near the
/// top, so a reader knows whether they hold research, a draft, a decision
/// or a plan.
fn check_status_lines(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let numbered = Regex::new(r"^\d{2}-").expect("valid regex");
    let docs = root.join("docs");
    let mut failures = Vec::new();
    for md in files {
        let Ok(inside) = md.strip_prefix(&docs) else {
            continue;
        };
        let Some(first) = inside.components().next() else {
            continue;
        };
        if !numbered.is_match(&first.as_os_str().to_string_lossy()) {
            continue;
        }
        let has_status = read(md)
            .lines()
            .take(20)
            .any(|line| line.starts_with("Status:"));
        if !has_status {
            failures.push(format!("no Status line near the top of {}", rel(root, md)));
        }
    }
    failures
}

/// Internal product and company names, and the name of the retired
/// planning corpus that preceded this repository, stay out of this public
/// repository; the predecessor engine is referred to as "the baseline
/// engine" and the corpus is not referred to at all.
fn check_forbidden(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let patterns = [r"(?i)joishi", r"(?i)softup", r"@jyotisha", r"(?i)pramana"];
    let compiled: Vec<Regex> = patterns
        .iter()
        .map(|pattern| Regex::new(pattern).expect("valid regex"))
        .collect();
    let mut failures = Vec::new();
    for md in files {
        let text = read(md);
        for (pattern, regex) in patterns.iter().zip(&compiled) {
            if regex.is_match(&text) {
                failures.push(format!("forbidden term {pattern} in {}", rel(root, md)));
            }
        }
    }
    failures
}

/// The tracker is only useful if it says when it was last true.
fn check_status_tracker(root: &Path) -> Vec<String> {
    let tracker = root.join("docs/STATUS.md");
    if !tracker.is_file() {
        return vec!["docs/STATUS.md is missing".to_string()];
    }
    if !read(&tracker).contains("Last updated:") {
        return vec!["docs/STATUS.md has no 'Last updated:' line".to_string()];
    }
    Vec::new()
}

// ── check-fixtures ─────────────────────────────────────────────────────────

const FIXTURE_SCHEMA: &str = "teistro-conformance/baseline-chart/1";
const MANIFEST_SCHEMA: &str = "teistro-conformance/baseline-manifest/1";
const TOLERANCES_SCHEMA: &str = "teistro-conformance/tolerances/1";
const BODY_ORDER: [&str; 10] = [
    "SUN", "MOON", "MARS", "MERCURY", "JUPITER", "VENUS", "SATURN", "RAHU", "KETU", "LAGNA",
];

/// The golden-vector corpus is only usable if every fixture parses, declares
/// the schema it follows, carries the settings hash of the profile it claims,
/// holds every section it lists, and is listed in the manifest with nothing
/// unlisted beside it. A fixture is also text that reaches the public
/// repository, so it passes the same forbidden-terms rule as the docs.
fn check_fixtures() -> i32 {
    let root = repo_root();
    let base = root.join("fixtures/baseline");
    let mut failures = Vec::new();
    let manifest = match parse_json(&base.join("manifest.json")) {
        Ok(value) => value,
        Err(err) => {
            println!("FAIL  {err}");
            return 1;
        }
    };
    if manifest["schema"].as_str() != Some(MANIFEST_SCHEMA) {
        failures.push("manifest schema is not the expected one".to_string());
    }
    let engine_version = manifest["provenance"]["engine_version"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    if engine_version.is_empty() {
        failures.push("manifest has no provenance.engine_version".to_string());
    }
    let listed: Vec<&Value> = manifest["fixtures"]
        .as_array()
        .map_or_else(Vec::new, |a| a.iter().collect());
    if listed.is_empty() {
        failures.push("manifest lists no fixtures".to_string());
    }
    let mut listed_files = Vec::new();
    for entry in &listed {
        let file = entry["file"].as_str().unwrap_or_default();
        let profile = entry["profile"].as_str().unwrap_or_default();
        listed_files.push(file.to_string());
        let profile_hash = manifest["profiles"][profile]["settings_hash"]["value"]
            .as_str()
            .unwrap_or_default();
        let path = base.join(file);
        if !path.is_file() {
            failures.push(format!("manifest lists a missing file: {file}"));
            continue;
        }
        failures.extend(check_fixture_file(
            &path,
            file,
            entry,
            profile_hash,
            &engine_version,
        ));
    }
    for dir in ["charts", "variants"] {
        let Ok(entries) = fs::read_dir(base.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let name = format!("{dir}/{}", entry.file_name().to_string_lossy());
            if entry.path().extension().is_some_and(|ext| ext == "json")
                && !listed_files.contains(&name)
            {
                failures.push(format!("fixture not listed in the manifest: {name}"));
            }
        }
    }
    let forbidden = Regex::new(r"(?i)joishi|softup|@jyotisha|pramana").expect("valid regex");
    if forbidden.is_match(&read(&base.join("manifest.json"))) {
        failures.push("forbidden term in fixtures/baseline/manifest.json".to_string());
    }
    match parse_json(&root.join("fixtures/tolerances.json")) {
        Ok(value) if value["schema"].as_str() == Some(TOLERANCES_SCHEMA) => {}
        Ok(_) => failures.push("fixtures/tolerances.json has the wrong schema".to_string()),
        Err(err) => failures.push(err),
    }
    for failure in &failures {
        println!("FAIL  {failure}");
    }
    println!(
        "checked {} fixtures: {} failure(s)",
        listed.len(),
        failures.len()
    );
    i32::from(!failures.is_empty())
}

/// The checks on one fixture file; every failure names the file.
fn check_fixture_file(
    path: &Path,
    file: &str,
    entry: &Value,
    profile_hash: &str,
    engine_version: &str,
) -> Vec<String> {
    let mut failures = Vec::new();
    let text = read(path);
    let forbidden = Regex::new(r"(?i)joishi|softup|@jyotisha|pramana").expect("valid regex");
    if forbidden.is_match(&text) {
        failures.push(format!("forbidden term in {file}"));
    }
    let fixture: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(err) => {
            failures.push(format!("{file} is not valid JSON: {err}"));
            return failures;
        }
    };
    if fixture["schema"].as_str() != Some(FIXTURE_SCHEMA) {
        failures.push(format!("{file}: schema is not {FIXTURE_SCHEMA}"));
    }
    if fixture["id"] != entry["id"] {
        failures.push(format!("{file}: id differs from the manifest"));
    }
    let hash = fixture["settings_hash"]["value"]
        .as_str()
        .unwrap_or_default();
    let hex16 = hash.len() == 16 && hash.chars().all(|c| c.is_ascii_hexdigit());
    if !hex16 {
        failures.push(format!(
            "{file}: settings_hash.value is not 16 hex characters"
        ));
    }
    if hash != profile_hash || entry["settings_hash"].as_str() != Some(hash) {
        failures.push(format!(
            "{file}: settings hash disagrees with the manifest profile"
        ));
    }
    if fixture["provenance"]["engine_version"].as_str() != Some(engine_version) {
        failures.push(format!(
            "{file}: provenance.engine_version differs from the manifest"
        ));
    }
    let jd = fixture["input"]["resolved"]["jd_ut"]
        .as_f64()
        .unwrap_or(f64::NAN);
    if !(jd.is_finite() && jd > 0.0) {
        failures.push(format!(
            "{file}: input.resolved.jd_ut is not a positive number"
        ));
    }
    match fixture["sections"].as_array() {
        Some(sections) if !sections.is_empty() => {
            for section in sections {
                let name = section.as_str().unwrap_or_default();
                if !fixture[name].is_object() {
                    failures.push(format!("{file}: listed section `{name}` is missing"));
                }
            }
        }
        _ => failures.push(format!("{file}: no sections listed")),
    }
    // The JSON map sorts its keys, so the check is on the set of bodies; the
    // exporter writes them in canonical order and the harness reads by key.
    if let Some(bodies) = fixture["positions"]["bodies"].as_object() {
        let mut keys: Vec<&str> = bodies.keys().map(String::as_str).collect();
        keys.sort_unstable();
        let mut expected = BODY_ORDER.to_vec();
        expected.sort_unstable();
        if keys != expected {
            failures.push(format!(
                "{file}: positions.bodies are not exactly the ten bodies"
            ));
        }
    }
    failures
}

fn parse_json(path: &Path) -> Result<Value, String> {
    let text =
        fs::read_to_string(path).map_err(|err| format!("cannot read {}: {err}", path.display()))?;
    serde_json::from_str(&text)
        .map_err(|err| format!("{} is not valid JSON: {err}", path.display()))
}

// ── check-dco ──────────────────────────────────────────────────────────────

/// Every commit in `base..head` carries a `Signed-off-by` line, which is
/// what certifies the Developer Certificate of Origin.
fn check_dco(base: &str, head: &str) -> i32 {
    let signoff = Regex::new(r"(?m)^Signed-off-by: .+ <.+@.+>$").expect("valid regex");
    let range = format!("{base}..{head}");
    let Some(listing) = git(&["rev-list", &range]) else {
        return 2;
    };
    let mut missing = 0;
    for sha in listing.lines().map(str::trim).filter(|sha| !sha.is_empty()) {
        let Some(body) = git(&["show", "-s", "--format=%B", sha]) else {
            return 2;
        };
        if !signoff.is_match(&body) {
            let subject = body.lines().next().unwrap_or("");
            println!("missing sign-off: {sha} {subject}");
            missing += 1;
        }
    }
    println!("checked commits in {range}: {missing} missing sign-off");
    i32::from(missing != 0)
}

/// Runs `git` with the given arguments and returns its standard output, or
/// `None` after printing why it could not.
fn git(args: &[&str]) -> Option<String> {
    match Command::new("git").args(args).output() {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).into_owned())
        }
        Ok(output) => {
            eprintln!(
                "git {} failed: {}",
                args.join(" "),
                String::from_utf8_lossy(&output.stderr).trim()
            );
            None
        }
        Err(err) => {
            eprintln!("cannot run git: {err}");
            None
        }
    }
}
