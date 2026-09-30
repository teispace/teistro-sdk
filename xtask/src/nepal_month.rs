//! The measurement pass over **Nepal's lunar month**
//! (`03-design/calendar-indian-lunisolar.md` §9, cruxes C177 and C183).
//!
//! Two things the patro does that the almanac must do the same way:
//!
//! - **Malmas**, the adhika month. The Nepal Panchanga Nirnayak Vikas
//!   Samiti announces each one's span, and the press prints it in Bikram
//!   Sambat days. This founds the almanac at Kathmandu on Nepal's clock
//!   under three readings and holds the days it marks `ADHIKA` to every
//!   announcement: the shipped `nepali-default` profile over the built-in
//!   ephemeris; the `surya-siddhanta` profile over the text's own sky and
//!   zodiac; and the `nepali-default` profile over the text's sky, a rival
//!   kept to show that the zodiac is the text's or the month moves. One
//!   announcement gives instants rather than days, and those are set
//!   beside each sky's new moons.
//! - **The month's name.** Nepal names a lunar month from full moon to
//!   full moon: Gai Jatra, the day after Shravana's full moon, is *Bhadra*
//!   Krishna Pratipada. Festival days the press dated and designated are
//!   held to the month `nepali-default` leads with, and every adhika day's
//!   purnimanta month to its amanta one, since a purnimanta almanac keeps
//!   an adhika month's name through its dark fortnight.
//!
//! `cargo xtask nepal-month` writes the page; `check-nepal-month`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::Panchanga;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, UtcOffset, jd_of_fixed};
use teistro_calendar::lunisolar::MonthKind;
use teistro_core::catalogue::{Calendar, Masa, Tithi};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

const PAGE: &str = "docs/03-design/nepal-month-measured.md";

/// Kathmandu, where the committee sits, and Nepal's clock.
const LATITUDE: f64 = 27.7172;
const LONGITUDE: f64 = 85.324;
const ALTITUDE: f64 = 1400.0;
const CLOCK_HOURS: f64 = 5.75;

/// A Bikram Sambat (month, day).
type MonthDay = (u8, u8);

/// A Bikram Sambat day and a time on Nepal's clock, (hour, minute).
type Instant = (MonthDay, (u8, u8));

/// What an announcement gives.
#[derive(Clone, Copy)]
enum Span {
    /// The first and the last day of the month.
    Days(MonthDay, MonthDay),
    /// The instants it opens and closes; its days are those whose sunrise
    /// falls between them.
    Instants(Instant, Instant),
}

/// One adhika month as announced.
struct Announced {
    /// The Bikram Sambat year.
    year: i32,
    span: Span,
    /// Where the announcement is printed.
    source: &'static str,
}

/// The announcements, as the press printed the committee's words.
const RECORD: [Announced; 5] = [
    Announced {
        year: 2072,
        span: Span::Instants(((3, 1), (23, 23)), ((3, 31), (6, 22))),
        source: "ekantipur, 16 June 2015, quoting the committee's chair",
    },
    Announced {
        year: 2075,
        span: Span::Days((2, 2), (2, 30)),
        source: "a1samachar, 18 May 2018, and the year's patros",
    },
    Announced {
        year: 2077,
        span: Span::Days((6, 2), (6, 30)),
        source: "Nagarik and Ratopati, 18 September 2020, quoting the committee",
    },
    Announced {
        year: 2080,
        span: Span::Days((4, 2), (4, 31)),
        source: "Gorkhapatra and Ratopati, July and August 2023, quoting the committee",
    },
    Announced {
        year: 2083,
        span: Span::Days((2, 3), (3, 1)),
        source: "Onlinekhabar and Ratopati, May and June 2026, quoting the committee",
    },
];

/// A festival day as the press dated it, and the lunar month its
/// designation names.
struct Designated {
    /// The day, in the calendar the source dates it by.
    date: (Calendar, i32, u8, u8),
    /// The festival and its designation, as the source spells it.
    what: &'static str,
    month: Masa,
    source: &'static str,
}

/// Designated days, each in a month the conventions name apart unless
/// said otherwise.
const DESIGNATED: [Designated; 6] = [
    Designated {
        date: (Calendar::BikramSambat, 2072, 2, 26),
        what: "a day's own designation, शुद्ध आषाढ कृष्ण सप्तमी; the solar month is Jestha",
        month: Masa::Ashadha,
        source: "Nepali Wikipedia, अधिकमास, its worked example",
    },
    Designated {
        date: (Calendar::Gregorian, 2023, 9, 14),
        what: "Kushe Aunsi, भाद्र कृष्ण औंसी",
        month: Masa::Bhadrapada,
        source: "Purbasandesh, 14 September 2023",
    },
    Designated {
        date: (Calendar::Gregorian, 2026, 2, 15),
        what: "Mahashivaratri, फाल्गुन कृष्ण चतुर्दशी, the committee's rule",
        month: Masa::Phalguna,
        source: "Nepal Press, 15 February 2026",
    },
    Designated {
        date: (Calendar::Gregorian, 2026, 8, 28),
        what: "Janai Purnima, श्रावण शुक्ल पूर्णिमा, where the conventions agree",
        month: Masa::Shravana,
        source: "the day before Gai Jatra",
    },
    Designated {
        date: (Calendar::Gregorian, 2026, 8, 29),
        what: "Gai Jatra, भाद्र कृष्ण प्रतिपदा",
        month: Masa::Bhadrapada,
        source: "Prasashan, 29 August 2026",
    },
    Designated {
        date: (Calendar::Gregorian, 2026, 9, 11),
        what: "Kushe Aunsi, भाद्र कृष्ण औंसी",
        month: Masa::Bhadrapada,
        source: "Arthikpati, 11 September 2026",
    },
];

/// The days founded either side of an announcement, wide enough to catch
/// a reading that puts the adhika month one month away.
const MARGIN_DAYS: i64 = 40;

/// One way of founding the almanac.
struct Reading {
    name: &'static str,
    profile: &'static str,
    ephemeris: fn() -> Ephemeris,
    /// Whether the page expects it to hold; the rival is kept to fail.
    shipped: bool,
}

const READINGS: [Reading; 3] = [
    Reading {
        name: "`nepali-default` over the built-in ephemeris",
        profile: "nepali-default",
        ephemeris: || Ephemeris::Builtin,
        shipped: true,
    },
    Reading {
        name: "`surya-siddhanta` over the text",
        profile: "surya-siddhanta",
        ephemeris: || Ephemeris::SuryaSiddhanta,
        shipped: true,
    },
    Reading {
        name: "`nepali-default` over the text's sky",
        profile: "nepali-default",
        ephemeris: || Ephemeris::SuryaSiddhanta,
        shipped: false,
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
        Ok(text) => {
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask nepal-month") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// What one reading found for one announcement.
struct Found {
    /// The adhika days, in order.
    days: Vec<CalendarDate>,
    /// The new moons that open and close the month found, as UTC Julian
    /// days.
    new_moons: Option<(f64, f64)>,
    /// Adhika days whose purnimanta month is not their amanta one.
    renamed: usize,
}

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(LATITUDE),
        Longitude::literal(LONGITUDE),
        Altitude::literal(ALTITUDE),
    )
}

/// The month each designated day's almanac names under the shipped
/// profile, in `DESIGNATED`'s order: the one it leads with, and its amanta
/// month.
fn designated() -> Result<Vec<(Masa, Masa)>, String> {
    let context = Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|e| e.to_string())?;
    DESIGNATED
        .iter()
        .map(|designated| {
            let (calendar, year, month, day) = designated.date;
            let date = CalendarDate::defined(calendar, year, month, day);
            let days = context
                .almanac()
                .of(&date, &date, &kathmandu(), UtcOffset::literal(5, 45, 0))
                .map_err(|e| format!("{}: {e}", designated.what))?;
            days.value
                .first()
                .map(|found| (found.month.month, found.month.amanta))
                .ok_or_else(|| format!("{}: no day", designated.what))
        })
        .collect()
}

fn bs(year: i32, (month, day): MonthDay) -> CalendarDate {
    CalendarDate::defined(Calendar::BikramSambat, year, month, day)
}

/// An instant on Nepal's clock, as a UTC Julian day.
fn jd_of(context: &Context, year: i32, (day, (hour, minute)): Instant) -> Result<f64, String> {
    let fixed = context
        .calendar()
        .fixed_of(&bs(year, day))
        .map_err(|e| format!("{year}: {e}"))?;
    let minutes = f64::from(hour) * 60.0 + f64::from(minute);
    Ok(jd_of_fixed(fixed) + minutes / 1440.0 - CLOCK_HOURS / 24.0)
}

/// Where the day's Amavasya ends, when it ends inside the day.
fn new_moon_in(day: &Panchanga) -> Option<f64> {
    day.limbs
        .tithi
        .iter()
        .filter(|span| span.member == Tithi::Amavasya)
        .map(|span| span.whole.to.get())
        .find(|&end| end >= day.day.sunrise.get() && end < day.day.next_sunrise.get())
}

/// The announcement's own days, the instants read by the reading's
/// sunrises.
fn announced_days(
    context: &Context,
    announced: &Announced,
    days: &[Panchanga],
) -> Result<Vec<CalendarDate>, String> {
    let year = announced.year;
    match announced.span {
        Span::Days(from, to) => {
            let calendar = context.calendar();
            let first = calendar
                .fixed_of(&bs(year, from))
                .map_err(|e| e.to_string())?;
            let last = calendar
                .fixed_of(&bs(year, to))
                .map_err(|e| e.to_string())?;
            (0..=first.days_until(last))
                .map(|k| {
                    calendar
                        .date_of(Calendar::BikramSambat, first.plus_days(k))
                        .map_err(|e| e.to_string())
                })
                .collect()
        }
        Span::Instants(from, to) => {
            let (from, to) = (jd_of(context, year, from)?, jd_of(context, year, to)?);
            Ok(days
                .iter()
                .filter(|day| (from..to).contains(&day.day.sunrise.get()))
                .map(|day| day.day.date.clone())
                .collect())
        }
    }
}

/// The span founded for an announcement: its first and last days, or
/// the instants' own days, widened by the margin.
fn founded(context: &Context, announced: &Announced) -> Result<Vec<Panchanga>, String> {
    let year = announced.year;
    let (from, to) = match announced.span {
        Span::Days(from, to) | Span::Instants((from, _), (to, _)) => (from, to),
    };
    let calendar = context.calendar();
    let edge = |day: MonthDay, by: i64| -> Result<CalendarDate, String> {
        let fixed = calendar
            .fixed_of(&bs(year, day))
            .map_err(|e| e.to_string())?;
        calendar
            .date_of(Calendar::BikramSambat, fixed.plus_days(by))
            .map_err(|e| e.to_string())
    };
    context
        .almanac()
        .of(
            &edge(from, -MARGIN_DAYS)?,
            &edge(to, MARGIN_DAYS)?,
            &kathmandu(),
            UtcOffset::literal(5, 45, 0),
        )
        .map(|days| days.value)
        .map_err(|e| format!("{year}: {e}"))
}

fn found(days: &[Panchanga]) -> Found {
    let adhika: Vec<usize> = (0..days.len())
        .filter(|&k| {
            days.get(k)
                .is_some_and(|day| day.month.kind == MonthKind::Adhika)
        })
        .collect();
    let new_moons = match (adhika.first(), adhika.last()) {
        (Some(&first), Some(&last)) => first
            .checked_sub(1)
            .and_then(|before| days.get(before))
            .and_then(new_moon_in)
            .zip(days.get(last).and_then(new_moon_in)),
        _ => None,
    };
    let at = |k: &usize| days.get(*k);
    Found {
        days: adhika
            .iter()
            .filter_map(at)
            .map(|day| day.day.date.clone())
            .collect(),
        new_moons,
        renamed: adhika
            .iter()
            .filter_map(at)
            .filter(|day| day.month.purnimanta != day.month.amanta)
            .count(),
    }
}

/// One reading's answer for every announcement: what it found, and the
/// announcement's days as its sunrises read them.
type Answers = Vec<Answer>;

/// What a reading found for an announcement, and the announcement's days.
type Answer = (Found, Vec<CalendarDate>);

/// One reading over one announcement.
fn answer(reading: &Reading, announced: &Announced) -> Result<Answer, String> {
    let context = Context::builder()
        .profile(reading.profile)
        .ephemeris([(reading.ephemeris)()])
        .build()
        .map_err(|e| format!("{}: {e}", reading.profile))?;
    let days = founded(&context, announced)?;
    let wanted = announced_days(&context, announced, &days)?;
    Ok((found(&days), wanted))
}

/// Each reading over every announcement, a task a pair, the tasks dealt
/// round the machine's cores: a reading's five months cost unevenly, so a
/// worker a reading waits on the slowest. The answers do not depend on
/// which worker found them.
fn measure() -> Result<Vec<Answers>, String> {
    let tasks: Vec<(usize, usize)> = (0..READINGS.len())
        .flat_map(|reading| (0..RECORD.len()).map(move |announced| (reading, announced)))
        .collect();
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let found: Vec<((usize, usize), Answer)> = std::thread::scope(|scope| {
        let spawned: Vec<_> = (0..workers.min(tasks.len()))
            .map(|worker| {
                let mine: Vec<(usize, usize)> = tasks
                    .iter()
                    .copied()
                    .skip(worker)
                    .step_by(workers)
                    .collect();
                scope.spawn(move || {
                    mine.into_iter()
                        .map(|(reading, announced)| {
                            let (Some(r), Some(a)) = (READINGS.get(reading), RECORD.get(announced))
                            else {
                                return Err(String::from("a task past the tables"));
                            };
                            Ok(((reading, announced), answer(r, a)?))
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut all = Vec::new();
        for worker in spawned {
            all.extend(
                worker
                    .join()
                    .map_err(|_| String::from("a worker founding the almanac panicked"))??,
            );
        }
        Ok::<_, String>(all)
    })?;
    let mut answers: Vec<Answers> = READINGS.iter().map(|_| Vec::new()).collect();
    let mut sorted = found;
    sorted.sort_by_key(|(at, _)| *at);
    for ((reading, _), pair) in sorted {
        if let Some(list) = answers.get_mut(reading) {
            list.push(pair);
        }
    }
    Ok(answers)
}

/// A day as the page spells it.
fn spell(date: &CalendarDate) -> String {
    format!("{} {}/{:02}/{:02}", "BS", date.year, date.month, date.day)
}

/// A found span as the page spells it.
fn spell_span(days: &[CalendarDate]) -> String {
    match (days.first(), days.last()) {
        (Some(first), Some(last)) => format!("{} to {}", spell(first), spell(last)),
        _ => String::from("none"),
    }
}

/// Minutes from one instant to another, signed.
fn minutes(from: f64, to: f64) -> String {
    format!("{:+.0}", (to - from) * 1440.0)
}

/// The header row of a table with a column a reading.
fn header(out: &mut String, first: &str, columns: usize) {
    out.push_str(first);
    for reading in &READINGS {
        let _ = write!(out, " {} |", reading.name);
    }
    out.push('\n');
    out.push_str(&"|---".repeat(columns));
    out.push('|');
    out.push_str(&"---|".repeat(READINGS.len()));
    out.push('\n');
}

/// §1: a claim a reading, and the purnimanta name, with a problem for
/// every verdict the page does not expect.
fn claims(
    out: &mut String,
    answers: &[Answers],
    names: &[(Masa, Masa)],
    problems: &mut Vec<String>,
) {
    let mut claims = Vec::new();
    for (reading, answers) in READINGS.iter().zip(answers) {
        let wrong = answers
            .iter()
            .filter(|(found, wanted)| found.days != *wanted)
            .count();
        let claim = Claim::counted(
            format!("{}: the adhika days are the committee's", reading.name),
            wrong,
            answers.len(),
        );
        if reading.shipped != (wrong == 0) {
            problems.push(format!(
                "{} {} the committee's months, and the page expects it {}",
                reading.name,
                if wrong == 0 { "reproduces" } else { "misses" },
                if reading.shipped { "to" } else { "not to" }
            ));
        }
        claims.push(if reading.shipped {
            claim
        } else {
            claim.with_note("a rival: the text's Sun read in Lahiri's zodiac")
        });
    }
    let (renamed, adhika_days) = answers
        .iter()
        .flatten()
        .fold((0, 0), |(renamed, days), (found, _)| {
            (renamed + found.renamed, days + found.days.len())
        });
    claims.push(Claim::counted(
        "an adhika day's purnimanta month is its amanta month",
        renamed,
        adhika_days,
    ));
    if renamed != 0 {
        problems.push(format!(
            "{renamed} adhika days name the next month under purnimanta"
        ));
    }
    let misnamed = |pick: fn(&(Masa, Masa)) -> Masa| {
        DESIGNATED
            .iter()
            .zip(names)
            .filter(|(designated, found)| pick(found) != designated.month)
            .count()
    };
    let (led, amanta) = (misnamed(|found| found.0), misnamed(|found| found.1));
    claims.push(Claim::counted(
        "`nepali-default` leads with the month a Nepali designation names",
        led,
        DESIGNATED.len(),
    ));
    claims.push(
        Claim::counted(
            "the amanta month is the one a Nepali designation names",
            amanta,
            DESIGNATED.len(),
        )
        .with_note("a rival: the root profile's lead"),
    );
    if led != 0 || amanta == 0 {
        problems.push(format!(
            "`nepali-default` misnames {led} designated days and the amanta month {amanta}; the page expects none and some"
        ));
    }
    out.push_str("## 1. The claims\n\n");
    out.push_str(&table(&claims));
}

/// §4: each designated day beside the month the shipped profile leads
/// with.
fn designations(out: &mut String, names: &[(Masa, Masa)]) {
    out.push_str(
        "\n## 4. The month a Nepali designation names\n\n\
         Nepal names a lunar month from full moon to full moon, so a dark\n\
         fortnight carries the next amanta month's name. Each day is founded\n\
         at Kathmandu under `nepali-default` over the built-in ephemeris.\n\n\
         | day | designation | source | designated | `nepali-default` leads with | amanta |\n\
         |---|---|---|---|---|---|\n",
    );
    for (designated, (led, amanta)) in DESIGNATED.iter().zip(names) {
        let (calendar, year, month, day) = designated.date;
        let _ = writeln!(
            out,
            "| {} {year}/{month:02}/{day:02} | {} | {} | {} | {} | {} |",
            if calendar == Calendar::BikramSambat {
                "BS"
            } else {
                "AD"
            },
            designated.what,
            designated.source,
            designated.month.key(),
            led.key(),
            amanta.key()
        );
    }
}

/// §2: each announcement beside what each reading found.
fn announcements(out: &mut String, answers: &[Answers]) {
    out.push_str(
        "\n## 2. Each announcement\n\n\
         An announcement that gives instants is read by each reading's own\n\
         sunrises: its days are those whose sunrise falls between them.\n\n",
    );
    header(out, "| year | announced | source |", 3);
    for (k, announced) in RECORD.iter().enumerate() {
        let said = match announced.span {
            Span::Days(from, to) => format!(
                "{} to {}",
                spell(&bs(announced.year, from)),
                spell(&bs(announced.year, to))
            ),
            Span::Instants((from, (fh, fm)), (to, (th, tm))) => format!(
                "{} {fh:02}:{fm:02} to {} {th:02}:{tm:02}",
                spell(&bs(announced.year, from)),
                spell(&bs(announced.year, to))
            ),
        };
        let _ = write!(
            out,
            "| {} | {said} | {} |",
            announced.year, announced.source
        );
        for answers in answers {
            let cell = answers.get(k).map_or_else(String::new, |(found, wanted)| {
                let span = spell_span(&found.days);
                if found.days == *wanted {
                    format!("{span}, as announced")
                } else {
                    format!("{span}, not {}", spell_span(wanted))
                }
            });
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
}

/// §3: an announcement's instants beside each reading's new moons.
fn instants(out: &mut String, answers: &[Answers], context: &Context) -> Result<(), String> {
    out.push_str(
        "\n## 3. The announced instants against each sky's new moons\n\n\
         Minutes from the announced instant to the new moon each reading\n\
         found, positive when the new moon is later; a reading that found\n\
         another month has no new moon to set beside it.\n\n",
    );
    header(out, "| year | boundary | announced (UTC JD) |", 3);
    for (k, announced) in RECORD.iter().enumerate() {
        let Span::Instants(from, to) = announced.span else {
            continue;
        };
        let bounds = [
            ("opens", jd_of(context, announced.year, from)?),
            ("closes", jd_of(context, announced.year, to)?),
        ];
        for (which, (name, at)) in bounds.iter().enumerate() {
            let _ = write!(out, "| {} | {name} | {at:.5} |", announced.year);
            for answers in answers {
                let cell = match answers.get(k) {
                    Some((found, wanted)) if found.days != *wanted => String::from("another month"),
                    Some((found, _)) => found.new_moons.map_or_else(
                        || String::from("none"),
                        |(opens, closes)| minutes(*at, if which == 0 { opens } else { closes }),
                    ),
                    None => String::from("none"),
                };
                let _ = write!(out, " {cell} |");
            }
            out.push('\n');
        }
    }
    Ok(())
}

fn page() -> Result<String, String> {
    let answers = measure()?;
    let context = Context::builder()
        .ephemeris([Ephemeris::None])
        .build()
        .map_err(|e| e.to_string())?;

    let mut out = String::new();
    let names = designated()?;
    out.push_str("# Nepal's lunar month, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask nepal-month`. Do not edit:\n\
         `check-nepal-month` regenerates this page and fails on any difference.\n\n\
         It measures `calendar-indian-lunisolar.md` §9 at Kathmandu on Nepal's\n\
         clock: the days the almanac marks `ADHIKA` against the adhika months\n\
         the Nepal Panchanga Nirnayak Vikas Samiti announced, which Nepal\n\
         calls *malmas* (C177), and the month it names against the month a\n\
         Nepali designation names (C183).\n\n",
    );
    let mut problems = Vec::new();
    claims(&mut out, &answers, &names, &mut problems);
    announcements(&mut out, &answers);
    instants(&mut out, &answers, &context)?;
    designations(&mut out, &names);
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems.join("\n      "))
    }
}
