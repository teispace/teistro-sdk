//! The measurement pass over **festival rules** (`03-design/festival-rules.md`
//! §5).
//!
//! Over the built-in ephemeris at Delhi, through the façade:
//!
//! - the shipped rules against the days the Government of India published
//!   as holidays for its offices in Delhi, each disagreement named with
//!   its cause, which the pass checks both ways;
//! - a decade of each rule, counting the case each year met and the guard
//!   that decided, with every guard no year reached given its reason.
//!
//! Only the seasons the rules live in are founded, March to May and
//! August to November, since a year of almanac is what the pass costs.
//!
//! `cargo xtask festival` writes the page; `check-festival` regenerates it
//! in memory and fails on any difference.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use teistro::festival::{Decided, FestivalRule, Observance};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, FestivalPack, FestivalRequest, UtcOffset};
use teistro_core::catalogue::Calendar;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

const PAGE: &str = "docs/03-design/festival-measured.md";

/// Delhi, whose offices the published list is for, and its clock.
const LATITUDE: f64 = 28.6139;
const LONGITUDE: f64 = 77.209;
const ALTITUDE: f64 = 216.0;

/// The seasons founded each year: `(from, to)` as (month, day).
const SEASONS: [((u8, u8), (u8, u8)); 2] = [((3, 1), (5, 5)), ((8, 1), (11, 20))];

/// The decade the reach is counted over.
const REACH: std::ops::RangeInclusive<i32> = 2021..=2030;

/// Which list a published day comes from.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Listed {
    /// Annexure I, the compulsory holidays.
    Compulsory,
    /// Annexure I's Janmashtami, marked Vaishnava.
    Vaishnava,
    /// Annexure II's Janmashtami, marked Smarta.
    Smarta,
}

impl Listed {
    const fn spelled(self) -> &'static str {
        match self {
            Listed::Compulsory => "compulsory",
            Listed::Vaishnava => "Vaishnava",
            Listed::Smarta => "Smarta",
        }
    }
}

/// The days the Department of Personnel and Training's office memoranda
/// list for the administrative offices of the central government in
/// Delhi, read off copies other offices republished (the department's own
/// server refused the pass's requests): 2021 F.No.12/9/2020-JCA (Delhi
/// Development Authority), 2023 F.No.12/5/2022-JCA (the Comptroller and
/// Auditor General), 2024 F.No.12/2/2023-JCA (REC Limited), 2025
/// F.No.12/2/2023-JCA (the Indian Council of Agricultural Research, a
/// scan; its Rama Navami row is illegible and left out) and 2027
/// F.No.12/2/2023-JCA (CSIR). The compulsory Janmashtami is the Vaishnava
/// day; the restricted list gives a Smarta day beside it in 2021, 2023 and
/// 2025, and *Dharmasindhu* is a Smarta text, so both are listed.
const PUBLISHED: [(i32, &str, Listed, (u8, u8)); 22] = [
    (2021, "RAMA_NAVAMI", Listed::Compulsory, (4, 21)),
    (2021, "JANMASHTAMI", Listed::Vaishnava, (8, 30)),
    (2021, "JANMASHTAMI", Listed::Smarta, (8, 30)),
    (2021, "VIJAYA_DASHAMI", Listed::Compulsory, (10, 15)),
    (2021, "LAKSHMI_PUJA", Listed::Compulsory, (11, 4)),
    (2023, "RAMA_NAVAMI", Listed::Compulsory, (3, 30)),
    (2023, "JANMASHTAMI", Listed::Vaishnava, (9, 7)),
    (2023, "JANMASHTAMI", Listed::Smarta, (9, 6)),
    (2023, "VIJAYA_DASHAMI", Listed::Compulsory, (10, 24)),
    (2023, "LAKSHMI_PUJA", Listed::Compulsory, (11, 12)),
    (2024, "RAMA_NAVAMI", Listed::Compulsory, (4, 17)),
    (2024, "JANMASHTAMI", Listed::Vaishnava, (8, 26)),
    (2024, "VIJAYA_DASHAMI", Listed::Compulsory, (10, 12)),
    (2024, "LAKSHMI_PUJA", Listed::Compulsory, (10, 31)),
    (2025, "JANMASHTAMI", Listed::Vaishnava, (8, 16)),
    (2025, "JANMASHTAMI", Listed::Smarta, (8, 15)),
    (2025, "VIJAYA_DASHAMI", Listed::Compulsory, (10, 2)),
    (2025, "LAKSHMI_PUJA", Listed::Compulsory, (10, 20)),
    (2027, "RAMA_NAVAMI", Listed::Compulsory, (4, 15)),
    (2027, "JANMASHTAMI", Listed::Vaishnava, (8, 25)),
    (2027, "VIJAYA_DASHAMI", Listed::Compulsory, (10, 9)),
    (2027, "LAKSHMI_PUJA", Listed::Compulsory, (10, 29)),
];

/// Why the shipped rule parts from a published day, per (year, rule,
/// list). The pass fails on a disagreement with no entry, and on an entry
/// whose row agrees.
const PARTS: [(i32, &str, Listed, &str); 4] = [
    (2023, "JANMASHTAMI", Listed::Vaishnava, VAISHNAVA),
    (
        2024,
        "LAKSHMI_PUJA",
        Listed::Compulsory,
        "the new moon lasts more than a ghati past the later day's sunset, which p. 77 calls beyond doubt; the list takes the earlier day, over which the calendars of that year were divided",
    ),
    (2025, "JANMASHTAMI", Listed::Vaishnava, VAISHNAVA),
    (2027, "JANMASHTAMI", Listed::Vaishnava, VAISHNAVA),
];

/// The cause the Vaishnava Janmashtami rows share.
const VAISHNAVA: &str = "the Vaishnava day, the later one, whose sunrise the 8th holds; the shipped rule is the Smarta one of p. 49, which takes the earlier day when it alone holds nishitha, and *Dharmasindhu* states no Vaishnava rule for Janmashtami (C175)";

/// Why no year of the decade reached a guard, per (rule, guard index),
/// with the crate test that reaches it on synthetic days. The pass fails
/// on an unreached guard with no entry, on an entry for a guard a year
/// reached, and on a test the crate does not have.
const UNREACHED: [(&str, usize, &str, &str); 2] = [
    (
        "VIJAYA_DASHAMI",
        1,
        "vijaya_dashami_held_later_alone_yields_to_shravana_on_the_earlier_evening",
        "the 10th beginning after the earlier day's aparahna with Shravana joining it only that evening, which no year of the decade had",
    ),
    (
        "VIJAYA_DASHAMI",
        6,
        "vijaya_dashami_held_on_both_days_or_neither_goes_to_shravana_alone",
        "the 10th holding both aparahnas, neither or each in part, with Shravana joining it on the later day only, which no year of the decade had",
    ),
];

/// The crate tests an `UNREACHED` entry may cite.
const CRATE_TESTS: &str = include_str!("../../crates/panchanga/tests/festival.rs");

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask festival") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn place() -> Place {
    Place::new(
        Latitude::literal(LATITUDE),
        Longitude::literal(LONGITUDE),
        Altitude::literal(ALTITUDE),
    )
}

fn date(year: i32, (month, day): (u8, u8)) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// Each rule's observances over a year's seasons.
fn year_of(
    context: &Context,
    request: &FestivalRequest,
    year: i32,
) -> Result<Vec<Observance>, String> {
    let mut all = Vec::new();
    for (from, to) in SEASONS {
        let found = context
            .almanac()
            .festivals(
                &date(year, from),
                &date(year, to),
                &place(),
                UtcOffset::literal(5, 30, 0),
                request,
            )
            .map_err(|e| format!("{year}: {e}"))?;
        if !found.value.unjudged.is_empty() {
            return Err(format!("{year}: unjudged {:?}", found.value.unjudged));
        }
        all.extend(found.value.observances);
    }
    Ok(all)
}

struct Row {
    year: i32,
    rule: &'static str,
    listed: Listed,
    published: (u8, u8),
    found: Option<Observance>,
}

impl Row {
    fn agrees(&self) -> bool {
        self.found
            .as_ref()
            .is_some_and(|found| (found.day.month, found.day.day) == self.published)
    }
}

/// Every year's observances, found by as many workers as the machine has
/// cores, each with its own context, over consecutive runs of the years.
/// A year's answer does not depend on which worker found it, so the map
/// is the one a single thread would build; the refusal reported is the
/// first worker's, in the years' order.
fn years_of(request: &FestivalRequest) -> Result<BTreeMap<i32, Vec<Observance>>, String> {
    let wanted: Vec<i32> = PUBLISHED
        .iter()
        .map(|row| row.0)
        .chain(REACH)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let run = wanted.len().div_ceil(workers).max(1);
    std::thread::scope(|scope| {
        let found: Vec<_> = wanted
            .chunks(run)
            .map(|part| {
                scope.spawn(move || {
                    let context = Context::builder()
                        .ephemeris([Ephemeris::Builtin])
                        .build()
                        .map_err(|e| e.to_string())?;
                    part.iter()
                        .map(|&year| Ok((year, year_of(&context, request, year)?)))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut years = BTreeMap::new();
        for worker in found {
            years.extend(
                worker
                    .join()
                    .map_err(|_| String::from("a worker finding the festivals panicked"))??,
            );
        }
        Ok(years)
    })
}

fn page() -> Result<String, String> {
    let request = FestivalRequest::from(FestivalPack::Dharmasindhu);
    let rules = request.rules();
    let years = years_of(&request)?;

    let mut rows: Vec<Row> = PUBLISHED
        .iter()
        .map(|&(year, rule, listed, published)| Row {
            year,
            rule,
            listed,
            published,
            found: years
                .get(&year)
                .and_then(|found| found.iter().find(|o| o.rule == rule).cloned()),
        })
        .collect();
    rows.sort_by_key(|row| (row.year, row.rule, row.listed.spelled()));

    let mut problems = Vec::new();
    for row in &rows {
        let excused = PARTS
            .iter()
            .any(|p| (p.0, p.1, p.2) == (row.year, row.rule, row.listed));
        if !row.agrees() && !excused {
            problems.push(format!(
                "{} {} ({}) published {:?}, found {:?}: a disagreement needs its cause in PARTS",
                row.year,
                row.rule,
                row.listed.spelled(),
                row.published,
                row.found
                    .as_ref()
                    .map(|o| (o.day.month, o.day.day, o.case, o.decided_by))
            ));
        }
        if row.agrees() && excused {
            problems.push(format!(
                "{} {} ({}) agrees and is excused in PARTS: remove the entry",
                row.year,
                row.rule,
                row.listed.spelled()
            ));
        }
    }

    let (cases, decided) = reach(rules, &years, &mut problems);
    if !problems.is_empty() {
        return Err(problems.join("\n      "));
    }
    Ok(render(rules, &rows, &cases, &decided))
}

/// How many years met each (rule, case).
type Cases<'r> = BTreeMap<(&'r str, String), usize>;

/// How many years each (rule, guard) decided; `None` is the rule's
/// `otherwise`.
type Deciders = BTreeMap<(String, Option<usize>), usize>;

/// Each rule's case and deciding guard over the decade, with a problem for
/// every guard reached and listed, or neither.
fn reach<'r>(
    rules: &'r [FestivalRule],
    years: &BTreeMap<i32, Vec<Observance>>,
    problems: &mut Vec<String>,
) -> (Cases<'r>, Deciders) {
    let mut cases = Cases::new();
    let mut decided = Deciders::new();
    for year in REACH {
        for observance in years.get(&year).into_iter().flatten() {
            *cases
                .entry((
                    rule_key(rules, &observance.rule),
                    format!("{:?}", observance.case),
                ))
                .or_default() += 1;
            let guard = match observance.decided_by {
                Decided::Guard { index } => Some(index),
                Decided::Otherwise => None,
            };
            *decided.entry((observance.rule.clone(), guard)).or_default() += 1;
        }
    }
    for rule in rules {
        for index in 0..rule.decide.len() {
            let reached = decided.contains_key(&(rule.key.clone(), Some(index)));
            let listed = UNREACHED
                .iter()
                .find(|u| (u.0, u.1) == (rule.key.as_str(), index));
            if let Some((_, _, test, _)) = listed {
                if !CRATE_TESTS.contains(&format!("fn {test}()")) {
                    problems.push(format!(
                        "{} guard {index} cites the test {test}, which crates/panchanga/tests/festival.rs does not have",
                        rule.key
                    ));
                }
            }
            let listed = listed.is_some();
            if !reached && !listed {
                problems.push(format!(
                    "{} guard {index} ({:?}) decided no year of {REACH:?}: it needs its reason in UNREACHED",
                    rule.key, rule.decide[index].when
                ));
            }
            if reached && listed {
                problems.push(format!(
                    "{} guard {index} decided a year and is listed in UNREACHED: remove the entry",
                    rule.key
                ));
            }
        }
    }
    (cases, decided)
}

fn rule_key<'r>(rules: &'r [FestivalRule], key: &str) -> &'r str {
    rules
        .iter()
        .find(|rule| rule.key == key)
        .map_or("", |rule| rule.key.as_str())
}

fn spell((month, day): (u8, u8)) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let name = MONTHS
        .get(usize::from(month).saturating_sub(1))
        .copied()
        .unwrap_or("?");
    format!("{day} {name}")
}

/// §1: each published row beside the day found, and the partings' causes.
fn published(out: &mut String, rows: &[Row]) {
    out.push_str("## 1. Against the published days\n\n");
    out.push_str(
        "The days the Government of India listed as holidays for its offices in Delhi,\n\
         read off copies of each year's office memorandum (the pass's source names them).\n\
         Janmashtami is listed twice where the restricted list gives a Smarta day apart\n\
         from the Vaishnava one; *Dharmasindhu* is a Smarta text.\n\n",
    );
    out.push_str("| year | rule | list | published | found | case | decided by |\n|---:|---|---|---|---|---|---|\n");
    for row in rows {
        let (found, case, by) = row.found.as_ref().map_or_else(
            || ("none".to_owned(), String::new(), String::new()),
            |o| {
                (
                    spell((o.day.month, o.day.day)),
                    format!("{:?}", o.case),
                    match o.decided_by {
                        Decided::Guard { index } => format!("guard {index}"),
                        Decided::Otherwise => "otherwise".to_owned(),
                    },
                )
            },
        );
        let _ = writeln!(
            *out,
            "| {} | {} | {} | {} | {}{} | {} | {} |",
            row.year,
            row.rule,
            row.listed.spelled(),
            spell(row.published),
            found,
            if row.agrees() { "" } else { " ✗" },
            case,
            by
        );
    }
    let wrong = rows.iter().filter(|row| !row.agrees()).count();
    out.push('\n');
    // The pass has refused a parting without its cause, so every one
    // counted here is named below.
    out.push_str(&table(&[Claim::counted(
        "each shipped rule falls on the day the published list gives",
        wrong,
        rows.len(),
    )
    .with_note("each parting is named below with its cause")]));
    out.push('\n');
    if PARTS.is_empty() {
        out.push_str("No row parts from the published day.\n\n");
    } else {
        out.push_str("Where the rule parts from the published day, and why:\n\n");
        for (year, rule, listed, why) in PARTS {
            let _ = writeln!(*out, "- {year} {rule} ({}): {why}", listed.spelled());
        }
        out.push('\n');
    }
}

fn render(rules: &[FestivalRule], rows: &[Row], cases: &Cases<'_>, decided: &Deciders) -> String {
    let mut out = String::new();
    out.push_str("# Festival rules, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask festival`. Do not edit:\n\
         `check-festival` regenerates this page and fails on any difference.\n\n\
         It measures `festival-rules.md` §5.2 and §5.3: the shipped rules\n\
         (`FestivalRule::dharmasindhu()`) through `sdk.almanac().festivals`, over\n\
         the built-in ephemeris at Delhi on India's clock.\n\n",
    );

    published(&mut out, rows);
    let _ = writeln!(
        out,
        "## 2. What a decade reaches\n\n\
         Each rule over {}–{}: the case each year met between the tithi's two\n\
         days, and the guard that decided. A guard no year reached is listed\n\
         with its reason; the pass fails on one that is not, and on a listed\n\
         one a year did reach.\n",
        REACH.start(),
        REACH.end()
    );
    out.push_str("| rule | cases met | decided by |\n|---|---|---|\n");
    for rule in rules {
        let met: Vec<String> = cases
            .iter()
            .filter(|((key, _), _)| *key == rule.key)
            .map(|((_, case), n)| format!("{case} {n}"))
            .collect();
        let by: Vec<String> = decided
            .iter()
            .filter(|((key, _), _)| *key == rule.key)
            .map(|((_, guard), n)| match guard {
                Some(index) => format!("guard {index}: {n}"),
                None => format!("otherwise: {n}"),
            })
            .collect();
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            rule.key,
            met.join(", "),
            by.join(", ")
        );
    }
    out.push('\n');
    if UNREACHED.is_empty() {
        out.push_str("Every guard decided at least one year.\n");
    } else {
        out.push_str("The guards no year reached, and why:\n\n");
        for (rule, index, test, why) in UNREACHED {
            let _ = writeln!(
                out,
                "- {rule} guard {index}: {why}; the crate test `{test}` reaches it."
            );
        }
    }
    out
}
