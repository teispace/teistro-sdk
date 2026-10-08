//! The score a conformance test keeps as it compares the SDK with a corpus
//! section, for `CONFORMANCE.md` (`docs/03-design/generated-pages.md` §2).
//!
//! Shared by every crate's corpus tests through `#[path]`, so the score has
//! one shape and one writer. A test compares through a [`Tally`] instead of
//! a bare `assert_eq!`: each comparison is counted as it is made, a miss
//! still fails the test, and a miss a named entry explains (a `KNOWN`
//! divergence, a crux) is counted against that entry. At the end the test
//! calls [`Tally::record`], which appends one JSON line to
//! `scores.jsonl` in the directory `TEISTRO_CONFORMANCE_DIR` names, and
//! does nothing without it, so an ordinary `cargo test` leaves no file.
//!
//! A section whose test failed records nothing, and `cargo xtask
//! conformance` refuses a section with no score: an unexplained miss can
//! never reach the page as a number.

#![allow(
    dead_code,
    reason = "each test binary uses the part of this module it needs"
)]

use std::fmt::{Debug, Write as _};
use std::io::Write as _;

/// The directory a score is written into, when set.
pub(crate) const DIR_ENV: &str = "TEISTRO_CONFORMANCE_DIR";

/// The file the scores are appended to, inside that directory.
pub(crate) const FILE: &str = "scores.jsonl";

/// One corpus section's comparisons, counted as they are made.
#[derive(Debug)]
#[must_use = "a tally records nothing until `record` is called"]
pub(crate) struct Tally {
    /// The corpus path the section is (`baseline/shadbala`), as
    /// `fixtures/corpus.json` names it.
    section: &'static str,
    /// What the comparisons were made under (`the engine's reading`).
    reading: String,
    /// Every comparison made, explained misses included.
    compared: usize,
    /// The misses each named entry explains, in the order first met.
    explained: Vec<(&'static str, usize)>,
}

impl Tally {
    /// A tally for one section under one reading.
    pub(crate) fn new(section: &'static str, reading: impl Into<String>) -> Self {
        Self {
            section,
            reading: reading.into(),
            compared: 0,
            explained: Vec::new(),
        }
    }

    /// One comparison that must agree exactly.
    #[track_caller]
    #[allow(
        clippy::needless_pass_by_value,
        reason = "taken by value as `assert_eq!` takes them, so a call reads as one"
    )]
    pub(crate) fn same<A: PartialEq<B> + Debug, B: Debug>(
        &mut self,
        ours: A,
        recorded: B,
        what: impl Fn() -> String,
    ) {
        self.compared += 1;
        assert_eq!(ours, recorded, "{}", what());
    }

    /// One comparison that must agree within `bound`.
    #[track_caller]
    pub(crate) fn within(
        &mut self,
        ours: f64,
        recorded: f64,
        bound: f64,
        what: impl Fn() -> String,
    ) {
        self.compared += 1;
        assert!(
            (ours - recorded).abs() <= bound,
            "{}: {ours} against {recorded}, past {bound}",
            what()
        );
    }

    /// One comparison stated as a condition that must hold.
    #[track_caller]
    pub(crate) fn holds(&mut self, agrees: bool, what: impl Fn() -> String) {
        self.compared += 1;
        assert!(agrees, "{}", what());
    }

    /// `count` comparisons that agreed, where the test decides agreement
    /// itself (a convention it counts rather than fails on).
    pub(crate) fn agreed(&mut self, count: usize) {
        self.compared += count;
    }

    /// `count` comparisons that miss, each explained by `entry`: compared,
    /// and counted against the entry rather than as agreement.
    pub(crate) fn explained(&mut self, entry: &'static str, count: usize) {
        if count == 0 {
            return;
        }
        self.compared += count;
        match self.explained.iter_mut().find(|(named, _)| *named == entry) {
            Some((_, held)) => *held += count,
            None => self.explained.push((entry, count)),
        }
    }

    /// How many comparisons were made.
    pub(crate) fn compared(&self) -> usize {
        self.compared
    }

    /// Writes the score, when `TEISTRO_CONFORMANCE_DIR` is set.
    ///
    /// # Panics
    ///
    /// When nothing was compared, which is a section silently skipped
    /// (`count-attempts-not-reports`), or when the file cannot be written.
    pub(crate) fn record(self) {
        assert!(self.compared > 0, "{}: nothing was compared", self.section);
        let Some(dir) = std::env::var_os(DIR_ENV) else {
            return;
        };
        let mut explained = String::new();
        for (index, (entry, count)) in self.explained.iter().enumerate() {
            let comma = if index == 0 { "" } else { "," };
            let _ = write!(
                explained,
                "{comma}{{\"entry\":{},\"count\":{count}}}",
                quoted(entry)
            );
        }
        let line = format!(
            "{{\"section\":{},\"reading\":{},\"compared\":{},\"explained\":[{explained}]}}\n",
            quoted(self.section),
            quoted(&self.reading),
            self.compared,
        );
        let path = std::path::Path::new(&dir).join(FILE);
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        // One write per line, so tests running at once never interleave.
        file.write_all(line.as_bytes())
            .unwrap_or_else(|err| panic!("{}: {err}", path.display()));
    }
}

/// A JSON string, for the plain ASCII names a section and an entry are.
fn quoted(text: &str) -> String {
    assert!(
        text.chars()
            .all(|c| c != '"' && c != '\\' && !c.is_control()),
        "{text:?} needs escaping, which a section or entry name never should"
    );
    format!("\"{text}\"")
}
