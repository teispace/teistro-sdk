//! The falsification pass over the eight nakshatra-seeded dasha systems
//! beside Vimshottari: Ashtottari, Yogini, Dwadashottari, Panchottari,
//! Shatabdika, Chaturashiti-sama, Dwisaptati-sama and Tribhagi (Phase 5).
//!
//! The corpus's `baseline/dasha-systems` records each of them for every
//! fixture that records a dasha, computed from that fixture's recorded Moon
//! and nakshatra span, so an answer here is the dasha arithmetic and nothing
//! astronomical. What `dasha-kernels.md` designed is one kernel with the
//! systems as rows; this page asks whether every system really is a row of
//! it, and **derives** each row's seat rather than taking the rule data the
//! corpus states: every reference nakshatra, direction and offset is tried,
//! and the page reports which the recorded first lords leave standing.
//!
//! It also settles what building Vimshottari could not: how a temporal
//! balance reads a window of three nakshatras, and what Tribhagi's scale
//! is, and it measures the readings each rule is chosen over.
//!
//! `cargo xtask dasha-systems` writes the page; `check-dasha-systems`
//! regenerates it in memory and fails on any difference.

use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;

use crate::dashas::{Parts, parts};
use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, worst};

const PAGE: &str = "docs/03-design/dasha-systems-measured.md";
const ROOT: &str = "fixtures/baseline";
const DIRS: [&str; 2] = ["charts", "variants"];

/// A nakshatra's degrees.
const PER_NAKSHATRA: f64 = 360.0 / 27.0;

/// Differences inside this many days are the last bits of a double.
const SAME_DAY: f64 = 1e-8;

/// The nakshatras, for naming the seeds the corpus leaves untested.
const NAKSHATRAS: [&str; 27] = [
    "Ashwini",
    "Bharani",
    "Krittika",
    "Rohini",
    "Mrigashira",
    "Ardra",
    "Punarvasu",
    "Pushya",
    "Ashlesha",
    "Magha",
    "Purva Phalguni",
    "Uttara Phalguni",
    "Hasta",
    "Chitra",
    "Swati",
    "Vishakha",
    "Anuradha",
    "Jyeshtha",
    "Mula",
    "Purva Ashadha",
    "Uttara Ashadha",
    "Shravana",
    "Dhanishta",
    "Shatabhisha",
    "Purva Bhadrapada",
    "Uttara Bhadrapada",
    "Revati",
];

// ── Reading the corpus ─────────────────────────────────────────────────────

/// Which way a seed is counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Count {
    From,
    To,
}

/// A system's seat rule: where its count starts, which way, how many
/// nakshatras a lord covers, and what is added before going round.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Seat {
    reference: usize,
    count: Count,
    span: usize,
    offset: usize,
}

/// A system's rule data as the corpus states it.
#[derive(Clone, Debug, PartialEq)]
struct Rules {
    lords: Vec<(String, f64)>,
    total_years: f64,
    seat: Seat,
    /// A mahadasha's length factor, its numerator and denominator, and how
    /// many times the sequence runs.
    scale: f64,
    fraction: (usize, usize),
    rounds: usize,
}

/// One period, or one link of a chain (its path then its index).
#[derive(Clone, Debug, PartialEq)]
struct Row {
    path: String,
    lord: String,
    start: f64,
    end: f64,
}

/// One system's answer for one fixture under one balance method.
struct Answer {
    birth: f64,
    longitude: f64,
    method: String,
    span: Option<(f64, f64)>,
    first_lord: String,
    parts: [u64; 5],
    total_days: f64,
    periods: Vec<Row>,
    active: Vec<(f64, Vec<Row>)>,
}

/// One system over the whole corpus.
struct System {
    key: String,
    rules: Rules,
    answers: Vec<Answer>,
}

fn number(value: &Value, what: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("{what} is not a number"))
}

fn text(value: &Value, what: &str) -> Result<String, String> {
    value
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("{what} is not a string"))
}

fn whole(value: &Value, what: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what} is not a whole number"))
}

fn read(path: &Path) -> Result<Value, String> {
    let body = std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
    serde_json::from_str(&body).map_err(|err| format!("{}: {err}", path.display()))
}

fn rules(value: &Value) -> Result<Rules, String> {
    let lords = value["sequence"]
        .as_array()
        .ok_or("sequence is not a list")?
        .iter()
        .map(|lord| {
            Ok((
                text(&lord["lord"], "lord")?,
                number(&lord["years"], "years")?,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    let count = match value["count"].as_str() {
        Some("from_reference") => Count::From,
        Some("to_reference") => Count::To,
        other => return Err(format!("count {other:?} is neither direction")),
    };
    let (fraction, rounds) = match &value["scale"] {
        Value::Null => ((1, 1), 1),
        scale => (
            (
                whole(&scale["numerator"], "numerator")?,
                whole(&scale["denominator"], "denominator")?,
            ),
            whole(&scale["rounds"], "rounds")?,
        ),
    };
    #[expect(
        clippy::cast_precision_loss,
        reason = "a scale's terms are single digits"
    )]
    let scale = fraction.0 as f64 / fraction.1 as f64;
    Ok(Rules {
        lords,
        total_years: number(&value["total_years"], "total_years")?,
        seat: Seat {
            reference: whole(&value["reference_nakshatra"], "reference_nakshatra")?,
            count,
            span: whole(&value["span"], "span")?,
            offset: whole(&value["offset"], "offset")?,
        },
        scale,
        fraction,
        rounds,
    })
}

fn rows(value: &Value, what: &str) -> Result<Vec<Row>, String> {
    value
        .as_array()
        .ok_or_else(|| format!("{what} is not a list"))?
        .iter()
        .map(|cells| {
            Ok(Row {
                path: text(&cells[0], what)?,
                lord: text(&cells[1], what)?,
                start: number(&cells[2], what)?,
                end: number(&cells[3], what)?,
            })
        })
        .collect()
}

fn answer(fixture: &Value, method: &str, value: &Value) -> Result<Answer, String> {
    let dasha = &fixture["dashas"];
    let span = match &dasha["methods"][method]["nakshatra_span"] {
        Value::Null => None,
        span => Some((
            number(&span["entry_jd"], "entry_jd")?,
            number(&span["exit_jd"], "exit_jd")?,
        )),
    };
    let balance = &value["balance"];
    let active = value["active_at"]
        .as_array()
        .ok_or("active_at is not a list")?
        .iter()
        .map(|at| {
            let chain = at["chain"]
                .as_array()
                .ok_or("chain is not a list")?
                .iter()
                .map(|link| {
                    Ok(Row {
                        path: whole(&link[1], "index")?.to_string(),
                        lord: text(&link[2], "lord")?,
                        start: number(&link[3], "start_jd")?,
                        end: number(&link[4], "end_jd")?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok((number(&at["jd"], "jd")?, chain))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Answer {
        birth: number(&fixture["input"]["resolved"]["jd_ut"], "jd_ut")?,
        longitude: number(&dasha["moon_sidereal_longitude_deg"], "longitude")?,
        method: method.to_owned(),
        span,
        first_lord: text(&value["first_lord"], "first_lord")?,
        parts: ["years", "months", "days", "hours", "minutes"]
            .map(|part| balance[part].as_u64().unwrap_or(u64::MAX)),
        total_days: number(&balance["total_days"], "total_days")?,
        periods: rows(&value["periods"], "periods")?,
        active,
    })
}

fn systems(root: &Path) -> Result<Vec<System>, String> {
    let base = root.join(ROOT);
    let mut found: Vec<System> = Vec::new();
    for dir in DIRS {
        let directory = base.join("dasha-systems").join(dir);
        let mut paths: Vec<_> = std::fs::read_dir(&directory)
            .map_err(|err| format!("{}: {err}", directory.display()))?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let file = read(&path)?;
            let name = text(&file["fixture"], "fixture")?;
            let fixture = read(&base.join(&name))?;
            let at = |err: String| format!("{name}: {err}");
            for (key, value) in file["systems"]
                .as_object()
                .ok_or_else(|| at("no systems".into()))?
            {
                let stated = rules(value).map_err(&at)?;
                let index = if let Some(index) = found.iter().position(|system| &system.key == key)
                {
                    index
                } else {
                    found.push(System {
                        key: key.clone(),
                        rules: stated.clone(),
                        answers: Vec::new(),
                    });
                    found.len() - 1
                };
                let system = &mut found[index];
                if system.rules != stated {
                    return Err(at(format!("{key} states different rule data")));
                }
                for (method, recorded) in value["methods"]
                    .as_object()
                    .ok_or_else(|| at("no methods".into()))?
                {
                    system
                        .answers
                        .push(answer(&fixture, method, recorded).map_err(&at)?);
                }
            }
        }
    }
    Ok(found)
}

// ── The rules proposed ─────────────────────────────────────────────────────

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude over a nakshatra's width is 0 to 26"
)]
fn nakshatra(longitude: f64) -> usize {
    (longitude / PER_NAKSHATRA).floor() as usize % 27
}

/// The first lord's index, how many whole nakshatras of its window are
/// behind the seed, and whether the seed lay past the windows the lords
/// cover (and so wrapped to the first).
fn seat(seat: Seat, lords: usize, seed: usize) -> (usize, usize) {
    let counted = match seat.count {
        Count::From => (seed + 27 - seat.reference) % 27,
        Count::To => (seat.reference + 27 - seed) % 27,
    };
    (
        (counted / seat.span + seat.offset) % lords,
        counted % seat.span,
    )
}

/// How a temporal balance reads a window of several nakshatras.
#[derive(Clone, Copy)]
enum Window {
    /// The time elapsed in the Moon's own nakshatra, counted as that
    /// nakshatra's part of the window.
    OwnNakshatra,
    /// The whole window's remainder taken from the nakshatra's time fraction
    /// alone, as though the window were one nakshatra.
    AsOne,
}

/// What remains of the first period at birth.
fn remaining(rules: &Rules, answer: &Answer, window: Window) -> f64 {
    let into = answer.longitude / PER_NAKSHATRA - (answer.longitude / PER_NAKSHATRA).floor();
    let elapsed = match (answer.method.as_str(), answer.span) {
        ("temporal", Some((entry, exit))) => (answer.birth - entry) / (exit - entry),
        _ => into,
    };
    let (_, within) = seat(rules.seat, rules.lords.len(), nakshatra(answer.longitude));
    #[expect(
        clippy::cast_precision_loss,
        reason = "a window is one to three nakshatras"
    )]
    let (span, within) = (rules.seat.span as f64, within as f64);
    match window {
        Window::OwnNakshatra => (span - within - elapsed) / span,
        Window::AsOne => 1.0 - elapsed,
    }
}

/// Which way a period's children start.
#[derive(Clone, Copy)]
enum Children {
    /// From the period's own lord round.
    FromOwnLord,
    /// From the sequence's first lord every time.
    FromFirst,
}

/// The tree to depth two, under the given scale and child order.
fn tree(rules: &Rules, answer: &Answer, scale: f64, rounds: usize, children: Children) -> Vec<Row> {
    let n = rules.lords.len();
    let (first, _) = seat(rules.seat, n, nakshatra(answer.longitude));
    let rest = remaining(rules, answer, Window::OwnNakshatra);
    let mut rows = Vec::new();
    let mut at = answer.birth;
    for round in 0..rounds {
        for step in 0..n {
            let lord = (first + step) % n;
            let (name, years) = &rules.lords[lord];
            let whole = years * scale * JULIAN_YEAR;
            let length = if round == 0 && step == 0 {
                whole * rest
            } else {
                whole
            };
            let path = (round * n + step).to_string();
            rows.push(Row {
                path: path.clone(),
                lord: name.clone(),
                start: at,
                end: at + length,
            });
            let mut child_at = at;
            for k in 0..n {
                let child = match children {
                    Children::FromOwnLord => (lord + k) % n,
                    Children::FromFirst => k,
                };
                let (child_name, child_years) = &rules.lords[child];
                let child_length = length * child_years / rules.total_years;
                rows.push(Row {
                    path: format!("{path}/{k}"),
                    lord: child_name.clone(),
                    start: child_at,
                    end: child_at + child_length,
                });
                child_at += child_length;
            }
            at += length;
        }
    }
    rows
}

/// The only year length these records use.
const JULIAN_YEAR: f64 = 365.25;

/// How many rows disagree in path or lord, of how many, and the worst bound.
fn rows_agree(got: &[Row], want: &[Row]) -> (usize, usize, f64) {
    let mut wrong = got.len().abs_diff(want.len());
    let mut far = 0.0_f64;
    for (a, b) in got.iter().zip(want) {
        if a.path != b.path || a.lord != b.lord {
            wrong += 1;
        }
        far = far
            .max((a.start - b.start).abs())
            .max((a.end - b.end).abs());
    }
    (wrong, got.len().max(want.len()), far)
}

/// The chain holding `jd`, walked down the children of each level to
/// `depth`; empty before birth and past the end of the recorded cycle.
fn chain(rules: &Rules, answer: &Answer, jd: f64, depth: usize) -> Vec<Row> {
    let n = rules.lords.len();
    let (first, _) = seat(rules.seat, n, nakshatra(answer.longitude));
    let mds = tree(
        rules,
        answer,
        rules.scale,
        rules.rounds,
        Children::FromOwnLord,
    );
    let mut out = Vec::new();
    let Some((index, maha)) = mds
        .iter()
        .filter(|row| !row.path.contains('/'))
        .enumerate()
        .find(|(_, row)| row.start <= jd && jd < row.end)
    else {
        return out;
    };
    let mut lord = (first + index) % n;
    let mut row = maha.clone();
    out.push(row.clone());
    while out.len() < depth {
        let length = row.end - row.start;
        let mut at = row.start;
        let mut next = None;
        for k in 0..n {
            let child = (lord + k) % n;
            let child_length = length * rules.lords[child].1 / rules.total_years;
            if at <= jd && jd < at + child_length {
                next = Some((
                    child,
                    Row {
                        path: k.to_string(),
                        lord: rules.lords[child].0.clone(),
                        start: at,
                        end: at + child_length,
                    },
                ));
                break;
            }
            at += child_length;
        }
        let Some((child, found)) = next else {
            break;
        };
        lord = child;
        row = found;
        out.push(row.clone());
    }
    out
}

// ── The page ───────────────────────────────────────────────────────────────

fn name(key: &str) -> String {
    key.split('_')
        .map(|word| {
            let mut letters = word.chars();
            letters
                .next()
                .map(|first| first.to_uppercase().chain(letters).collect::<String>())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("-")
}

/// Every seat rule the recorded first lords leave standing, at the stated
/// span, over every reference nakshatra, direction and offset.
fn standing(system: &System) -> Vec<Seat> {
    let n = system.rules.lords.len();
    let mut out = Vec::new();
    for reference in 0..27 {
        for count in [Count::From, Count::To] {
            for offset in 0..n {
                let candidate = Seat {
                    reference,
                    count,
                    span: system.rules.seat.span,
                    offset,
                };
                let holds = system.answers.iter().all(|answer| {
                    let (lord, _) = seat(candidate, n, nakshatra(answer.longitude));
                    system.rules.lords[lord].0 == answer.first_lord
                });
                if holds {
                    out.push(candidate);
                }
            }
        }
    }
    out
}

/// The row of the summary table a system gets.
fn summary(system: &System) -> String {
    let rules = &system.rules;
    let n = rules.lords.len();
    let seats_wrong = system
        .answers
        .iter()
        .filter(|answer| {
            let (lord, _) = seat(rules.seat, n, nakshatra(answer.longitude));
            rules.lords[lord].0 != answer.first_lord
        })
        .count();
    let balance_far = worst(system.answers.iter().map(|answer| {
        let (lord, _) = seat(rules.seat, n, nakshatra(answer.longitude));
        (remaining(rules, answer, Window::OwnNakshatra)
            * rules.lords[lord].1
            * rules.scale
            * JULIAN_YEAR
            - answer.total_days)
            .abs()
    }));
    let (mut wrong, mut of, mut far) = (0, 0, 0.0_f64);
    for answer in &system.answers {
        let (w, o, f) = rows_agree(
            &tree(
                rules,
                answer,
                rules.scale,
                rules.rounds,
                Children::FromOwnLord,
            ),
            &answer.periods,
        );
        wrong += w;
        of += o;
        far = far.max(f);
    }
    let (mut chains, mut chains_wrong, mut past) = (0, 0, 0);
    for answer in &system.answers {
        for (jd, recorded) in &answer.active {
            chains += 1;
            past += usize::from(recorded.is_empty());
            let proposed = chain(rules, answer, *jd, 4);
            let agrees = proposed.len() == recorded.len()
                && proposed.iter().zip(recorded).all(|(a, b)| {
                    a.path == b.path
                        && a.lord == b.lord
                        && (a.start - b.start).abs() < SAME_DAY
                        && (a.end - b.end).abs() < SAME_DAY
                });
            chains_wrong += usize::from(!agrees);
        }
    }
    let standing = standing(system).len();
    let scale = if rules.rounds > 1 {
        format!(
            ", × {}/{} {} round",
            rules.fraction.0,
            rules.fraction.1,
            crate::measure::times(rules.rounds)
        )
    } else {
        String::new()
    };
    let seat = format!(
        "{} {}, window {}, offset {}",
        if rules.seat.count == Count::From {
            "from"
        } else {
            "to"
        },
        NAKSHATRAS[rules.seat.reference],
        rules.seat.span,
        rules.seat.offset
    );
    format!(
        "| {name} | {n} × {years} years{scale} | {seat} | {standing} | {seats} | {balance:.1e} | {rows} | {far:.1e} | {chains} ({past} past the end) |\n",
        name = name(&system.key),
        years = rules.total_years,
        seats = right(seats_wrong, system.answers.len()),
        balance = balance_far,
        rows = right(wrong, of),
        chains = right(chains_wrong, chains),
    )
}

/// `of` right, or how many were wrong.
fn right(wrong: usize, of: usize) -> String {
    if wrong == 0 {
        format!("all {}", count(of))
    } else {
        format!("**{} wrong** of {}", count(wrong), count(of))
    }
}

/// Every system's answers, with the system.
fn each(systems: &[System]) -> impl Iterator<Item = (&System, &Answer)> {
    systems
        .iter()
        .flat_map(|s| s.answers.iter().map(move |a| (s, a)))
}

/// The rules every system is held to, and the readings they are chosen over.
fn claims(systems: &[System]) -> Vec<Claim> {
    let mut claims = seat_claims(systems);
    claims.extend(balance_claims(systems));
    claims.extend(tree_claims(systems));
    claims
}

/// The first lord, and a seed past a windowed row's windows.
fn seat_claims(systems: &[System]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let answers: usize = systems.iter().map(|s| s.answers.len()).sum();
    let each = || each(systems);

    let wrong = each()
        .filter(|(s, a)| {
            let (lord, _) = seat(s.rules.seat, s.rules.lords.len(), nakshatra(a.longitude));
            s.rules.lords[lord].0 != a.first_lord
        })
        .count();
    claims.push(Claim::counted(
        "the first lord is the count from the reference nakshatra to the seed (or from the seed to it), in windows of the row's span, plus its offset, taken round the lords",
        wrong,
        answers,
    ));
    // A seed past the windows a row's lords cover once: Ashtottari's eight
    // windows of three leave the three nakshatras before Ardra.
    let past: Vec<_> = each()
        .filter(|(s, a)| {
            let seed = nakshatra(a.longitude);
            let counted = match s.rules.seat.count {
                Count::From => (seed + 27 - s.rules.seat.reference) % 27,
                Count::To => (s.rules.seat.reference + 27 - seed) % 27,
            };
            s.rules.seat.span > 1 && counted / s.rules.seat.span >= s.rules.lords.len()
        })
        .collect();
    let past_wrong = past
        .iter()
        .filter(|(s, a)| {
            let (lord, within) = seat(s.rules.seat, s.rules.lords.len(), nakshatra(a.longitude));
            #[expect(
                clippy::cast_precision_loss,
                reason = "a window is one to three nakshatras"
            )]
            let (span, within) = (s.rules.seat.span as f64, within as f64);
            let rest = (span - within - (a.longitude / PER_NAKSHATRA).fract()) / span;
            lord != 0
                || s.rules.lords[0].0 != a.first_lord
                || (a.method == "spatial"
                    && (rest * s.rules.lords[0].1 * JULIAN_YEAR - a.total_days).abs() > SAME_DAY)
        })
        .count();
    let past_seeds: BTreeSet<&str> = past
        .iter()
        .map(|(_, a)| NAKSHATRAS[nakshatra(a.longitude)])
        .collect();
    claims.push(
        Claim::counted(
            "a seed past the windows of a row whose lords cover several nakshatras each goes round to the first lord's window, keeping its place in it",
            past_wrong,
            past.len(),
        )
        .with_note(format!(
            "at {}",
            past_seeds.into_iter().collect::<Vec<_>>().join(" and ")
        )),
    );

    claims
}

/// A temporal balance over a window, and how a balance is written.
fn balance_claims(systems: &[System]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let answers: usize = systems.iter().map(|s| s.answers.len()).sum();
    let each = || each(systems);

    let temporal_windows: Vec<_> = each()
        .filter(|(s, a)| a.method == "temporal" && s.rules.seat.span > 1)
        .collect();
    let far = |window: Window| {
        worst(temporal_windows.iter().map(|(s, a)| {
            let (lord, _) = seat(s.rules.seat, s.rules.lords.len(), nakshatra(a.longitude));
            (remaining(&s.rules, a, window) * s.rules.lords[lord].1 * JULIAN_YEAR - a.total_days)
                .abs()
        }))
    };
    let own = far(Window::OwnNakshatra);
    claims.push(Claim::stated(
        "a temporal balance over a window of nakshatras counts the time elapsed in the Moon's own nakshatra as that nakshatra's part of the window, the whole nakshatras before it in the window already gone",
        if temporal_windows.is_empty() {
            Verdict::Untested
        } else if own < SAME_DAY {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!("worst {own:.1e} days over {} Ashtottari answers", count(temporal_windows.len())),
    ));
    let as_one = temporal_windows
        .iter()
        .filter(|(s, a)| {
            let (lord, _) = seat(s.rules.seat, s.rules.lords.len(), nakshatra(a.longitude));
            (remaining(&s.rules, a, Window::AsOne) * s.rules.lords[lord].1 * JULIAN_YEAR
                - a.total_days)
                .abs()
                > SAME_DAY
        })
        .count();
    claims.push(Claim::counted(
        "the same with the window's remainder taken from the nakshatra's time fraction alone",
        as_one,
        temporal_windows.len(),
    ));
    claims.push(Claim::stated(
        "the time fraction of the Moon's stay across the whole window",
        Verdict::Untested,
        String::from("the corpus records the span of one nakshatra, never of a window"),
    ));

    let written_wrong = each()
        .filter(|(_, a)| parts(a.total_days, JULIAN_YEAR, Parts::RoundedMinutes) != a.parts)
        .count();
    claims.push(Claim::counted(
        "every system writes its balance as Vimshottari does, the minutes rounded",
        written_wrong,
        answers,
    ));

    claims
}

/// The tree, Tribhagi's scale, and the end of the cycle.
fn tree_claims(systems: &[System]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let answers: usize = systems.iter().map(|s| s.answers.len()).sum();
    let each = || each(systems);

    let (mut own_wrong, mut rows) = (0, 0);
    let mut first_wrong = 0;
    for (s, a) in each() {
        let (w, o, _) = rows_agree(
            &tree(
                &s.rules,
                a,
                s.rules.scale,
                s.rules.rounds,
                Children::FromOwnLord,
            ),
            &a.periods,
        );
        own_wrong += w;
        rows += o;
        first_wrong += usize::from(
            rows_agree(
                &tree(
                    &s.rules,
                    a,
                    s.rules.scale,
                    s.rules.rounds,
                    Children::FromFirst,
                ),
                &a.periods,
            )
            .0 > 0,
        );
    }
    claims.push(Claim::counted(
        "every system's tree is Vimshottari's: the birth period compressed into the balance, each period's children from its own lord round, each its lord's share of the row's years",
        own_wrong,
        rows,
    ));
    claims.push(Claim::counted(
        "children start from the sequence's first lord instead",
        first_wrong,
        answers,
    ));

    if let Some(tribhagi) = systems.iter().find(|s| s.rules.rounds > 1) {
        let thirds = tribhagi
            .answers
            .iter()
            .filter(|a| {
                rows_agree(
                    &tree(&tribhagi.rules, a, 1.0 / 3.0, 3, Children::FromOwnLord),
                    &a.periods,
                )
                .0 > 0
            })
            .count();
        claims.push(Claim::counted(
            "Tribhagi runs Vimshottari's nine at a **third** of their years, three times round (forty years a round)",
            thirds,
            tribhagi.answers.len(),
        ));
    }

    // Under a sequence that begins again a period runs at every instant
    // after birth, so every instant the corpus records none refutes it.
    let empty = each()
        .flat_map(|(_, a)| a.active.iter())
        .filter(|(_, recorded)| recorded.is_empty())
        .count();
    claims.push(Claim::counted(
        "past the end of its cycle a system begins its sequence again",
        empty,
        empty,
    ));

    claims
}

fn page(root: &Path) -> Result<String, String> {
    let systems = systems(root)?;
    if systems.is_empty() {
        return Err(String::from("the corpus records no dasha systems"));
    }
    let answers: usize = systems.iter().map(|s| s.answers.len()).sum();
    let seeds: BTreeSet<usize> = systems
        .iter()
        .flat_map(|s| s.answers.iter().map(|a| nakshatra(a.longitude)))
        .collect();
    let untested: Vec<&str> = (0..27)
        .filter(|seed| !seeds.contains(seed))
        .map(|seed| NAKSHATRAS[seed])
        .collect();

    let mut out = String::new();
    let _ = write!(
        out,
        "# The other nakshatra-seeded dashas, measured\n\n\
         Status: `generated` by `cargo xtask dasha-systems` over the conformance corpus's \
         `baseline/dasha-systems`, 2026-09-15. Do not edit: `check-dasha-systems` \
         regenerates this page and fails on any difference. The design it measures is \
         [`dasha-kernels.md`](dasha-kernels.md); Vimshottari's own page is \
         [`dasha-measured.md`](dasha-measured.md).\n\n\
         The corpus records {systems} systems beside Vimshottari, each for the same \
         fixtures and balance methods Vimshottari is recorded under: {answers} answers, \
         each with its first lord, its balance, its tree to antardashas and the chain of \
         four levels at two instants. They were computed from each fixture's recorded Moon \
         and nakshatra span, so what they test is the dasha and not the astronomy. The \
         Moon stands in {seeds} of the 27 nakshatras across them, so the seat rules are \
         untested at {untested}.\n\n\
         ## Each system\n\n\
         The seat is the rule data the corpus states. **Standing** is how many seat rules \
         at the same window survive every recorded first lord when every reference \
         nakshatra, both directions and every offset are tried: more than one means the \
         corpus cannot tell them apart, and all of them are the same map on the seeds it \
         records.\n\n\
         | system | lords | seat | standing | first lords | balance, worst days | tree rows | worst boundary, days | chains |\n\
         |---|---|---|---|---|---|---|---|---|\n",
        systems = count(systems.len()),
        answers = count(answers),
        seeds = seeds.len(),
        untested = untested.join(", "),
    );
    for system in &systems {
        out.push_str(&summary(system));
    }
    let _ = write!(
        out,
        "\n## What the corpus decides\n\n{}\n",
        table(&claims(&systems))
    );
    out.push_str(
        "## What it means for the module\n\n\
         **Every one is a row of the K-udu kernel.** Eight systems, one tree: the seat, \
         the balance, the compressed birth period, the children and the end of the cycle \
         are Vimshottari's in every one, so a system is its lords, its reference, its \
         direction, its window and its offset, and nothing is a module of its own. The \
         corpus decides five of the seats outright: one reference, direction and offset \
         survives. Yogini's and Dwadashottari's leave a second, which differs only at a \
         seed the corpus does not record, and Tribhagi's nine lords divide the 27 exactly, \
         so any reference with the matching offset is the same map. The rule data comes \
         from the recording engine's cited rows; where the corpus cannot choose, it \
confirms it without choosing it.\n\n\
         **A temporal balance reads a window through the Moon's own nakshatra.** Building \
         Vimshottari refused a temporal balance over a window because no source defined \
         one. The recording engine defines it: the whole nakshatras of the window behind the \
         seed are gone, and the time elapsed in the Moon's own nakshatra is that \
         nakshatra's share of the window. Every Ashtottari answer agrees, so the kernel \
         takes that reading. The other reading, the Moon's time across the whole window, \
         is not recorded and stays unbuilt.\n\n\
         **Tribhagi is Vimshottari scaled, not a sequence of its own.** Each mahadasha is \
         two thirds of its Vimshottari years and the sequence runs twice, eighty years a \
         round, with the sub-period shares still over a hundred and twenty. The reading \
         that runs a third of the years three times round is refused by every answer. So \
         a row carries a scale and a number of rounds, which is the design page's scale \
         decorator as two fields.\n\n\
         **The cycle ends in every system**, Yogini's thirty-six years included, which is \
         the corpus's reading and the `dasha.after_cycle` default.\n",
    );
    Ok(fill(&out))
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
        Ok(text) => i32::from(
            check(
                root,
                &[Output::new(PAGE, text)],
                "cargo xtask dasha-systems",
            ) != 0,
        ),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_window_and_a_backward_count_each_move_the_seat() {
        let ardra = Seat {
            reference: 5,
            count: Count::From,
            span: 3,
            offset: 0,
        };
        assert_eq!(seat(ardra, 8, 5), (0, 0));
        assert_eq!(
            seat(ardra, 8, 16),
            (3, 2),
            "Anuradha is Mercury's, two into it"
        );
        assert_eq!(
            seat(ardra, 8, 2),
            (0, 0),
            "Krittika goes round to the first"
        );
        let revati = Seat {
            reference: 26,
            count: Count::To,
            span: 1,
            offset: 0,
        };
        assert_eq!(seat(revati, 8, 25), (1, 0));
        let yogini = Seat {
            reference: 0,
            count: Count::From,
            span: 1,
            offset: 3,
        };
        assert_eq!(seat(yogini, 8, 0), (3, 0));
    }

    #[test]
    fn a_system_name_reads_as_written() {
        assert_eq!(name("chaturashiti_sama"), "Chaturashiti-Sama");
        assert_eq!(name("yogini"), "Yogini");
    }
}
