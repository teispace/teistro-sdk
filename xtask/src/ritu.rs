//! The measurement pass over **the season** (cruxes C178 and C186,
//! `03-design/ritu-measured.md`).
//!
//! The Surya Siddhanta (XIV.10) counts six seasons from the Sun's entry
//! into Capricorn, two signs each. Which *day* a season begins on is a
//! second question: a sankranti falls at an instant and a month begins on
//! a day. This pass holds the almanac's `sun.ritu` to the season Nepal's
//! daily panchanga printed on 341 days, under the shipped reading and
//! every rival the knobs offer, each rival held to miss at least one day
//! and the shipped reading held to miss none; and the almanac's `ayana`
//! to the same records.
//!
//! `cargo xtask ritu` writes the page; `check-ritu` regenerates it in
//! memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Ayana, Calendar, Ritu};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, Panchanga, UtcOffset};
use teistro_panchanga::sky::ritu_of;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

const PAGE: &str = "docs/03-design/ritu-measured.md";

/// Nepal's daily panchanga as Makalukhabar printed it ("आजको पञ्चाङ्ग"),
/// a month a row: the season and the ayana of each day, `.` for a day
/// with no post. Seasons are numbered from Vasanta, 1 to 6, in the
/// Surya Siddhanta's order; `U` is uttarayana and `D` dakshinayana.
/// Collected on 1 October 2026 from every post of the series, each read
/// for its "इ.सन्" date, its season ("… ऋतु") and its ayana.
const RECORDS: [(i32, u8, &str, &str); 18] = [
    (
        2025,
        4,
        "........1...1.11.11.111..1...1",
        "........U...U.UU.UU.UUU..U...U",
    ),
    (
        2025,
        5,
        "....11111..11.2.222....22222...",
        "....UUUUU..UU.U.UUU....UUUUU...",
    ),
    (
        2025,
        6,
        "2.222.22..2..222222.22..2222.2",
        "U.UUU.UU..U..UUUUUU.UU..UUUU.U",
    ),
    (
        2025,
        7,
        "...2.22222.2222.33333..3.333.33",
        "...U.UUUUU.UUUU.DDDDD..D.DDD.DD",
    ),
    (
        2025,
        8,
        "333.333..33.33333.3..3.3.33.3..",
        "DDD.DDD..DD.DDDDD.D..D.D.DD.D..",
    ),
    (
        2025,
        9,
        "3..3...............4..........",
        "D..D...............D..........",
    ),
    (
        2025,
        10,
        "...............4444...4..444444",
        "...............DDDD...D..DDDDDD",
    ),
    (
        2025,
        11,
        "..4.4.44.44.444455555555......",
        "..D.D.DD.DD.DDDDDDDDDDDD......",
    ),
    (
        2025,
        12,
        "..............5.55.55.......5..",
        "..............D.DD.DD.......D..",
    ),
    (
        2026,
        1,
        ".5.555555...5..6.....666..6.6.6",
        ".D.DDDDDD...D..U.....UUU..U.U.U",
    ),
    (
        2026,
        2,
        "666.6..6..66.............66.",
        "UUU.U..U..UU.............UU.",
    ),
    (
        2026,
        3,
        "6666.6.66666.61111.1.11111111.1",
        "UUUU.U.UUUUU.UUUUU.U.UUUUUUUU.U",
    ),
    (
        2026,
        4,
        "11..11111111111111111111111111",
        "UU..UUUUUUUUUUUUUUUUUUUUUUUUUU",
    ),
    (
        2026,
        5,
        ".111111111111122222222.22222222",
        ".UUUUUUUUUUUUUUUUUUUUU.UUUUUUUU",
    ),
    (
        2026,
        6,
        "22222222222222.22222222222.2.2",
        "UUUUUUUUUUUUUU.UUUUUUUUUUU.U.U",
    ),
    (
        2026,
        7,
        "22222222.22222223.33333333333.3",
        "UUUUUUUU.UUUUUUUD.DDDDDDDDDDD.D",
    ),
    (
        2026,
        8,
        "333.3.333333333333333.333333333",
        "DDD.D.DDDDDDDDDDDDDDD.DDDDDDDDD",
    ),
    (
        2026,
        9,
        "33333333333.3.3344.44444444444",
        "DDDDDDDDDDD.D.DDDD.DDDDDDDDDDD",
    ),
];

/// The seasons in the order the records number them.
const SEASONS: [Ritu; 6] = [
    Ritu::Vasanta,
    Ritu::Grishma,
    Ritu::Varsha,
    Ritu::Sharad,
    Ritu::Hemanta,
    Ritu::Shishira,
];

/// The Bikram Sambat years a rival the records cannot separate is
/// measured over: the last of the official span, founded around each
/// month a season begins with.
const FORWARD: core::ops::RangeInclusive<i32> = 2070..=2095;

/// The Bikram Sambat month each season begins with, in `SEASONS` order:
/// Chaitra (Pisces), Jestha, Shrawan, Ashwin, Mangsir and Magh.
const SEASON_MONTHS: [u8; 6] = [12, 2, 4, 6, 8, 10];

/// The threads the forward measurement is split over.
const FORWARD_THREADS: usize = 4;

/// A reading's misses are listed day by day up to this many.
const LISTED: usize = 12;

/// One recorded day.
#[derive(Clone, Copy)]
struct Record {
    date: (i32, u8, u8),
    ritu: Ritu,
    ayana: Ayana,
}

/// How a reading takes the season off a founded day.
#[derive(Clone, Copy)]
enum Take {
    /// The almanac's `sun.ritu`.
    Ritu,
    /// The season of the sign the Sun holds as the day opens, which is
    /// no setting: the reading the knob's first draft shipped.
    Opening,
}

/// What the page expects of a reading over the records.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Expect {
    /// It names every recorded season.
    Agrees,
    /// It misses at least one.
    Misses,
}

/// A reading of the season, and what the page expects of it.
struct Reading {
    name: &'static str,
    /// The column head in the table of changes, for the readings that
    /// differ only in the day a month begins on.
    head: Option<&'static str>,
    /// The settings patch over `nepali-default`, JSON.
    patch: Option<&'static str>,
    take: Take,
    shipped: bool,
    expect: Expect,
}

const READINGS: [Reading; 10] = [
    Reading {
        name: "the Bikram Sambat calendar's months (`BIKRAM_SAMBAT`, `nepali-default`'s)",
        head: Some("BIKRAM_SAMBAT"),
        patch: None,
        take: Take::Ritu,
        shipped: true,
        expect: Expect::Agrees,
    },
    Reading {
        name: "the punya-kala over the modern Sun (`PUNYAKALA`, the root's)",
        head: Some("PUNYAKALA"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "PUNYAKALA"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "the civil day of the modern sankranti (`SANKRANTI_DAY`)",
        head: Some("SANKRANTI_DAY"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "SANKRANTI_DAY"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "the civil day after it (`FOLLOWING_DAY`)",
        head: Some("FOLLOWING_DAY"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "FOLLOWING_DAY"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "the almanac day holding it (`SUNRISE_TO_SUNRISE`)",
        head: Some("SUNRISE_TO_SUNRISE"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "SUNRISE_TO_SUNRISE"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "before sunset, else the next day (`BEFORE_SUNSET`)",
        head: Some("BEFORE_SUNSET"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "BEFORE_SUNSET"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "before aparahna, else the next day (`BEFORE_APARAHNA`)",
        head: Some("BEFORE_APARAHNA"),
        patch: Some(r#"{"panchanga": {"solar_month_start": "BEFORE_APARAHNA"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Agrees,
    },
    Reading {
        name: "the modern Sun's sign as the day opens (no setting)",
        head: Some("sign at opening"),
        patch: None,
        take: Take::Opening,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "the lunar month's season (`panchanga.ritu` `LUNAR`)",
        head: None,
        patch: Some(r#"{"panchanga": {"ritu": "LUNAR"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
    Reading {
        name: "the tropical Sun's (`panchanga.ritu` `TROPICAL`)",
        head: None,
        patch: Some(r#"{"panchanga": {"ritu": "TROPICAL"}}"#),
        take: Take::Ritu,
        shipped: false,
        expect: Expect::Misses,
    },
];

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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask ritu") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn kathmandu() -> (Place, UtcOffset) {
    (
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        ),
        UtcOffset::literal(5, 45, 0),
    )
}

fn context(patch: Option<&str>) -> Result<Context, String> {
    let builder = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin]);
    match patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .map_err(|e| e.to_string())
}

fn gregorian((y, m, d): (i32, u8, u8)) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, y, m, d)
}

/// The records, day by day, refusing a malformed row.
fn records() -> Result<Vec<Record>, String> {
    let mut out = Vec::new();
    for (year, month, ritus, ayanas) in RECORDS {
        if ritus.len() != ayanas.len() {
            return Err(format!("{year}-{month:02}: the rows differ in length"));
        }
        for (k, (r, a)) in ritus.chars().zip(ayanas.chars()).enumerate() {
            let day = u8::try_from(k + 1).map_err(|e| e.to_string())?;
            let ritu = match r.to_digit(10) {
                Some(n @ 1..=6) => SEASONS.get(n as usize - 1).copied(),
                _ => None,
            };
            let ayana = match a {
                'U' => Some(Ayana::Uttarayana),
                'D' => Some(Ayana::Dakshinayana),
                _ => None,
            };
            match (ritu, ayana, r, a) {
                (Some(ritu), Some(ayana), _, _) => out.push(Record {
                    date: (year, month, day),
                    ritu,
                    ayana,
                }),
                (None, None, '.', '.') => {}
                _ => {
                    return Err(format!(
                        "{year}-{month:02}-{day:02}: '{r}' '{a}' is no record"
                    ));
                }
            }
        }
    }
    Ok(out)
}

/// Each record's founded day under a context, founded a month at a time:
/// a range holds at most a year, and a month's is what a caller asks for.
fn days(context: &Context, records: &[Record]) -> Result<Vec<Panchanga>, String> {
    let (place, offset) = kathmandu();
    let calendar = context.calendar();
    let mut out = Vec::with_capacity(records.len());
    for month in records.chunk_by(|a, b| (a.date.0, a.date.1) == (b.date.0, b.date.1)) {
        let (Some(first), Some(last)) = (month.first(), month.last()) else {
            continue;
        };
        let origin = calendar
            .fixed_of(&gregorian(first.date))
            .map_err(|e| e.to_string())?;
        let span = context
            .almanac()
            .of(
                &gregorian(first.date),
                &gregorian(last.date),
                &place,
                offset,
            )
            .map_err(|e| e.to_string())?
            .value;
        for record in month {
            let fixed = calendar
                .fixed_of(&gregorian(record.date))
                .map_err(|e| e.to_string())?;
            let index = usize::try_from(origin.days_until(fixed)).map_err(|e| e.to_string())?;
            let day = span
                .get(index)
                .filter(|day| {
                    let date = &day.day.date;
                    (date.year, date.month, date.day) == record.date
                })
                .ok_or_else(|| format!("no founded day for {}", iso(record.date)))?;
            out.push(day.clone());
        }
    }
    Ok(out)
}

fn season(take: Take, day: &Panchanga) -> Ritu {
    match take {
        Take::Ritu => day.sun.ritu,
        Take::Opening => day.sun.sign().map_or(day.sun.ritu, ritu_of),
    }
}

fn iso((y, m, d): (i32, u8, u8)) -> String {
    format!("{y}-{m:02}-{d:02}")
}

/// Each reading's seasons over the records, in `READINGS` order, and the
/// shipped reading's days. A reading taken off the day as it opens is
/// read from the shipped days, and every other is founded in its own
/// context on its own thread: a day founds the Moon's series, which is
/// most of the pass's cost and none of the season's.
fn readings(records: &[Record]) -> Result<(Vec<Vec<Ritu>>, Vec<Panchanga>), String> {
    let founded: Vec<Option<Vec<Panchanga>>> = std::thread::scope(|scope| {
        let handles: Vec<_> = READINGS
            .iter()
            .map(|reading| {
                matches!(reading.take, Take::Ritu)
                    .then(|| scope.spawn(|| days(&context(reading.patch)?, records)))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| {
                handle
                    .map(|h| {
                        h.join()
                            .map_err(|_| String::from("a founding thread panicked"))?
                    })
                    .transpose()
            })
            .collect::<Result<_, String>>()
    })?;
    let shipped = READINGS
        .iter()
        .zip(&founded)
        .find_map(|(reading, days)| reading.shipped.then_some(days.as_ref()).flatten())
        .cloned()
        .ok_or("no shipped reading founded")?;
    let seasons = READINGS
        .iter()
        .zip(&founded)
        .map(|(reading, days)| {
            days.as_ref()
                .unwrap_or(&shipped)
                .iter()
                .map(|day| season(reading.take, day))
                .collect()
        })
        .collect();
    Ok((seasons, shipped))
}

/// A reading over the season changes the Bikram Sambat calendar makes in
/// `FORWARD`: each change is right when the reading names the old season
/// on the day before the month and the new one on its first day. The
/// misses, as Bikram Sambat dates, and the changes measured.
fn forward(reading: &Reading) -> Result<(Vec<String>, usize), String> {
    let years: Vec<i32> = FORWARD.collect();
    let chunk = years.len().div_ceil(FORWARD_THREADS).max(1);
    let parts: Vec<Result<(Vec<String>, usize), String>> = std::thread::scope(|scope| {
        let handles: Vec<_> = years
            .chunks(chunk)
            .map(|part| scope.spawn(move || forward_years(reading, part)))
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .map_err(|_| String::from("a forward thread panicked"))?
            })
            .collect()
    });
    let mut misses = Vec::new();
    let mut measured = 0;
    for part in parts {
        let (m, n) = part?;
        misses.extend(m);
        measured += n;
    }
    Ok((misses, measured))
}

fn forward_years(reading: &Reading, years: &[i32]) -> Result<(Vec<String>, usize), String> {
    let context = context(reading.patch)?;
    let calendar = context.calendar();
    let (place, offset) = kathmandu();
    let mut misses = Vec::new();
    let mut measured = 0;
    for year in years {
        for (k, month) in SEASON_MONTHS.iter().enumerate() {
            let first = calendar
                .fixed_of(&CalendarDate::defined(
                    Calendar::BikramSambat,
                    *year,
                    *month,
                    1,
                ))
                .map_err(|e| e.to_string())?;
            let on = |fixed| {
                calendar
                    .date_of(Calendar::Gregorian, fixed)
                    .map_err(|e| e.to_string())
            };
            let days = context
                .almanac()
                .of(&on(first.plus_days(-1))?, &on(first)?, &place, offset)
                .map_err(|e| e.to_string())?
                .value;
            let before = SEASONS.get((k + 5) % 6).copied();
            let after = SEASONS.get(k).copied();
            let named: Vec<Option<Ritu>> = days
                .iter()
                .map(|day| Some(season(reading.take, day)))
                .collect();
            measured += 1;
            if named != [before, after] {
                misses.push(format!("{year}-{month:02}-01"));
            }
        }
    }
    Ok((misses, measured))
}

/// The records' days a season changed on, where the day before is also
/// recorded: the days that decide between the readings.
fn changes(records: &[Record]) -> Vec<usize> {
    records
        .windows(2)
        .enumerate()
        .filter_map(|(k, pair)| match pair {
            [before, after] if before.ritu != after.ritu => Some(k + 1),
            _ => None,
        })
        .collect()
}

/// What the page's sections gather: the claims, and a problem for every
/// verdict the page does not expect.
#[derive(Default)]
struct Findings {
    claims: Vec<Claim>,
    problems: Vec<String>,
}

/// A cell listing a reading's misses, or their count when there are many.
fn listed(misses: &[String], of: usize) -> String {
    if misses.is_empty() {
        String::from("none")
    } else if misses.len() <= LISTED {
        misses.join(", ")
    } else {
        format!("{} of {of}", misses.len())
    }
}

/// §3: each reading over the records. Also whether each rule over the
/// modern Sun misses a recorded day, which §6 counts.
fn over_records(
    records: &[Record],
    seasons: &[Vec<Ritu>],
    found: &mut Findings,
) -> (String, Vec<bool>) {
    let mut out = String::from(
        "\n## 3. Each reading over the records\n\n\
         | reading | days it misses |\n|---|---|\n",
    );
    let mut modern_rules = Vec::new();
    for (reading, column) in READINGS.iter().zip(seasons) {
        let wrong: Vec<String> = records
            .iter()
            .zip(column)
            .filter(|(record, ritu)| record.ritu != **ritu)
            .map(|(record, ritu)| format!("{} ({})", iso(record.date), ritu.key()))
            .collect();
        let agrees = wrong.is_empty();
        if reading.head.is_some() && reading.patch.is_some() {
            modern_rules.push(!agrees);
        }
        if (reading.expect == Expect::Agrees) != agrees {
            found.problems.push(format!(
                "{} {} the records, and the page expects it {}",
                reading.name,
                if agrees { "reproduces" } else { "misses" },
                if reading.expect == Expect::Agrees {
                    "to"
                } else {
                    "not to"
                }
            ));
        }
        let claim = Claim::counted(
            format!("{}: the seasons Nepal printed", reading.name),
            wrong.len(),
            records.len(),
        );
        found.claims.push(match (reading.shipped, reading.expect) {
            (true, _) => claim,
            (false, Expect::Agrees) => claim.with_note("a rival the records cannot separate"),
            (false, Expect::Misses) => claim.with_note("a rival"),
        });
        let _ = writeln!(
            out,
            "| {} | {} |",
            reading.name,
            listed(&wrong, records.len())
        );
    }
    (out, modern_rules)
}

/// The ayana the shipped days name, against the records.
fn ayana(records: &[Record], shipped: &[Panchanga], found: &mut Findings) {
    let wrong: Vec<String> = records
        .iter()
        .zip(shipped)
        .filter(|(record, day)| record.ayana != day.sun.ayana)
        .map(|(record, _)| iso(record.date))
        .collect();
    if !wrong.is_empty() {
        found.problems.push(format!(
            "the ayana misses {} recorded days: {}",
            wrong.len(),
            wrong.join(", ")
        ));
    }
    found.claims.push(Claim::counted(
        "the ayana, the sidereal Sun's sign as the day opens: the ayana Nepal printed",
        wrong.len(),
        records.len(),
    ));
}

/// §4: the recorded days a season changed on, under each reading of the
/// day a month begins on.
fn changed(records: &[Record], seasons: &[Vec<Ritu>]) -> Result<String, String> {
    let context = context(None)?;
    let mut out = String::from(
        "\n## 4. The days a season changed\n\n\
         Each recorded day whose season differs from the recorded day before\n\
         it, with its Bikram Sambat date. A cell is what the reading names;\n\
         a miss is marked.\n\n\
         | day | Bikram Sambat | printed |",
    );
    let rules: Vec<(usize, &str)> = READINGS
        .iter()
        .enumerate()
        .filter_map(|(k, reading)| reading.head.map(|head| (k, head)))
        .collect();
    for (_, head) in &rules {
        let _ = write!(out, " {head} |");
    }
    out.push_str("\n|---|---|---|");
    out.push_str(&"---|".repeat(rules.len()));
    out.push('\n');
    for index in changes(records) {
        let Some(record) = records.get(index) else {
            continue;
        };
        let bs = context
            .calendar()
            .convert(&gregorian(record.date), Calendar::BikramSambat)
            .map_err(|e| e.to_string())?;
        let _ = write!(
            out,
            "| {} | {} {:02} {:02} | {} |",
            iso(record.date),
            bs.year,
            bs.month,
            bs.day,
            record.ritu.key()
        );
        for (k, _) in &rules {
            let ritu = seasons
                .get(*k)
                .and_then(|column| column.get(index))
                .copied()
                .ok_or("a reading without the day")?;
            if ritu == record.ritu {
                let _ = write!(out, " {} |", ritu.key());
            } else {
                let _ = write!(out, " **{}** |", ritu.key());
            }
        }
        out.push('\n');
    }
    Ok(out)
}

/// §5: each rival the records cannot separate, over the calendar's season
/// changes in `FORWARD`.
fn ahead(found: &mut Findings) -> Result<String, String> {
    let mut out = String::from(
        "\n## 5. Where the records cannot separate a rival\n\n\
         A rival that names every recorded season is founded around each\n\
         month the Bikram Sambat calendar begins a season with, over the\n\
         years below: the day before the month and its first day.\n\n\
         | reading | Bikram Sambat years | season changes it misses |\n|---|---|---|\n",
    );
    let years = format!("{} to {}", FORWARD.start(), FORWARD.end());
    for reading in READINGS
        .iter()
        .filter(|reading| !reading.shipped && reading.expect == Expect::Agrees)
    {
        let (misses, measured) = forward(reading)?;
        if misses.is_empty() {
            found.problems.push(format!(
                "{} names every season change of the forward years too, and §6 says a rule the records cannot separate misses them",
                reading.name
            ));
        }
        found.claims.push(
            Claim::counted(
                format!(
                    "{}: the season changes of Bikram Sambat {years}",
                    reading.name
                ),
                misses.len(),
                measured,
            )
            .with_note("a rival the records cannot separate"),
        );
        let _ = writeln!(
            out,
            "| {} | {years} | {} |",
            reading.name,
            listed(&misses, measured)
        );
    }
    Ok(out)
}

fn page() -> Result<String, String> {
    let records = records()?;
    let (seasons, shipped) = readings(&records)?;
    let mut found = Findings::default();
    let (misses, modern_rules) = over_records(&records, &seasons, &mut found);
    ayana(&records, &shipped, &mut found);
    let changed = changed(&records, &seasons)?;
    let ahead = ahead(&mut found)?;

    let mut out = String::new();
    out.push_str("# The season, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask ritu`. Do not edit: `check-ritu`\n\
         regenerates this page and fails on any difference.\n\n\
         It measures the almanac's `sun.ritu` and `sun.ayana` at Kathmandu\n\
         against Nepal's daily panchanga. The Surya Siddhanta (XIV.10) counts\n\
         six seasons from the Sun's entry into Capricorn, two signs each\n\
         (C178); which *day* a season begins on is the day its solar month\n\
         begins on, and a sankranti falls at an instant, so the month's first\n\
         day is placed by a rule (C186). `panchanga.ritu` chooses the months\n\
         the season is read from and `panchanga.solar_month_start` the rule.\n",
    );
    out.push_str("\n## 1. The claims\n\n");
    out.push_str(&table(&found.claims));
    let _ = write!(
        out,
        "\n## 2. The records\n\n\
         Makalukhabar's daily \"आजको पञ्चाङ्ग\" posts, {} days from {} to {},\n\
         each read for its Gregorian date, its season and its ayana; the page\n\
         holds them in `xtask/src/ritu.rs`, a month a row. {}.\n",
        records.len(),
        records.first().map_or_else(String::new, |r| iso(r.date)),
        records.last().map_or_else(String::new, |r| iso(r.date)),
        SEASONS
            .iter()
            .map(|ritu| format!(
                "{} {}",
                records.iter().filter(|r| r.ritu == *ritu).count(),
                ritu.key()
            ))
            .collect::<Vec<_>>()
            .join(", ")
    );
    out.push_str(&misses);
    out.push_str(&changed);
    out.push_str(&ahead);
    let _ = write!(
        out,
        "\n## 6. What the records decide\n\n\
         The season follows the civil calendar's month. Of the {rules} rules\n\
         over the modern Sun, {missing} miss a recorded day, and the Sun's\n\
         sign as the day opens misses the months whose sankranti fell after\n\
         dawn; a rule that names every recorded season still misses the\n\
         calendar's season changes in §5, so it agrees with these months by\n\
         coincidence and not by rule. So `nepali-default` reads the shipped\n\
         Bikram Sambat calendar, and the root, which has no civil calendar\n\
         of Nepal's, places the modern sankranti by the punya-kala, the\n\
         Dharmasindhu's rule the calendar itself follows\n\
         (`calendar-bikram-sambat.md` §4, where the modern Sun's months are\n\
         measured against the official table).\n",
        rules = modern_rules.len(),
        missing = modern_rules.iter().filter(|missed| **missed).count(),
    );
    if found.problems.is_empty() {
        Ok(out)
    } else {
        Err(found.problems.join("\n      "))
    }
}
