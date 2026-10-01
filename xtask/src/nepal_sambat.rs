//! The measurement pass over **Nepal Sambat**
//! (`03-design/calendar-indian-lunisolar.md` §11).
//!
//! Two things are held here. The committee's national panchanga prints
//! the Nepal Sambat year, month and half in every page header, one page a
//! half-month; the 51 headers of VS 2082 and 2083, read off the page
//! images, are held to the reading of each page's first day, said through
//! the `ne-Deva-NP` messages a consumer would say it with, over the
//! committee's sky and the modern one. And the year's count, an estimate
//! of its middle, is held to the lunar years the almanac itself finds, on
//! the days around each year's turn across six centuries, with how near a
//! civil new year the estimate came.
//!
//! `cargo xtask nepal-sambat` writes the page; `check-nepal-sambat`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::messages::sdk::calendar::{NepalSambatHalf, NepalSambatMonth};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, LunarYear, NepalSambatDate, Panchanga, UtcOffset};
use teistro_calendar::FixedDay;
use teistro_calendar::gregorian::{fixed_from_gregorian, year_from_fixed};
use teistro_core::catalogue::{Calendar, Tithi};
use teistro_panchanga::nepal_sambat::middle_of;

use crate::generated::{Output, check, write};
use crate::measure::{Claim, count, table};

const PAGE: &str = "docs/03-design/nepal-sambat-measured.md";

/// A Gregorian day.
type Day = (i32, u8, u8);

/// One page header as the committee printed it.
struct Header {
    /// The page, as `VS-page`.
    page: &'static str,
    /// The page's first civil day.
    first: Day,
    /// The Nepal Sambat year printed.
    year: i32,
    /// The month and the half, as printed together: `कछलाथ्व`.
    printed: &'static str,
}

const fn header(page: &'static str, first: Day, year: i32, printed: &'static str) -> Header {
    Header {
        page,
        first,
        year,
        printed,
    }
}

/// The headers of the committee's national panchanga for VS 2082 and
/// 2083, page 2 onwards, each "ने.सं. <year> (<month><half>)", with the
/// date of the page's first row. A visarga some headers add after थ्व is
/// left out.
const HEADERS: [Header; 51] = [
    header("2082-02", (2025, 4, 14), 1145, "चौलागा"),
    header("2082-03", (2025, 4, 28), 1145, "वछलाथ्व"),
    header("2082-04", (2025, 5, 13), 1145, "वछलागा"),
    header("2082-05", (2025, 5, 28), 1145, "तछलाथ्व"),
    header("2082-06", (2025, 6, 12), 1145, "तछलागा"),
    header("2082-07", (2025, 6, 26), 1145, "दिल्लाथ्व"),
    header("2082-08", (2025, 7, 11), 1145, "दिल्लागा"),
    header("2082-09", (2025, 7, 25), 1145, "गुंलाथ्व"),
    header("2082-10", (2025, 8, 10), 1145, "गुंलागा"),
    header("2082-11", (2025, 8, 24), 1145, "ञँलाथ्व"),
    header("2082-12", (2025, 9, 8), 1145, "ञँलागा"),
    header("2082-13", (2025, 9, 22), 1145, "कौलाथ्व"),
    header("2082-14", (2025, 10, 8), 1145, "कौलागा"),
    header("2082-15", (2025, 10, 22), 1146, "कछलाथ्व"),
    header("2082-16", (2025, 11, 6), 1146, "कछलागा"),
    header("2082-17", (2025, 11, 21), 1146, "थिंल्लाथ्व"),
    header("2082-18", (2025, 12, 5), 1146, "थिंल्लागा"),
    header("2082-19", (2025, 12, 20), 1146, "पोहेलाथ्व"),
    header("2082-20", (2026, 1, 4), 1146, "पोहेलागा"),
    header("2082-21", (2026, 1, 19), 1146, "सिल्लाथ्व"),
    header("2082-22", (2026, 2, 2), 1146, "सिल्लागा"),
    header("2082-23", (2026, 2, 18), 1146, "चिल्लाथ्व"),
    header("2082-24", (2026, 3, 4), 1146, "चिल्लागा"),
    header("2082-25", (2026, 3, 20), 1146, "चौलाथ्व"),
    header("2082-26", (2026, 4, 3), 1146, "चौलागा"),
    header("2083-02", (2026, 4, 14), 1146, "चौलागा"),
    header("2083-03", (2026, 4, 18), 1146, "वछलाथ्व"),
    header("2083-04", (2026, 5, 2), 1146, "वछलागा"),
    header("2083-05", (2026, 5, 17), 1146, "अनलाथ्व"),
    header("2083-06", (2026, 6, 1), 1146, "अनलागा"),
    header("2083-07", (2026, 6, 16), 1146, "तछलाथ्व"),
    header("2083-08", (2026, 6, 30), 1146, "तछलागा"),
    header("2083-09", (2026, 7, 15), 1146, "दिल्लाथ्व"),
    header("2083-10", (2026, 7, 30), 1146, "दिल्लागा"),
    header("2083-11", (2026, 8, 13), 1146, "गुंलाथ्व"),
    header("2083-12", (2026, 8, 29), 1146, "गुंलागा"),
    header("2083-13", (2026, 9, 12), 1146, "ञँलाथ्व"),
    header("2083-14", (2026, 9, 27), 1146, "ञँलागा"),
    header("2083-15", (2026, 10, 11), 1146, "कौलाथ्व"),
    header("2083-16", (2026, 10, 27), 1146, "कौलागा"),
    header("2083-17", (2026, 11, 10), 1147, "कछलाथ्व"),
    header("2083-18", (2026, 11, 25), 1147, "कछलागा"),
    header("2083-19", (2026, 12, 9), 1147, "थिंल्लाथ्व"),
    header("2083-20", (2026, 12, 25), 1147, "थिंल्लागा"),
    header("2083-21", (2027, 1, 8), 1147, "पोहेलाथ्व"),
    header("2083-22", (2027, 1, 23), 1147, "पोहेलागा"),
    header("2083-23", (2027, 2, 7), 1147, "सिल्लाथ्व"),
    header("2083-24", (2027, 2, 21), 1147, "सिल्लागा"),
    header("2083-25", (2027, 3, 9), 1147, "चिल्लाथ्व"),
    header("2083-26", (2027, 3, 23), 1147, "चिल्लागा"),
    header("2083-27", (2027, 4, 7), 1147, "चौलाथ्व"),
];

/// The civil years whose turn of the Nepal Sambat year is founded, every
/// `SAMPLE_STEP`th from `SAMPLE_FROM` to `SAMPLE_TO`: six centuries
/// around the print, which is as far as the built-in sky reaches
/// (1800 to 2400), and it gives the committee's profile its sunrise (C39).
const SAMPLE_FROM: i32 = 1810;
const SAMPLE_TO: i32 = 2390;
const SAMPLE_STEP: usize = 20;

/// The days founded around each turn, (month, day) to (month, day): from
/// before the earliest Kartika new moon to after the latest, so Kaula's
/// last days and Kachhala's first are both among them.
const TURN: ((u8, u8), (u8, u8)) = ((10, 1), (12, 5));

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

fn gregorian((year, month, day): Day) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, year, month, day)
}

/// A sky the days are founded under.
#[derive(Clone, Copy)]
enum Sky {
    /// `nepali-committee`, the committee's own.
    Committee,
    /// `nepali-default`, the modern sky under Lahiri.
    Modern,
}

impl Sky {
    fn context(self) -> Result<Context, String> {
        let (profile, ephemeris) = match self {
            Sky::Committee => ("nepali-committee", Ephemeris::SuryaSiddhanta),
            Sky::Modern => ("nepali-default", Ephemeris::Builtin),
        };
        Context::builder()
            .profile(profile)
            .locale("ne-Deva-NP")
            .ephemeris([ephemeris])
            .build()
            .map_err(|e| e.to_string())
    }
}

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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask nepal-sambat") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// A date as the committee's header prints it, through the messages a
/// consumer says it with: the year, and the month and the half together.
fn said(context: &Context, date: NepalSambatDate) -> Result<String, String> {
    let intl = context.intl();
    let month = intl.render_typed(&NepalSambatMonth {
        month: i64::from(date.month),
        kind: date.kind.key().to_owned(),
    });
    let half = intl.render_typed(&NepalSambatHalf {
        paksha: date.paksha.key().to_owned(),
    });
    if let Some(warning) = month.warnings.iter().chain(&half.warnings).next() {
        return Err(format!("{date:?} is not said cleanly: {warning:?}"));
    }
    Ok(format!("{}{}", month.text, half.text))
}

/// The page's first day under a sky: its year, its month and half said,
/// and the tithi its sunrise falls in.
fn read(context: &Context, header: &Header) -> Result<Read, String> {
    let (place, clock) = kathmandu();
    let day = gregorian(header.first);
    let days = context
        .almanac()
        .of(&day, &day, &place, clock)
        .map_err(|e| format!("{}: {e}", header.page))?;
    let day = days
        .value
        .first()
        .ok_or_else(|| format!("{}: no day", header.page))?;
    let tithi = day
        .limbs
        .tithi
        .first()
        .ok_or_else(|| format!("{}: no tithi", header.page))?
        .member;
    let date = day.nepal_sambat();
    Ok(Read {
        year: date.year,
        said: said(context, date)?,
        tithi,
    })
}

/// What one sky reads for a page's first day.
struct Read {
    year: i32,
    /// The month and the half, said together.
    said: String,
    /// The tithi at the day's sunrise.
    tithi: Tithi,
}

/// One header and what each sky reads.
struct Row<'a> {
    header: &'a Header,
    committee: Read,
    modern: Read,
}

impl Row<'_> {
    fn agrees(&self, read: &Read) -> bool {
        read.year == self.header.year && read.said == self.header.printed
    }
}

/// What the count of the year does across the turns founded.
struct Turns {
    /// Days founded.
    days: usize,
    /// Days whose year the count gives otherwise than the lunar year
    /// holding the day, each as (date, counted, the lunar year's).
    wrong: Vec<(String, i32, i32)>,
    /// How near a civil new year the estimate of the middle came, days,
    /// and on which day.
    nearest: (i64, String),
}

/// The Nepal Sambat year the lunar year holding a day gives: its Vikrama
/// number less 937 until Kartika, and less 936 from Kachhala on.
fn by_the_lunar_year(day: &Panchanga, years: &[LunarYear]) -> Option<i32> {
    let sunrise = day.day.sunrise.get();
    let year = years
        .iter()
        .find(|year| (year.began.get()..year.ended.get()).contains(&sunrise))?;
    let from_kachhala = teistro_panchanga::nepal_sambat::month_of(day.month.amanta) <= 5;
    Some(year.vikrama - 937 + i32::from(from_kachhala))
}

/// Days from a fixed day to the nearer civil new year.
fn from_new_year(day: FixedDay) -> i64 {
    let year = year_from_fixed(day);
    let opened = fixed_from_gregorian(year, 1, 1);
    let next = fixed_from_gregorian(year + 1, 1, 1);
    opened.days_until(day).min(day.days_until(next))
}

/// The turns of the year across the sample, a worker per civil year,
/// each with its own context.
fn turns() -> Result<Turns, String> {
    let (place, clock) = kathmandu();
    let ((from_month, from_day), (to_month, to_day)) = TURN;
    let sampled: Vec<i32> = (SAMPLE_FROM..=SAMPLE_TO).step_by(SAMPLE_STEP).collect();
    let per_year = std::thread::scope(|scope| {
        let workers: Vec<_> = sampled
            .iter()
            .map(|&year| {
                let place = &place;
                scope.spawn(move || {
                    let context = Sky::Committee.context()?;
                    let (from, to) = (
                        gregorian((year, from_month, from_day)),
                        gregorian((year, to_month, to_day)),
                    );
                    let days = context
                        .almanac()
                        .of(&from, &to, place, clock)
                        .map_err(|e| format!("{year}: {e}"))?;
                    let years = context
                        .almanac()
                        .years(&from, &to, place, clock)
                        .map_err(|e| format!("{year}: {e}"))?;
                    Ok::<_, String>((days.value, years.value))
                })
            })
            .collect();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| String::from("a worker panicked"))?
            })
            .collect::<Result<Vec<_>, String>>()
    })?;
    let mut turns = Turns {
        days: 0,
        wrong: Vec::new(),
        nearest: (i64::MAX, String::new()),
    };
    for (days, years) in &per_year {
        for day in days {
            let at = &day.day.date;
            let date = format!("{}-{:02}-{:02}", at.year, at.month, at.day);
            let counted = day.nepal_sambat().year;
            let reference = by_the_lunar_year(day, years)
                .ok_or_else(|| format!("{date}: no lunar year holds its sunrise"))?;
            if counted != reference {
                turns.wrong.push((date.clone(), counted, reference));
            }
            let margin = from_new_year(middle_of(day.month.amanta, day.day.sunrise));
            if margin < turns.nearest.0 {
                turns.nearest = (margin, date);
            }
            turns.days += 1;
        }
    }
    Ok(turns)
}

fn page() -> Result<String, String> {
    let (committee, modern) = (Sky::Committee.context()?, Sky::Modern.context()?);
    let rows = HEADERS
        .iter()
        .map(|header| {
            Ok(Row {
                header,
                committee: read(&committee, header)?,
                modern: read(&modern, header)?,
            })
        })
        .collect::<Result<Vec<Row<'_>>, String>>()?;
    let mut problems = Vec::new();
    for row in &rows {
        if !row.agrees(&row.committee) {
            problems.push(format!(
                "{} printed {} {}, read {} {} over the committee's sky: reopen §11",
                row.header.page,
                row.header.year,
                row.header.printed,
                row.committee.year,
                row.committee.said
            ));
        }
        // A modern miss is the sky's, not the reading's: the two skies
        // must part on the tithi at the day's sunrise.
        if !row.agrees(&row.modern) && row.modern.tithi == row.committee.tithi {
            problems.push(format!(
                "{} read {} {} over the modern sky with the committee's sunrise tithi: the reading, not the sky, parts",
                row.header.page, row.modern.year, row.modern.said
            ));
        }
    }
    let turns = turns()?;
    for (date, counted, reference) in &turns.wrong {
        problems.push(format!(
            "{date}: the count gives {counted} and the lunar year holding the day {reference}"
        ));
    }
    if !problems.is_empty() {
        return Err(problems.join("\n      "));
    }
    Ok(render(&rows, &turns))
}

fn render(rows: &[Row<'_>], turns: &Turns) -> String {
    let mut out = String::from(
        "# Nepal Sambat, measured\n\n\
         Status: `generated` by `cargo xtask nepal-sambat`. Do not edit:\n\
         `check-nepal-sambat` regenerates this page and fails on any difference.\n\n\
         It measures `calendar-indian-lunisolar.md` §11.\n\n\
         ## 1. Against the committee's page headers\n\n\
         The Nepal Panchanga Decision Committee's national panchanga for VS 2082\n\
         and 2083 gives each half-month a page, and each page's header prints the\n\
         Nepal Sambat year, month and half: \"ने.सं. ११४६ (कछलाथ्व)\". Every\n\
         header from page 2 on, read off the page images, beside the reading of\n\
         the page's first day at Kathmandu on Nepal's clock, said through the\n\
         `ne-Deva-NP` messages `sdk.calendar.nepalSambatMonth` and\n\
         `nepalSambatHalf`: over the committee's own sky (`nepali-committee`, the\n\
         Surya Siddhanta with its bija, C187), which the pass holds to every\n\
         header, and over the modern sky (`nepali-default`).\n\n\
         | page | first day | printed | committee's sky | modern sky |\n\
         |---|---|---|---|---|\n",
    );
    let shown = |read: &Read| format!("{} {}", read.year, read.said);
    for row in rows {
        let (year, month, day) = row.header.first;
        let _ = writeln!(
            out,
            "| {} | {year}-{month:02}-{day:02} | {} {} | {} | {} |",
            row.header.page,
            row.header.year,
            row.header.printed,
            shown(&row.committee),
            shown(&row.modern),
        );
    }
    let missed = |committee: bool| {
        rows.iter()
            .filter(|row| {
                !row.agrees(if committee {
                    &row.committee
                } else {
                    &row.modern
                })
            })
            .count()
    };
    out.push('\n');
    out.push_str(&table(&[
        Claim::counted(
            "the committee's sky reads the printed year, month and half",
            missed(true),
            rows.len(),
        )
        .with_note("the pass refuses any miss"),
        Claim::counted(
            "the modern sky reads the printed year, month and half",
            missed(false),
            rows.len(),
        )
        .with_note("each miss is a page's first day whose sunrise the modern sky puts in the tithi before, its new or full moon falling just after sunrise; the pass refuses a miss on a day the skies' sunrise tithis agree"),
    ]));
    let _ = write!(
        out,
        "\n## 2. The year's count\n\n\
         The year is the civil year an estimate of its middle falls in, less 880:\n\
         the day's sunrise moved by the months between it and the year's middle.\n\
         Held here to the lunar years the almanac itself finds\n\
         (`almanac().years`), whose Vikrama number less 937, or less 936 from\n\
         Kachhala on, is the Nepal Sambat year: on every day from {} to {}\n\
         of every {}th civil year from {SAMPLE_FROM} to {SAMPLE_TO}, over the committee's\n\
         sky, which holds each year's last days of Kaula and first of Kachhala,\n\
         where the estimate stands furthest from the day.\n\n",
        spell(TURN.0),
        spell(TURN.1),
        SAMPLE_STEP,
    );
    out.push_str(&table(&[Claim::counted(
        "the count gives the lunar year's Nepal Sambat year",
        turns.wrong.len(),
        turns.days,
    )
    .with_note("the pass refuses any difference")]));
    let _ = write!(
        out,
        "\nThe estimate of the middle came within {} days of a civil new year at\n\
         its nearest, on {}: the margin a sidereal month's drift through the\n\
         civil year, a day in seventy years, has before the count is wrong.\n",
        count(usize::try_from(turns.nearest.0).unwrap_or(0)),
        turns.nearest.1,
    );
    out
}

/// A (month, day) as the page spells it.
fn spell((month, day): (u8, u8)) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    format!("{day} {}", MONTHS[usize::from(month - 1)])
}
