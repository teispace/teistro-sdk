//! The falsification pass over the Vimshottari dasha, which the dasha
//! module is designed from (Phase 5).
//!
//! The corpus records the answer **and** its inputs here, which makes it
//! the `vargas` shape rather than the `aspect` one: every fixture carries
//! the Moon's sidereal longitude, its nakshatra and the elapsed part of
//! it, the year length, and the whole period tree the recording engine
//! built from them, under both balance methods. So a proposed rule is
//! right or wrong rather than plausible, and the page settles the
//! questions `dasha-kernels.md` left as a design:
//!
//! - which reading of the birth period's sub-periods the engine takes
//!   (compressed into the balance, or sized against the whole period with
//!   the elapsed ones before birth);
//! - how the balance is written as years, months, days, hours and
//!   minutes;
//! - whether the boundaries can be reproduced to the bit, and so what
//!   arithmetic the SDK should choose;
//! - what the engine answers past the end of the cycle.
//!
//! What it cannot settle is the year length (crux C6): every record uses
//! 365.25 days, so the page says so and marks the other lengths untested.
//!
//! `cargo xtask dashas` writes the page; `check-dashas` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, count, fill, table, worst};

const PAGE: &str = "docs/03-design/dasha-measured.md";
const DIRS: [&str; 2] = ["fixtures/baseline/charts", "fixtures/baseline/variants"];

/// The Vimshottari lords in sequence from Ashwini, with their years.
const VIMSHOTTARI: [(&str, u32); 9] = [
    ("KETU", 7),
    ("VENUS", 20),
    ("SUN", 6),
    ("MOON", 10),
    ("MARS", 7),
    ("RAHU", 18),
    ("JUPITER", 16),
    ("SATURN", 19),
    ("MERCURY", 17),
];

/// The only year length the corpus records, in days.
const JULIAN_YEAR: f64 = 365.25;

/// The cycle's years.
const TOTAL_YEARS: u32 = 120;

/// A nakshatra's degrees.
const PER_NAKSHATRA: f64 = 360.0 / 27.0;

/// Differences inside this many days are the last bits of a double: a
/// quarter of a millisecond, where the corpus's own tolerance is a
/// thousandth of a day.
const SAME_DAY: f64 = 1e-8;

/// Two fractions this close are the same fraction.
const SAME_FRACTION: f64 = 1e-12;

// ── Reading the corpus ─────────────────────────────────────────────────────

/// One recorded dasha: the inputs and every method's tree.
struct Record {
    birth: f64,
    longitude: f64,
    index: u64,
    elapsed: f64,
    start: String,
    year_length: f64,
    total_years: u64,
    methods: Vec<Method>,
}

/// One balance method's recorded answer.
struct Method {
    name: String,
    remaining: f64,
    span: Option<(f64, f64)>,
    balance: Balance,
    depth: usize,
    periods: Vec<Row>,
    children: Vec<(String, Vec<Row>)>,
    active: Vec<(f64, Vec<Row>)>,
}

/// A balance as the corpus writes it.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Balance {
    parts: [u64; 5],
    total_days: f64,
}

/// One period: where it sits in the tree, its lord and its bounds.
#[derive(Clone, Debug, PartialEq)]
struct Row {
    path: String,
    lord: String,
    start: f64,
    end: f64,
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

fn whole(value: &Value, what: &str) -> Result<u64, String> {
    value
        .as_u64()
        .ok_or_else(|| format!("{what} is not a whole number"))
}

/// A `[path, lord, start, end]` row, or `[index, lord, start, end]`.
fn row(value: &Value, what: &str) -> Result<Row, String> {
    let cells = value
        .as_array()
        .filter(|cells| cells.len() == 4)
        .ok_or_else(|| format!("{what} is not a row of four"))?;
    let path = match &cells[0] {
        Value::String(path) => path.clone(),
        other => whole(other, what)?.to_string(),
    };
    Ok(Row {
        path,
        lord: text(&cells[1], what)?,
        start: number(&cells[2], what)?,
        end: number(&cells[3], what)?,
    })
}

fn method(name: &str, value: &Value) -> Result<Method, String> {
    let balance = &value["balance"];
    let parts = ["years", "months", "days", "hours", "minutes"]
        .map(|part| whole(&balance[part], part).unwrap_or(u64::MAX));
    let span = match &value["nakshatra_span"] {
        Value::Null => None,
        span => Some((
            number(&span["entry_jd"], "entry_jd")?,
            number(&span["exit_jd"], "exit_jd")?,
        )),
    };
    let rows = |items: &Value, what: &str| -> Result<Vec<Row>, String> {
        items
            .as_array()
            .ok_or_else(|| format!("{what} is not a list"))?
            .iter()
            .map(|item| row(item, what))
            .collect()
    };
    let children = value["children_at_path"]
        .as_object()
        .ok_or("children_at_path is not an object")?
        .iter()
        .map(|(path, items)| Ok((path.clone(), rows(items, "children")?)))
        .collect::<Result<Vec<_>, String>>()?;
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
                        path: whole(&link["index"], "index")?.to_string(),
                        lord: text(&link["lord"], "lord")?,
                        start: number(&link["start_jd"], "start_jd")?,
                        end: number(&link["end_jd"], "end_jd")?,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok((number(&at["jd"], "jd")?, chain))
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Method {
        name: name.to_owned(),
        remaining: number(&value["remaining_fraction"], "remaining_fraction")?,
        span,
        balance: Balance {
            parts,
            total_days: number(&balance["total_days"], "total_days")?,
        },
        depth: usize::try_from(whole(&value["tree_depth"], "tree_depth")?)
            .map_err(|_| "tree_depth is too deep")?,
        periods: rows(&value["periods"], "periods")?,
        children,
        active,
    })
}

fn records(root: &Path) -> Result<Vec<Record>, String> {
    let mut found = Vec::new();
    for dir in DIRS {
        let mut paths: Vec<_> = std::fs::read_dir(root.join(dir))
            .map_err(|err| format!("{dir}: {err}"))?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();
        for path in paths {
            let name = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            let json: Value = serde_json::from_str(
                &std::fs::read_to_string(&path).map_err(|err| format!("{name}: {err}"))?,
            )
            .map_err(|err| format!("{name}: {err}"))?;
            let dasha = &json["dashas"];
            if dasha.is_null() {
                continue;
            }
            let at = |err: String| format!("{name}: {err}");
            let methods = dasha["methods"]
                .as_object()
                .ok_or_else(|| at(String::from("methods is not an object")))?
                .iter()
                .map(|(method_name, value)| method(method_name, value).map_err(&at))
                .collect::<Result<Vec<_>, String>>()?;
            found.push(Record {
                birth: number(&json["input"]["resolved"]["jd_ut"], "jd_ut").map_err(&at)?,
                longitude: number(&dasha["moon_sidereal_longitude_deg"], "longitude")
                    .map_err(&at)?,
                index: whole(&dasha["moon_nakshatra_index"], "index").map_err(&at)?,
                elapsed: number(&dasha["moon_nakshatra_elapsed_fraction"], "elapsed")
                    .map_err(&at)?,
                start: text(&dasha["starting_lord"], "starting_lord").map_err(&at)?,
                year_length: number(&dasha["year_length_days"], "year_length_days").map_err(&at)?,
                total_years: whole(&dasha["total_years"], "total_years").map_err(&at)?,
                methods,
            });
        }
    }
    Ok(found)
}

// ── The rules proposed ─────────────────────────────────────────────────────

/// Where a lord sits in the sequence.
fn position(lord: &str) -> Option<usize> {
    VIMSHOTTARI.iter().position(|(name, _)| *name == lord)
}

/// A lord's years.
fn years(lord: &str) -> f64 {
    position(lord).map_or(f64::NAN, |at| f64::from(VIMSHOTTARI[at].1))
}

/// The nine lords from `lord` round.
fn from(lord: &str) -> impl Iterator<Item = &'static str> {
    let at = position(lord).unwrap_or(0);
    (0..VIMSHOTTARI.len()).map(move |step| VIMSHOTTARI[(at + step) % VIMSHOTTARI.len()].0)
}

/// How a period's length is cut among its children.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Accumulate {
    /// Each child starts where the last ended, its length the parent's
    /// times its share.
    Sequential,
    /// Each boundary is the parent's start plus the parent's length times
    /// the shares so far.
    Cumulative,
}

/// The children of a period of `length` days from `start`, from its own
/// lord round, each proportional to its lord's years.
fn children(lord: &str, start: f64, length: f64, arithmetic: Accumulate) -> Vec<Row> {
    let mut rows = Vec::with_capacity(VIMSHOTTARI.len());
    let (mut at, mut shares) = (start, 0.0);
    for (index, child) in from(lord).enumerate() {
        let share = years(child);
        let (begin, end) = match arithmetic {
            Accumulate::Sequential => (at, at + length * share / f64::from(TOTAL_YEARS)),
            Accumulate::Cumulative => {
                let begin = start + length * shares / f64::from(TOTAL_YEARS);
                shares += share;
                (begin, start + length * shares / f64::from(TOTAL_YEARS))
            }
        };
        at = end;
        rows.push(Row {
            path: index.to_string(),
            lord: child.to_owned(),
            start: begin,
            end,
        });
    }
    rows
}

/// The mahadashas: the first from birth for the balance, then each whole
/// period in sequence.
fn mahadashas(record: &Record, balance_days: f64) -> Vec<Row> {
    let mut at = record.birth;
    from(&record.start)
        .enumerate()
        .map(|(index, lord)| {
            let length = if index == 0 {
                balance_days
            } else {
                years(lord) * record.year_length
            };
            let row = Row {
                path: index.to_string(),
                lord: lord.to_owned(),
                start: at,
                end: at + length,
            };
            at += length;
            row
        })
        .collect()
}

/// The tree to `depth`, every level's children compressed into their
/// parent, in the corpus's row order (depth first).
fn tree(record: &Record, method: &Method, depth: usize, arithmetic: Accumulate) -> Vec<Row> {
    fn walk(row: Row, level: usize, depth: usize, arithmetic: Accumulate, out: &mut Vec<Row>) {
        let kids = if level < depth {
            children(&row.lord, row.start, row.end - row.start, arithmetic)
        } else {
            Vec::new()
        };
        let path = row.path.clone();
        out.push(row);
        for kid in kids {
            let child = Row {
                path: format!("{path}/{}", kid.path),
                ..kid
            };
            walk(child, level + 1, depth, arithmetic, out);
        }
    }
    let mut out = Vec::new();
    for maha in mahadashas(record, method.balance.total_days) {
        walk(maha, 1, depth, arithmetic, &mut out);
    }
    out
}

/// The birth period's sub-periods under the other reading: sized against
/// the whole period, which began before birth, the ones already over
/// dropped and the one running at birth cut at it.
fn elapsed_reading(record: &Record, method: &Method) -> Vec<Row> {
    let whole = years(&record.start) * record.year_length;
    let began = record.birth - (whole - method.balance.total_days);
    children(&record.start, began, whole, Accumulate::Sequential)
        .into_iter()
        .filter(|kid| kid.end > record.birth)
        .map(|kid| Row {
            start: kid.start.max(record.birth),
            ..kid
        })
        .collect()
}

/// The children of the period at `path` (`2/5/3`), found by descending to
/// it rather than building the tree around it.
fn children_at(record: &Record, method: &Method, path: &str) -> Vec<Row> {
    let mut level = mahadashas(record, method.balance.total_days);
    for step in path.split('/') {
        let Some(period) = step
            .parse::<usize>()
            .ok()
            .and_then(|at| level.get(at).cloned())
        else {
            return Vec::new();
        };
        level = children(
            &period.lord,
            period.start,
            period.end - period.start,
            Accumulate::Sequential,
        );
    }
    level
}

/// The chain of periods holding `jd`, to `depth` levels; empty past the
/// end of the cycle, as the corpus records it.
fn chain(record: &Record, method: &Method, jd: f64, depth: usize) -> Vec<Row> {
    let mut found: Vec<Row> = Vec::with_capacity(depth);
    let mut level = mahadashas(record, method.balance.total_days);
    while found.len() < depth {
        let Some(holding) = level
            .into_iter()
            .find(|row| row.start <= jd && jd < row.end)
        else {
            break;
        };
        level = children(
            &holding.lord,
            holding.start,
            holding.end - holding.start,
            Accumulate::Sequential,
        );
        found.push(holding);
    }
    found
}

/// How a balance's remainder is written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Parts {
    /// Whole years of the year length, whole months of a twelfth of it,
    /// whole days, then hours and minutes rounded to the nearest minute.
    RoundedMinutes,
    /// The same, every part floored.
    FlooredMinutes,
    /// Months of thirty days.
    ThirtyDayMonths,
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a balance is a few thousand days, and each part is floored or rounded first"
)]
fn parts(total_days: f64, year_length: f64, rule: Parts) -> [u64; 5] {
    let month = match rule {
        Parts::ThirtyDayMonths => 30.0,
        Parts::RoundedMinutes | Parts::FlooredMinutes => year_length / 12.0,
    };
    let years = (total_days / year_length).floor();
    let mut rest = total_days - years * year_length;
    let months = (rest / month).floor();
    rest -= months * month;
    let days = rest.floor();
    rest -= days;
    let minutes = match rule {
        Parts::RoundedMinutes => (rest * 1440.0).round(),
        Parts::FlooredMinutes | Parts::ThirtyDayMonths => (rest * 1440.0).floor(),
    } as u64;
    [
        years as u64,
        months as u64,
        days as u64,
        minutes / 60,
        minutes % 60,
    ]
}

// ── The page ───────────────────────────────────────────────────────────────

/// Every method of every record, with its record.
fn each(records: &[Record]) -> impl Iterator<Item = (&Record, &Method)> {
    records
        .iter()
        .flat_map(|record| record.methods.iter().map(move |method| (record, method)))
}

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

/// Every claim, in the page's order.
fn claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = seed_claims(records);
    claims.extend(balance_claims(records));
    claims.extend(tree_claims(records));
    claims.extend(deeper_claims(records));
    claims.extend(year_claims(records));
    claims
}

/// The seed, the first lord and the cycle.
fn seed_claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = Vec::new();

    let seed_wrong = records
        .iter()
        .filter(|r| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a longitude over a nakshatra's width is 0 to 26"
            )]
            let index = (r.longitude / PER_NAKSHATRA).floor() as u64;
            index != r.index
        })
        .count();
    claims.push(Claim::counted(
        "the seed is the nakshatra the Moon's sidereal longitude falls in, counted from Ashwini",
        seed_wrong,
        records.len(),
    ));
    let lord_wrong = records
        .iter()
        .filter(|r| {
            let at = usize::try_from(r.index).unwrap_or(usize::MAX) % VIMSHOTTARI.len();
            VIMSHOTTARI.get(at).map(|(lord, _)| *lord) != Some(r.start.as_str())
        })
        .count();
    claims.push(Claim::counted(
        "the first lord is the sequence's, Ketu 7 · Venus 20 · Sun 6 · Moon 10 · Mars 7 · Rahu 18 · Jupiter 16 · Saturn 19 · Mercury 17, indexed by the seed modulo nine",
        lord_wrong,
        records.len(),
    ));
    let total_wrong = records
        .iter()
        .filter(|r| r.total_years != u64::from(TOTAL_YEARS))
        .count();
    claims.push(Claim::counted(
        "the cycle is 120 years, the sum of the nine",
        total_wrong,
        records.len(),
    ));
    let elapsed_far = worst(
        records
            .iter()
            .map(|r| ((r.longitude.rem_euclid(PER_NAKSHATRA) / PER_NAKSHATRA) - r.elapsed).abs()),
    );
    claims.push(Claim::stated(
        "the elapsed fraction is the Moon's longitude into its nakshatra over the nakshatra's width",
        if elapsed_far < 1e-9 { Verdict::Holds } else { Verdict::Falsified },
        format!("worst {elapsed_far:.1e}"),
    ));

    claims
}

/// What remains of the first period, and how its balance is written.
fn balance_claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let methods = each(records).count();
    let spatial = each(records).filter(|(_, m)| m.name == "spatial");
    let spatial_far = worst(spatial.map(|(r, m)| ((1.0 - r.elapsed) - m.remaining).abs()));
    claims.push(Claim::stated(
        "spatially, what remains is one less the elapsed fraction",
        if spatial_far < SAME_FRACTION {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!("worst {spatial_far:.1e}"),
    ));
    let temporal: Vec<_> = each(records)
        .filter_map(|(r, m)| m.span.map(|span| (r, m, span)))
        .collect();
    let temporal_far = worst(
        temporal
            .iter()
            .map(|(r, m, (entry, exit))| ((exit - r.birth) / (exit - entry) - m.remaining).abs()),
    );
    claims.push(Claim::stated(
        "temporally, what remains is the time from birth to the Moon leaving its nakshatra over the time the Moon spends in it",
        if temporal_far < SAME_FRACTION { Verdict::Holds } else { Verdict::Falsified },
        format!("worst {temporal_far:.1e} over {}", count(temporal.len())),
    ));
    let balance_far = worst(each(records).map(|(r, m)| {
        (m.remaining * years(&r.start) * r.year_length - m.balance.total_days).abs()
    }));
    claims.push(Claim::stated(
        "the balance is what remains times the first lord's years times the year length",
        if balance_far < SAME_DAY {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!("worst {balance_far:.1e} days over {}", count(methods)),
    ));

    for (rule, parts_rule) in [
        (
            "the balance is written as whole years of the year length, whole months of a twelfth of it and whole days, the rest **rounded** to the minute",
            Parts::RoundedMinutes,
        ),
        (
            "the same with the rest **floored** to the minute",
            Parts::FlooredMinutes,
        ),
        (
            "the same with months of thirty days",
            Parts::ThirtyDayMonths,
        ),
    ] {
        let wrong = each(records)
            .filter(|(r, m)| {
                parts(m.balance.total_days, r.year_length, parts_rule) != m.balance.parts
            })
            .count();
        claims.push(Claim::counted(rule, wrong, methods));
    }

    claims
}

/// The tree, both readings of the birth period, and its arithmetic.
fn tree_claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let methods = each(records).count();
    // The tree under the compressed reading, both arithmetics.
    let mut row_count = 0;
    let mut tree_wrong = 0;
    let mut sequential_far = 0.0_f64;
    let mut cumulative_far = 0.0_f64;
    let mut bits = [0_usize; 2];
    let identical = |rows: &[Row], recorded: &[Row]| {
        rows.iter()
            .zip(recorded)
            .filter(|(a, b)| {
                a.start.to_bits() == b.start.to_bits() && a.end.to_bits() == b.end.to_bits()
            })
            .count()
    };
    for (r, m) in each(records) {
        let sequential = tree(r, m, m.depth, Accumulate::Sequential);
        let cumulative = tree(r, m, m.depth, Accumulate::Cumulative);
        let (wrong, of, far) = rows_agree(&sequential, &m.periods);
        tree_wrong += wrong;
        row_count += of;
        sequential_far = sequential_far.max(far);
        cumulative_far = cumulative_far.max(rows_agree(&cumulative, &m.periods).2);
        bits[0] += identical(&sequential, &m.periods);
        bits[1] += identical(&cumulative, &m.periods);
    }
    claims.push(
        Claim::counted(
            "the mahadashas run from birth for the balance, then each lord's whole years in sequence; every period's children run from its own lord, each its share of **the period it is in**, the birth period's too",
            tree_wrong,
            row_count,
        )
        .with_note(format!(
            "every bound within {:.1e} days ({})",
            sequential_far,
            crate::measure::seconds(sequential_far * 86_400.0)
        )),
    );
    let elapsed_wrong = each(records)
        .filter(|(r, m)| {
            let recorded: Vec<Row> = m
                .periods
                .iter()
                .filter(|row| row.path.starts_with("0/") && row.path.matches('/').count() == 1)
                .cloned()
                .collect();
            let proposed = elapsed_reading(r, m);
            proposed.len() != recorded.len()
                || proposed.iter().zip(&recorded).any(|(a, b)| {
                    a.lord != b.lord
                        || (a.start - b.start).abs() > 1e-3
                        || (a.end - b.end).abs() > 1e-3
                })
        })
        .count();
    claims.push(Claim::counted(
        "the birth period's children are sized against the **whole** period, which began before birth, the ones already over dropped",
        elapsed_wrong,
        methods,
    ));
    claims.push(Claim::stated(
        "a boundary reproduces the recorded double to the bit",
        Verdict::Falsified,
        format!(
            "{} of {} rows bit-identical adding lengths in turn, {} adding shares so far; worst {:.1e} and {:.1e} days",
            count(bits[0]),
            count(row_count),
            count(bits[1]),
            sequential_far,
            cumulative_far
        ),
    ));

    claims
}

/// The deeper children and the running chains.
fn deeper_claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = Vec::new();
    let mut children_rows = 0;
    let mut children_wrong = 0;
    for (r, m) in each(records) {
        for (path, rows) in &m.children {
            let proposed = children_at(r, m, path);
            let (wrong, of, far) = rows_agree(&proposed, rows);
            children_rows += of;
            children_wrong += wrong + usize::from(far > SAME_DAY);
        }
    }
    claims.push(Claim::counted(
        "the children the corpus samples at deeper paths are the same rule's, to the fifth level",
        children_wrong,
        children_rows,
    ));

    let mut chains = 0;
    let mut chain_wrong = 0;
    let mut past_end = 0;
    for (r, m) in each(records) {
        for (jd, recorded) in &m.active {
            chains += 1;
            let proposed = chain(r, m, *jd, recorded.len().max(5));
            if recorded.is_empty() {
                past_end += 1;
                chain_wrong += usize::from(!proposed.is_empty());
                continue;
            }
            let agrees = proposed.len() == recorded.len()
                && proposed.iter().zip(recorded).all(|(a, b)| {
                    a.path == b.path
                        && a.lord == b.lord
                        && (a.start - b.start).abs() < SAME_DAY
                        && (a.end - b.end).abs() < SAME_DAY
                });
            chain_wrong += usize::from(!agrees);
        }
    }
    claims.push(
        Claim::counted(
            "the periods running at an instant are found by descending the same tree, five levels deep",
            chain_wrong,
            chains,
        )
        .with_note(format!(
            "{} of the instants fall past the ninth mahadasha, where the corpus records no period at all and the cycle does not begin again",
            count(past_end)
        )),
    );

    claims
}

/// The year length.
fn year_claims(records: &[Record]) -> Vec<Claim> {
    let mut claims = Vec::new();
    // Compared to the bit: a year length is a recorded constant, and 365.25
    // is exact in binary.
    let other_lengths = records
        .iter()
        .filter(|r| r.year_length.to_bits() != JULIAN_YEAR.to_bits())
        .count();
    claims.push(Claim::stated(
        "a year of 365.25 days",
        if other_lengths == 0 {
            Verdict::Holds
        } else {
            Verdict::Falsified
        },
        format!("every one of the {} records uses it", count(records.len())),
    ));
    claims.push(Claim::stated(
        "any other year length (360, sidereal, tropical, lunar, 324)",
        Verdict::Untested,
        String::from("no record uses one, so crux C6 is not settled here"),
    ));
    claims
}

fn page(root: &Path) -> Result<String, String> {
    let records = records(root)?;
    if records.is_empty() {
        return Err(String::from("the corpus records no dasha"));
    }
    let methods = each(&records).count();
    let depths: std::collections::BTreeSet<(String, usize)> = each(&records)
        .map(|(_, m)| (m.name.clone(), m.depth))
        .collect();
    let mut out = String::new();
    let _ = write!(
        out,
        "# The Vimshottari dasha, measured\n\n\
         Status: `generated` by `cargo xtask dashas` over the conformance corpus's \
         `dashas` section, 2026-09-15. Do not edit: `check-dashas` regenerates this page \
         and fails on any difference. The design it measures is \
         [`dasha-kernels.md`](dasha-kernels.md).\n\n\
         The corpus records {records} Vimshottari dashas: 55 charts under the default \
         profile and the rest under the variant profiles that change the Moon (other \
         ayanamshas, a geocentric Moon, the Surya Siddhanta, a temporal balance). \
         {methods} method answers in all, each with its inputs, its balance, its period \
         tree ({depths}), the children at sampled deeper paths, and the chain of periods \
         running at two instants. So every rule below is decided rather than argued.\n\n\
         ## What the corpus decides\n\n{table}\n",
        records = count(records.len()),
        methods = count(methods),
        depths = depths
            .iter()
            .map(|(name, depth)| format!("{name} to depth {depth}"))
            .collect::<Vec<_>>()
            .join(", "),
        table = table(&claims(&records)),
    );
    out.push_str(
        "## What it means for the module\n\n\
         **The birth period's sub-periods are compressed.** Each child is its share of \
         the period it is in, and the birth period is only as long as its balance, so \
         its antardashas are that much shorter. The other reading, sizing them against \
         the whole period with the elapsed ones dropped, is refused by every record. \
         Both readings are taught: a tutorial works the example the compressed way, and \
         another says the sub-periods are sized against the full period. Neither is a \
         rank-1 text, so the module ships both as a knob, `dasha.birth_period`, with \
         the corpus's reading as the default, and the question as crux C48.\n\n\
         **The cycle ends.** Past the ninth mahadasha the recording engine answers no \
         period rather than beginning the sequence again, so a chart older than its \
         cycle has no running dasha. That is a choice rather than a law, and the module \
         makes it a knob, `dasha.after_cycle`, defaulting to the corpus's.\n\n\
         **Boundaries are within a quarter of a millisecond, and not to the bit.** No \
         order of float arithmetic reproduces the recorded doubles, and the corpus's \
         tolerance for a boundary is a thousandth of a day. So the module computes a \
         boundary as its period's start plus an exact rational share of its length, \
         which is the design page's `Ratio` arithmetic, and the golden comparison is to \
         the tolerance.\n\n\
         **The balance is written with its minutes rounded.** Floored minutes disagree \
         with half the records and thirty-day months with nearly all.\n\n\
         **The year length is not settled.** Every record uses 365.25 days, so the other \
         lengths the settings offer stay a knob with no measurement behind any of them, \
         which is crux C6 as it stood.\n",
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask dashas") != 0),
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
    fn the_sequence_is_the_cycle() {
        let total: u32 = VIMSHOTTARI.iter().map(|(_, years)| years).sum();
        assert_eq!(total, TOTAL_YEARS);
        assert_eq!(
            from("SATURN").collect::<Vec<_>>()[..3],
            ["SATURN", "MERCURY", "KETU"]
        );
    }

    #[test]
    fn children_fill_their_parent_whichever_way_they_are_added() {
        for arithmetic in [Accumulate::Sequential, Accumulate::Cumulative] {
            let kids = children("VENUS", 100.0, 7305.0, arithmetic);
            assert_eq!(kids.len(), 9);
            assert_eq!(kids[0].lord, "VENUS");
            assert!((kids[8].end - 7405.0).abs() < 1e-9, "{arithmetic:?}");
            for pair in kids.windows(2) {
                assert!((pair[0].end - pair[1].start).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn a_balance_is_written_with_its_minutes_rounded() {
        // Six years, eleven months, thirteen days and 11:20 of a 365.25-day
        // year, the first fixture's balance.
        assert_eq!(
            parts(2_539.784_984_723_759_4, 365.25, Parts::RoundedMinutes),
            [6, 11, 13, 11, 20]
        );
        // Half a minute rounds up where flooring does not.
        let half = 10.0 + 30.5 / 1440.0;
        assert_eq!(parts(half, 365.25, Parts::RoundedMinutes)[4], 31);
        assert_eq!(parts(half, 365.25, Parts::FlooredMinutes)[4], 30);
    }
}
