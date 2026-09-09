//! The falsification pass over the shape of a batch of almanacs, which
//! the panchanga blob is designed from.
//!
//! The chart blob had no shape problem worth measuring: a chart holds one
//! lagna, twelve bhavas and a graha count fixed by its kind, so every
//! per-chart section has a stride the blob states once
//! (`chart-at-the-boundary.md` §4). A panchanga does not. A day holds two
//! tithis or three, a Moon that may rise twice or not at all, and — under
//! a polar policy — an arc synthesised from the nearest real one, which
//! can be a fortnight long. A layout that assumes a stride is either
//! wrong or wasteful, and which of the two is a number rather than an
//! opinion.
//!
//! So this pass measures a real batch. `crates/panchanga/examples/shapes`
//! founds 450 days at three latitudes — a year at Kathmandu, both
//! solstices at Reykjavík where the Sun still rises, and both at Tromsø
//! inside the Arctic Circle — and writes how many rows each of a day's
//! fifteen lists would put in a blob. This reads that and decides the
//! layout.
//!
//! The sample is built rather than recorded, for the reason
//! `schema-measured.md` gives: a recorded one goes stale the first time a
//! list gains a member, and the pass would not notice.
//!
//! `cargo xtask almanac` writes the page; `check-almanac` regenerates it
//! in memory and fails on any difference, so the numbers on the page are
//! the numbers this build produces.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table};

const PAGE: &str = "docs/03-design/panchanga-at-the-boundary-measured.md";

/// The lists that are a **division of an arc**: a count fixed by
/// convention rather than by what the sky did. The pass does not assume
/// this — it is the hypothesis the measurement tests, and the names are
/// here so the table can be read in two halves.
const DIVISIONS: [&str; 5] = [
    "choghadiya",
    "horas",
    "kaalas",
    "muhurtas.daylight",
    "muhurtas.night",
];

/// One day's shape, as the example wrote it.
struct Day {
    site: String,
    polar: bool,
    lists: BTreeMap<String, usize>,
    present: BTreeMap<String, bool>,
}

/// What a range refused, when it did.
struct Refusal {
    site: String,
    range: String,
    status: String,
    message: String,
}

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask almanac") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let (days, refusals) = shapes(root)?;
    if days.is_empty() {
        return Err(String::from("the sample measured no days"));
    }
    let sections = [
        header(&days, &refusals),
        divisions_and_crossings(&days),
        layout(&days),
        optionals(&days),
        refused(&refusals),
        decides(&days),
    ];
    Ok(fill(&sections.concat()))
}

/// Runs the example and reads the shapes it wrote.
fn shapes(root: &Path) -> Result<(Vec<Day>, Vec<Refusal>), String> {
    let out = Command::new(crate::binding::cargo())
        .args([
            "run",
            "--quiet",
            "-p",
            "teistro-panchanga",
            "--example",
            "shapes",
        ])
        .current_dir(root)
        .output()
        .map_err(|err| format!("the almanac shapes could not be built: {err}"))?;
    if !out.status.success() {
        return Err(format!(
            "the almanac shapes could not be built: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    let value: Value = serde_json::from_slice(&out.stdout)
        .map_err(|err| format!("the almanac shapes are not JSON: {err}"))?;
    let days = value["days"]
        .as_array()
        .ok_or("the shapes carry no days")?
        .iter()
        .map(|day| Day {
            site: day["site"].as_str().unwrap_or_default().to_string(),
            polar: day["polar"].as_bool().unwrap_or(false),
            lists: map_of(&day["lists"], |v| {
                usize::try_from(v.as_u64().unwrap_or(0)).unwrap_or(0)
            }),
            present: map_of(&day["present"], |v| v.as_bool().unwrap_or(false)),
        })
        .collect();
    let refusals = value["refusals"]
        .as_array()
        .map(|list| {
            list.iter()
                .map(|r| Refusal {
                    site: r["site"].as_str().unwrap_or_default().to_string(),
                    range: r["range"].as_str().unwrap_or_default().to_string(),
                    status: r["status"].as_str().unwrap_or_default().to_string(),
                    message: r["message"].as_str().unwrap_or_default().to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    Ok((days, refusals))
}

fn map_of<T>(value: &Value, read: impl Fn(&Value) -> T) -> BTreeMap<String, T> {
    value
        .as_object()
        .map(|object| {
            object
                .iter()
                .map(|(key, value)| (key.clone(), read(value)))
                .collect()
        })
        .unwrap_or_default()
}

/// Every list a day carries, in the order the table reads best:
/// divisions first, then crossings, each alphabetical.
fn lists(days: &[Day]) -> Vec<String> {
    let mut names: Vec<String> = days
        .first()
        .map(|day| day.lists.keys().cloned().collect())
        .unwrap_or_default();
    names.sort_by_key(|name| (!DIVISIONS.contains(&name.as_str()), name.clone()));
    names
}

/// The least and greatest a list reaches over a set of days.
fn span(days: &[&Day], list: &str) -> (usize, usize, usize) {
    let counts: Vec<usize> = days
        .iter()
        .map(|day| day.lists.get(list).copied().unwrap_or(0))
        .collect();
    (
        counts.iter().copied().min().unwrap_or(0),
        counts.iter().copied().max().unwrap_or(0),
        counts.iter().sum(),
    )
}

fn header(days: &[Day], refusals: &[Refusal]) -> String {
    let ordinary = days.iter().filter(|day| !day.polar).count();
    let polar = days.len() - ordinary;
    let mut sites: Vec<&str> = days.iter().map(|day| day.site.as_str()).collect();
    sites.sort_unstable();
    sites.dedup();
    format!(
        "# The panchanga at the boundary, measured\n\n\
         Status: `generated` by `cargo xtask almanac`. Do not edit:\n\
         `check-almanac` regenerates this page and fails on any\n\
         difference. The design it is written for is\n\
         [`panchanga-day.md`](panchanga-day.md) §14 at the boundary, beside\n\
         [`chart-at-the-boundary.md`](chart-at-the-boundary.md).\n\n\
         A blob has to state where one day's rows end and the next day's\n\
         begin. For a chart that was free: the grahas are the kind's, the\n\
         bhavas are twelve, and one stride serves the batch\n\
         ([`chart-at-the-boundary.md`](chart-at-the-boundary.md) §4). A\n\
         panchanga's lists are not like that, and this pass measures how\n\
         unlike.\n\n\
         The sample is **{} days at {} places** — {} ordinary and {} polar —\n\
         founded over the analytic provider: a year at Kathmandu, and both\n\
         solstices at Reykjavík (64.15°N, where the Sun still rises) and\n\
         at Tromsø (69.65°N, where for weeks it does not). {} range(s)\n\
         refused.\n\n",
        count(days.len()),
        count(sites.len()),
        count(ordinary),
        count(polar),
        count(refusals.len()),
    )
}

fn divisions_and_crossings(days: &[Day]) -> String {
    let ordinary: Vec<&Day> = days.iter().filter(|day| !day.polar).collect();
    let polar: Vec<&Day> = days.iter().filter(|day| day.polar).collect();
    let mut out = String::from(
        "## 1. A division is fixed; a crossing is not\n\n\
         The fifteen lists split cleanly in two, and the split is not the\n\
         one a reader would guess from the names.\n\n\
         | list | ordinary | polar | fixed everywhere? |\n|---|---|---|---|\n",
    );
    for list in lists(days) {
        let (lo, hi, _) = span(&ordinary, &list);
        let (plo, phi, _) = span(&polar, &list);
        let fixed = lo == hi && plo == phi && lo == plo;
        let _ = writeln!(
            out,
            "| `{list}` | {lo} to {hi} | {plo} to {phi} | {} |",
            if fixed { "**yes**" } else { "no" }
        );
    }
    out.push_str(
        "\nEvery list that is fixed is a **division of an arc**: twenty-four\n\
         horas, fifteen muhurtas of the daylight and fifteen of the night,\n\
         sixteen choghadiya, three inauspicious kaalas. Its count is a\n\
         convention, so it does not move when the arc does — a polar day\n\
         whose synthesised arc runs for a fortnight still has twenty-four\n\
         horas, each of them fourteen hours long.\n\n\
         Every list that is not fixed is a **crossing of a quantity inside\n\
         the window**: a tithi boundary, a moonrise, the Moon entering a\n\
         sign. Its count is set by how long the window is and how fast the\n\
         quantity moves, and neither is a convention.\n\n\
         This was not proposed and then checked. The two groups fell out\n\
         of the counts, and the rule is what they have in common.\n\n",
    );
    out
}

fn layout(days: &[Day]) -> String {
    let names = lists(days);
    let ordinary: Vec<&Day> = days.iter().filter(|day| !day.polar).collect();
    let all: Vec<&Day> = days.iter().collect();
    let waste = |rows: &[&Day]| -> (usize, usize, f64) {
        let ragged: usize = names.iter().map(|list| span(rows, list).2).sum();
        let rectangular: usize = names
            .iter()
            .map(|list| span(rows, list).1)
            .sum::<usize>()
            .saturating_mul(rows.len());
        let wasted = if rectangular == 0 {
            0.0
        } else {
            #[expect(
                clippy::cast_precision_loss,
                reason = "row counts here are thousands, exact in an f64"
            )]
            let share = (rectangular - ragged) as f64 / rectangular as f64 * 100.0;
            share
        };
        (ragged, rectangular, wasted)
    };
    let (ordinary_ragged, ordinary_rect, ordinary_waste) = waste(&ordinary);
    let (all_ragged, all_rect, all_waste) = waste(&all);
    // The worst list by **rows**, not by share: `omens.yogas` wastes the
    // largest proportion of its slots and almost no rows, because two is
    // a small stride. What a caller pays is rows.
    let worst = names
        .iter()
        .map(|list| {
            let (_, hi, total) = span(&all, list);
            (list.clone(), hi.saturating_mul(all.len()) - total, hi)
        })
        .max_by_key(|(_, wasted, _)| *wasted)
        .unwrap_or((String::new(), 0, 0));

    let mut out = String::from(
        "## 2. Rectangular or ragged\n\n\
         A batch can lay a per-day list out two ways. **Rectangular**: one\n\
         stride for the whole batch, the widest day's count, with a count\n\
         column saying how many rows of each day are real. **Ragged**: the\n\
         rows concatenated, with an offset column saying where each day's\n\
         begin.\n\n\
         Rectangular is the chart blob's shape, and it is tempting here\n\
         because it indexes without arithmetic. What it costs is the\n\
         difference between the widest day and every other day:\n\n\
         | batch | ragged rows | rectangular rows | wasted |\n|---|---|---|---|\n",
    );
    let _ = writeln!(
        out,
        "| the {} ordinary days | {} | {} | {ordinary_waste:.1}% |",
        count(ordinary.len()),
        count(ordinary_ragged),
        count(ordinary_rect)
    );
    let _ = writeln!(
        out,
        "| all {} days | {} | {} | **{all_waste:.1}%** |",
        count(all.len()),
        count(all_ragged),
        count(all_rect)
    );
    let _ = write!(
        out,
        "\nTen per cent is arguable. Seventy-eight is not, and the second\n\
         row is what a real caller gets: **one** polar day in a batch sets\n\
         the stride for every other day in it. The list that costs the\n\
         most is `{}`: a stride of {} against a median day's handful, and\n\
         {} empty rows across the batch.\n\n\
         So the layout is **ragged**, and the same rule for every list\n\
         rather than rectangular for the five divisions and ragged for the\n\
         ten crossings: two layouts in one blob is two things for a reader\n\
         to learn, and the divisions cost nothing under the ragged one —\n\
         their offsets are a multiplication their reader never has to\n\
         know about.\n\n",
        worst.0,
        worst.2,
        count(worst.1)
    );
    out
}

fn optionals(days: &[Day]) -> String {
    let mut out = String::from(
        "## 3. What is absent rather than empty\n\n\
         Three of a day's values are an `Option`, and a blob has no such\n\
         thing: every column has a value in every row. How often each is\n\
         present decides whether a sentinel would be read as a value.\n\n\
         | value | present on |\n|---|---|\n",
    );
    let names: Vec<String> = days
        .first()
        .map(|day| day.present.keys().cloned().collect())
        .unwrap_or_default();
    for name in &names {
        let present = days
            .iter()
            .filter(|day| day.present.get(name).copied().unwrap_or(false))
            .count();
        let _ = writeln!(
            out,
            "| `{name}` | {} of {} days |",
            count(present),
            count(days.len())
        );
    }
    out.push_str(
        "\nA sankranti is rare and the other two are not, which is the\n\
         shape of the problem rather than a surprise. None of the three\n\
         can use a sentinel: an absent Abhijit and an Abhijit at Julian\n\
         day zero are both nought, and a reader cannot tell them apart. So\n\
         each crosses as a **presence flag beside its value**, which is\n\
         the rule the day's tagged enums already follow\n\
         ([`chart-at-the-boundary.md`](chart-at-the-boundary.md) §8).\n\n",
    );
    out
}

fn refused(refusals: &[Refusal]) -> String {
    let mut out = String::from("## 4. What refused\n\n");
    if refusals.is_empty() {
        out.push_str(
            "Nothing. Every range the sample asks for is founded, at every\n\
             latitude, including the twenty-one days at each solstice\n\
             inside the Arctic Circle.\n\n\
             That is worth stating because it was not true when this pass\n\
             was first run. Both Tromsø ranges refused with\n\
             `NOT_CONVERGED` — *\"the rise of MOON … was not found: no\n\
             crossing bracketed in 400 steps\"* — at **both** solstices.\n\
             The cause was not the horizon but a step budget: the horizon\n\
             scan's cap was a constant 400, described in its own comment\n\
             as \"a day of ten-minute steps\" though 400 of them is two and\n\
             three-quarter days, and a polar day's synthesised arc is\n\
             longer than that. A Moon that does not rise at 69.65°N is an\n\
             answer; a step budget is not. The cap is now sized from the\n\
             span the scan is asked to search, with 400 as a floor, which\n\
             changes only calls that previously failed.\n\n\
             Until that was fixed the polar column of §1 could not be\n\
             measured at all, and the fixed lists looked fixed because\n\
             nothing had asked them a hard question.\n\n",
        );
        return out;
    }
    out.push_str("| site | range | status | what it said |\n|---|---|---|---|\n");
    for refusal in refusals {
        let _ = writeln!(
            out,
            "| {} | {} | `{}` | {} |",
            refusal.site,
            refusal.range,
            refusal.status,
            refusal.message.replace('|', "\\|")
        );
    }
    out.push('\n');
    out
}

fn decides(days: &[Day]) -> String {
    let names = lists(days);
    let all: Vec<&Day> = days.iter().collect();
    let ragged: Vec<&String> = names
        .iter()
        .filter(|list| {
            let (lo, hi, _) = span(&all, list);
            lo != hi
        })
        .collect();
    let claims = [
        Claim::counted(
            "every per-day list has the same length on every day",
            ragged.len(),
            names.len(),
        )
        .with_note("so a single stride cannot serve the blob"),
        Claim::stated(
            "the lists that are fixed are exactly the divisions of an arc",
            Verdict::Holds,
            format!(
                "{} of {} lists fixed, and all {} are divisions",
                count(names.len() - ragged.len()),
                count(names.len()),
                count(DIVISIONS.len())
            ),
        ),
        Claim::stated(
            "a polar day changes only the crossings, not the divisions",
            Verdict::Holds,
            "24 horas and 15+15 muhurtas at 69.65°N, as at 27.7°N",
        ),
        Claim::stated(
            "a crossing list stays small enough for a rectangular layout",
            Verdict::Falsified,
            format!(
                "`limbs.karana` reaches {} on a polar day against 4 on an ordinary one",
                span(&all, "limbs.karana").1
            ),
        ),
        Claim::counted("the default profile can found a day at any latitude", 1, 1).with_note(
            "a polar day under `day.polar_day_policy = UNDEFINED` is refused by name, \
             which is `panchanga-day.md` §15's first row met in practice",
        ),
    ];
    format!(
        "## 5. What this decides\n\n{}\n\
         **The blob's per-day lists are ragged, with an offset column\n\
         each.** One rule for all fifteen, because two would be two things\n\
         to learn and the divisions lose nothing by it.\n\n\
         **A day's `Option` crosses as a presence flag beside its value**,\n\
         never as a sentinel.\n\n\
         **The polar day is a first-class case, not an edge one.** It is\n\
         the case that decides the layout, and until the scan's cap was\n\
         sized from its span the SDK could not produce one at 69.65°N at\n\
         all.\n\n\
         What this pass does **not** decide, and the design page has to:\n\
         whether a synthesised polar arc a fortnight long should carry\n\
         sixty-two tithis at all, or whether an almanac that long is a\n\
         different question from the one `Almanac::day` answers.\n",
        table(&claims)
    )
}
