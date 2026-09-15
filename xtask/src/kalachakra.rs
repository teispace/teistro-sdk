//! The falsification pass over the Kalachakra dasha (Phase 5).
//!
//! The corpus's `baseline/kalachakra` records the recording engine's
//! Kalachakra for every dasha fixture, computed from the fixture's recorded
//! Moon and nakshatra span, and its manifest carries the rule data the
//! engine ran: each sign's years and the four pada tables with the
//! nakshatras each serves.
//!
//! Kalachakra is where the sources read disagree most, and they disagree on
//! structure rather than arithmetic, so every rule the engine takes is
//! measured beside the reading the sources give instead:
//!
//! - which pada table a nakshatra takes — the engine's membership, or the
//!   one the texts' lists follow, by the nakshatra's place in its triad;
//! - the balance — the unelapsed part of the pada taken of the first sign's
//!   years, or the elapsed part taken of the pada's whole span and the
//!   signs it covers skipped;
//! - what follows the ninth mahadasha — the same nine signs reversed, the
//!   same nine again, or the next pada's nine;
//! - where each mahadasha's antardashas begin.
//!
//! `cargo xtask kalachakra` writes the page; `check-kalachakra` regenerates
//! it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, worst};

const PAGE: &str = "docs/03-design/kalachakra-measured.md";
const ROOT: &str = "fixtures/baseline/kalachakra";

const YEAR: f64 = 365.25;
const SAME_DAY: f64 = 1e-8;
const PER_NAKSHATRA: f64 = 360.0 / 27.0;

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

/// The rule data the manifest carries.
struct Rules {
    years: [f64; 12],
    /// Each table's key, the nakshatras it serves, and its four padas.
    tables: Vec<(String, Vec<usize>, [[usize; 9]; 4])>,
}

/// One period.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    path: String,
    sign: usize,
    start: f64,
    end: f64,
}

/// One recorded answer.
struct Answer {
    birth: f64,
    longitude: f64,
    nakshatra: usize,
    pada: usize,
    span: Option<(f64, f64)>,
    temporal: bool,
    total_days: f64,
    periods: Vec<Row>,
    active: Vec<(f64, Vec<Row>)>,
}

fn number(value: &Value, what: &str) -> Result<f64, String> {
    value
        .as_f64()
        .ok_or_else(|| format!("{what} is not a number"))
}

fn index(value: &Value, what: &str) -> Result<usize, String> {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| format!("{what} is not a whole number"))
}

fn read(path: &Path) -> Result<Value, String> {
    serde_json::from_str(
        &std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?,
    )
    .map_err(|err| format!("{}: {err}", path.display()))
}

fn rules(manifest: &Value) -> Result<Rules, String> {
    let mut years = [0.0; 12];
    for (slot, value) in years.iter_mut().zip(
        manifest["rules"]["sign_years"]
            .as_array()
            .ok_or("sign_years")?,
    ) {
        *slot = number(value, "sign_years")?;
    }
    let tables = manifest["rules"]["tables"]
        .as_array()
        .ok_or("tables")?
        .iter()
        .map(|table| {
            let nakshatras = table["nakshatras"]
                .as_array()
                .ok_or("nakshatras")?
                .iter()
                .map(|n| index(n, "nakshatra"))
                .collect::<Result<Vec<_>, String>>()?;
            let mut padas = [[0; 9]; 4];
            for (pada, signs) in padas
                .iter_mut()
                .zip(table["padas"].as_array().ok_or("padas")?)
            {
                for (slot, sign) in pada.iter_mut().zip(signs.as_array().ok_or("pada")?) {
                    *slot = index(sign, "sign")?;
                }
            }
            Ok((
                table["key"].as_str().unwrap_or_default().to_owned(),
                nakshatras,
                padas,
            ))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Rules { years, tables })
}

fn rows(value: &Value, path_at: Option<usize>) -> Result<Vec<Row>, String> {
    value
        .as_array()
        .ok_or("rows")?
        .iter()
        .map(|cells| {
            let (path, at) = match path_at {
                None => (cells[0].as_str().ok_or("path")?.to_owned(), 1),
                Some(at) => (index(&cells[at], "index")?.to_string(), at + 1),
            };
            Ok(Row {
                path,
                sign: index(&cells[at], "sign")?,
                start: number(&cells[at + 2], "start")?,
                end: number(&cells[at + 3], "end")?,
            })
        })
        .collect()
}

fn answers(root: &Path) -> Result<(Rules, Vec<Answer>), String> {
    let base = root.join(ROOT);
    let rules = rules(&read(&base.join("manifest.json"))?)?;
    let mut out = Vec::new();
    for dir in ["charts", "variants"] {
        let mut paths: Vec<_> = std::fs::read_dir(base.join(dir))
            .map_err(|err| format!("{dir}: {err}"))?
            .flatten()
            .map(|entry| entry.path())
            .collect();
        paths.sort();
        for path in paths {
            let file = read(&path)?;
            let fixture = read(
                &root
                    .join("fixtures/baseline")
                    .join(file["fixture"].as_str().unwrap_or_default()),
            )?;
            for (method, recorded) in file["methods"].as_object().ok_or("methods")? {
                let span = &fixture["dashas"]["methods"][method]["nakshatra_span"];
                out.push(Answer {
                    birth: number(&fixture["input"]["resolved"]["jd_ut"], "jd_ut")?,
                    longitude: number(&file["moon_sidereal_longitude_deg"], "longitude")?,
                    nakshatra: index(&file["nakshatra_index"], "nakshatra_index")?,
                    pada: index(&file["pada_index"], "pada_index")?,
                    span: match span {
                        Value::Null => None,
                        span => Some((
                            number(&span["entry_jd"], "entry")?,
                            number(&span["exit_jd"], "exit")?,
                        )),
                    },
                    temporal: method == "temporal",
                    total_days: number(&recorded["balance"]["total_days"], "total_days")?,
                    periods: rows(&recorded["periods"], None)?,
                    active: recorded["active_at"]
                        .as_array()
                        .ok_or("active_at")?
                        .iter()
                        .map(|at| Ok((number(&at["jd"], "jd")?, rows(&at["chain"], Some(1))?)))
                        .collect::<Result<Vec<_>, String>>()?,
                });
            }
        }
    }
    Ok((rules, out))
}

// ── The rules proposed ─────────────────────────────────────────────────────

/// Which table a nakshatra takes.
#[derive(Clone, Copy)]
enum Membership {
    /// The manifest's lists.
    Stated,
    /// By the nakshatra's place in its triad: the savya triads (0, 2, 4, 6,
    /// 8) and apasavya (1, 3, 5, 7) each have two tables, the middle
    /// nakshatra taking the second and the outer two the first.
    ByTriad,
}

/// How the balance at birth is taken.
#[derive(Clone, Copy)]
enum BalanceRule {
    /// The unelapsed part of the pada, of the first sign's years.
    FirstSign,
    /// The elapsed part of the pada, of the pada's whole span, the signs it
    /// covers skipped and the rest of the one it ends in remaining.
    WholePada,
}

/// What follows the ninth mahadasha.
#[derive(Clone, Copy)]
enum AfterNinth {
    /// The same nine signs reversed, and back again.
    Reversed,
    /// The same nine in the same order.
    Repeated,
    /// The next pada's nine.
    NextPada,
}

#[derive(Clone, Copy)]
struct Reading {
    membership: Membership,
    balance: BalanceRule,
    after: AfterNinth,
    /// Whether antardashas begin from the mahadasha's own place in its nine
    /// (else from the first of the nine).
    from_own: bool,
}

const RECORDED: Reading = Reading {
    membership: Membership::Stated,
    balance: BalanceRule::FirstSign,
    after: AfterNinth::Reversed,
    from_own: true,
};

fn table_of(rules: &Rules, nakshatra: usize, membership: Membership) -> Option<&[[usize; 9]; 4]> {
    let key = match membership {
        Membership::Stated => {
            return rules
                .tables
                .iter()
                .find(|(_, stars, _)| stars.contains(&nakshatra))
                .map(|(_, _, t)| t);
        }
        Membership::ByTriad => {
            let savya = (nakshatra / 3) % 2 == 0;
            let middle = nakshatra % 3 == 1;
            match (savya, middle) {
                (true, false) => "savya_1",
                (true, true) => "savya_2",
                (false, false) => "apasavya_1",
                (false, true) => "apasavya_2",
            }
        }
    };
    rules
        .tables
        .iter()
        .find(|(k, _, _)| k == key)
        .map(|(_, _, t)| t)
}

/// The mahadasha signs of 27, and the nine each one's antardashas run over,
/// with its place in them.
fn mahadashas(
    rules: &Rules,
    answer: &Answer,
    reading: Reading,
) -> Option<Vec<(usize, [usize; 9], usize)>> {
    let tables = table_of(rules, answer.nakshatra, reading.membership)?;
    let mut nine = tables[answer.pada];
    let mut out = Vec::new();
    let mut pada = answer.pada;
    let mut nakshatra = answer.nakshatra;
    let mut nakshatra_tables = *tables;
    for cycle in 0..3 {
        if cycle > 0 {
            nine = match reading.after {
                AfterNinth::Reversed => {
                    let mut reversed = nine;
                    reversed.reverse();
                    reversed
                }
                AfterNinth::Repeated => nine,
                AfterNinth::NextPada => {
                    pada = (pada + 1) % 4;
                    if pada == 0 {
                        nakshatra = (nakshatra + 1) % 27;
                        nakshatra_tables = *table_of(rules, nakshatra, reading.membership)?;
                    }
                    nakshatra_tables[pada]
                }
            };
        }
        for (place, sign) in nine.iter().enumerate() {
            out.push((*sign, nine, place));
        }
    }
    Some(out)
}

/// The elapsed part of the pada the answer's method reads.
fn elapsed(answer: &Answer) -> f64 {
    let spatial = (answer.longitude % PER_NAKSHATRA) / (PER_NAKSHATRA / 4.0);
    let spatial = spatial - spatial.floor();
    match (answer.temporal, answer.span) {
        (true, Some((entry, exit))) => {
            let by_time = (answer.birth - entry) / (exit - entry) * 4.0;
            by_time - by_time.floor()
        }
        _ => spatial,
    }
}

/// The tree to antardashas under a reading, and the balance in days.
fn tree(rules: &Rules, answer: &Answer, reading: Reading) -> Option<(Vec<Row>, f64)> {
    let mds = mahadashas(rules, answer, reading)?;
    let fraction = elapsed(answer);
    let years = |sign: usize| rules.years[sign];
    // The first mahadasha's remaining years, and how many whole ones the
    // balance skips.
    let (skip, first_years) = match reading.balance {
        BalanceRule::FirstSign => (0, (1.0 - fraction) * years(mds[0].0)),
        BalanceRule::WholePada => {
            let total: f64 = mds.iter().take(9).map(|(s, _, _)| years(*s)).sum();
            let mut gone = fraction * total;
            let mut skip = 0;
            while skip < 8 && gone >= years(mds[skip].0) {
                gone -= years(mds[skip].0);
                skip += 1;
            }
            (skip, years(mds[skip].0) - gone)
        }
    };
    let mut rows = Vec::new();
    let mut at = answer.birth;
    for (i, (sign, nine, place)) in mds.iter().skip(skip).enumerate() {
        let length = if i == 0 { first_years } else { years(*sign) } * YEAR;
        rows.push(Row {
            path: i.to_string(),
            sign: *sign,
            start: at,
            end: at + length,
        });
        let total: f64 = nine.iter().map(|s| years(*s)).sum();
        let mut child = at;
        for k in 0..9 {
            let sub = nine[if reading.from_own { (place + k) % 9 } else { k }];
            let share = length * years(sub) / total;
            rows.push(Row {
                path: format!("{i}/{k}"),
                sign: sub,
                start: child,
                end: child + share,
            });
            child += share;
        }
        at += length;
    }
    Some((rows, first_years * YEAR))
}

fn agrees(got: &[Row], want: &[Row]) -> bool {
    got.len() >= want.len()
        && got.iter().zip(want).all(|(a, b)| {
            a.path == b.path
                && a.sign == b.sign
                && (a.start - b.start).abs() < SAME_DAY
                && (a.end - b.end).abs() < SAME_DAY
        })
}

// ── The page ───────────────────────────────────────────────────────────────

fn wrong(rules: &Rules, answers: &[Answer], reading: Reading) -> usize {
    answers
        .iter()
        .filter(|answer| {
            tree(rules, answer, reading).is_none_or(|(rows, days)| {
                !agrees(&rows, &answer.periods) || (days - answer.total_days).abs() > SAME_DAY
            })
        })
        .count()
}

/// The mahadashas running at the recorded instants.
fn chain_claim(rules: &Rules, answers: &[Answer]) -> Claim {
    let (mut chains, mut chains_wrong, mut past) = (0, 0, 0);
    let mut far = 0.0_f64;
    for answer in answers {
        let Some((rows, _)) = tree(rules, answer, RECORDED) else {
            continue;
        };
        for (jd, recorded) in &answer.active {
            chains += 1;
            past += usize::from(recorded.is_empty());
            let maha = rows
                .iter()
                .filter(|r| !r.path.contains('/'))
                .position(|r| r.start <= *jd && *jd < r.end);
            let agrees = match (maha, recorded.first()) {
                (None, None) => true,
                (Some(i), Some(link)) => {
                    let top = rows.iter().find(|r| r.path == i.to_string());
                    far = far.max(top.map_or(0.0, |t| (t.start - link.start).abs()));
                    top.is_some_and(|t| t.sign == link.sign) && link.path == i.to_string()
                }
                _ => false,
            };
            chains_wrong += usize::from(!agrees);
        }
    }
    Claim::counted(
        "the mahadasha running at an instant is the same tree's",
        chains_wrong,
        chains,
    )
    .with_note(format!(
        "{} of the instants fall past the third cycle; worst start {far:.1e} days",
        count(past)
    ))
}

fn claims(rules: &Rules, answers: &[Answer]) -> Vec<Claim> {
    let of = answers.len();
    let mut claims = vec![Claim::counted(
        "the Moon's nakshatra picks the table the manifest lists it under and its pada the row; the first sign runs the unelapsed part of the pada of its years, every other sign its whole years; after nine the same nine run reversed, three cycles in all; each mahadasha's nine antardashas begin from its own place in its nine and share it by their signs' years",
        wrong(rules, answers, RECORDED),
        of,
    )];
    for (rule, reading) in [
        (
            "the table is the nakshatra's by its place in its triad, the middle of each triad taking the chakra's second table",
            Reading {
                membership: Membership::ByTriad,
                ..RECORDED
            },
        ),
        (
            "the balance is the elapsed part of the pada taken of the pada's whole span, the signs it covers skipped",
            Reading {
                balance: BalanceRule::WholePada,
                ..RECORDED
            },
        ),
        (
            "after nine the same nine run again in the same order",
            Reading {
                after: AfterNinth::Repeated,
                ..RECORDED
            },
        ),
        (
            "after nine the next pada's nine run",
            Reading {
                after: AfterNinth::NextPada,
                ..RECORDED
            },
        ),
        (
            "every mahadasha's antardashas begin from the first of its nine",
            Reading {
                from_own: false,
                ..RECORDED
            },
        ),
    ] {
        claims.push(Claim::counted(rule, wrong(rules, answers, reading), of));
    }

    let differ: Vec<&str> = (0..27)
        .filter(|&n| {
            table_of(rules, n, Membership::Stated).map(|t| t[0])
                != table_of(rules, n, Membership::ByTriad).map(|t| t[0])
        })
        .map(|n| NAKSHATRAS[n])
        .collect();
    claims.push(Claim::stated(
        "the manifest's lists are the triad rule",
        if differ.is_empty() {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!("they differ at {}", differ.join(", ")),
    ));

    let temporal: Vec<&Answer> = answers
        .iter()
        .filter(|a| a.temporal && a.span.is_some())
        .collect();
    let crossed = temporal
        .iter()
        .filter(|a| {
            a.span.is_some_and(|(entry, exit)| {
                let by_time = ((a.birth - entry) / (exit - entry) * 4.0).floor();
                (by_time - f64::from(u8::try_from(a.pada).unwrap_or(0))).abs() > 0.5
            })
        })
        .count();
    claims.push(Claim::counted(
        "a temporal balance's pada, the Moon's time in its nakshatra taken in quarters, is the pada the table was chosen by",
        crossed,
        temporal.len(),
    ));

    claims.push(chain_claim(rules, answers));
    claims
}

fn page(root: &Path) -> Result<String, String> {
    let (rules, answers) = answers(root)?;
    if answers.is_empty() {
        return Err(String::from("the corpus records no Kalachakra"));
    }
    let seeds: std::collections::BTreeSet<(usize, usize)> =
        answers.iter().map(|a| (a.nakshatra, a.pada)).collect();
    let worst_balance =
        worst(answers.iter().filter_map(|a| {
            tree(&rules, a, RECORDED).map(|(_, days)| (days - a.total_days).abs())
        }));
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Kalachakra dasha, measured\n\n\
         Status: `generated` by `cargo xtask kalachakra` over the conformance corpus's \
         `baseline/kalachakra`, 2026-09-15. Do not edit: `check-kalachakra` regenerates this page \
         and fails on any difference. The design it measures is \
         [`dasha-kernels.md`](dasha-kernels.md).\n\n\
         The corpus records the recording engine's Kalachakra for {answers} answers, computed from \
         the recorded Moon and nakshatra span of every dasha fixture, each a tree of 27 mahadashas \
         and their antardashas and the periods running at two instants. The Moon stands in {seeds} of \
         the 108 padas, so a table row the corpus never reaches is untested; the worst balance is \
         {worst_balance:.1e} days.\n\n\
         ## What the corpus decides\n\n{table}\n",
        answers = count(answers.len()),
        seeds = seeds.len(),
        table = table(&claims(&rules, &answers)),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **The engine's Kalachakra is one reading at every fork, and the sources read differently \
         at most of them.** The corpus confirms which reading the engine takes and can say nothing \
         about which is right, so the kernel ships the engine's as its defaults and each fork is a \
         crux (C54–C58): the pada tables' membership, the balance, what follows the ninth \
         mahadasha, the temporal balance's pada, and anything below the antardashas.\n\n\
         **The tables are sound and the membership is not settled.** The four pada tables agree sign \
         for sign with the published chakra tables read; the nakshatras each table serves do not at \
         the five named above, where the texts' lists follow the triad rule and the engine's do not.\n\n\
         **The engine builds nothing below the antardashas**, and no source read defines a third \
         level, so the kernel stops there too.\n",
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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask kalachakra") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}
