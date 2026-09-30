//! The measurement pass over **the sixty-year cycle** (crux C180 and
//! C184, `03-design/samvatsara-measured.md`).
//!
//! The Surya Siddhanta counts the signs its mean Jupiter has crossed since
//! the Kali age and reads the count from Vijaya (I.55); Nepal's panchanga
//! committee names a lunar year by the Jovian year that meets Chaitra
//! Shukla Pratipada. This pass holds the almanac's years to:
//!
//! - **the text's worked example**, Burgess's note to I.55: the Jovian
//!   year begun in February 1859 by the text, April with the bija;
//! - **the years Nepal named**, as the press printed the committee, under
//!   the shipped count and its rivals, each rival held to fail;
//! - **the Jovian years the press dated**;
//! - **six centuries of years** at Kathmandu, where the shipped count's
//!   short walk back is held to the full recursion it stands for, and the
//!   years the two Barhaspatya readings name apart are listed.
//!
//! `cargo xtask samvatsara` writes the page; `check-samvatsara`
//! regenerates it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Calendar, Samvatsara};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::settings::SamvatsaraCount;
use teistro::{CalendarDate, Context, Ephemeris, LunarYear, UtcOffset};
use teistro_calendar::samvatsara::{Bija, JovianYear, Parameters};
use teistro_panchanga::year::{MOST_ADVANCED_YEARS, name_counts};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

const PAGE: &str = "docs/03-design/samvatsara-measured.md";

/// The years the six-century section founds, Vikrama: the built-in
/// ephemeris covers 1800 to 2400, and the naming reads a few years back.
const CENTURIES: core::ops::RangeInclusive<i32> = 1865..=2455;

/// Burgess's bija for Jupiter: its year "about 12 minutes longer", eight
/// revolutions fewer an age.
const BIJA: Bija = Bija {
    moon: 0,
    moon_apsis: 0,
    moon_node: 0,
    mars: 0,
    mercury: 0,
    jupiter: -8,
    venus: 0,
    saturn: 0,
};

/// A lunar year as the press named it.
struct Named {
    vikrama: i32,
    name: Samvatsara,
    source: &'static str,
}

/// The years Nepal named, as the press printed the committee's words.
const NAMED: [Named; 7] = [
    Named {
        vikrama: 2076,
        name: Samvatsara::Paridhaavi,
        source: "Onlinekhabar, 25 March 2020: परिधावी ran until the new year",
    },
    Named {
        vikrama: 2077,
        name: Samvatsara::Pramadicha,
        source: "Onlinekhabar, 25 March 2020: प्रमादी from that day",
    },
    Named {
        vikrama: 2078,
        name: Samvatsara::Rakshasa,
        source: "Lokpath, 13 April 2021, the committee's chair: आनन्द did not meet the pratipada",
    },
    Named {
        vikrama: 2079,
        name: Samvatsara::Nala,
        source: "Ratopati, 14 April 2022, the committee's chair: नल from Chaitra 19",
    },
    Named {
        vikrama: 2080,
        name: Samvatsara::Pingala,
        source: "Makalukhabar, January 2023, the committee on the year's patros",
    },
    Named {
        vikrama: 2081,
        name: Samvatsara::Kalayukta,
        source: "Nagarik, April 2024",
    },
    Named {
        vikrama: 2082,
        name: Samvatsara::Siddharthi,
        source: "Nepal Khabar, 14 April 2025: सिद्धार्थी from Chaitra 17",
    },
];

/// A Jovian year's first day as the press dated it, on India's clock.
struct Dated {
    name: Samvatsara,
    date: (i32, u8, u8),
    source: &'static str,
}

const DATED: [Dated; 2] = [
    Dated {
        name: Samvatsara::Ananda,
        date: (2020, 4, 4),
        source: "Patrika (Jaipur), April 2021",
    },
    Dated {
        name: Samvatsara::Rakshasa,
        date: (2021, 4, 1),
        source: "Patrika (Jaipur), April 2021",
    },
];

/// India's clock, hours east, which the dated records are on.
const INDIA_HOURS: f64 = 5.5;

/// How far a dated Jovian year's first day may fall from the computed
/// one: the press prints whole days, and its panchangs round the
/// ingress to one.
const DATED_DAYS: f64 = 1.5;

/// How far the text's worked example may fall from Burgess's dates: he
/// worked from his table's 1850 places, rounded.
const EXAMPLE_DAYS: f64 = 2.0;

/// One way of naming a year from its founded record, for the named
/// years: the shipped count read off the SDK, the rest computed from the
/// same years' first sunrises.
#[derive(Clone, Copy)]
enum Reading {
    /// The SDK's own answer under a count.
    Sdk(SamvatsaraCount),
    /// The text's Jupiter with Burgess's bija, advancing as the shipped
    /// count does.
    Bija,
    /// The Jovian year running at the Bikram Sambat new year, Baishakh 1.
    Baishakh,
    /// The baseline engine's fixed phase, `(vikrama + 9) mod 60`.
    FixedPhase,
}

struct Rival {
    name: &'static str,
    reading: Reading,
    shipped: bool,
}

const READINGS: [Rival; 6] = [
    Rival {
        name: "`BARHASPATYA`, shipped",
        reading: Reading::Sdk(SamvatsaraCount::Barhaspatya),
        shipped: true,
    },
    Rival {
        name: "`BARHASPATYA_RUNNING`",
        reading: Reading::Sdk(SamvatsaraCount::BarhaspatyaRunning),
        shipped: false,
    },
    Rival {
        name: "`CHANDRAMANA`",
        reading: Reading::Sdk(SamvatsaraCount::Chandramana),
        shipped: false,
    },
    Rival {
        name: "the text with Burgess's bija",
        reading: Reading::Bija,
        shipped: false,
    },
    Rival {
        name: "running at Baishakh 1",
        reading: Reading::Baishakh,
        shipped: false,
    },
    Rival {
        name: "the baseline engine's fixed phase",
        reading: Reading::FixedPhase,
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask samvatsara") != 0)
        }
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

fn context(count: SamvatsaraCount) -> Result<Context, String> {
    Context::builder()
        .profile("nepali-default")
        .ephemeris([Ephemeris::Builtin])
        .settings_json(format!(
            r#"{{"calendars": {{"samvatsara": "{}"}}}}"#,
            count.key()
        ))
        .build()
        .map_err(|e| e.to_string())
}

/// The lunar year holding a mid-year day of a Vikrama year.
fn year_of(context: &Context, vikrama: i32) -> Result<LunarYear, String> {
    let (place, clock) = kathmandu();
    let day = CalendarDate::defined(Calendar::Gregorian, vikrama - 57, 8, 1);
    let years = context
        .almanac()
        .years(&day, &day, &place, clock)
        .map_err(|e| format!("VS {vikrama}: {e}"))?;
    years
        .value
        .into_iter()
        .next()
        .ok_or_else(|| format!("VS {vikrama}: no year"))
}

/// A Jovian count's name.
fn member(count: i64) -> Result<Samvatsara, String> {
    JovianYear::numbered(&Parameters::TEXT, count)
        .map(|year| year.member)
        .map_err(|e| e.to_string())
}

/// The names a reading gives the named years, in `NAMED`'s order.
fn named_by(reading: Reading, shipped: &[LunarYear]) -> Result<Vec<Samvatsara>, String> {
    match reading {
        Reading::Sdk(count) => {
            let context = context(count)?;
            NAMED
                .iter()
                .map(|named| year_of(&context, named.vikrama).map(|year| year.samvatsara))
                .collect()
        }
        Reading::Bija => {
            let params = Parameters::TEXT.with_bija(&BIJA);
            // The shipped years' first sunrises and the four before the
            // first, which the advancing walk may read.
            let context = context(SamvatsaraCount::Barhaspatya)?;
            let first = NAMED.first().map_or(0, |named| named.vikrama);
            let mut before = Vec::new();
            for vikrama in first - 4..first {
                before.push(year_of(&context, vikrama)?);
            }
            let running = before
                .iter()
                .chain(shipped)
                .map(|year| JovianYear::at(&params, year.began).map(|jovian| jovian.count))
                .collect::<Result<Vec<i64>, _>>()
                .map_err(|e| e.to_string())?;
            (0..shipped.len())
                .map(|k| {
                    let count =
                        name_counts(&running, before.len() + k, SamvatsaraCount::Barhaspatya)
                            .ok_or_else(|| String::from("the bija's walk ran out of years"))?;
                    JovianYear::numbered(&params, count)
                        .map(|year| year.member)
                        .map_err(|e| e.to_string())
                })
                .collect()
        }
        Reading::Baishakh => {
            let context = context(SamvatsaraCount::Barhaspatya)?;
            let (place, clock) = kathmandu();
            NAMED
                .iter()
                .map(|named| {
                    let day = CalendarDate::defined(Calendar::BikramSambat, named.vikrama, 1, 1);
                    let sunrise = context
                        .almanac()
                        .of(&day, &day, &place, clock)
                        .map_err(|e| e.to_string())?
                        .value
                        .first()
                        .map(|found| found.day.sunrise)
                        .ok_or_else(|| String::from("no Baishakh 1"))?;
                    JovianYear::at(&Parameters::TEXT, sunrise)
                        .map(|year| year.member)
                        .map_err(|e| e.to_string())
                })
                .collect()
        }
        Reading::FixedPhase => NAMED
            .iter()
            .map(|named| member(i64::from(named.vikrama) + 9 - 26))
            .collect(),
    }
}

/// §1 and §2: each reading over the named years, a problem for every
/// verdict the page does not expect.
fn named_years(out: &mut String, problems: &mut Vec<String>) -> Result<Vec<Claim>, String> {
    let barhaspatya = context(SamvatsaraCount::Barhaspatya)?;
    let shipped = NAMED
        .iter()
        .map(|named| year_of(&barhaspatya, named.vikrama))
        .collect::<Result<Vec<_>, _>>()?;
    let mut claims = Vec::new();
    let mut columns = Vec::new();
    for rival in &READINGS {
        let names = named_by(rival.reading, &shipped)?;
        let wrong = NAMED
            .iter()
            .zip(&names)
            .filter(|(named, name)| named.name != **name)
            .count();
        if rival.shipped != (wrong == 0) {
            problems.push(format!(
                "{} {} the years Nepal named, and the page expects it {}",
                rival.name,
                if wrong == 0 { "names" } else { "misnames" },
                if rival.shipped { "to" } else { "not to" }
            ));
        }
        let claim = Claim::counted(
            format!("{}: the years Nepal named", rival.name),
            wrong,
            NAMED.len(),
        );
        claims.push(if rival.shipped {
            claim
        } else {
            claim.with_note("a rival")
        });
        columns.push(names);
    }
    out.push_str(
        "\n## 2. The years Nepal named\n\n\
         Each year is the one holding 1 August of its Vikrama year, founded at\n\
         Kathmandu under `nepali-default`; a mismatch is marked.\n\n\
         | Vikrama | named | source |",
    );
    for rival in &READINGS {
        let _ = write!(out, " {} |", rival.name);
    }
    out.push_str("\n|---|---|---|");
    out.push_str(&"---|".repeat(READINGS.len()));
    out.push('\n');
    for (k, named) in NAMED.iter().enumerate() {
        let _ = write!(
            out,
            "| {} | {} | {} |",
            named.vikrama,
            named.name.key(),
            named.source
        );
        for names in &columns {
            let cell = names.get(k).map_or_else(String::new, |name| {
                if *name == named.name {
                    name.key().to_string()
                } else {
                    format!("**{}**", name.key())
                }
            });
            let _ = write!(out, " {cell} |");
        }
        out.push('\n');
    }
    Ok(claims)
}

/// Days from a civil date on a clock to an instant.
fn days_from(date: (i32, u8, u8), hours: f64, instant: f64) -> f64 {
    let (y, m, d) = date;
    let midnight = teistro::jd_of_fixed(teistro_calendar::gregorian::fixed_from_gregorian(y, m, d));
    instant - (midnight - hours / 24.0)
}

/// §3: the text's worked example and the press's dated Jovian years.
fn dated(
    out: &mut String,
    claims: &mut Vec<Claim>,
    problems: &mut Vec<String>,
) -> Result<(), String> {
    let text = JovianYear::at(
        &Parameters::TEXT,
        teistro::quantity::JulianDay::try_new(teistro::jd_of_fixed(
            teistro_calendar::gregorian::fixed_from_gregorian(1859, 6, 1),
        ))
        .map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let bija = JovianYear::numbered(&Parameters::TEXT.with_bija(&BIJA), text.count)
        .map_err(|e| e.to_string())?;
    // Burgess dates on the Indian meridian's day; his rounding is the bound.
    let text_off = days_from((1859, 2, 23), 0.0, text.from.get());
    let bija_off = days_from((1859, 4, 3), 0.0, bija.from.get());
    let example = text.count + 1 == 5019
        && text.member == Samvatsara::Prajotpatti
        && bija.member == Samvatsara::Prajotpatti
        && text_off.abs() <= EXAMPLE_DAYS
        && bija_off.abs() <= EXAMPLE_DAYS;
    claims.push(Claim::counted(
        "the text's count names Burgess's worked example: the 5019th year, Prajapati, begun 23 February 1859 or 3 April with the bija",
        usize::from(!example),
        1,
    ));
    if !example {
        problems.push(String::from(
            "the text's count misses Burgess's worked example",
        ));
    }
    out.push_str(
        "\n## 3. Jovian years dated\n\n\
         Days from the recorded date's midnight to the computed first instant:\n\
         Burgess's on Greenwich's clock, the press's on India's.\n\n\
         | Jovian year | recorded | source | computed first instant (UTC JD) | days |\n\
         |---|---|---|---|---|\n",
    );
    let _ = writeln!(
        out,
        "| {} (the text) | 1859-02-23 | Burgess, note to I.55 | {:.4} | {text_off:+.2} |",
        text.member.key(),
        text.from.get()
    );
    let _ = writeln!(
        out,
        "| {} (with the bija) | 1859-04-03 | Burgess, note to I.55 | {:.4} | {bija_off:+.2} |",
        bija.member.key(),
        bija.from.get()
    );
    let mut worst: f64 = 0.0;
    for record in &DATED {
        let (y, m, d) = record.date;
        let midday =
            teistro::jd_of_fixed(teistro_calendar::gregorian::fixed_from_gregorian(y, m, d));
        let running = JovianYear::at(
            &Parameters::TEXT,
            teistro::quantity::JulianDay::try_new(midday + 0.5).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        // The year the record names: the one running that day, or the
        // next when it begins the day after.
        let year = if running.member == record.name {
            running
        } else {
            JovianYear::numbered(&Parameters::TEXT, running.count + 1).map_err(|e| e.to_string())?
        };
        let off = days_from(record.date, INDIA_HOURS, year.from.get());
        worst = worst.max(off.abs());
        let _ = writeln!(
            out,
            "| {} | {y}-{m:02}-{d:02} | {} | {:.4} | {off:+.2} |",
            year.member.key(),
            record.source,
            year.from.get()
        );
        if year.member != record.name {
            problems.push(format!(
                "no Jovian year {} near {y}-{m}-{d}",
                record.name.key()
            ));
        }
    }
    let dated_wrong = usize::from(worst > DATED_DAYS);
    claims.push(Claim::counted(
        format!("the press's dated Jovian years begin within {DATED_DAYS} days of the text's"),
        dated_wrong,
        DATED.len(),
    ));
    if dated_wrong != 0 {
        problems.push(format!(
            "a dated Jovian year is {worst:.2} days from the text's"
        ));
    }
    Ok(())
}

/// Each year of the centuries, founded under the shipped count, the
/// tasks dealt round the machine's cores.
fn centuries() -> Result<Vec<LunarYear>, String> {
    let years: Vec<i32> = CENTURIES.collect();
    let workers = std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let found = std::thread::scope(|scope| {
        let spawned: Vec<_> = (0..workers.min(years.len()))
            .map(|worker| {
                let mine: Vec<i32> = years
                    .iter()
                    .copied()
                    .skip(worker)
                    .step_by(workers)
                    .collect();
                scope.spawn(move || {
                    let context = context(SamvatsaraCount::Barhaspatya)?;
                    mine.into_iter()
                        .map(|vikrama| year_of(&context, vikrama))
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut all = Vec::new();
        for worker in spawned {
            all.extend(
                worker
                    .join()
                    .map_err(|_| String::from("a worker founding the years panicked"))??,
            );
        }
        Ok::<_, String>(all)
    })?;
    let mut sorted = found;
    sorted.sort_by_key(|year| year.vikrama);
    Ok(sorted)
}

/// Names a list of Vikrama years compactly.
fn listed(years: &[i32]) -> String {
    if years.is_empty() {
        return String::from("none");
    }
    years
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

/// The count each year of the centuries is named by under the shipped
/// count, held to the full recursion it stands for: a year takes the
/// running count or one past the year before's name. Seeded at the first
/// year, so the walk's bound of years is left out of the comparison.
fn walked(
    years: &[LunarYear],
    running: &[i64],
    claims: &mut Vec<Claim>,
    problems: &mut Vec<String>,
) -> Vec<i64> {
    let mut recursion: Vec<i64> = Vec::with_capacity(running.len());
    for &own in running {
        let named = recursion.last().map_or(own, |&before| own.max(before + 1));
        recursion.push(named);
    }
    let shipped: Vec<i64> = years
        .iter()
        .zip(running)
        .map(|(year, &own)| {
            let ahead = member(own + 1).is_ok_and(|next| next == year.samvatsara)
                && member(own).is_ok_and(|now| now != year.samvatsara);
            own + i64::from(ahead)
        })
        .collect();
    let differ = shipped
        .iter()
        .zip(&recursion)
        .skip(MOST_ADVANCED_YEARS)
        .filter(|(a, b)| a != b)
        .count();
    claims.push(Claim::counted(
        format!(
            "the shipped count's walk of at most {MOST_ADVANCED_YEARS} years back is the full recursion"
        ),
        differ,
        years.len().saturating_sub(MOST_ADVANCED_YEARS),
    ));
    if differ != 0 {
        problems.push(format!(
            "the walk back and the recursion name {differ} years apart"
        ));
    }
    shipped
}

/// The longest run of consecutive years in a list.
fn longest_run(years: &[i32]) -> usize {
    years
        .iter()
        .fold((0, 0, None::<i32>), |(best, run, last), &vikrama| {
            let run = if last == Some(vikrama - 1) {
                run + 1
            } else {
                1
            };
            (usize::max(best, run), run, Some(vikrama))
        })
        .0
}

/// §4: the centuries, the shortcut held to the recursion, and the years
/// the two Barhaspatya readings name apart.
fn six_centuries(
    out: &mut String,
    claims: &mut Vec<Claim>,
    problems: &mut Vec<String>,
) -> Result<(), String> {
    let years = centuries()?;
    let contiguous = years
        .windows(2)
        .all(|pair| matches!(pair, [a, b] if a.vikrama + 1 == b.vikrama && a.ended == b.began));
    if !contiguous {
        problems.push(String::from("the centuries' years do not abut"));
    }
    let running: Vec<i64> = years
        .iter()
        .map(|year| year.jovian.first().map_or(0, |jovian| jovian.count))
        .collect();
    let shipped = walked(&years, &running, claims, problems);
    let at = |k: usize| years.get(k).map_or(0, |year| year.vikrama);
    let advanced: Vec<i32> = (0..years.len())
        .filter(|&k| shipped.get(k) != running.get(k))
        .map(at)
        .collect();
    let repeats: Vec<i32> = (1..running.len())
        .filter(|&k| running.get(k) == running.get(k - 1))
        .map(at)
        .collect();
    let lupta: Vec<String> = years
        .iter()
        .filter_map(|year| {
            year.lupta
                .map(|name| format!("{} ({})", year.vikrama, name.key()))
        })
        .collect();
    let longest = longest_run(&advanced);
    if longest >= MOST_ADVANCED_YEARS {
        problems.push(format!(
            "a run of {longest} advanced years reaches the walk's bound of {MOST_ADVANCED_YEARS}"
        ));
    }
    // A year whose Jovian year changed between its opening new moon and
    // its first sunrise: its name depends on reading the pratipada as the
    // tithi or as the day.
    let straddled: Vec<i32> = years
        .iter()
        .filter(|year| {
            year.jovian
                .first()
                .is_some_and(|jovian| jovian.from.get() > year.opened.get())
        })
        .map(|year| year.vikrama)
        .collect();
    let _ = write!(
        out,
        "\n## 4. {} to {}, at Kathmandu\n\n\
         Every year founded under `nepali-default`. A repeat is a year whose\n\
         running Jovian year already ran at the year before's pratipada; the\n\
         shipped count names it one ahead, and the lead lasts until a year\n\
         whose count rises by two, where the expunged name falls. The longest\n\
         run of advanced years is {longest}, inside the walk's bound of\n\
         {MOST_ADVANCED_YEARS}.\n\n\
         | what | count | years |\n|---|---|---|\n",
        CENTURIES.start(),
        CENTURIES.end()
    );
    let _ = writeln!(out, "| years founded | {} | |", years.len());
    for (what, list) in [
        ("repeats under `BARHASPATYA_RUNNING`", &repeats),
        ("named one ahead under `BARHASPATYA`", &advanced),
        (
            "the Jovian year changed between the opening new moon and the first sunrise",
            &straddled,
        ),
    ] {
        let _ = writeln!(out, "| {what} | {} | {} |", list.len(), listed(list));
    }
    let _ = writeln!(
        out,
        "| a Jovian year expunged under `BARHASPATYA` | {} | {} |",
        lupta.len(),
        if lupta.is_empty() {
            String::from("none")
        } else {
            lupta.join(", ")
        }
    );
    Ok(())
}

fn page() -> Result<String, String> {
    let mut out = String::new();
    out.push_str("# The sixty-year cycle, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask samvatsara`. Do not edit:\n\
         `check-samvatsara` regenerates this page and fails on any difference.\n\n\
         It measures `teistro_calendar::samvatsara` and `Almanac::years`: the\n\
         Surya Siddhanta's Jovian count (I.55), and the name Nepal's panchanga\n\
         committee gives a lunar year, the Jovian year that meets Chaitra\n\
         Shukla Pratipada (C180), with a name never naming two years (C184).\n",
    );
    let mut problems = Vec::new();
    let mut body = String::new();
    let mut claims = named_years(&mut body, &mut problems)?;
    dated(&mut body, &mut claims, &mut problems)?;
    six_centuries(&mut body, &mut claims, &mut problems)?;
    out.push_str("\n## 1. The claims\n\n");
    out.push_str(&table(&claims));
    out.push_str(&body);
    if problems.is_empty() {
        Ok(out)
    } else {
        Err(problems.join("\n      "))
    }
}
