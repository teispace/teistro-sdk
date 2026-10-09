//! The conformance document: `conformance` runs the crates' corpus tests
//! with `TEISTRO_CONFORMANCE_DIR` set, so each records the comparisons it
//! made through `crates/core/tests/support/conformance.rs`, and renders
//! `docs/05-testing/CONFORMANCE.md` from those scores and
//! `fixtures/corpus.json`; `check-conformance` renders in memory and
//! compares, so the checked-in page can never drift from what the tests
//! compare (`03-design/generated-pages.md` §2).
//!
//! A test fails on any miss no named entry explains, and a failed test
//! records nothing, so an unexplained miss never reaches the page as a
//! number: the gate fails instead. Every corpus section is scored or
//! declared unscored here with its reason, both ways. Every entry a score
//! names is a `KNOWN` divergence (the charts' or DE440's) or cites a page that exists, and the
//! corpus the scores were made against is the tagged release its index
//! names.
//!
//! The tiers a default build does not compile are **recorded**: verify's
//! tier jobs write their score lines, `conformance --from DIR` keeps them in
//! `docs/05-testing/conformance.json` with the run they came from, and
//! `check-conformance --from DIR` holds a later run to the record exactly,
//! since a corpus report is deterministic (§2.3).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use teistro_ephemeris_kit::{corpus, jpl};

use crate::generated::{Output, check, write};

const DOCUMENT: &str = "docs/05-testing/CONFORMANCE.md";
const CORPUS: &str = "fixtures/corpus.json";
const SCORES: &str = "scores.jsonl";
const DIR_ENV: &str = "TEISTRO_CONFORMANCE_DIR";
const RECORD: &str = "docs/05-testing/conformance.json";
const RECORD_SCHEMA: &str = "teistro-conformance-record/1";

/// The built-in tiers a default build does not compile, so their scores
/// come from verify's tier jobs, each uploading `conformance-<tier>`.
const RECORDED_TIERS: &[&str] = &["compact", "full"];

/// The corpus tests that score a section, by crate.
const TESTS: &[(&str, &[&str])] = &[
    ("teistro-ephemeris-builtin", &["conformance"]),
    (
        "teistro-dasha",
        &["baseline", "systems", "rashi", "kalachakra", "pyjhora"],
    ),
    ("teistro-strength", &["baseline"]),
    ("teistro-rules", &["baseline", "doshas"]),
    ("teistro-calendar", &["official"]),
    ("teistro-siddhanta", &["official"]),
];

/// The corpus sections compared elsewhere and not scored here, each with
/// where it is held instead.
const UNSCORED: &[(&str, &str)] = &[
    (
        "baseline/names.json",
        "vocabulary, not answers: the source `sdk.entity`'s names were migrated from (`crates/intl/src/migrate.rs`, whose tests read it), and a name is vetted by a speaker rather than scored",
    ),
    (
        "teimeris",
        "an engine's own outputs, measured rather than scored: the worst difference of each is on `ACCURACY.md`, held by `cargo xtask check-accuracy`",
    ),
];

#[derive(Deserialize)]
struct Corpus {
    version: String,
    released: String,
    repository: String,
    corpora: Vec<Section>,
}

#[derive(Deserialize)]
struct Section {
    path: String,
    source: String,
    evidence_rank: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Score {
    section: String,
    reading: String,
    compared: usize,
    explained: Vec<Explained>,
    /// The commit and run that wrote it, where a CI run did.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    run: Option<String>,
}

impl Score {
    /// The score without where it came from, which is what two runs of one
    /// corpus must agree on.
    fn answer(&self) -> (&str, &str, usize, &[Explained]) {
        (&self.section, &self.reading, self.compared, &self.explained)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Explained {
    entry: String,
    count: usize,
}

/// The recorded tiers' scores, and the corpus they were made against.
#[derive(Debug, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
struct Record {
    schema: String,
    /// `corpus.json`'s version when the run was recorded, so a bumped
    /// corpus with old scores fails.
    corpus: String,
    tiers: BTreeMap<String, Vec<Score>>,
}

/// Score lines, one JSON object each.
fn lines(text: &str) -> Result<Vec<Score>, String> {
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).map_err(|e| format!("a score line: {e}")))
        .collect()
}

/// The page's order, which is not the order tests ran in.
fn sorted(mut scores: Vec<Score>) -> Vec<Score> {
    scores.sort_by(|a, b| (&a.section, &a.reading).cmp(&(&b.section, &b.reading)));
    scores
}

/// Runs the corpus tests and collects what they recorded.
fn measure(root: &Path) -> Vec<Score> {
    let dir = root.join("target").join("conformance");
    std::fs::create_dir_all(&dir).expect("the conformance directory");
    let file = dir.join(SCORES);
    let _ = std::fs::remove_file(&file);
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    for (krate, tests) in TESTS {
        let mut command = Command::new(&cargo);
        command.args(["test", "--quiet", "-p", krate]);
        for test in *tests {
            command.args(["--test", test]);
        }
        let status = command
            .env(DIR_ENV, &dir)
            .current_dir(root)
            .status()
            .expect("cargo runs");
        assert!(status.success(), "{krate}'s corpus tests failed");
    }
    let text = std::fs::read_to_string(&file).unwrap_or_default();
    sorted(lines(&text).expect("the score file"))
}

/// The tiers a downloaded run holds: each `conformance-<tier>` directory's
/// `scores.jsonl`, for the tiers recorded.
fn downloaded(dir: &Path) -> Result<BTreeMap<String, Vec<Score>>, String> {
    let mut tiers = BTreeMap::new();
    for tier in RECORDED_TIERS {
        let path = dir.join(format!("conformance-{tier}")).join(SCORES);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("{}: {e}; the tier job uploads it", path.display()))?;
        let scores = sorted(lines(&text)?);
        if scores.is_empty() {
            return Err(format!(
                "{}: no score, so the tier compared nothing",
                path.display()
            ));
        }
        tiers.insert((*tier).to_owned(), scores);
    }
    Ok(tiers)
}

fn read_record(root: &Path) -> Result<Record, String> {
    let path = root.join(RECORD);
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{RECORD}: {e}"))?;
    let record: Record = serde_json::from_str(&text).map_err(|e| format!("{RECORD}: {e}"))?;
    if record.schema != RECORD_SCHEMA {
        return Err(format!(
            "{RECORD}: schema {}, not {RECORD_SCHEMA}",
            record.schema
        ));
    }
    Ok(record)
}

/// What is wrong with the record against the corpus and the tiers it
/// covers: another corpus's scores, a tier missing, or one it should not
/// hold.
fn stale(record: &Record, corpus: &Corpus) -> Vec<String> {
    let mut found = Vec::new();
    if record.corpus != corpus.version {
        found.push(format!(
            "{RECORD} was recorded against corpus {}, and {CORPUS} is {}; record a verify run with `cargo xtask conformance --from DIR`",
            record.corpus, corpus.version
        ));
    }
    for tier in RECORDED_TIERS {
        if !record.tiers.contains_key(*tier) {
            found.push(format!("{RECORD} has no `{tier}` tier"));
        }
    }
    for tier in record.tiers.keys() {
        if !RECORDED_TIERS.contains(&tier.as_str()) {
            found.push(format!(
                "{RECORD} records `{tier}`, which is not a recorded tier"
            ));
        }
    }
    found
}

/// Scores without their provenance.
fn answers(scores: &[Score]) -> Vec<(&str, &str, usize, &[Explained])> {
    scores.iter().map(Score::answer).collect()
}

/// A run's tiers against the record, exactly, where they move.
fn moved(record: &Record, run: &BTreeMap<String, Vec<Score>>) -> Vec<String> {
    let mut found = Vec::new();
    for (tier, scores) in run {
        let recorded = record
            .tiers
            .get(tier)
            .map(Vec::as_slice)
            .unwrap_or_default();
        if answers(scores) != answers(recorded) {
            found.push(format!(
                "the `{tier}` tier scored {:?}, and {RECORD} has {:?}",
                answers(scores),
                answers(recorded)
            ));
        }
    }
    found
}

/// `conformance --from DIR` writes the record from a downloaded run, which
/// `conformance` then renders; `check-conformance --from DIR` holds the
/// run to the record, without running the live tests, which fast-check
/// does.
pub(crate) fn recorded(root: &Path, command: &str, args: &[String]) -> i32 {
    let [_, flag, dir] = args else {
        eprintln!("usage: cargo xtask {command} --from DIR");
        return 2;
    };
    if flag != "--from" {
        eprintln!("usage: cargo xtask {command} --from DIR");
        return 2;
    }
    let run = match downloaded(Path::new(dir)) {
        Ok(run) => run,
        Err(why) => {
            println!("FAIL  {why}");
            return 1;
        }
    };
    let corpus = read_corpus(root);
    if command == "conformance" {
        let record = Record {
            schema: RECORD_SCHEMA.to_owned(),
            corpus: corpus.version.clone(),
            tiers: run,
        };
        let text = serde_json::to_string_pretty(&record).unwrap_or_default() + "\n";
        if let Err(e) = std::fs::write(root.join(RECORD), text) {
            println!("FAIL  {RECORD}: {e}");
            return 1;
        }
        println!("wrote {RECORD}; `cargo xtask conformance` renders the page over it");
        return 0;
    }
    let found = match read_record(root) {
        Ok(record) => {
            let mut found = stale(&record, &corpus);
            found.extend(moved(&record, &run));
            found
        }
        Err(why) => vec![why],
    };
    for line in &found {
        println!("FAIL  {line}");
    }
    if found.is_empty() {
        println!("ok    the recorded tiers score as {RECORD} records");
    }
    i32::from(!found.is_empty())
}

fn read_corpus(root: &Path) -> Corpus {
    serde_json::from_str(&std::fs::read_to_string(root.join(CORPUS)).expect("the corpus index"))
        .expect("a valid corpus index")
}

/// What is wrong with the scores against the corpus's sections, if
/// anything: a section neither scored nor declared, declared and scored
/// too, declared and not in the corpus, or scored and not in it.
fn mismatches(corpus: &Corpus, scores: &[Score]) -> Vec<String> {
    let mut found = Vec::new();
    let has_score = |path: &str| scores.iter().any(|score| score.section == path);
    let declared = |path: &str| UNSCORED.iter().any(|(section, _)| *section == path);
    for section in &corpus.corpora {
        match (has_score(&section.path), declared(&section.path)) {
            (false, false) => found.push(format!(
                "{}: no test scores it and `UNSCORED` gives no reason",
                section.path
            )),
            (true, true) => found.push(format!(
                "{}: scored, and declared unscored too",
                section.path
            )),
            _ => {}
        }
    }
    let known = |path: &str| corpus.corpora.iter().any(|section| section.path == path);
    for (section, _) in UNSCORED {
        if !known(section) {
            found.push(format!("{section}: declared unscored and not in {CORPUS}"));
        }
    }
    for score in scores {
        if !known(&score.section) {
            found.push(format!(
                "{}: a test scored it and it is not in {CORPUS}",
                score.section
            ));
        }
    }
    found
}

/// What an explained entry fails to name, if anything. An entry is a
/// `KNOWN` divergence by its name, or a reading that ends by citing its
/// page, `(page.md)` or `(page.md, ANCHOR)`: the page must be one file
/// under `docs`, and the anchor (a crux, a finding, a rank) a word on it.
fn uncited(docs: &[(String, String)], entry: &str) -> Option<String> {
    if corpus::KNOWN
        .iter()
        .chain(&jpl::KNOWN)
        .any(|divergence| divergence.name == entry)
    {
        return None;
    }
    let Some(cited) = entry
        .strip_suffix(')')
        .and_then(|rest| rest.rsplit_once('('))
        .map(|(_, cited)| cited)
    else {
        return Some(format!(
            "{entry:?}: neither a `KNOWN` divergence nor a reading citing its page"
        ));
    };
    let (page, anchor) = cited
        .split_once(", ")
        .map_or((cited, None), |(page, anchor)| (page, Some(anchor)));
    let found: Vec<&(String, String)> = docs.iter().filter(|(name, _)| name == page).collect();
    let [(_, text)] = found.as_slice() else {
        return Some(format!(
            "{entry:?}: cites `{page}`, which is {} pages under docs, not one",
            found.len()
        ));
    };
    let word = |word: &str| {
        text.split(|c: char| !c.is_ascii_alphanumeric())
            .any(|w| w == word)
    };
    match anchor {
        Some(anchor) if !word(anchor) => Some(format!("{entry:?}: `{page}` never names {anchor}")),
        _ => None,
    }
}

/// Every markdown page under `docs`, by file name.
fn pages(root: &Path) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut stack = vec![root.join("docs")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("the docs tree").flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                let name = entry.file_name().to_string_lossy().into_owned();
                found.push((name, std::fs::read_to_string(&path).expect("a page")));
            }
        }
    }
    found
}

/// Whether the submodule is pinned at the tag `corpus.json` names, read
/// from the corpus repository itself, so a corpus pinned between releases
/// or a bumped index with a stale pin fails.
fn untagged(root: &Path, corpus: &Corpus) -> Option<String> {
    let git = |args: &[&str]| {
        Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let pinned = git(&["-C", "fixtures", "rev-parse", "HEAD"])?
        .trim()
        .to_owned();
    let tag = format!("refs/tags/v{}", corpus.version);
    let peeled = format!("{tag}^{{}}");
    let Some(listed) = git(&["ls-remote", &corpus.repository, &tag, &peeled]) else {
        return Some(format!(
            "{CORPUS}: could not list {tag} in {}, so the pin is unchecked",
            corpus.repository
        ));
    };
    let commit = |name: &str| {
        listed.lines().find_map(|line| {
            let (commit, named) = line.split_once('\t')?;
            (named == name).then_some(commit)
        })
    };
    match commit(&peeled).or_else(|| commit(&tag)) {
        Some(commit) if commit == pinned => None,
        Some(commit) => Some(format!(
            "fixtures is pinned at {pinned}, and {tag} is {commit}"
        )),
        None => Some(format!(
            "{CORPUS} names {}, and {} has no tag {tag}",
            corpus.version, corpus.repository
        )),
    }
}

fn render(corpus: &Corpus, scores: &[Score], record: &Record) -> String {
    let mut by_section: BTreeMap<&str, Vec<&Score>> = BTreeMap::new();
    for score in scores {
        by_section
            .entry(score.section.as_str())
            .or_default()
            .push(score);
    }
    let mut s = String::new();
    s.push_str("# Conformance\n\n");
    s.push_str(
        "Status: `generated`, by `cargo xtask conformance` from the crates' corpus tests\n\
         and `fixtures/corpus.json`, held by `cargo xtask check-conformance`; do not edit.\n\n",
    );
    let _ = writeln!(
        s,
        "The SDK against the conformance corpus {} (released {},\n\
         {}), one section per corpus directory. Each row is one test's comparisons under one reading:\n\
         **compared** counts every comparison the test attempted, **explained** the\n\
         misses a named entry accounts for, listed below with its count, and\n\
         **agree** the rest. A miss no entry explains fails its test, so the page\n\
         cannot be regenerated with one: **unexplained misses: 0**. There is no total\n\
         across sections, since a yoga decision and a longitude are not one unit\n\
         (`03-design/generated-pages.md` §2.1).\n",
        corpus.version, corpus.released, corpus.repository
    );
    let runs: Vec<String> = record
        .tiers
        .iter()
        .map(|(tier, scores)| {
            let from = match scores.first().map(|score| (&score.run, &score.commit)) {
                Some((Some(run), Some(commit))) => {
                    let commit: String = commit.chars().take(12).collect();
                    format!(" from run `{run}` at commit `{commit}`")
                }
                _ => " by hand, outside CI".to_owned(),
            };
            format!("`{tier}`{from}")
        })
        .collect();
    let _ = writeln!(
        s,
        "The default tier's positions are scored live; the other built-in tiers are\n\
         recorded from verify's tier jobs ({}), and verify holds each later run to\n\
         the record exactly.\n",
        runs.join(", ")
    );
    s.push_str("| section | rank | reading | compared | agree | explained |\n");
    s.push_str("|---|---|---|---|---|---|\n");
    for section in &corpus.corpora {
        let Some(list) = by_section.get(section.path.as_str()) else {
            continue;
        };
        for score in list {
            let explained: usize = score.explained.iter().map(|e| e.count).sum();
            let _ = writeln!(
                s,
                "| `{}` | {} | {} | {} | {} | {} |",
                section.path,
                section.evidence_rank,
                score.reading,
                score.compared,
                score.compared - explained,
                explained
            );
        }
    }
    s.push_str("\n## Explained misses\n\n");
    let mut any = false;
    for section in &corpus.corpora {
        for score in by_section.get(section.path.as_str()).into_iter().flatten() {
            for entry in &score.explained {
                any = true;
                let _ = writeln!(
                    s,
                    "- `{}`, {}: {} — {}",
                    section.path, score.reading, entry.entry, entry.count
                );
            }
        }
    }
    if !any {
        s.push_str("None.\n");
    }
    s.push_str("\n## Not scored here\n\n");
    for section in &corpus.corpora {
        if let Some((_, why)) = UNSCORED.iter().find(|(path, _)| *path == section.path) {
            let _ = writeln!(s, "- `{}` ({}): {why}.", section.path, section.source);
        }
    }
    s
}

fn outputs(root: &Path) -> Vec<Output> {
    let corpus = read_corpus(root);
    let record = read_record(root).unwrap_or_else(|why| panic!("{why}"));
    let live = measure(root);
    let docs = pages(root);
    let mut found = mismatches(&corpus, &live);
    found.extend(stale(&record, &corpus));
    let scores = sorted(
        live.into_iter()
            .chain(record.tiers.values().flatten().cloned())
            .collect(),
    );
    found.extend(
        scores
            .iter()
            .flat_map(|score| &score.explained)
            .filter_map(|explained| uncited(&docs, &explained.entry)),
    );
    found.extend(untagged(root, &corpus));
    assert!(
        found.is_empty(),
        "the scores against the corpus:\n{}",
        found.join("\n")
    );
    vec![Output::new(DOCUMENT, render(&corpus, &scores, &record))]
}

pub(crate) fn generate(root: &Path) -> i32 {
    write(root, &outputs(root))
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    let failures = check(root, &outputs(root), "cargo xtask conformance");
    i32::from(failures != 0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn corpus(paths: &[&str]) -> Corpus {
        Corpus {
            version: "0.0.0".into(),
            released: "2026-01-01".into(),
            repository: "r".into(),
            corpora: paths
                .iter()
                .map(|path| Section {
                    path: (*path).into(),
                    source: "s".into(),
                    evidence_rank: 2,
                })
                .collect(),
        }
    }

    #[test]
    fn an_entry_is_known_or_cites_a_page_that_names_it() {
        let docs = vec![
            ("one.md".to_owned(), "R2: the committee's (C12).".to_owned()),
            ("twice.md".to_owned(), String::new()),
            ("twice.md".to_owned(), String::new()),
        ];
        assert_eq!(uncited(&docs, corpus::KNOWN[0].name), None);
        assert_eq!(uncited(&docs, jpl::KNOWN[0].name), None);
        assert_eq!(uncited(&docs, "a reading (one.md)"), None);
        assert_eq!(uncited(&docs, "a reading (one.md, R2)"), None);
        assert_eq!(uncited(&docs, "a reading (one.md, C12)"), None);
        let refused = [
            "an unnamed reading",
            "a reading (gone.md)",
            "a reading (twice.md)",
            "a reading (one.md, R3)",
            // A word inside another is not the word.
            "a reading (one.md, C1)",
        ];
        for entry in refused {
            assert!(uncited(&docs, entry).is_some(), "{entry}");
        }
    }

    fn score(section: &str) -> Score {
        Score {
            section: section.into(),
            reading: "r".into(),
            compared: 1,
            explained: Vec::new(),
            commit: None,
            run: None,
        }
    }

    fn record(version: &str, tiers: &[&str]) -> Record {
        Record {
            schema: RECORD_SCHEMA.into(),
            corpus: version.into(),
            tiers: tiers
                .iter()
                .map(|tier| ((*tier).to_owned(), vec![score("baseline")]))
                .collect(),
        }
    }

    #[test]
    fn a_record_is_of_this_corpus_and_every_recorded_tier() {
        let corpus = corpus(&[]);
        assert_eq!(
            stale(&record("0.0.0", RECORDED_TIERS), &corpus),
            Vec::<String>::new()
        );
        let found = stale(&record("0.0.1", RECORDED_TIERS), &corpus);
        assert!(
            found
                .iter()
                .any(|f| f.contains("recorded against corpus 0.0.1")),
            "{found:?}"
        );
        let found = stale(&record("0.0.0", &RECORDED_TIERS[1..]), &corpus);
        assert!(
            found.iter().any(|f| f.contains("no `compact`")),
            "{found:?}"
        );
        let mut tiers = RECORDED_TIERS.to_vec();
        tiers.push("standard");
        let found = stale(&record("0.0.0", &tiers), &corpus);
        assert!(
            found.iter().any(|f| f.contains("`standard`, which is not")),
            "{found:?}"
        );
    }

    #[test]
    fn a_run_is_held_to_the_record_exactly_and_not_to_its_provenance() {
        let held = record("0.0.0", RECORDED_TIERS);
        let mut run = held.tiers.clone();
        for scores in run.values_mut() {
            for score in scores {
                score.commit = Some("abc".into());
                score.run = Some("1".into());
            }
        }
        assert_eq!(moved(&held, &run), Vec::<String>::new());
        if let Some(score) = run.get_mut("full").and_then(|scores| scores.first_mut()) {
            score.compared += 1;
        }
        let found = moved(&held, &run);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].contains("`full`"), "{found:?}");
    }

    #[test]
    fn a_download_holds_every_recorded_tier() {
        let dir = std::env::temp_dir().join(format!("teistro-conformance-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let line = r#"{"section":"baseline","reading":"r","compared":3,"explained":[],"commit":"c","run":"9"}"#;
        for tier in RECORDED_TIERS {
            let tier_dir = dir.join(format!("conformance-{tier}"));
            std::fs::create_dir_all(&tier_dir).unwrap();
            std::fs::write(tier_dir.join(SCORES), format!("{line}\n")).unwrap();
        }
        let run = downloaded(&dir).unwrap();
        assert_eq!(run.len(), RECORDED_TIERS.len());
        std::fs::write(dir.join("conformance-full").join(SCORES), "").unwrap();
        assert!(downloaded(&dir).unwrap_err().contains("compared nothing"));
        std::fs::remove_dir_all(dir.join("conformance-compact")).unwrap();
        assert!(
            downloaded(&dir)
                .unwrap_err()
                .contains("conformance-compact")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_section_is_scored_or_declared_both_ways() {
        let declared: Vec<&str> = UNSCORED.iter().map(|(path, _)| *path).collect();
        let mut paths = declared.clone();
        paths.push("baseline/yogas");
        assert_eq!(
            mismatches(&corpus(&paths), &[score("baseline/yogas")]),
            Vec::<String>::new()
        );
        // A section nothing scores and nothing declares.
        paths.push("baseline/new");
        let found = mismatches(&corpus(&paths), &[score("baseline/yogas")]);
        assert_eq!(found.len(), 1, "{found:?}");
        assert!(found[0].starts_with("baseline/new"));
        // A declared section a test scores too.
        let found = mismatches(&corpus(&declared), &[score(declared[0])]);
        assert!(
            found.iter().any(|f| f.contains("declared unscored too")),
            "{found:?}"
        );
        // A declared section the corpus no longer has.
        let found = mismatches(&corpus(&declared[1..]), &[]);
        assert!(found.iter().any(|f| f.contains("not in")), "{found:?}");
        // A score for a section the corpus does not have.
        let found = mismatches(&corpus(&declared), &[score("elsewhere")]);
        assert!(
            found.iter().any(|f| f.starts_with("elsewhere")),
            "{found:?}"
        );
    }
}
