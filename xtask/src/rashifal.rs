//! The measurement pass over the **rashifal** (`03-design/rashifal.md`
//! step 3): what each of C361's differences moves, over a year of days
//! at Kathmandu.
//!
//! No text scores a rashifal and nothing records one, so this pass holds
//! the SDK's reading against the baseline engine's choices one at a time,
//! each over the same sky: its tables (D1 to D3), its 06:00 clock (B5),
//! the events its scan cannot report (B1 to B3) and the day its panchanga
//! is taken from (B6). Each count is a difference the SDK's default makes
//! to a consumer of the baseline's numbers.
//!
//! `cargo xtask rashifal` writes the page; `check-rashifal` regenerates it
//! in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::{Calendar, Graha};
use teistro::gochar::Verdict;
use teistro::gochar::hits::HitEvent;
use teistro::quantity::{Altitude, JulianDay, Latitude, Longitude, Place, Utc};
use teistro::rashifal::baseline::{Period, baseline_gochar, baseline_score};
use teistro::rashifal::{PeriodEvent, RashiReading};
use teistro::{
    CalendarDate, Context, Ephemeris, FixedDay, RashifalPeriod, RashifalRequest, Snapshot,
    UtcOffset,
};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict as Decided, capitalised, count, spelled, table};

const PAGE: &str = "docs/03-design/rashifal-measured.md";

/// Kathmandu's civil offset, the baseline engine's one reckoning.
const NEPAL: UtcOffset = UtcOffset::literal(5, 45, 0);

/// The year measured, July 2026 to June 2027, which holds the nodes'
/// ingress and Saturn's: its first month, and how many.
const FROM: (i32, u8) = (2026, 7);
const MONTHS: u8 = 12;

/// The first day of the `n`th month from [`FROM`].
fn month_start(n: u8) -> CalendarDate {
    let (year, month) = FROM;
    let at = i32::from(month - 1) + i32::from(n);
    CalendarDate::defined(
        Calendar::Gregorian,
        year + at.div_euclid(12),
        u8::try_from(at.rem_euclid(12) + 1).unwrap_or(1),
        1,
    )
}

/// The baseline's clock, 06:00 at the reckoning's offset (B5).
const SIX: Snapshot = Snapshot::Clock { hour: 6, minute: 0 };

/// The grahas the baseline's weekly scan reads (Jupiter, Saturn and the
/// nodes); its monthly scan reads every one but the Moon.
const WEEKLY_SCAN: [Graha; 4] = [Graha::Jupiter, Graha::Saturn, Graha::Rahu, Graha::Ketu];

fn kathmandu() -> Place {
    Place::new(
        Latitude::literal(27.7172),
        Longitude::literal(85.324),
        Altitude::literal(1400.0),
    )
}

pub(crate) fn generate(root: &Path) -> i32 {
    match page() {
        Ok(text) => write(root, &[Output::new(PAGE, text)]),
        Err(why) => {
            eprintln!("rashifal: {why}");
            1
        }
    }
}

pub(crate) fn check_generated(root: &Path) -> i32 {
    match page() {
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask rashifal") != 0),
        Err(why) => {
            eprintln!("check-rashifal: {why}");
            1
        }
    }
}

/// The days measured, as fixed days, and the calendar that names them.
struct Days {
    first: FixedDay,
    count: i64,
}

impl Days {
    fn of(sdk: &Context) -> Result<Days, String> {
        let fixed = |n: u8| {
            sdk.calendar()
                .fixed_of(&month_start(n))
                .map_err(|why| why.to_string())
        };
        let first = fixed(0)?;
        let end = fixed(MONTHS)?;
        Ok(Days {
            first,
            count: first.days_until(end),
        })
    }

    fn date(sdk: &Context, day: FixedDay) -> Result<CalendarDate, String> {
        sdk.calendar()
            .date_of(Calendar::Gregorian, day)
            .map_err(|why| why.to_string())
    }
}

fn read(sdk: &Context, requests: &[RashifalRequest]) -> Result<Vec<RashifalPeriod>, String> {
    sdk.chart()
        .rashifal_many(requests)
        .map(|answer| answer.value)
        .map_err(|why| why.to_string())
}

/// What the tables and the clock move over the days (D1 to D3, B5).
#[derive(Default)]
struct OverDays {
    cells: usize,
    by_tables: [usize; 9],
    by_clock: usize,
    moon_moved: usize,
    sign_days: usize,
    score_by_tables: usize,
    score_by_clock: usize,
    worst_by_tables: u8,
    worst_by_clock: u8,
}

fn verdicts(reading: &RashiReading) -> [Verdict; 9] {
    reading.gochar.grahas.each_ref().map(|one| one.verdict)
}

fn as_the_baseline(reading: &RashiReading) -> RashiReading {
    RashiReading {
        gochar: baseline_gochar(&reading.gochar),
        ..reading.clone()
    }
}

fn over_days(sunrise: &[RashifalPeriod], six: &[RashifalPeriod]) -> OverDays {
    let mut over = OverDays::default();
    for (dawn, clock) in sunrise.iter().zip(six) {
        if dawn.transits[Graha::Moon as usize].sign != clock.transits[Graha::Moon as usize].sign {
            over.moon_moved += 1;
        }
        for (text, at_six) in dawn.readings.iter().zip(&clock.readings) {
            let baseline = as_the_baseline(text);
            over.sign_days += 1;
            for ((graha, ours), (theirs, later)) in verdicts(text)
                .iter()
                .enumerate()
                .zip(verdicts(&baseline).iter().zip(verdicts(at_six).iter()))
            {
                over.cells += 1;
                if ours != theirs
                    && let Some(moved) = over.by_tables.get_mut(graha)
                {
                    *moved += 1;
                }
                if ours != later {
                    over.by_clock += 1;
                }
            }
            let score = |reading: &RashiReading, period: &RashifalPeriod| {
                baseline_score(reading, &period.retrograde, period.panchanga, Period::Daily).overall
            };
            let ours = score(text, dawn);
            let by_tables = ours.abs_diff(score(&baseline, dawn));
            let by_clock = ours.abs_diff(score(at_six, clock));
            if by_tables > 0 {
                over.score_by_tables += 1;
                over.worst_by_tables = over.worst_by_tables.max(by_tables);
            }
            if by_clock > 0 {
                over.score_by_clock += 1;
                over.worst_by_clock = over.worst_by_clock.max(by_clock);
            }
        }
    }
    over
}

/// Why the baseline engine's scan cannot report an event, the first that
/// holds, or `None` when it can.
fn missed(event: &PeriodEvent, weekly: bool, last_six: JulianDay<Utc>) -> Option<usize> {
    let graha = event.hit.graha;
    if matches!(graha, Graha::Rahu | Graha::Ketu) {
        // B1: the nodes have no ephemeris body in its scan.
        return Some(0);
    }
    if weekly
        && (!WEEKLY_SCAN.contains(&graha) || matches!(event.hit.event, HitEvent::Station { .. }))
    {
        // Its week scans four grahas and finds no station.
        return Some(1);
    }
    if weekly && graha == Graha::Saturn {
        // B2: a step of seven days in a window of six.
        return Some(2);
    }
    if event.hit.instant.get() > last_six.get() {
        // B3: its window closes at 06:00 on the last day.
        return Some(3);
    }
    None
}

const MISSED: [&str; 4] = [
    "a node's ingress (B1)",
    "in a week, a graha its weekly scan leaves out, or a station",
    "in a week, Saturn's ingress (B2)",
    "after 06:00 on the period's last day (B3)",
];

/// The periods' events, against what the baseline's scan reports.
#[derive(Default)]
struct OverEvents {
    periods: usize,
    found: usize,
    missed: [usize; 4],
    outside: usize,
    utc_day: usize,
}

fn over_events(sdk: &Context, periods: &[(RashifalRequest, bool)]) -> Result<OverEvents, String> {
    let requests: Vec<RashifalRequest> = periods.iter().map(|(one, _)| one.clone()).collect();
    let read = read(sdk, &requests)?;
    let mut over = OverEvents::default();
    let calendar = sdk.calendar();
    for ((request, weekly), period) in periods.iter().zip(&read) {
        over.periods += 1;
        let midnight = |date: &CalendarDate, plus: i64| -> Result<f64, String> {
            let fixed = calendar.fixed_of(date).map_err(|why| why.to_string())?;
            Ok(teistro::jd_of_fixed(fixed.plus_days(plus)) - f64::from(NEPAL.seconds()) / 86_400.0)
        };
        let (from, to) = (midnight(request.first(), 0)?, midnight(request.last(), 1)?);
        let last_six = JulianDay::<Utc>::literal(to - 18.0 / 24.0);
        // Every sign holds the same events, counted from it.
        let Some(aries) = period.readings.first() else {
            return Err(String::from("a period read no sign"));
        };
        for one in &aries.events {
            over.found += 1;
            let at = one.event.hit.instant.get();
            if !(from..to).contains(&at) {
                over.outside += 1;
            }
            if let Some(why) = missed(&one.event, *weekly, last_six)
                && let Some(counted) = over.missed.get_mut(why)
            {
                *counted += 1;
            }
            // B4: its dates are UTC dates.
            let local = teistro::fixed_of_jd(at + f64::from(NEPAL.seconds()) / 86_400.0).0;
            if teistro::fixed_of_jd(at).0 != local {
                over.utc_day += 1;
            }
        }
    }
    Ok(over)
}

/// How many sign-months score otherwise with the panchanga of the month's
/// first day, as the baseline takes it from the day a row is made (B6).
fn over_panchanga(months: &[RashifalPeriod], first_days: &[RashifalPeriod]) -> (usize, usize) {
    let mut moved = 0;
    let mut of = 0;
    for (month, first) in months.iter().zip(first_days) {
        for reading in &month.readings {
            of += 1;
            let score = |panchanga| {
                baseline_score(reading, &month.retrograde, panchanga, Period::Monthly).overall
            };
            if score(month.panchanga) != score(first.panchanga) {
                moved += 1;
            }
        }
    }
    (moved, of)
}

fn name(graha: Graha) -> String {
    let key = graha.key();
    let bare = key.rsplit('.').next().unwrap_or(key);
    capitalised(&bare.to_lowercase())
}

#[allow(
    clippy::too_many_lines,
    reason = "a page reads top to bottom, and splitting it would scatter its prose"
)]
fn page() -> Result<String, String> {
    let sdk = Context::builder()
        .ephemeris([Ephemeris::Builtin])
        .build()
        .map_err(|why| why.to_string())?;
    let days = Days::of(&sdk)?;
    let mut daily = Vec::new();
    for at in 0..days.count {
        let date = Days::date(&sdk, days.first.plus_days(at))?;
        daily.push(RashifalRequest::day(date, kathmandu(), NEPAL).with_events([]));
    }
    let sunrise = read(&sdk, &daily)?;
    let six: Vec<RashifalRequest> = daily.iter().map(|one| one.clone().at(SIX)).collect();
    let six = read(&sdk, &six)?;
    let over = over_days(&sunrise, &six);

    // Sunday to Saturday, every week wholly inside the years; and every
    // month. A fixed day's remainder by seven is 0 on a Sunday.
    let mut periods = Vec::new();
    let mut sunday = days
        .first
        .plus_days((7 - days.first.get().rem_euclid(7)) % 7);
    let end = days.first.plus_days(days.count);
    while sunday.plus_days(6).get() < end.get() {
        periods.push((
            RashifalRequest::between(
                Days::date(&sdk, sunday)?,
                Days::date(&sdk, sunday.plus_days(6))?,
                kathmandu(),
                NEPAL,
            ),
            true,
        ));
        sunday = sunday.plus_days(7);
    }
    let weeks = periods.len();
    let mut months = Vec::new();
    let mut first_days = Vec::new();
    for n in 0..MONTHS {
        let first = month_start(n);
        let last = sdk
            .calendar()
            .fixed_of(&month_start(n + 1))
            .map_err(|why| why.to_string())?
            .plus_days(-1);
        let request =
            RashifalRequest::between(first.clone(), Days::date(&sdk, last)?, kathmandu(), NEPAL);
        months.push(request.clone());
        first_days.push(RashifalRequest::day(first, kathmandu(), NEPAL).with_events([]));
        periods.push((request, false));
    }
    let events = over_events(&sdk, &periods)?;
    let (panchanga_moved, panchanga_of) =
        over_panchanga(&read(&sdk, &months)?, &read(&sdk, &first_days)?);

    let mut out = String::new();
    let _ = writeln!(
        out,
        "# Rashifal, measured\n\n\
         Status: `generated` by `cargo xtask rashifal`. Do not edit:\n\
         `check-rashifal` regenerates this page and fails on any difference.\n\n\
         The design this measures is `rashifal.md`. **No text scores a\n\
         rashifal and nothing records one**, so the kernel is held to the\n\
         gochar it is built from by the crate's tests, and this page counts\n\
         what each of the baseline engine's choices moves (C361), one at a\n\
         time over the same sky: every day from July 2026 to June 2027 at\n\
         Kathmandu, a year that holds the nodes' ingress and Saturn's, read\n\
         for all twelve signs under the default profile.\n"
    );
    let mut rows = format!(
        "| graha | verdicts moved, of {} |\n|---|---:|\n",
        count(over.cells / 9)
    );
    for graha in teistro::gochar::GRAHAS {
        let _ = writeln!(
            rows,
            "| {} | {} |",
            name(graha),
            count(over.by_tables.get(graha as usize).copied().unwrap_or(0))
        );
    }
    let tables: usize = over.by_tables.iter().sum();
    let _ = writeln!(
        out,
        "## 1. The baseline's tables (D1 to D3)\n\n\
         Each day's reading at sunrise judged again by the baseline engine's\n\
         vedha tables, `baseline_gochar`: Venus's 11th and 12th exchanged,\n\
         the nodes' hybrid vedha, and the Sun and the Moon its only\n\
         exemption, with every node obstructing. {} of {} verdicts move, by\n\
         graha:\n\n{rows}\n\
         The daily score moves on {} of {} sign-days, by {} at most.\n",
        count(tables),
        count(over.cells),
        count(over.score_by_tables),
        count(over.sign_days),
        over.worst_by_tables
    );
    let _ = writeln!(
        out,
        "## 2. 06:00 against sunrise (C358, B5)\n\n\
         The same days read at 06:00 at +05:45, the baseline's clock, under\n\
         the text's tables. The Moon is in another sign on {} of {} days,\n\
         {} of {} verdicts move, and the daily score moves on {} of {}\n\
         sign-days, by {} at most.\n",
        count(over.moon_moved),
        count(sunrise.len()),
        count(over.by_clock),
        count(over.cells),
        count(over.score_by_clock),
        count(over.sign_days),
        over.worst_by_clock
    );
    let mut missed_rows =
        String::from("| why the baseline cannot report it | events |\n|---|---:|\n");
    for (why, counted) in MISSED.iter().zip(events.missed) {
        let _ = writeln!(missed_rows, "| {why} | {} |", count(counted));
    }
    let missed: usize = events.missed.iter().sum();
    let _ = writeln!(
        out,
        "## 3. The events (C360, B1 to B4)\n\n\
         Every Sunday-to-Saturday week wholly inside the year ({}) and\n\
         every month ({}), each with its ingresses and stations of every\n\
         graha but the Moon. The SDK finds {} events, each counted once in every period\n\
         that holds it, and the baseline's\n\
         scan cannot report {} of them, each counted under the first reason\n\
         that holds:\n\n{missed_rows}\n\
         Of the events found, {} fall on another UTC date than their\n\
         Kathmandu one, which is the date the baseline prints (B4).\n",
        spelled(weeks),
        spelled(events.periods - weeks),
        count(events.found),
        count(missed),
        count(events.utc_day)
    );
    let _ = writeln!(
        out,
        "## 4. The panchanga's day (B6)\n\n\
         Each month scored with its reference day's panchanga, the SDK's,\n\
         and with its first day's, one day the baseline can take it from:\n\
         {} of {} sign-months score otherwise.\n",
        count(panchanga_moved),
        count(panchanga_of)
    );

    let claims = vec![
        Claim::counted(
            "every event a period reports falls between local midnight of its first day and local midnight after its last",
            events.outside,
            events.found,
        ),
        Claim::counted(
            "the baseline engine's vedha tables give the text's verdicts",
            tables,
            over.cells,
        ),
        Claim::counted(
            "reading at 06:00 rather than at sunrise moves no verdict",
            over.by_clock,
            over.cells,
        ),
        Claim::counted(
            "the baseline engine's scan reports every event a period holds",
            missed,
            events.found,
        ),
        Claim::counted(
            "a month scores alike whichever of its days the panchanga is taken from",
            panchanga_moved,
            panchanga_of,
        ),
    ];
    let _ = writeln!(out, "## 5. What this pass decides\n");
    let _ = writeln!(out, "{}", table(&claims));
    let falsified = claims
        .iter()
        .filter(|claim| claim.verdict == Decided::Falsified)
        .count();
    let _ = writeln!(
        out,
        "{} The first holds the façade's own window; each of the others\n\
         is one of the baseline engine's choices, held against the SDK's\n\
         default over the same sky.",
        match falsified {
            0 => format!("None of the {} claims is falsified.", spelled(claims.len())),
            1 => format!("One of the {} claims is falsified.", spelled(claims.len())),
            _ => format!(
                "{} of the {} claims are falsified.",
                capitalised(&spelled(falsified)),
                spelled(claims.len())
            ),
        }
    );
    Ok(crate::measure::fill(&out))
}
