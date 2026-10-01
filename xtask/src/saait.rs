//! The measurement pass over the **saait packs**
//! (`03-design/muhurta.md` §4.6): the rites beyond marriage held to the
//! days Nepal's national panchanga committee printed for them.
//!
//! The committee's VS 2083 muhurta sheet prints, by Bikram Sambat date
//! and weekday, the days of the year it finds for a thread ceremony
//! (bratabandha), a marriage (vivaha), the first feeding (pasni) and a
//! house entry (griha pravesh). Each row, read off the page image, is
//! founded at Kathmandu on Nepal's clock and searched with the rite
//! Raman's *Muhurtha* gives it, over the committee's own sky and the
//! modern one. A printed day **agrees** when the search finds a clean
//! window in its daylight: one no bar strikes and no grade of the rite's
//! own rejects. A day that does not is attributed to the clauses that
//! reject every daylight window it has.
//!
//! The sheet's weekday is checked against its date first; a row whose
//! two disagree contradicts itself, and is listed by name rather than
//! read either way.
//!
//! `cargo xtask saait` writes the page; `check-saait` regenerates it in
//! memory and fails on any difference.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use teistro::muhurta::activity::MonthWithSun;
use teistro::muhurta::clause::{ClauseKey, ClauseKind};
use teistro::muhurta::{ActivityRules, Bar, Grade, Judgement, MonthRule};
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{Activity, CalendarDate, Context, Ephemeris, MuhurtaRequest, Panchanga, UtcOffset};
use teistro_core::catalogue::{Calendar, Masa, Rashi, Vara};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, table};

const PAGE: &str = "docs/03-design/saait-measured.md";

/// The Bikram Sambat year the sheet is for.
const YEAR: i32 = 2083;

/// A rite the sheet prints days for.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Rite {
    Bratabandha,
    Vivaha,
    Pasni,
    GrihaPravesh,
}

impl Rite {
    const ALL: [Rite; 4] = [
        Rite::Bratabandha,
        Rite::Vivaha,
        Rite::Pasni,
        Rite::GrihaPravesh,
    ];

    /// The rules the SDK ships for it.
    const fn activity(self) -> Activity {
        match self {
            Rite::Bratabandha => Activity::RamanUpanayana,
            Rite::Vivaha => Activity::RamanMarriage,
            Rite::Pasni => Activity::RamanAnnaprasana,
            Rite::GrihaPravesh => Activity::RamanGrihaPravesha,
        }
    }

    /// Its place in [`Rite::ALL`].
    const fn index(self) -> usize {
        self as usize
    }

    /// Its name on the sheet.
    const fn name(self) -> &'static str {
        match self {
            Rite::Bratabandha => "bratabandha",
            Rite::Vivaha => "vivaha",
            Rite::Pasni => "pasni",
            Rite::GrihaPravesh => "griha pravesh",
        }
    }
}

/// One row of the sheet: the rite, the Bikram Sambat month and day, and
/// the weekday printed beside it.
struct Printed {
    rite: Rite,
    month: u8,
    day: u8,
    vara: Vara,
}

const fn row(rite: Rite, month: u8, day: u8, vara: Vara) -> Printed {
    Printed {
        rite,
        month,
        day,
        vara,
    }
}

use Rite::{Bratabandha as B, GrihaPravesh as G, Pasni as P, Vivaha as V};
use Vara::{
    Budhavara as WED, Guruvara as THU, Mangalavara as TUE, Ravivara as SUN, Shanivara as SAT,
    Shukravara as FRI, Somavara as MON,
};

/// The committee's VS 2083 muhurta sheet, its three tables in the order
/// printed: each rite's months left to right, each month's days top to
/// bottom.
const SHEET: [Printed; 94] = [
    row(B, 1, 20, SUN),
    row(B, 1, 21, MON),
    row(B, 1, 23, WED),
    row(B, 3, 10, WED),
    row(B, 10, 25, MON),
    row(B, 10, 21, MON),
    row(B, 10, 23, WED),
    row(B, 11, 10, MON),
    row(B, 11, 13, THU),
    row(B, 11, 26, WED),
    row(B, 12, 3, WED),
    row(B, 12, 4, MON),
    row(B, 12, 10, WED),
    row(B, 12, 25, THU),
    row(B, 12, 28, SUN),
    row(V, 1, 7, MON),
    row(V, 1, 8, TUE),
    row(V, 1, 22, TUE),
    row(V, 1, 23, WED),
    row(V, 1, 24, THU),
    row(V, 1, 25, FRI),
    row(V, 1, 30, WED),
    row(V, 1, 31, THU),
    row(V, 3, 9, TUE),
    row(V, 3, 13, SAT),
    row(V, 3, 14, MON),
    row(V, 3, 15, TUE),
    row(V, 3, 17, WED),
    row(V, 3, 18, THU),
    row(V, 3, 23, TUE),
    row(V, 8, 9, WED),
    row(V, 8, 10, THU),
    row(V, 8, 16, WED),
    row(V, 8, 17, THU),
    row(V, 8, 19, SAT),
    row(V, 8, 24, THU),
    row(V, 8, 25, FRI),
    row(V, 8, 26, SAT),
    row(V, 10, 4, MON),
    row(V, 10, 12, TUE),
    row(V, 10, 15, FRI),
    row(V, 10, 19, TUE),
    row(V, 10, 20, WED),
    row(V, 10, 27, WED),
    row(V, 11, 3, MON),
    row(V, 11, 4, TUE),
    row(V, 11, 11, TUE),
    row(V, 11, 12, WED),
    row(V, 11, 15, SAT),
    row(V, 11, 16, SUN),
    row(V, 11, 17, MON),
    row(V, 11, 18, TUE),
    row(V, 11, 20, THU),
    row(V, 11, 25, TUE),
    row(V, 11, 26, WED),
    row(V, 11, 30, SUN),
    row(P, 1, 7, MON),
    row(P, 1, 10, THU),
    row(P, 1, 21, MON),
    row(P, 2, 4, MON),
    row(P, 2, 7, THU),
    row(P, 2, 22, FRI),
    row(P, 3, 3, WED),
    row(P, 3, 10, WED),
    row(P, 3, 17, WED),
    row(P, 3, 18, THU),
    row(P, 4, 4, MON),
    row(P, 4, 8, FRI),
    row(P, 4, 15, FRI),
    row(P, 4, 18, MON),
    row(P, 4, 20, WED),
    row(P, 5, 1, MON),
    row(P, 5, 3, WED),
    row(P, 5, 10, WED),
    row(P, 5, 29, MON),
    row(P, 6, 5, MON),
    row(P, 6, 26, MON),
    row(P, 7, 13, FRI),
    row(P, 7, 30, MON),
    row(P, 8, 10, THU),
    row(P, 8, 17, THU),
    row(P, 9, 1, WED),
    row(P, 9, 15, WED),
    row(P, 10, 15, FRI),
    row(P, 10, 25, WED),
    row(P, 11, 7, FRI),
    row(P, 11, 13, THU),
    row(P, 11, 26, WED),
    row(P, 12, 3, WED),
    row(P, 12, 10, WED),
    row(P, 12, 18, THU),
    row(P, 12, 25, THU),
    row(G, 1, 31, THU),
    row(G, 11, 20, THU),
];

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

/// A sky the days are founded under.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sky {
    /// `nepali-committee`, the committee's own.
    Committee,
    /// `nepali-default`, the modern sky under Lahiri.
    Modern,
}

impl Sky {
    const ALL: [Sky; 2] = [Sky::Committee, Sky::Modern];

    /// Its place in [`Sky::ALL`].
    const fn index(self) -> usize {
        self as usize
    }

    /// Its name on the page.
    const fn name(self) -> &'static str {
        match self {
            Sky::Committee => "the committee's",
            Sky::Modern => "modern",
        }
    }

    fn context(self) -> Result<Context, String> {
        let (profile, ephemeris) = match self {
            Sky::Committee => ("nepali-committee", Ephemeris::SuryaSiddhanta),
            Sky::Modern => ("nepali-default", Ephemeris::Builtin),
        };
        Context::builder()
            .profile(profile)
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
        Ok(text) => i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask saait") != 0),
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

/// What a sky made of one printed day.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Verdict {
    /// A clean window in the daylight.
    Clean,
    /// The season closed the day, by these blackouts.
    Closed(Vec<String>),
    /// Every daylight window rejected; these clauses reject all of them,
    /// or, where none does, each is rejected by one of these.
    Rejected {
        every: Vec<String>,
        some: Vec<String>,
    },
}

impl Verdict {
    fn shown(&self) -> String {
        match self {
            Verdict::Clean => String::from("clean"),
            Verdict::Closed(by) => format!("closed: {}", by.join(", ")),
            Verdict::Rejected { every, some } if every.is_empty() => {
                format!("each window by one of: {}", some.join(", "))
            }
            Verdict::Rejected { every, .. } => every.join(", "),
        }
    }
}

/// The level a rejection is at: the day's limbs, vara and month, which
/// hold for hours, or the sky at the instant, the lagna and the
/// placements, which turn every two.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Level {
    Day,
    Instant,
}

/// Why a window is not clean at a level: each grade of the rite's own
/// that rejects it, and at the instant each bar that struck it, as the
/// page names them.
fn rejections(window: &Judgement, level: Level) -> BTreeSet<String> {
    let mut why: BTreeSet<String> = window
        .clauses
        .iter()
        .filter_map(|clause| rejected_grade(&clause.kind, level))
        .collect();
    if level == Level::Instant {
        why.extend(window.barred_by.iter().map(bar_name));
    }
    why
}

fn key_name(key: ClauseKey) -> String {
    serde_json::to_value(key)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// A bar as the page names it: its key, and a placement's house.
fn bar_name(bar: &Bar) -> String {
    match bar {
        Bar::Key(key) => key_name(*key),
        Bar::Clause(ClauseKind::UnwantedPlacement { house, .. }) => {
            format!("UNWANTED_PLACEMENT {house}")
        }
        Bar::Clause(kind) => key_name(kind.key()),
    }
}

/// A grade of the rite's own that rejects at a level, named with its
/// member.
fn rejected_grade(kind: &ClauseKind, level: Level) -> Option<String> {
    let rejected = |grade: &Grade| *grade == Grade::Rejected;
    let member = match (kind, level) {
        (ClauseKind::Tithi { tithi, grade }, Level::Day) => rejected(grade).then(|| tithi.key()),
        (ClauseKind::Nakshatra { nakshatra, grade }, Level::Day) => {
            rejected(grade).then(|| nakshatra.key())
        }
        (ClauseKind::Yoga { yoga, grade }, Level::Day) => rejected(grade).then(|| yoga.key()),
        (ClauseKind::Karana { karana, grade }, Level::Day) => rejected(grade).then(|| karana.key()),
        (ClauseKind::Vara { vara, grade }, Level::Day) => rejected(grade).then(|| vara.key()),
        (ClauseKind::Month { masa, grade }, Level::Day) => rejected(grade).then(|| masa.key()),
        (ClauseKind::SolarMonth { sign, grade }, Level::Day)
        | (ClauseKind::Lagna { sign, grade }, Level::Instant) => {
            rejected(grade).then(|| sign.key())
        }
        (ClauseKind::Pada { pada }, Level::Instant) => Some(pada.nakshatra.key()),
        _ => None,
    }?;
    Some(format!("{} {member}", key_name(kind.key())))
}

/// What the instant's clauses make of the windows a day leaves open.
fn judged(open: &[&Judgement]) -> Verdict {
    let why: Vec<BTreeSet<String>> = open.iter().map(|w| rejections(w, Level::Instant)).collect();
    if why.iter().any(BTreeSet::is_empty) {
        return Verdict::Clean;
    }
    let every = why
        .iter()
        .skip(1)
        .fold(why.first().cloned().unwrap_or_default(), |acc, w| {
            acc.intersection(w).cloned().collect()
        });
    let some: BTreeSet<String> = why.iter().flatten().cloned().collect();
    Verdict::Rejected {
        every: every.into_iter().collect(),
        some: some.into_iter().collect(),
    }
}

/// Whether spans, each clipped to `[from, to]`, cover all of it.
fn covers(mut spans: Vec<(f64, f64)>, from: f64, to: f64) -> bool {
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut reached = from;
    for (start, end) in spans {
        if start > reached {
            return false;
        }
        reached = reached.max(end);
    }
    reached >= to
}

/// The day's verdict at the day level, from the spans its clauses hold
/// over: clean when the rejecting spans leave some of the daylight, and
/// otherwise the clauses that reject all of it, or each of those that
/// reject some.
///
/// A search cuts a day's windows at every clause's edge, so a clause
/// holds over every daylight window exactly when its spans cover the
/// daylight: this is the verdict the windows give, without cutting them.
fn day_verdict(day: &Panchanga, rules: &ActivityRules) -> Verdict {
    let (rise, set) = (day.day.sunrise.get(), day.day.sunset.get());
    let mut held = teistro::muhurta::clauses(day, None, &rules.day);
    held.extend(rules.month_clauses(day));
    let mut by_name: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for clause in &held {
        let Some(name) = rejected_grade(&clause.kind, Level::Day) else {
            continue;
        };
        let span = (clause.at.from.get().max(rise), clause.at.to.get().min(set));
        if span.0 < span.1 {
            by_name.entry(name).or_default().push(span);
        }
    }
    if !covers(by_name.values().flatten().copied().collect(), rise, set) {
        return Verdict::Clean;
    }
    let every = by_name
        .iter()
        .filter(|(_, spans)| covers((*spans).clone(), rise, set))
        .map(|(name, _)| name.clone())
        .collect();
    Verdict::Rejected {
        every,
        some: by_name.into_keys().collect(),
    }
}

/// What one sky makes of a printed day: the day's verdict, and, on a day
/// it leaves open, the instant's.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Reading {
    day: Verdict,
    instant: Option<Verdict>,
}

impl Reading {
    fn shown(&self) -> String {
        match &self.instant {
            None => self.day.shown(),
            Some(Verdict::Clean) => String::from("clean"),
            Some(instant) => format!("day clean; {}", instant.shown()),
        }
    }
}

/// The instant's verdict on a day the day rules leave open: the day's
/// windows, searched, and those of its daylight no day-level clause
/// rejects, judged. The spans said such a window exists; a search that
/// finds none is the two readings parting, and refused.
fn instant(
    context: &Context,
    rules: &ActivityRules,
    date: &CalendarDate,
) -> Result<Verdict, String> {
    let (place, clock) = kathmandu();
    let request = MuhurtaRequest::new(rules.clone())
        .with_windows_on(1)
        .at_most(usize::MAX);
    let found = context
        .almanac()
        .muhurta_with_days(date, date, &place, clock, &request)
        .map_err(|e| format!("{}: {e}", iso(date)))?;
    let day = found
        .days
        .value
        .first()
        .ok_or_else(|| format!("{}: no day", iso(date)))?;
    let (rise, set) = (day.day.sunrise.get(), day.day.sunset.get());
    let open: Vec<&Judgement> = found
        .answer
        .value
        .windows
        .iter()
        .filter(|w| w.at.from.get() < set && w.at.to.get() > rise)
        .filter(|w| rejections(w, Level::Day).is_empty())
        .collect();
    if open.is_empty() {
        return Err(format!(
            "{}: the spans leave daylight open and no window is",
            iso(date)
        ));
    }
    Ok(judged(&open))
}

/// The rows whose printed weekday is not their date's, each as (rite,
/// month, day): the sheet contradicts itself there, and the row is read
/// neither way. Refused both ways, so a row added to the list that agrees,
/// or a row that disagrees and is not listed, fails the pass.
const MISPRINTS: [(Rite, u8, u8); 6] = [
    (B, 10, 21),
    (B, 10, 23),
    (B, 12, 4),
    (V, 3, 14),
    (V, 3, 15),
    (P, 10, 25),
];

/// One printed row and what was found of it.
struct Found<'a> {
    printed: &'a Printed,
    /// The Gregorian day.
    date: CalendarDate,
    /// The weekday the date has.
    vara: Vara,
    /// Each sky's reading, in [`Sky::ALL`]'s order, on a row whose
    /// weekday agrees.
    readings: Option<[Reading; 2]>,
}

impl Found<'_> {
    fn reading(&self, sky: Sky) -> Option<&Reading> {
        self.readings.as_ref().and_then(|r| r.get(sky.index()))
    }
}

/// A date as the page spells it.
fn iso(date: &CalendarDate) -> String {
    format!("{}-{:02}-{:02}", date.year, date.month, date.day)
}

/// VS 2083's first and last days, as Gregorian dates.
fn the_year(context: &Context) -> Result<(CalendarDate, CalendarDate), String> {
    let calendar = context.calendar();
    let first_of = |year| {
        calendar
            .convert(
                &CalendarDate::defined(Calendar::BikramSambat, year, 1, 1),
                Calendar::Gregorian,
            )
            .map_err(|e| e.to_string())
    };
    let next = calendar
        .fixed_of(&first_of(YEAR + 1)?)
        .map_err(|e| e.to_string())?;
    let last = calendar
        .date_of(Calendar::Gregorian, next.plus_days(-1))
        .map_err(|e| e.to_string())?;
    Ok((first_of(YEAR)?, last))
}

/// A sky's year: every day of it, and, rite by rite, the days the rite's
/// season closes with the blackouts that close them.
struct Year {
    days: Vec<Panchanga>,
    closed: Vec<BTreeMap<String, Vec<String>>>,
}

impl Year {
    /// The day of a date.
    fn day(&self, date: &CalendarDate) -> Option<&Panchanga> {
        let date = iso(date);
        self.days.iter().find(|day| iso(&day.day.date) == date)
    }

    /// The days the rite's rules leave daylight open on, the season's
    /// closed days left out.
    fn open(&self, rite: Rite, rules: &ActivityRules) -> usize {
        let closed = self.closed.get(rite.index());
        self.days
            .iter()
            .filter(|day| !closed.is_some_and(|c| c.contains_key(&iso(&day.day.date))))
            .filter(|day| day_verdict(day, rules) == Verdict::Clean)
            .count()
    }
}

/// Founds a sky's year. The season is searched only for a rite that
/// heeds one; a rite heeding none closes nothing.
fn year(sky: Sky) -> Result<Year, String> {
    let context = sky.context()?;
    let (place, clock) = kathmandu();
    let (first, last) = the_year(&context)?;
    let days = context
        .almanac()
        .of(&first, &last, &place, clock)
        .map_err(|e| format!("{}'s year: {e}", sky.name()))?;
    let closed = Rite::ALL
        .iter()
        .map(|rite| {
            let rules = rite.activity().rules();
            if rules.heeds.is_empty() {
                return Ok(BTreeMap::new());
            }
            let request = MuhurtaRequest::new(rules).with_windows_on(1).at_most(1);
            let answer = context
                .almanac()
                .muhurta(&first, &last, &place, clock, &request)
                .map_err(|e| format!("{}'s season: {e}", rite.name()))?;
            Ok(answer
                .value
                .closed
                .iter()
                .map(|day| {
                    let by = day.by.iter().map(|kind| kind.key().to_owned()).collect();
                    (iso(&day.date), by)
                })
                .collect())
        })
        .collect::<Result<_, String>>()?;
    Ok(Year {
        days: days.value,
        closed,
    })
}

/// How many workers search the printed days' windows, each with its own
/// contexts, since a context is not shared between threads.
const WORKERS: usize = 4;

/// Each task's instant verdict, a task a sky, a set of rules and a date,
/// spread over [`WORKERS`] threads.
fn instants(tasks: &[(Sky, ActivityRules, CalendarDate)]) -> Result<Vec<Verdict>, String> {
    let chunk = tasks.len().div_ceil(WORKERS).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = tasks
            .chunks(chunk)
            .map(|part| {
                scope.spawn(move || {
                    let mut contexts: [Option<Context>; 2] = [None, None];
                    part.iter()
                        .map(|(sky, rules, date)| {
                            let slot = contexts
                                .get_mut(sky.index())
                                .ok_or("a sky without a slot")?;
                            if slot.is_none() {
                                *slot = Some(sky.context()?);
                            }
                            let context = slot.as_ref().ok_or("a context")?;
                            instant(context, rules, date)
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
            })
            .collect();
        let mut all = Vec::new();
        for worker in workers {
            all.extend(worker.join().map_err(|_| "a worker panicked")??);
        }
        Ok(all)
    })
}

/// Raman's marriage with the sages' Pausha he reports and does not take:
/// the month allowed, middling, while the Sun is in Capricorn.
fn with_the_sages_pausha() -> ActivityRules {
    let mut rules = ActivityRules::raman_marriage();
    if let MonthRule::Lunar { with_sun, .. } = &mut rules.months {
        with_sun.push(MonthWithSun {
            masa: Masa::Pausha,
            sun: Rashi::Capricorn,
        });
    }
    rules
}

/// What the page measures.
struct Measured<'a> {
    found: Vec<Found<'a>>,
    /// Each sky's open days of the year, rite by rite in [`Rite::ALL`]'s
    /// order, and the days founded.
    open: [Vec<usize>; 2],
    days: usize,
    /// The printed marriages' days the sages' Pausha leaves open over the
    /// committee's sky, and those Raman's own rules leave open.
    pausha: (usize, usize),
}

/// A day of the sheet read against each sky's day rules: the rows, the
/// days left for the instant's search, and each row's day verdicts.
type DayLevel = (
    Vec<Found<'static>>,
    Vec<(Sky, ActivityRules, CalendarDate)>,
    Vec<Option<[Verdict; 2]>>,
);

/// Each printed row converted, its weekday checked against `MISPRINTS`
/// both ways, and judged at the day level under each sky.
fn day_level(years: &[Year; 2], rules: &[ActivityRules]) -> Result<DayLevel, String> {
    let calendar_context = Sky::Committee.context()?;
    let calendar = calendar_context.calendar();

    // The day level from each sky's year, and the windows to search.
    let mut problems = Vec::new();
    let mut found = Vec::new();
    let mut tasks = Vec::new();
    let mut days_of: Vec<Option<[Verdict; 2]>> = Vec::new();
    for printed in &SHEET {
        let bs = CalendarDate::defined(Calendar::BikramSambat, YEAR, printed.month, printed.day);
        let date = calendar
            .convert(&bs, Calendar::Gregorian)
            .map_err(|e| format!("BS {YEAR}-{}-{}: {e}", printed.month, printed.day))?;
        let fixed = calendar.fixed_of(&date).map_err(|e| e.to_string())?;
        let vara = Vara::from_id(fixed.weekday() as u16).unwrap_or(Vara::Ravivara);
        let listed = MISPRINTS.contains(&(printed.rite, printed.month, printed.day));
        let misprinted = vara != printed.vara;
        if listed != misprinted {
            problems.push(format!(
                "{} BS {YEAR}-{}-{}: printed {}, the date's {}, and {} in MISPRINTS",
                printed.rite.name(),
                printed.month,
                printed.day,
                printed.vara.key(),
                vara.key(),
                if listed { "listed" } else { "not listed" }
            ));
        }
        let rite_rules = rules.get(printed.rite.index()).ok_or("a rite's rules")?;
        let mut verdicts = Vec::new();
        for sky in Sky::ALL {
            let year = &years[sky.index()];
            let closed = year
                .closed
                .get(printed.rite.index())
                .and_then(|closed| closed.get(&iso(&date)));
            let verdict = if let Some(by) = closed {
                Verdict::Closed(by.clone())
            } else {
                let day = year
                    .day(&date)
                    .ok_or_else(|| format!("{}: not in the year", iso(&date)))?;
                day_verdict(day, rite_rules)
            };
            if !misprinted && verdict == Verdict::Clean {
                tasks.push((sky, rite_rules.clone(), date.clone()));
            }
            verdicts.push(verdict);
        }
        days_of.push(if misprinted {
            None
        } else {
            Some(verdicts.try_into().map_err(|_| "two skies")?)
        });
        found.push(Found {
            printed,
            date,
            vara,
            readings: None,
        });
    }
    if !problems.is_empty() {
        return Err(problems.join("\n      "));
    }
    Ok((found, tasks, days_of))
}

fn page() -> Result<String, String> {
    let years = std::thread::scope(|scope| {
        let workers: Vec<_> = Sky::ALL
            .iter()
            .map(|sky| scope.spawn(move || year(*sky)))
            .collect();
        workers
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .map_err(|_| String::from("a worker panicked"))?
            })
            .collect::<Result<Vec<Year>, String>>()
    })?;
    let years: [Year; 2] = years.try_into().map_err(|_| "two skies")?;
    let rules: Vec<ActivityRules> = Rite::ALL.iter().map(|r| r.activity().rules()).collect();
    let (mut found, tasks, days_of) = day_level(&years, &rules)?;

    // The instant on the days the day rules leave open.
    let mut instants = instants(&tasks)?.into_iter();
    for (f, days) in found.iter_mut().zip(days_of) {
        let Some(days) = days else {
            continue;
        };
        let mut readings = Vec::new();
        for day in days {
            let instant = if day == Verdict::Clean {
                Some(instants.next().ok_or("an instant short")?)
            } else {
                None
            };
            readings.push(Reading { day, instant });
        }
        f.readings = Some(readings.try_into().map_err(|_| "two skies")?);
    }

    let open = years
        .iter()
        .map(|year| {
            Rite::ALL
                .iter()
                .zip(&rules)
                .map(|(rite, rules)| year.open(*rite, rules))
                .collect()
        })
        .collect::<Vec<Vec<usize>>>()
        .try_into()
        .map_err(|_| "two skies")?;
    let committee = &years[Sky::Committee.index()];
    let pausha = pausha(&found, committee)?;
    Ok(render(&Measured {
        found,
        open,
        days: committee.days.len(),
        pausha,
    }))
}

/// The printed marriages' days the sages' Pausha and Raman's own rules
/// each leave day-clean over the committee's sky.
fn pausha(found: &[Found<'_>], committee: &Year) -> Result<(usize, usize), String> {
    let sages = with_the_sages_pausha();
    let raman = ActivityRules::raman_marriage();
    let mut pausha = (0, 0);
    for f in found
        .iter()
        .filter(|f| f.printed.rite == Rite::Vivaha && f.readings.is_some())
    {
        let day = committee.day(&f.date).ok_or("a printed day")?;
        pausha.0 += usize::from(day_verdict(day, &sages) == Verdict::Clean);
        pausha.1 += usize::from(day_verdict(day, &raman) == Verdict::Clean);
    }
    Ok(pausha)
}

fn render(m: &Measured<'_>) -> String {
    let mut out = String::from(
        "# Saait, measured\n\n\
         Status: `generated` by `cargo xtask saait`. Do not edit:\n\
         `check-saait` regenerates this page and fails on any difference.\n\n\
         It measures `muhurta.md` §4.6: the rites Raman's *Muhurtha* gives\n\
         beyond marriage, and his marriage, against the days the Nepal Panchanga\n\
         Nirnayak Samiti printed for them on its VS 2083 muhurta sheet. Each\n\
         printed day is founded at Kathmandu (27.7172° N, 85.324° E, 1400 m) on\n\
         Nepal's clock, over the committee's own sky (`nepali-committee`, the\n\
         Surya Siddhanta) and the modern one (`nepali-default`), and searched\n\
         with the rite's shipped rules. A day is **day-clean** when the rite's\n\
         grades of the tithi, nakshatra, yoga, karana, vara and month leave some\n\
         of its daylight unrejected, and **clean** when some window of that\n\
         daylight is also struck by no bar and rejected by no lagna or quarter.\n\
         Where a day is not, the clauses named are those rejecting every\n\
         window, or, where no one clause does, each window's.\n\n\
         ## 1. The sheet\n\n",
    );
    let misprints: Vec<String> = m
        .found
        .iter()
        .filter(|f| f.vara != f.printed.vara)
        .map(|f| {
            format!(
                "{} {}-{} (printed {}, the date's {})",
                f.printed.rite.name(),
                f.printed.month,
                f.printed.day,
                f.printed.vara.key(),
                f.vara.key()
            )
        })
        .collect();
    let per_rite: Vec<String> = Rite::ALL
        .iter()
        .map(|rite| {
            let n = m.found.iter().filter(|f| f.printed.rite == *rite).count();
            format!("{n} {}", rite.name())
        })
        .collect();
    let _ = writeln!(
        out,
        "The sheet prints {} rows ({}), each a Bikram Sambat date and its\n\
         weekday, read off the page images. Each date is converted by the shipped\n\
         Bikram Sambat calendar and its weekday compared with the one printed;\n\
         a row whose two disagree contradicts itself and is read neither way.\n",
        SHEET.len(),
        per_rite.join(", ")
    );
    out.push_str(&table(&[Claim::counted(
        "every printed weekday is its date's",
        misprints.len(),
        SHEET.len(),
    )
    .with_note(format!(
        "the pass refuses a change to these: {}",
        misprints.join("; ")
    ))]));
    each_rite(&mut out, m);
    what_parts_them(&mut out, m);
    every_row(&mut out, m);
    out
}

/// §2: each rite's printed days and the year's open days, by sky.
fn each_rite(out: &mut String, m: &Measured<'_>) {
    out.push_str(
        "\n## 2. Each rite against its printed days\n\n\
         The year is VS 2083's days at Kathmandu; a day the rite's season closes\n\
         is not open. A printed day the day rules leave open is necessarily one\n\
         of the year's open days, so the year's count says how much the rules\n\
         narrow the year before the committee chose.\n\n\
         | rite | rules | sky | printed and read | day-clean | clean | open days of the year |\n\
         |---|---|---|---:|---:|---:|---:|\n",
    );
    for rite in Rite::ALL {
        for sky in Sky::ALL {
            let read: Vec<&Reading> = m
                .found
                .iter()
                .filter(|f| f.printed.rite == rite)
                .filter_map(|f| f.reading(sky))
                .collect();
            let day_clean = read.iter().filter(|r| r.day == Verdict::Clean).count();
            let clean = read
                .iter()
                .filter(|r| r.instant == Some(Verdict::Clean))
                .count();
            let open = m.open[sky.index()].get(rite.index()).copied().unwrap_or(0);
            let _ = writeln!(
                out,
                "| {} | `{}` | {} | {} | {day_clean} | {clean} | {open} of {} |",
                rite.name(),
                rite.activity().key(),
                sky.name(),
                read.len(),
                m.days,
            );
        }
    }
    out.push('\n');
    let claims: Vec<Claim> = Rite::ALL
        .iter()
        .map(|rite| {
            let read: Vec<&Reading> = m
                .found
                .iter()
                .filter(|f| f.printed.rite == *rite)
                .filter_map(|f| f.reading(Sky::Committee))
                .collect();
            let unclean = read
                .iter()
                .filter(|r| r.instant != Some(Verdict::Clean))
                .count();
            Claim::counted(
                format!(
                    "over the committee's sky, every printed {} day has a clean window",
                    rite.name()
                ),
                unclean,
                read.len(),
            )
        })
        .collect();
    out.push_str(&table(&claims));
}

/// §3: the clauses that part a printed day from the rules.
fn what_parts_them(out: &mut String, m: &Measured<'_>) {
    out.push_str(
        "\n## 3. What parts them\n\n\
         Over the committee's sky, each clause that rejects every daylight\n\
         window of a printed day, counted over the days it does: a day-level\n\
         clause on a day not day-clean, an instant one on a day-clean day with\n\
         no clean window. A day whose windows are each rejected by a different\n\
         clause is counted under \"no one clause\".\n\n\
         | rite | clause | printed days |\n|---|---|---:|\n",
    );
    for rite in Rite::ALL {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in m.found.iter().filter(|f| f.printed.rite == rite) {
            let Some(read) = f.reading(Sky::Committee) else {
                continue;
            };
            let verdict = read.instant.as_ref().unwrap_or(&read.day);
            match verdict {
                Verdict::Clean => {}
                Verdict::Closed(by) => {
                    *counts
                        .entry(format!("closed by {}", by.join(", ")))
                        .or_default() += 1;
                }
                Verdict::Rejected { every, .. } if every.is_empty() => {
                    *counts.entry(String::from("no one clause")).or_default() += 1;
                }
                Verdict::Rejected { every, .. } => {
                    for clause in every {
                        *counts.entry(clause.clone()).or_default() += 1;
                    }
                }
            }
        }
        let mut counts: Vec<(String, usize)> = counts.into_iter().collect();
        counts.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        for (clause, n) in counts {
            let _ = writeln!(out, "| {} | {clause} | {n} |", rite.name());
        }
    }
    let _ = write!(
        out,
        "\nRaman reports that \"some sages\" allow a marriage in Pausha while the\n\
         Sun is in Capricorn, and does not take it (`ActivityRules::raman_marriage`).\n\
         Read with it, over the committee's sky, {} of the printed marriages'\n\
         days are day-clean, against {} under his rules as shipped.\n",
        m.pausha.0, m.pausha.1
    );
}

/// §4: every row of the sheet under both skies.
fn every_row(out: &mut String, m: &Measured<'_>) {
    out.push_str(
        "\n## 4. Every row\n\n\
         | rite | BS 2083 | day | printed | committee's sky | modern sky |\n\
         |---|---|---|---|---|---|\n",
    );
    for f in &m.found {
        let shown = |sky| {
            f.reading(sky)
                .map_or_else(|| String::from("—"), Reading::shown)
        };
        let printed = if f.vara == f.printed.vara {
            f.printed.vara.key().to_owned()
        } else {
            format!("{} (the date's {})", f.printed.vara.key(), f.vara.key())
        };
        let _ = writeln!(
            out,
            "| {} | {}-{:02} | {} | {printed} | {} | {} |",
            f.printed.rite.name(),
            f.printed.month,
            f.printed.day,
            iso(&f.date),
            shown(Sky::Committee),
            shown(Sky::Modern),
        );
    }
}
