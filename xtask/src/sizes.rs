//! `SIZES.md`: what every shipped artefact weighs, raw and gzipped, from
//! the CI run that built it (`03-design/generated-pages.md` §1).
//!
//! Every platform's library is built on its own runner, so no machine can
//! measure the page live. The figures are **recorded** instead:
//!
//! - `check-package` and `check-wasm` each end by writing a *fragment*,
//!   what they just built and measured, to `target/sizes/`, which verify
//!   uploads from every row;
//! - `cargo xtask sizes --from DIR` merges a run's fragments into the
//!   record, `docs/05-testing/sizes.json`, and renders the page from it;
//!   `sizes --render` renders it from the record as it stands (after a
//!   budget is edited by hand), and `sizes --measure` writes the fragments
//!   of whatever this machine last staged;
//! - `cargo xtask check-sizes` holds the page to the record and the record
//!   to the lists it covers (every platform, wasm profile, bundler and
//!   entry), everywhere; with `--from DIR`, verify's `sizes` job also holds
//!   every fragment of the run to the record within [`DRIFT`].
//!
//! The record also carries the budgets `check-wasm` holds the modules to,
//! with a history row for each re-budget, so a figure is never typed into
//! prose (`a-prose-claim-is-the-part-that-rots`).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::generated::{Output, check, write};
use crate::platform::{PLATFORMS, Platform};

/// The record the page is rendered from.
pub(crate) const RECORD: &str = "docs/05-testing/sizes.json";
/// The page.
const PAGE: &str = "docs/05-testing/SIZES.md";
/// Where a gate writes the fragment of what it built.
pub(crate) const FRAGMENTS: &str = "target/sizes";
/// The record's and the fragments' schema.
const SCHEMA: &str = "teistro-sizes/1";
/// How far a run's figure may move from the record before the record is
/// stale: 2%, the budget's own headroom unit, so the page is never more
/// than one budget step behind and a feature PR need not re-record.
const DRIFT: f64 = 0.02;
/// How far under its budget a module may fall before the budget is too
/// loose to protect anything: 5%, which a toolchain's own drift stays
/// inside and a real saving does not.
pub(crate) const SLACK: f64 = 0.95;
/// The bundlers `check-wasm` bundles the installed package with.
pub(crate) const BUNDLERS: [&str; 3] = ["esbuild", "vite", "webpack"];
/// The wasm package's full module, which no profile names.
pub(crate) const FULL: &str = "full";

/// What an artefact is, which decides which of its sizes are measured.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Artefact {
    /// The shared library: loaded raw, downloaded gzipped by the Dart
    /// installer.
    Library,
    /// The C bundle, a `.tar.gz`: its size is the download.
    CBundle,
    /// The Node addon: raw, and gzipped as npm's tarball carries it.
    Addon,
    /// The Python wheel, a zip: its size is the download.
    Wheel,
    /// The Java classifier jar, a zip: its size is the download.
    Jar,
    /// A wasm profile's module: what a browser compiles, and downloads
    /// gzipped.
    Module,
    /// A wasm profile's glue JavaScript.
    Glue,
    /// One entry of the installed package, bundled.
    Bundle,
}

impl Artefact {
    /// Whether a gzipped figure is measured: where the artefact is not
    /// already compressed, and the C bundle, which is a gzipped tarball and
    /// so has only that figure.
    const fn gzipped(self) -> bool {
        matches!(
            self,
            Artefact::Library
                | Artefact::CBundle
                | Artefact::Addon
                | Artefact::Module
                | Artefact::Glue
        )
    }

    /// Whether a raw figure is measured: everything but the C bundle, whose
    /// raw size is the sum of what it unpacks to and nobody downloads.
    const fn raw(self) -> bool {
        !matches!(self, Artefact::CBundle)
    }
}

/// One artefact's measured sizes.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Row {
    /// A platform's name, a wasm profile, or `<bundler> <entry>`.
    pub(crate) subject: String,
    /// What it is.
    pub(crate) artefact: Artefact,
    /// Its bytes, where [`Artefact::raw`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) raw: Option<u64>,
    /// Its bytes gzipped at the best level, where [`Artefact::gzipped`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) gzip: Option<u64>,
}

impl Row {
    /// A row, measured from the bytes it has: raw where the artefact counts
    /// raw, gzipped where it counts gzipped, and the C bundle's bytes as
    /// they are, since they are already gzipped.
    fn of(subject: impl Into<String>, artefact: Artefact, bytes: &[u8]) -> Row {
        let gzip = match artefact {
            Artefact::CBundle => len(bytes.len()),
            _ => len(gzip_best(bytes).len()),
        };
        Row {
            subject: subject.into(),
            artefact,
            raw: artefact.raw().then(|| len(bytes.len())),
            gzip: artefact.gzipped().then_some(gzip),
        }
    }

    /// Whether it carries exactly the figures its artefact measures.
    fn measured(&self) -> bool {
        self.raw.is_some() == self.artefact.raw() && self.gzip.is_some() == self.artefact.gzipped()
    }

    fn key(&self) -> (String, Artefact) {
        (self.subject.clone(), self.artefact)
    }
}

/// What a run measured, with where it came from.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Fragment {
    schema: String,
    commit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    run: Option<String>,
    rustc: String,
    rows: Vec<Row>,
}

/// The run a record's figures came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Recorded {
    run: String,
    commit: String,
    date: String,
    rustc: String,
    flate2: String,
}

/// What an artefact may weigh, both ways.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Allowed {
    pub(crate) raw: u64,
    pub(crate) gzip: u64,
}

/// A budgeted artefact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Budget {
    subject: String,
    artefact: Artefact,
    #[serde(flatten)]
    allowed: Allowed,
}

/// One re-budget: what the artefact measured when its budget was moved,
/// and the sentence saying what moved it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct History {
    run: String,
    date: String,
    subject: String,
    artefact: Artefact,
    raw: u64,
    gzip: u64,
    why: String,
}

/// `docs/05-testing/sizes.json`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    schema: String,
    why: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    recorded: Option<Recorded>,
    rows: Vec<Row>,
    budgets: Vec<Budget>,
    history: Vec<History>,
}

/// A byte count as `u64`.
fn len(n: usize) -> u64 {
    u64::try_from(n).unwrap_or(u64::MAX)
}

/// Bytes gzipped at the best level, as `check-wasm` and the Dart
/// installer's archive measure them.
pub(crate) fn gzip_best(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::best());
    // Writing to a `Vec` cannot fail.
    let _ = encoder.write_all(bytes);
    encoder.finish().unwrap_or_default()
}

/// The wasm subjects: the full module, then every profile.
fn wasm_subjects() -> Vec<&'static str> {
    std::iter::once(FULL)
        .chain(crate::wasm_binding::PROFILES.iter().map(|p| p.name))
        .collect()
}

/// The bundled entries: one member of `/catalogue`, the entry itself, and
/// each profile's subpath.
fn bundle_entries() -> Vec<&'static str> {
    ["catalogue", "everything"]
        .into_iter()
        .chain(crate::wasm_binding::PROFILES.iter().map(|p| p.name))
        .collect()
}

/// A bundle row's subject.
pub(crate) fn bundle_subject(bundler: &str, entry: &str) -> String {
    format!("{bundler} {entry}")
}

/// Every row a complete record has, in the page's order.
fn expected() -> Vec<(String, Artefact)> {
    let mut keys = Vec::new();
    for platform in PLATFORMS {
        for artefact in [
            Artefact::Library,
            Artefact::CBundle,
            Artefact::Addon,
            Artefact::Wheel,
            Artefact::Jar,
        ] {
            keys.push((platform.name(), artefact));
        }
    }
    for subject in wasm_subjects() {
        keys.push((subject.to_owned(), Artefact::Module));
        keys.push((subject.to_owned(), Artefact::Glue));
    }
    for bundler in BUNDLERS {
        for entry in bundle_entries() {
            keys.push((bundle_subject(bundler, entry), Artefact::Bundle));
        }
    }
    keys
}

// ── measuring, at the end of the gates that build ──────────────────────

/// The rows `check-package` built for a platform, from its staged files.
/// A file missing is a row missing, which the record's completeness
/// check names; it is not an error here, so one missing artefact does not
/// hide the rest.
pub(crate) fn platform_rows(root: &Path, platform: &Platform, version: &str) -> Vec<Row> {
    let dist = root.join("target/dist");
    let name = platform.name();
    let mut rows = Vec::new();
    let manifest: Option<serde_json::Value> =
        fs::read_to_string(dist.join(format!("teistro-{version}-{name}.json")))
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok());
    if let Some(manifest) = manifest {
        let archive = |suffix: &str| {
            manifest["archives"].as_array().and_then(|archives| {
                archives
                    .iter()
                    .filter_map(|a| Some((a["file"].as_str()?, a["bytes"].as_u64()?)))
                    // The library and the C bundle; the agent server's
                    // archive is a program, not a size a binding pays.
                    .filter(|(file, _)| !file.starts_with("teistro-mcp-"))
                    .find(|(file, _)| {
                        file.ends_with(suffix) && !(suffix == ".gz" && file.ends_with(".tar.gz"))
                    })
                    .map(|(_, bytes)| bytes)
            })
        };
        if let Some(raw) = manifest["library"]["bytes"].as_u64() {
            rows.push(Row {
                subject: name.clone(),
                artefact: Artefact::Library,
                raw: Some(raw),
                gzip: archive(".gz"),
            });
        }
        if let Some(gzip) = archive(".tar.gz") {
            rows.push(Row {
                subject: name.clone(),
                artefact: Artefact::CBundle,
                raw: None,
                gzip: Some(gzip),
            });
        }
        let addon = manifest["npm"]["directory"]
            .as_str()
            .zip(manifest["npm"]["addon"]["file"].as_str())
            .map(|(directory, file)| root.join(directory).join(file));
        if let Some(bytes) = addon.and_then(|path| fs::read(path).ok()) {
            rows.push(Row::of(&name, Artefact::Addon, &bytes));
        }
    }
    let files = [
        (
            Artefact::Wheel,
            dist.join("pypi/wheels").join(format!(
                "teistro-{version}-py3-none-{}.whl",
                platform.wheel_tag
            )),
        ),
        (
            Artefact::Jar,
            dist.join(format!(
                "maven/com/teispace/teistro/{version}/teistro-{version}-{name}.jar"
            )),
        ),
    ];
    for (artefact, path) in files {
        if let Ok(meta) = fs::metadata(&path) {
            rows.push(Row {
                subject: name.clone(),
                artefact,
                raw: Some(meta.len()),
                gzip: None,
            });
        }
    }
    rows
}

/// The rows of a staged wasm package's modules and glue, one pair per
/// subject.
pub(crate) fn wasm_rows(staged: &Path) -> Vec<Row> {
    let wasm = staged.join("wasm");
    let mut rows = Vec::new();
    for subject in wasm_subjects() {
        let dir = if subject == FULL {
            wasm.clone()
        } else {
            wasm.join(subject)
        };
        for (artefact, file) in [
            (Artefact::Module, "teistro_wasm_bg.wasm"),
            (Artefact::Glue, "teistro_wasm.js"),
        ] {
            if let Ok(bytes) = fs::read(dir.join(file)) {
                rows.push(Row::of(subject, artefact, &bytes));
            }
        }
    }
    rows
}

/// `cargo xtask sizes --measure`: the fragments of whatever this machine
/// last staged, the host's packages and the wasm package, without running
/// the gates that stage them. The bundles are measured only by
/// `check-wasm`.
fn measure(root: &Path) -> i32 {
    let platform = Platform::host();
    let version = crate::release::version(root);
    let host = write_fragment(
        root,
        &platform.name(),
        platform_rows(root, &platform, &version),
    );
    let wasm = write_fragment(
        root,
        "wasm",
        wasm_rows(&root.join(crate::wasm_binding::STAGED)),
    );
    i32::from(host.is_err() || wasm.is_err())
}

/// Writes a gate's fragment, named for what it measured.
pub(crate) fn write_fragment(root: &Path, name: &str, rows: Vec<Row>) -> Result<(), ()> {
    let fragment = Fragment {
        schema: SCHEMA.to_owned(),
        commit: commit(root),
        run: std::env::var("GITHUB_RUN_ID")
            .ok()
            .filter(|id| !id.is_empty()),
        rustc: output(Command::new("rustc").arg("-V")),
        rows,
    };
    let dir = root.join(FRAGMENTS);
    let path = dir.join(format!("sizes-{name}.json"));
    let text = serde_json::to_string_pretty(&fragment).unwrap_or_default() + "\n";
    fs::create_dir_all(&dir)
        .and_then(|()| fs::write(&path, text))
        .map_err(|e| println!("FAIL  {} could not be written: {e}", path.display()))?;
    println!(
        "ok    {} sizes recorded in {FRAGMENTS}/sizes-{name}.json",
        fragment.rows.len()
    );
    Ok(())
}

/// The commit measured: CI's own where it says so, since a container's
/// checkout may refuse `git`, else HEAD.
fn commit(root: &Path) -> String {
    std::env::var("GITHUB_SHA")
        .ok()
        .filter(|sha| !sha.is_empty())
        .unwrap_or_else(|| {
            output(
                Command::new("git")
                    .args(["rev-parse", "HEAD"])
                    .current_dir(root),
            )
        })
}

/// A command's trimmed standard output, empty when it cannot run.
fn output(command: &mut Command) -> String {
    command
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

// ── the record ─────────────────────────────────────────────────────────

fn read_record(root: &Path) -> Result<Record, String> {
    let text = fs::read_to_string(root.join(RECORD)).map_err(|e| format!("{RECORD}: {e}"))?;
    let record: Record = serde_json::from_str(&text).map_err(|e| format!("{RECORD}: {e}"))?;
    if record.schema != SCHEMA {
        return Err(format!("{RECORD} is `{}`, not `{SCHEMA}`", record.schema));
    }
    Ok(record)
}

/// The budget the record sets an artefact, for `check-wasm`.
pub(crate) fn budget(root: &Path, subject: &str, artefact: Artefact) -> Result<Allowed, String> {
    read_record(root)?
        .budgets
        .into_iter()
        .find(|b| b.subject == subject && b.artefact == artefact)
        .map(|b| b.allowed)
        .ok_or_else(|| format!("{RECORD} sets no budget for {subject}'s {artefact:?}"))
}

/// The budget to suggest for a measured size: 2% over it, to the next ten
/// kilobytes, so a toolchain's drift does not fail the next run.
pub(crate) fn suggested(measured: u64) -> u64 {
    (measured + measured / 50).div_ceil(10_000) * 10_000
}

/// Every fragment in a directory, searched to any depth, since a
/// download of several artefacts nests each in its own folder.
fn fragments(dir: &Path) -> Result<Vec<Fragment>, String> {
    let mut paths = Vec::new();
    collect(dir, &mut paths);
    paths.sort();
    if paths.is_empty() {
        return Err(format!("no sizes-*.json under {}", dir.display()));
    }
    paths
        .iter()
        .map(|path| {
            let text = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
            let fragment: Fragment =
                serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
            if fragment.schema == SCHEMA {
                Ok(fragment)
            } else {
                Err(format!(
                    "{} is `{}`, not `{SCHEMA}`",
                    path.display(),
                    fragment.schema
                ))
            }
        })
        .collect()
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for path in entries.flatten().map(|e| e.path()) {
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() {
            collect(&path, out);
        } else if name.starts_with("sizes-")
            && path
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            out.push(path);
        }
    }
}

/// A run's fragments as one set of rows in the page's order, refused if
/// they come from two commits or measure one artefact twice.
fn merged(fragments: &[Fragment]) -> Result<(String, Option<String>, String, Vec<Row>), String> {
    let first = fragments.first().ok_or("no fragments")?;
    let mut rows: BTreeMap<(String, Artefact), Row> = BTreeMap::new();
    for fragment in fragments {
        if fragment.commit != first.commit {
            return Err(format!(
                "the fragments measure two commits, {} and {}",
                first.commit, fragment.commit
            ));
        }
        for row in &fragment.rows {
            if rows.insert(row.key(), row.clone()).is_some() {
                return Err(format!(
                    "{}'s {:?} is measured twice",
                    row.subject, row.artefact
                ));
            }
        }
    }
    let order = expected();
    let mut sorted: Vec<Row> = rows.into_values().collect();
    sorted.sort_by_key(|row| {
        order
            .iter()
            .position(|key| *key == row.key())
            .unwrap_or(usize::MAX)
    });
    Ok((
        first.commit.clone(),
        first.run.clone(),
        first.rustc.clone(),
        sorted,
    ))
}

/// flate2's version from the lockfile, since gzip bytes depend on the
/// encoder.
fn flate2_version(root: &Path) -> String {
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap_or_default();
    lock.split("[[package]]")
        .find(|block| block.contains("name = \"flate2\""))
        .and_then(|block| {
            block
                .lines()
                .find_map(|line| line.strip_prefix("version = \""))
                .map(|v| v.trim_end_matches('"').to_owned())
        })
        .unwrap_or_default()
}

/// `sizes …` and `check-sizes …`, from the command line as given.
pub(crate) fn command(root: &Path, command: &str, args: &[String]) -> i32 {
    let rest = args.get(1..).unwrap_or_default();
    if command == "sizes" {
        run(root, rest)
    } else {
        check_sizes(root, rest)
    }
}

/// `cargo xtask sizes --from DIR [--why SENTENCE]`: a run's fragments
/// recorded, and the page rendered. With `--why`, every budget its module
/// has left (over it, or more than [`SLACK`] under) moves to
/// [`suggested`] and gains a history row with that sentence.
fn run(root: &Path, args: &[String]) -> i32 {
    if args.iter().any(|arg| arg == "--measure") {
        return measure(root);
    }
    if args.iter().any(|arg| arg == "--render") {
        return match read_record(root) {
            Ok(record) => write(root, &[Output::new(PAGE, render(&record))]),
            Err(why) => {
                println!("FAIL  {why}");
                1
            }
        };
    }
    let Some(from) = flag(args, "--from") else {
        eprintln!(
            "usage: cargo xtask sizes --from DIR [--why SENTENCE] | sizes --measure | sizes --render"
        );
        return 2;
    };
    match record_from(root, Path::new(&from), flag(args, "--why").as_deref()) {
        Ok(outputs) => write(root, &outputs),
        Err(why) => {
            println!("FAIL  {why}");
            1
        }
    }
}

fn record_from(root: &Path, from: &Path, why: Option<&str>) -> Result<Vec<Output>, String> {
    let mut record = read_record(root)?;
    let (commit, run, rustc, rows) = merged(&fragments(from)?)?;
    let run = run.unwrap_or_else(|| String::from("by hand"));
    let date = output(
        Command::new("git")
            .args(["show", "-s", "--format=%cs", &commit])
            .current_dir(root),
    );
    if let Some(why) = why {
        for budget in &mut record.budgets {
            let Some(row) = rows
                .iter()
                .find(|r| r.subject == budget.subject && r.artefact == budget.artefact)
            else {
                continue;
            };
            let (Some(raw), Some(gzip)) = (row.raw, row.gzip) else {
                continue;
            };
            let outside = |measured: u64, allowed: u64| {
                measured > allowed || ratio(measured, allowed) < SLACK
            };
            if outside(raw, budget.allowed.raw) || outside(gzip, budget.allowed.gzip) {
                budget.allowed = Allowed {
                    raw: suggested(raw),
                    gzip: suggested(gzip),
                };
                record.history.push(History {
                    run: run.clone(),
                    date: date.clone(),
                    subject: budget.subject.clone(),
                    artefact: budget.artefact,
                    raw,
                    gzip,
                    why: why.to_owned(),
                });
            }
        }
    }
    record.recorded = Some(Recorded {
        run,
        commit,
        date,
        rustc,
        flate2: flate2_version(root),
    });
    record.rows = rows;
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())? + "\n";
    Ok(vec![
        Output::new(RECORD, text),
        Output::new(PAGE, render(&record)),
    ])
}

fn flag(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|arg| arg == name)
        .and_then(|at| args.get(at + 1))
        .cloned()
}

#[allow(
    clippy::cast_precision_loss,
    reason = "a ratio of two sizes under a gigabyte, read to a percent"
)]
pub(crate) fn ratio(measured: u64, allowed: u64) -> f64 {
    measured as f64 / allowed.max(1) as f64
}

// ── the gate ───────────────────────────────────────────────────────────

/// `cargo xtask check-sizes [--from DIR]`.
fn check_sizes(root: &Path, args: &[String]) -> i32 {
    let record = match read_record(root) {
        Ok(record) => record,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    let mut failures = check(
        root,
        &[Output::new(PAGE, render(&record))],
        "cargo xtask sizes",
    );
    failures += complete(&record);
    if let Some(from) = flag(args, "--from") {
        failures += match fragments(Path::new(&from)).and_then(|f| merged(&f)) {
            Ok((_, _, _, rows)) => against(&record, &rows),
            Err(why) => {
                println!("FAIL  {why}");
                1
            }
        };
    }
    if failures == 0 {
        println!("ok    {RECORD}: {} sizes, complete", record.rows.len());
    }
    i32::from(failures != 0)
}

/// The record against the lists it covers, both ways, and each row against
/// what its artefact measures.
fn complete(record: &Record) -> i32 {
    let expected = expected();
    let mut failures = 0;
    for key in &expected {
        if !record.rows.iter().any(|row| row.key() == *key) {
            println!(
                "FAIL  {RECORD} has no {:?} for {}; record a verify run with `cargo xtask sizes --from DIR`",
                key.1, key.0
            );
            failures += 1;
        }
    }
    for row in &record.rows {
        if !expected.contains(&row.key()) {
            println!(
                "FAIL  {RECORD}'s {:?} for {} is for nothing the SDK ships",
                row.artefact, row.subject
            );
            failures += 1;
        }
        if !row.measured() {
            println!(
                "FAIL  {RECORD}'s {:?} for {} carries the wrong figures",
                row.artefact, row.subject
            );
            failures += 1;
        }
    }
    for budget in &record.budgets {
        if !expected.contains(&(budget.subject.clone(), budget.artefact)) {
            println!(
                "FAIL  {RECORD} budgets {:?} for {}, which the SDK does not ship",
                budget.artefact, budget.subject
            );
            failures += 1;
        }
    }
    failures
}

/// A run's figures against the record, each within [`DRIFT`], and the run
/// complete.
fn against(record: &Record, rows: &[Row]) -> i32 {
    let mut failures = 0;
    for key in expected() {
        if !rows.iter().any(|row| row.key() == key) {
            println!("FAIL  the run measured no {:?} for {}", key.1, key.0);
            failures += 1;
        }
    }
    for row in rows {
        let Some(recorded) = record.rows.iter().find(|r| r.key() == row.key()) else {
            continue;
        };
        for (what, measured, was) in [
            ("raw", row.raw, recorded.raw),
            ("gzipped", row.gzip, recorded.gzip),
        ] {
            let (Some(measured), Some(was)) = (measured, was) else {
                continue;
            };
            let moved = (ratio(measured, was) - 1.0).abs();
            if moved > DRIFT {
                println!(
                    "FAIL  {}'s {:?} is {measured} bytes {what}, {:.1}% from the record's {was}; re-record with `cargo xtask sizes --from DIR`",
                    row.subject,
                    row.artefact,
                    moved * 100.0
                );
                failures += 1;
            }
        }
    }
    failures
}

// ── the page ───────────────────────────────────────────────────────────

/// Bytes grouped in threes, as the page prints them.
fn grouped(bytes: u64) -> String {
    let digits = bytes.to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\u{a0}');
        }
        out.push(c);
    }
    out
}

fn cell(bytes: Option<u64>) -> String {
    bytes.map_or_else(|| String::from("—"), grouped)
}

fn render(record: &Record) -> String {
    let mut page = String::new();
    let _ = writeln!(page, "# Sizes\n");
    let _ = writeln!(
        page,
        "Status: generated by `cargo xtask sizes --from DIR` from `sizes.json`, held by `cargo xtask check-sizes`. Do not edit by hand.\n"
    );
    let _ = writeln!(
        page,
        "What every artefact a release ships weighs, in bytes. **Gzipped** is what a consumer downloads, so it is the column a decision reads; an artefact that is already compressed (an archive, a wheel, a jar) has only its own size. The figures are recorded from one CI run, since every platform builds on its own runner, and verify fails when a run moves any of them more than 2% from the record (`03-design/generated-pages.md` §1).\n"
    );
    match &record.recorded {
        Some(r) => {
            let _ = writeln!(
                page,
                "Recorded from run `{}` at commit `{}` ({}), with `{}` and flate2 {}.\n",
                r.run,
                r.commit.get(..12).unwrap_or(&r.commit),
                r.date,
                r.rustc,
                r.flate2
            );
        }
        None => {
            let _ = writeln!(page, "No run is recorded yet.\n");
        }
    }
    platforms(&mut page, record);
    wasm(&mut page, record);
    bundles(&mut page, record);
    history(&mut page, record);
    page
}

impl Record {
    fn find(&self, subject: &str, artefact: Artefact) -> Option<&Row> {
        self.rows
            .iter()
            .find(|r| r.subject == subject && r.artefact == artefact)
    }
}

fn platforms(page: &mut String, record: &Record) {
    let _ = writeln!(page, "## Per platform\n");
    let _ = writeln!(
        page,
        "| platform | library | library, gzipped | C bundle | Node addon | Node addon, gzipped | Python wheel | Java jar |"
    );
    let _ = writeln!(page, "|---|--:|--:|--:|--:|--:|--:|--:|");
    for platform in PLATFORMS {
        let name = platform.name();
        let library = record.find(&name, Artefact::Library);
        let addon = record.find(&name, Artefact::Addon);
        let _ = writeln!(
            page,
            "| `{name}` | {} | {} | {} | {} | {} | {} | {} |",
            cell(library.and_then(|r| r.raw)),
            cell(library.and_then(|r| r.gzip)),
            cell(record.find(&name, Artefact::CBundle).and_then(|r| r.gzip)),
            cell(addon.and_then(|r| r.raw)),
            cell(addon.and_then(|r| r.gzip)),
            cell(record.find(&name, Artefact::Wheel).and_then(|r| r.raw)),
            cell(record.find(&name, Artefact::Jar).and_then(|r| r.raw)),
        );
    }
    let _ = writeln!(
        page,
        "\nA native library is loaded, not downloaded, so only the wasm modules are held to a budget.\n"
    );
}

fn wasm(page: &mut String, record: &Record) {
    let _ = writeln!(page, "## The wasm package\n");
    let _ = writeln!(
        page,
        "| module | module | gzipped | budget, gzipped | left | glue | glue, gzipped |"
    );
    let _ = writeln!(page, "|---|--:|--:|--:|--:|--:|--:|");
    for subject in wasm_subjects() {
        let module = record.find(subject, Artefact::Module);
        let glue = record.find(subject, Artefact::Glue);
        let budget = record
            .budgets
            .iter()
            .find(|b| b.subject == subject && b.artefact == Artefact::Module);
        let left = budget.zip(module.and_then(|r| r.gzip)).map_or_else(
            || String::from("—"),
            |(b, gzip)| format!("{:.1}%", (1.0 - ratio(gzip, b.allowed.gzip)) * 100.0),
        );
        let _ = writeln!(
            page,
            "| `{subject}` | {} | {} | {} | {left} | {} | {} |",
            cell(module.and_then(|r| r.raw)),
            cell(module.and_then(|r| r.gzip)),
            cell(budget.map(|b| b.allowed.gzip)),
            cell(glue.and_then(|r| r.raw)),
            cell(glue.and_then(|r| r.gzip)),
        );
    }
}

fn bundles(page: &mut String, record: &Record) {
    let _ = writeln!(page, "\n## Bundled\n");
    let _ = writeln!(
        page,
        "The installed package bundled by each pinned bundler (`bindings/wasm/bundlers`), the module apart. One member of `/catalogue` must ship no module.\n"
    );
    let entries = bundle_entries();
    let heads: Vec<String> = entries.iter().map(|e| format!("`{e}`")).collect();
    let _ = writeln!(page, "| bundler | {} |", heads.join(" | "));
    let _ = writeln!(page, "|---|{}", "--:|".repeat(entries.len()));
    for bundler in BUNDLERS {
        let cells: Vec<String> = entries
            .iter()
            .map(|entry| {
                cell(
                    record
                        .find(&bundle_subject(bundler, entry), Artefact::Bundle)
                        .and_then(|r| r.raw),
                )
            })
            .collect();
        let _ = writeln!(page, "| {bundler} | {} |", cells.join(" | "));
    }
}

fn history(page: &mut String, record: &Record) {
    let _ = writeln!(page, "\n## Budgets moved\n");
    let _ = writeln!(page, "{}\n", record.why);
    if record.history.is_empty() {
        let _ = writeln!(page, "No budget has moved since the record began.");
        return;
    }
    let _ = writeln!(page, "| date | run | module | measured | gzipped | why |");
    let _ = writeln!(page, "|---|---|---|--:|--:|---|");
    for h in &record.history {
        let _ = writeln!(
            page,
            "| {} | `{}` | `{}` | {} | {} | {} |",
            h.date,
            h.run,
            h.subject,
            grouped(h.raw),
            grouped(h.gzip),
            h.why
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_budget_is_two_percent_over_to_the_next_ten_kilobytes() {
        assert_eq!(suggested(1_916_240), 1_960_000);
        assert_eq!(suggested(10_000), 10_200_u64.div_ceil(10_000) * 10_000);
    }

    #[test]
    fn bytes_are_grouped_in_threes() {
        assert_eq!(grouped(7), "7");
        assert_eq!(grouped(1_000), "1\u{a0}000");
        assert_eq!(grouped(7_218_676), "7\u{a0}218\u{a0}676");
    }

    #[test]
    fn every_expected_row_is_named_once() {
        let keys = expected();
        let mut unique = keys.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), keys.len());
    }

    fn fragment(commit: &str, rows: Vec<Row>) -> Fragment {
        Fragment {
            schema: SCHEMA.to_owned(),
            commit: commit.to_owned(),
            run: None,
            rustc: String::new(),
            rows,
        }
    }

    fn module(subject: &str, raw: u64, gzip: u64) -> Row {
        Row {
            subject: subject.to_owned(),
            artefact: Artefact::Module,
            raw: Some(raw),
            gzip: Some(gzip),
        }
    }

    #[test]
    fn fragments_of_two_commits_or_one_artefact_twice_are_refused() {
        let a = fragment("a", vec![module(FULL, 1, 1)]);
        let b = fragment("b", vec![module("panchanga", 1, 1)]);
        assert!(merged(&[a, b]).is_err());
        let a = fragment("a", vec![module(FULL, 1, 1)]);
        let again = fragment("a", vec![module(FULL, 2, 2)]);
        assert!(merged(&[a, again]).is_err());
    }

    #[test]
    fn a_run_is_held_to_the_record_within_its_drift() {
        let record = Record {
            schema: SCHEMA.to_owned(),
            why: String::new(),
            recorded: None,
            rows: vec![module(FULL, 1_000_000, 300_000)],
            budgets: Vec::new(),
            history: Vec::new(),
        };
        let within = against(&record, &[module(FULL, 1_019_000, 300_000)]);
        let beyond = against(&record, &[module(FULL, 1_021_000, 300_000)]);
        // Both runs are incomplete by every other row; the difference is
        // the one row that drifted.
        assert_eq!(beyond - within, 1);
    }

    #[test]
    fn a_row_carries_exactly_the_figures_its_artefact_measures() {
        let row = Row::of(FULL, Artefact::Module, &[0; 64]);
        assert!(row.raw.is_some() && row.gzip.is_some());
        let row = Row::of("linux-x64", Artefact::Wheel, &[0; 64]);
        assert!(row.raw.is_some() && row.gzip.is_none());
        // The C bundle is a gzipped tarball: its one figure is its size.
        let row = Row::of("linux-x64", Artefact::CBundle, &[0; 64]);
        assert_eq!((row.raw, row.gzip), (None, Some(64)));
        for artefact in [
            Artefact::Library,
            Artefact::CBundle,
            Artefact::Addon,
            Artefact::Wheel,
            Artefact::Jar,
            Artefact::Module,
            Artefact::Glue,
            Artefact::Bundle,
        ] {
            assert!(Row::of("s", artefact, &[0; 64]).measured(), "{artefact:?}");
        }
    }
}
