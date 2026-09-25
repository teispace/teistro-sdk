//! The instruction-count benchmarks: how many instructions the fixed
//! scenario costs, counted rather than timed.
//!
//! Wall-clock time on a shared runner is noise: a neighbouring job moves
//! it further than most changes do, so a wall-clock gate either passes
//! everything or fails at random. Callgrind counts instructions instead,
//! and the count barely moves — the same scenario counted twice differs
//! by a few parts in a million, on a busy machine and an idle one, where
//! wall-clock time differs by tens of per cent. That is what makes a 1%
//! threshold meaningful (ADR-0022's quality bar: fail above 3%, warn
//! above 1%).
//!
//! The scenario is `teistro-scenario`, the same code the determinism
//! matrix hashes, so neither gate measures a path the other never walks.
//! It is run once per section and once doing nothing at all; the
//! difference is what that section costs, with the process's own start-up
//! taken out.
//!
//! `cargo xtask bench [FILE]` writes the counts; `cargo xtask
//! compare-bench BASE HEAD` compares two such files. The comparison is
//! against the pull request's base commit rather than a checked-in
//! number, because an instruction count belongs to a compiler and a
//! target as much as to the source: measuring both sides in one job on
//! one machine is the only comparison that means anything.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use teistro_scenario::SECTIONS;

use crate::binding::{build, present};
use crate::{read, rel};

/// The name of the run that does no work: what the process costs before
/// any section is walked, and what every section's count has taken out of
/// it.
const NOTHING: &str = "nothing";

/// Above this, a change is reported as a regression and the gate fails.
const FAIL: f64 = 3.0;
/// Above this, a change is worth a reader's attention.
const WARN: f64 = 1.0;

// ── bench ──────────────────────────────────────────────────────────────────

/// Counts what each section of the scenario costs, and writes the counts
/// where a later run can compare them.
pub(crate) fn report(root: &Path, out: Option<&Path>) -> i32 {
    if !present("valgrind", "--version") {
        eprintln!(
            "no `valgrind` on this machine; instruction counts need it (Linux, `apt install valgrind`)"
        );
        return 0;
    }
    if build(root, "teistro-scenario", "the scenario").is_err() {
        return 1;
    }
    let binary = root.join("target/release/teistro-scenario");
    let into = root.join("target/callgrind");
    if std::fs::create_dir_all(&into).is_err() {
        println!("FAIL  {} could not be created", rel(root, &into));
        return 1;
    }

    let Some(overhead) = count(&binary, NOTHING, &into) else {
        println!("FAIL  callgrind could not count the empty run");
        return 1;
    };
    let mut counts: BTreeMap<&str, u64> = BTreeMap::new();
    for section in SECTIONS {
        let Some(total) = count(&binary, section, &into) else {
            println!("FAIL  callgrind could not count `{section}`");
            return 1;
        };
        // The empty run is a floor, not a promise: a section that somehow
        // costs less than start-up is reported as zero rather than as a
        // number that wrapped.
        counts.insert(section, total.saturating_sub(overhead));
    }

    println!("{:<12} {:>16}", "section", "instructions");
    for (section, count) in &counts {
        println!("{section:<12} {count:>16}");
    }
    println!("{NOTHING:<12} {overhead:>16}  (the empty run, taken out of every section above)");

    if let Some(path) = out {
        let mut text = String::new();
        for (section, count) in &counts {
            let _ = writeln!(text, "{section}\t{count}");
        }
        if let Err(err) = std::fs::write(path, text) {
            println!("FAIL  cannot write {}: {err}", path.display());
            return 1;
        }
        println!("wrote {}", path.display());
    }
    0
}

/// Runs one section under callgrind and reads the instruction count out
/// of the profile it wrote.
fn count(binary: &Path, section: &str, into: &Path) -> Option<u64> {
    let profile = into.join(format!("callgrind.{section}.out"));
    let status = Command::new("valgrind")
        .args([
            "--tool=callgrind",
            // Instructions alone: the cache and branch simulators model a
            // processor this build may never run on, and their numbers
            // move with the model rather than with the code.
            "--cache-sim=no",
            "--branch-sim=no",
        ])
        .arg(format!("--callgrind-out-file={}", profile.display()))
        .arg(binary)
        .arg(section)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
    if !status.is_ok_and(|s| s.success()) {
        return None;
    }
    instructions(&read(&profile))
}

/// The instruction count in a callgrind profile: its `totals:` line, or
/// its `summary:` line when the version wrote one.
fn instructions(profile: &str) -> Option<u64> {
    for prefix in ["totals:", "summary:"] {
        if let Some(count) = profile
            .lines()
            .find_map(|line| line.strip_prefix(prefix))
            .and_then(|rest| rest.split_whitespace().next())
            .and_then(|number| number.parse().ok())
        {
            return Some(count);
        }
    }
    None
}

// ── compare-bench ──────────────────────────────────────────────────────────

/// A cost a change declared on purpose: a commit's
/// `Instruction-cost: <section> +<percent>% (<reason>)` trailer.
///
/// The gate has one threshold and no memory, so a deliberate price — one
/// `libm` on every target is the first, ADR-0022 — would otherwise fail it
/// or be waved past it. The trailer is reviewed with the change, lives in
/// its history, and belongs to that change alone: the workflow reads only
/// the pull request's own commits, so an acceptance never outlives its
/// pull request and never excuses the next one.
#[derive(Debug, Clone, PartialEq)]
struct Accepted {
    section: String,
    percent: f64,
    reason: String,
}

/// The trailers' values, one a line, as `git log --format=
/// '%(trailers:key=Instruction-cost,valueonly)'` prints them; a line that
/// does not read as one is an error rather than a silence.
fn accepted_costs(text: &str) -> Result<Vec<Accepted>, String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| {
            let refuse = || {
                format!(
                    "`Instruction-cost: {line}` does not read as `<section> +<percent>% (<reason>)`"
                )
            };
            let (section, rest) = line.split_once(' ').ok_or_else(refuse)?;
            let (percent, reason) = rest.trim().split_once('%').ok_or_else(refuse)?;
            let percent: f64 = percent
                .trim()
                .strip_prefix('+')
                .and_then(|p| p.parse().ok())
                .ok_or_else(refuse)?;
            let reason = reason
                .trim()
                .strip_prefix('(')
                .and_then(|r| r.strip_suffix(')'))
                .map(str::trim)
                .filter(|r| !r.is_empty())
                .ok_or_else(refuse)?;
            Ok(Accepted {
                section: section.to_string(),
                percent,
                reason: reason.to_string(),
            })
        })
        .collect()
}

/// What a comparison found: its report, line by line, and its failures.
#[derive(Debug, Default)]
struct Judgement {
    lines: Vec<String>,
    failures: Vec<String>,
    worst: f64,
}

/// Judges two counted runs against the costs the change accepted.
///
/// **An acceptance is held both ways.** A section may cost up to what its
/// trailer says and no more; and a trailer for a section that did not
/// cost more than the threshold fails too, because an excuse nothing
/// needed is one that would silently cover the next regression.
fn judge(
    before: &BTreeMap<String, u64>,
    after: &BTreeMap<String, u64>,
    accepted: &[Accepted],
) -> Judgement {
    let mut out = Judgement::default();
    out.lines.push(format!(
        "{:<12} {:>16} {:>16} {:>9}",
        "section", "base", "head", "change"
    ));
    let mut sections: Vec<&String> = before.keys().chain(after.keys()).collect();
    sections.sort_unstable();
    sections.dedup();
    let mut changes: BTreeMap<&str, f64> = BTreeMap::new();
    for section in sections {
        match (before.get(section), after.get(section)) {
            (Some(&base), Some(&head)) => {
                #[expect(
                    clippy::cast_precision_loss,
                    reason = "an instruction count is far below the precision limit of a double"
                )]
                let change = if base == 0 {
                    0.0
                } else {
                    (head as f64 - base as f64) / base as f64 * 100.0
                };
                out.lines.push(format!(
                    "{section:<12} {base:>16} {head:>16} {change:>8.2}%"
                ));
                changes.insert(section, change);
                let declared = accepted.iter().find(|a| a.section == *section);
                match declared {
                    Some(a) if change > FAIL && change <= a.percent => out.lines.push(format!(
                        "accepted  {section} costs {change:.2}%, within the {}% its commit accepted: {}",
                        a.percent, a.reason
                    )),
                    Some(a) if change > a.percent && change > FAIL => out.failures.push(format!(
                        "{section} costs {change:.2}% more, which is more than the {}% its commit accepted",
                        a.percent
                    )),
                    _ if change > FAIL => out.failures.push(format!(
                        "{section} costs {change:.2}% more, which is more than the {FAIL}% a change may cost"
                    )),
                    _ => out.worst = out.worst.max(change),
                }
            }
            (None, Some(&head)) => out
                .lines
                .push(format!("{section:<12} {:>16} {head:>16} {:>9}", "—", "new")),
            (Some(&base), None) => out.lines.push(format!(
                "{section:<12} {base:>16} {:>16} {:>9}",
                "—", "gone"
            )),
            (None, None) => {}
        }
    }
    for a in accepted {
        match changes.get(a.section.as_str()) {
            None => out.failures.push(format!(
                "a commit accepts a cost for `{}`, which is not a section both runs counted",
                a.section
            )),
            Some(&change) if change <= FAIL => out.failures.push(format!(
                "a commit accepts {}% for `{}`, which cost {change:.2}%: the acceptance is not needed and would cover the next regression",
                a.percent, a.section
            )),
            Some(_) => {}
        }
    }
    out
}

/// Compares two counted runs and says what moved, by how much, and
/// whether that is a regression; `accepted`, when given, is the file of
/// `Instruction-cost` trailers the change's own commits carry.
pub(crate) fn compare(base: &Path, head: &Path, accepted: Option<&Path>) -> i32 {
    let (before, after) = (counts(base), counts(head));
    if before.is_empty() || after.is_empty() {
        println!(
            "FAIL  a run has no counts: {} and {}",
            base.display(),
            head.display()
        );
        return 1;
    }
    let declared = match accepted.map(|path| accepted_costs(&read(path))) {
        None => Vec::new(),
        Some(Ok(declared)) => declared,
        Some(Err(refusal)) => {
            println!("FAIL  {refusal}");
            return 1;
        }
    };
    let judgement = judge(&before, &after, &declared);
    for line in &judgement.lines {
        println!("{line}");
    }
    if !judgement.failures.is_empty() {
        for failure in &judgement.failures {
            println!("FAIL  {failure}");
        }
        return 1;
    }
    let worst = judgement.worst;
    if worst > WARN {
        println!(
            "warn  the worst section costs {worst:.2}% more, which is over {WARN}% and under {FAIL}%: worth a look, not a failure"
        );
    } else {
        println!("ok    nothing costs more than {WARN}% (the worst is {worst:.2}%)");
    }
    0
}

/// A counted run, read back.
fn counts(path: &Path) -> BTreeMap<String, u64> {
    read(path)
        .lines()
        .filter_map(|line| {
            let (section, count) = line.split_once('\t')?;
            Some((section.to_string(), count.trim().parse().ok()?))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Accepted, BTreeMap, accepted_costs, instructions, judge};

    fn run(pairs: &[(&str, u64)]) -> BTreeMap<String, u64> {
        pairs.iter().map(|(s, n)| ((*s).to_string(), *n)).collect()
    }

    fn accept(section: &str, percent: f64) -> Accepted {
        Accepted {
            section: section.to_string(),
            percent,
            reason: "a reason".to_string(),
        }
    }

    /// A trailer reads as its section, its percentage and its reason, and
    /// anything else is refused rather than ignored.
    #[test]
    fn a_trailer_reads_or_is_refused() {
        let read = accepted_costs("astro +12.5% (one libm, ADR-0022)\n\n").unwrap();
        assert_eq!(
            read,
            [Accepted {
                section: "astro".to_string(),
                percent: 12.5,
                reason: "one libm, ADR-0022".to_string(),
            }]
        );
        for wrong in ["astro 12%", "astro +12% ()", "astro +x% (why)", "astro"] {
            assert!(accepted_costs(wrong).is_err(), "{wrong}");
        }
    }

    /// Unaccepted, a section over the threshold fails; accepted, it passes
    /// up to what was declared and fails beyond it.
    #[test]
    fn an_acceptance_covers_what_it_declares_and_no_more() {
        let before = run(&[("astro", 1000), ("houses", 1000)]);
        let after = run(&[("astro", 1100), ("houses", 1005)]);
        assert_eq!(judge(&before, &after, &[]).failures.len(), 1);
        assert!(
            judge(&before, &after, &[accept("astro", 10.0)])
                .failures
                .is_empty()
        );
        assert_eq!(
            judge(&before, &after, &[accept("astro", 9.5)])
                .failures
                .len(),
            1
        );
    }

    /// An acceptance nothing needed fails, and so does one for a section
    /// that is not there.
    #[test]
    fn an_unneeded_acceptance_fails() {
        let before = run(&[("astro", 1000)]);
        let after = run(&[("astro", 1010)]);
        assert_eq!(
            judge(&before, &after, &[accept("astro", 5.0)])
                .failures
                .len(),
            1
        );
        assert_eq!(
            judge(&before, &after, &[accept("nowhere", 5.0)])
                .failures
                .len(),
            1
        );
    }

    #[test]
    fn a_profile_reports_its_total() {
        let profile =
            "version: 1\ncreator: callgrind-3.22\nevents: Ir\nsummary: 1234567\ntotals: 1234567\n";
        assert_eq!(instructions(profile), Some(1_234_567));
    }

    #[test]
    fn a_profile_with_several_events_reports_the_first() {
        // Instructions are the first event whatever else was collected.
        let profile = "events: Ir Dr Dw\ntotals: 900 100 50\n";
        assert_eq!(instructions(profile), Some(900));
    }

    #[test]
    fn a_profile_that_says_nothing_reports_nothing() {
        assert_eq!(instructions("version: 1\n"), None);
    }
}
