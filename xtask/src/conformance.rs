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
//! declared unscored here with its reason, both ways.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde::Deserialize;

use crate::generated::{Output, check, write};

const DOCUMENT: &str = "docs/05-testing/CONFORMANCE.md";
const CORPUS: &str = "fixtures/corpus.json";
const SCORES: &str = "scores.jsonl";
const DIR_ENV: &str = "TEISTRO_CONFORMANCE_DIR";

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

#[derive(Deserialize)]
struct Score {
    section: String,
    reading: String,
    compared: usize,
    explained: Vec<Explained>,
}

#[derive(Deserialize)]
struct Explained {
    entry: String,
    count: usize,
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
    let mut scores: Vec<Score> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("a score line"))
        .collect();
    // Tests run at once, so the file's order is not the page's.
    scores.sort_by(|a, b| (&a.section, &a.reading).cmp(&(&b.section, &b.reading)));
    scores
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

fn render(corpus: &Corpus, scores: &[Score]) -> String {
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
    let corpus: Corpus = serde_json::from_str(
        &std::fs::read_to_string(root.join(CORPUS)).expect("the corpus index"),
    )
    .expect("a valid corpus index");
    let scores = measure(root);
    let found = mismatches(&corpus, &scores);
    assert!(
        found.is_empty(),
        "the scores against the corpus:\n{}",
        found.join("\n")
    );
    vec![Output::new(DOCUMENT, render(&corpus, &scores))]
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

    fn score(section: &str) -> Score {
        Score {
            section: section.into(),
            reading: "r".into(),
            compared: 1,
            explained: Vec::new(),
        }
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
