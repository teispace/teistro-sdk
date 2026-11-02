//! The measurement pass over the **saait packs**
//! (`03-design/muhurta.md` §4.6): the rites held to the days Nepal's
//! panchanga printed for them.
//!
//! The national panchanga committee's VS 2083 muhurta sheet prints, by
//! Bikram Sambat date and weekday, the days of the year it finds for a
//! thread ceremony (bratabandha), a marriage (vivaha), the first feeding
//! (pasni), a house entry (griha pravesh), the laying of a house's
//! foundation (griharambha) and opening a shop (dokan). The Surya
//! Panchanga's VS 2082 page of "the days of the various saaits", which
//! its preface says the committee decided, prints the same three house
//! and shop rites for the year before, most rows without a weekday. Each
//! row, read off the page image, is founded at Kathmandu on Nepal's clock
//! and searched with each set of the SDK's rules for its rite — Raman's
//! *Muhurtha* and *Muhurta Chintamani* — over the committee's own sky and
//! the modern one. A printed day **agrees** when the search finds a clean
//! window in its daylight: one no bar strikes and no grade of the rite's
//! own rejects. A day that does not is attributed to the clauses that
//! reject every daylight window it has.
//!
//! A printed weekday is checked against its date first; a row whose two
//! disagree contradicts itself, and is listed by name rather than read
//! either way.
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

/// The Bikram Sambat year of the committee's sheet.
const YEAR: i32 = 2083;

/// The Bikram Sambat year of the Surya Panchanga's lists.
const YEAR_BEFORE: i32 = 2082;

/// The years the rows are of, each founded whole under each sky.
const YEARS: [i32; 2] = [YEAR_BEFORE, YEAR];

/// A rite a list prints days for.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Rite {
    Bratabandha,
    Vivaha,
    Pasni,
    GrihaPravesh,
    Griharambha,
    Dokan,
}

impl Rite {
    /// Its name on the page.
    const fn name(self) -> &'static str {
        match self {
            Rite::Bratabandha => "bratabandha",
            Rite::Vivaha => "vivaha",
            Rite::Pasni => "pasni",
            Rite::GrihaPravesh => "griha pravesh",
            Rite::Griharambha => "griharambha",
            Rite::Dokan => "dokan",
        }
    }
}

/// A rite's printed days read under one set of the SDK's rules: each
/// rite under the rules shipped for it, and the marriage under both
/// Raman's and *Muhurta Chintamani*'s (crux C206).
const TESTS: [(Rite, Activity); 7] = [
    (Rite::Bratabandha, Activity::RamanUpanayana),
    (Rite::Vivaha, Activity::RamanMarriage),
    (Rite::Vivaha, Activity::ChintamaniVivaha),
    (Rite::Pasni, Activity::RamanAnnaprasana),
    (Rite::GrihaPravesh, Activity::RamanGrihaPravesha),
    (Rite::Griharambha, Activity::ChintamaniGriharambha),
    (Rite::Dokan, Activity::ChintamaniVipani),
];

/// The rite and the rules of a test, by its place in [`TESTS`].
fn test(index: usize) -> (Rite, Activity) {
    TESTS
        .get(index)
        .copied()
        .unwrap_or((Rite::Vivaha, Activity::RamanMarriage))
}

/// One printed row: the Bikram Sambat year, the rite, the month and day,
/// and the weekday printed beside it, where one is.
struct Printed {
    year: i32,
    rite: Rite,
    month: u8,
    day: u8,
    vara: Option<Vara>,
}

impl Printed {
    /// The list it is printed in.
    const fn list(&self) -> &'static str {
        if self.year == YEAR {
            "the committee's sheet"
        } else {
            "the Surya Panchanga"
        }
    }
}

/// A row of the committee's VS 2083 sheet, which prints every weekday.
const fn row(rite: Rite, month: u8, day: u8, vara: Vara) -> Printed {
    Printed {
        year: YEAR,
        rite,
        month,
        day,
        vara: Some(vara),
    }
}

/// A row of the Surya Panchanga's VS 2082 lists, the weekday where
/// printed.
const fn before(rite: Rite, month: u8, day: u8, vara: Option<Vara>) -> Printed {
    Printed {
        year: YEAR_BEFORE,
        rite,
        month,
        day,
        vara,
    }
}

use Rite::{
    Bratabandha as B, Dokan as D, GrihaPravesh as G, Griharambha as H, Pasni as P, Vivaha as V,
};
use Vara::{
    Budhavara as WED, Guruvara as THU, Mangalavara as TUE, Ravivara as SUN, Shanivara as SAT,
    Shukravara as FRI, Somavara as MON,
};

/// The committee's VS 2083 muhurta sheet, its tables in the order
/// printed: each rite's months left to right, each month's days top to
/// bottom.
static SHEET: [Printed; 148] = [
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
    row(H, 1, 21, MON),
    row(H, 8, 5, SAT),
    row(H, 8, 9, WED),
    row(H, 8, 10, THU),
    row(H, 9, 1, WED),
    row(H, 9, 4, SAT),
    row(H, 9, 11, SAT),
    row(H, 11, 7, FRI),
    row(H, 11, 13, THU),
    row(D, 1, 7, MON),
    row(D, 1, 21, MON),
    row(D, 1, 25, FRI),
    row(D, 1, 31, THU),
    row(D, 3, 10, WED),
    row(D, 3, 13, SAT),
    row(D, 3, 18, THU),
    row(D, 5, 1, MON),
    row(D, 5, 4, THU),
    row(D, 5, 14, SUN),
    row(D, 5, 16, TUE),
    row(D, 5, 19, FRI),
    row(D, 5, 28, SUN),
    row(D, 5, 29, MON),
    row(D, 6, 1, THU),
    row(D, 6, 5, MON),
    row(D, 7, 15, SUN),
    row(D, 7, 16, MON),
    row(D, 7, 19, THU),
    row(D, 7, 29, SUN),
    row(D, 8, 5, SAT),
    row(D, 8, 6, SUN),
    row(D, 8, 9, WED),
    row(D, 8, 10, THU),
    row(D, 8, 13, SUN),
    row(D, 8, 17, THU),
    row(D, 8, 19, SAT),
    row(D, 8, 26, SAT),
    row(D, 9, 4, SAT),
    row(D, 9, 11, SAT),
    row(D, 9, 15, WED),
    row(D, 9, 25, SAT),
    row(D, 10, 9, SAT),
    row(D, 10, 14, THU),
    row(D, 10, 29, FRI),
    row(D, 11, 7, FRI),
    row(D, 11, 13, THU),
    row(D, 11, 20, THU),
    row(D, 11, 26, WED),
    row(D, 12, 4, THU),
    row(D, 12, 10, WED),
    row(D, 12, 18, THU),
    row(D, 12, 25, THU),
    row(D, 12, 28, SUN),
    row(D, 12, 29, MON),
];

/// The Surya Panchanga's VS 2082 lists (printed p. 80, "विभिन्न साइतका
/// दिनहरू"): the house's foundation, the house entry and the shop, each
/// month's days in the order printed.
static SURYA_2082: [Printed; 41] = [
    before(H, 4, 26, Some(MON)),
    before(H, 7, 21, Some(FRI)),
    before(H, 8, 11, Some(THU)),
    before(H, 8, 19, Some(FRI)),
    before(H, 8, 20, Some(SAT)),
    before(H, 11, 14, Some(THU)),
    before(G, 10, 28, Some(WED)),
    before(G, 11, 14, Some(THU)),
    before(G, 11, 30, Some(SAT)),
    before(D, 1, 7, None),
    before(D, 1, 8, None),
    before(D, 1, 17, Some(WED)),
    before(D, 2, 4, Some(SUN)),
    before(D, 2, 14, Some(WED)),
    before(D, 2, 17, Some(SAT)),
    before(D, 3, 27, None),
    before(D, 3, 28, None),
    before(D, 4, 5, None),
    before(D, 4, 29, None),
    before(D, 5, 9, None),
    before(D, 5, 15, None),
    before(D, 6, 11, None),
    before(D, 6, 25, None),
    before(D, 6, 26, None),
    before(D, 7, 7, None),
    before(D, 7, 17, None),
    before(D, 7, 21, None),
    before(D, 7, 29, None),
    before(D, 7, 30, None),
    before(D, 8, 14, None),
    before(D, 8, 19, None),
    before(D, 8, 20, None),
    before(D, 10, 24, None),
    before(D, 10, 28, None),
    before(D, 11, 14, None),
    before(D, 11, 30, None),
    before(D, 12, 6, None),
    before(D, 12, 11, None),
    before(D, 12, 14, None),
    before(D, 12, 27, None),
    before(D, 12, 28, None),
];

/// Every printed row, the committee's first.
fn rows() -> impl Iterator<Item = &'static Printed> {
    SHEET.iter().chain(&SURYA_2082)
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

/// A sky the days are founded under.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sky {
    /// `nepali-committee`, the committee's own.
    Committee,
    /// `nepali-drik`, the modern sky under Lahiri.
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
            Sky::Modern => ("nepali-drik", Ephemeris::Builtin),
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
    // The bull's count moves through a day's windows, so its rejection
    // is named by the clause alone and a day all of whose windows fall on
    // the bad limbs reads as one reason.
    if let (ClauseKind::VrishaVastu { grade, .. }, Level::Instant) = (kind, level) {
        return rejected(grade).then(|| key_name(kind.key()));
    }
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

/// The rows whose printed weekday is not their date's, each as (year,
/// rite, month, day): the list contradicts itself there, and the row is
/// read neither way. Refused both ways, so a row added to the list that
/// agrees, or a row that disagrees and is not listed, fails the pass.
const MISPRINTS: [(i32, Rite, u8, u8); 6] = [
    (YEAR, B, 10, 21),
    (YEAR, B, 10, 23),
    (YEAR, B, 12, 4),
    (YEAR, V, 3, 14),
    (YEAR, V, 3, 15),
    (YEAR, P, 10, 25),
];

/// One printed row read under one test, and what was found of it.
struct Found {
    printed: &'static Printed,
    /// Its test's place in [`TESTS`].
    test: usize,
    /// The Gregorian day.
    date: CalendarDate,
    /// The weekday the date has.
    vara: Vara,
    /// Each sky's reading, in [`Sky::ALL`]'s order, on a row whose
    /// weekday agrees.
    readings: Option<[Reading; 2]>,
}

impl Found {
    fn reading(&self, sky: Sky) -> Option<&Reading> {
        self.readings.as_ref().and_then(|r| r.get(sky.index()))
    }

    /// Whether the printed weekday contradicts the date.
    fn misprinted(&self) -> bool {
        self.printed.vara.is_some_and(|vara| vara != self.vara)
    }
}

/// A date as the page spells it.
fn iso(date: &CalendarDate) -> String {
    format!("{}-{:02}-{:02}", date.year, date.month, date.day)
}

/// A Bikram Sambat year's first and last days, as Gregorian dates.
fn the_year(context: &Context, bs: i32) -> Result<(CalendarDate, CalendarDate), String> {
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
        .fixed_of(&first_of(bs + 1)?)
        .map_err(|e| e.to_string())?;
    let last = calendar
        .date_of(Calendar::Gregorian, next.plus_days(-1))
        .map_err(|e| e.to_string())?;
    Ok((first_of(bs)?, last))
}

/// A sky's Bikram Sambat year: every day of it, and, test by test, the
/// days the rules' season closes with the blackouts that close them.
struct Year {
    sky: Sky,
    bs: i32,
    days: Vec<Panchanga>,
    closed: Vec<BTreeMap<String, Vec<String>>>,
}

impl Year {
    /// The day of a date.
    fn day(&self, date: &CalendarDate) -> Option<&Panchanga> {
        let date = iso(date);
        self.days.iter().find(|day| iso(&day.day.date) == date)
    }

    /// The days a test's rules leave daylight open on, the season's
    /// closed days left out.
    fn open(&self, test: usize, rules: &ActivityRules) -> usize {
        let closed = self.closed.get(test);
        self.days
            .iter()
            .filter(|day| !closed.is_some_and(|c| c.contains_key(&iso(&day.day.date))))
            .filter(|day| day_verdict(day, rules) == Verdict::Clean)
            .count()
    }
}

/// The founded year of a sky and a Bikram Sambat year.
fn year_of(years: &[Year], sky: Sky, bs: i32) -> Result<&Year, String> {
    years
        .iter()
        .find(|y| y.sky == sky && y.bs == bs)
        .ok_or_else(|| format!("{}'s VS {bs} not founded", sky.name()))
}

/// Founds a sky's Bikram Sambat year. The season is searched only for
/// rules that heed one; rules heeding none close nothing.
fn year(sky: Sky, bs: i32) -> Result<Year, String> {
    let context = sky.context()?;
    let (place, clock) = kathmandu();
    let (first, last) = the_year(&context, bs)?;
    let days = context
        .almanac()
        .of(&first, &last, &place, clock)
        .map_err(|e| format!("{}'s VS {bs}: {e}", sky.name()))?;
    let closed = TESTS
        .iter()
        .map(|(rite, activity)| {
            let rules = activity.rules();
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
        sky,
        bs,
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
struct Measured {
    found: Vec<Found>,
    /// The open days of each founded year, test by test in [`TESTS`]'s
    /// order, with the days founded.
    open: Vec<(Sky, i32, Vec<usize>, usize)>,
    /// The printed marriages' days the sages' Pausha leaves open over the
    /// committee's sky, and those Raman's own rules leave open.
    pausha: (usize, usize),
}

/// The rows read against each sky's day rules: the rows under each test,
/// the days left for the instant's search, and each row's day verdicts.
type DayLevel = (
    Vec<Found>,
    Vec<(Sky, ActivityRules, CalendarDate)>,
    Vec<Option<[Verdict; 2]>>,
);

/// Each printed row converted, its weekday checked against `MISPRINTS`
/// both ways, and judged under each of its rite's tests at the day level
/// under each sky.
fn day_level(years: &[Year], rules: &[ActivityRules]) -> Result<DayLevel, String> {
    let calendar_context = Sky::Committee.context()?;
    let calendar = calendar_context.calendar();

    let mut problems = Vec::new();
    let mut found = Vec::new();
    let mut tasks = Vec::new();
    let mut days_of: Vec<Option<[Verdict; 2]>> = Vec::new();
    for printed in rows() {
        let bs = CalendarDate::defined(
            Calendar::BikramSambat,
            printed.year,
            printed.month,
            printed.day,
        );
        let date = calendar
            .convert(&bs, Calendar::Gregorian)
            .map_err(|e| format!("BS {}-{}-{}: {e}", printed.year, printed.month, printed.day))?;
        let fixed = calendar.fixed_of(&date).map_err(|e| e.to_string())?;
        let vara = Vara::from_id(fixed.weekday() as u16).unwrap_or(Vara::Ravivara);
        let listed = MISPRINTS.contains(&(printed.year, printed.rite, printed.month, printed.day));
        let misprinted = printed.vara.is_some_and(|v| v != vara);
        if listed != misprinted {
            problems.push(format!(
                "{} BS {}-{}-{}: printed {}, the date's {}, and {} in MISPRINTS",
                printed.rite.name(),
                printed.year,
                printed.month,
                printed.day,
                printed.vara.map_or("no weekday", |v| v.key()),
                vara.key(),
                if listed { "listed" } else { "not listed" }
            ));
        }
        for (index, (rite, _)) in TESTS.iter().enumerate() {
            if *rite != printed.rite {
                continue;
            }
            let test_rules = rules.get(index).ok_or("a test's rules")?;
            let mut verdicts = Vec::new();
            for sky in Sky::ALL {
                let year = year_of(years, sky, printed.year)?;
                let closed = year
                    .closed
                    .get(index)
                    .and_then(|closed| closed.get(&iso(&date)));
                let verdict = if let Some(by) = closed {
                    Verdict::Closed(by.clone())
                } else {
                    let day = year
                        .day(&date)
                        .ok_or_else(|| format!("{}: not in the year", iso(&date)))?;
                    day_verdict(day, test_rules)
                };
                if !misprinted && verdict == Verdict::Clean {
                    tasks.push((sky, test_rules.clone(), date.clone()));
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
                test: index,
                date: date.clone(),
                vara,
                readings: None,
            });
        }
    }
    if !problems.is_empty() {
        return Err(problems.join("\n      "));
    }
    Ok((found, tasks, days_of))
}

fn page() -> Result<String, String> {
    let founding: Vec<(Sky, i32)> = Sky::ALL
        .iter()
        .flat_map(|sky| YEARS.iter().map(move |bs| (*sky, *bs)))
        .collect();
    let years = std::thread::scope(|scope| {
        let workers: Vec<_> = founding
            .iter()
            .map(|(sky, bs)| scope.spawn(move || year(*sky, *bs)))
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
    let rules: Vec<ActivityRules> = TESTS.iter().map(|(_, a)| a.rules()).collect();
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
            let open = rules
                .iter()
                .enumerate()
                .map(|(test, rules)| year.open(test, rules))
                .collect();
            (year.sky, year.bs, open, year.days.len())
        })
        .collect();
    let committee = year_of(&years, Sky::Committee, YEAR)?;
    let pausha = pausha(&found, committee)?;
    Ok(render(&Measured {
        found,
        open,
        pausha,
    }))
}

/// The printed marriages' days the sages' Pausha and Raman's own rules
/// each leave day-clean over the committee's sky.
fn pausha(found: &[Found], committee: &Year) -> Result<(usize, usize), String> {
    let sages = with_the_sages_pausha();
    let raman = ActivityRules::raman_marriage();
    let mut pausha = (0, 0);
    for f in found.iter().filter(|f| {
        test(f.test).1 == Activity::RamanMarriage && f.printed.year == YEAR && f.readings.is_some()
    }) {
        let day = committee.day(&f.date).ok_or("a printed day")?;
        pausha.0 += usize::from(day_verdict(day, &sages) == Verdict::Clean);
        pausha.1 += usize::from(day_verdict(day, &raman) == Verdict::Clean);
    }
    Ok(pausha)
}

fn render(m: &Measured) -> String {
    let mut out = String::from(
        "# Saait, measured\n\n\
         Status: `generated` by `cargo xtask saait`. Do not edit:\n\
         `check-saait` regenerates this page and fails on any difference.\n\n\
         It measures `muhurta.md` §4.6: the rites Raman's *Muhurtha* gives,\n\
         his marriage, and *Muhurta Chintamani*'s house, shop and marriage\n\
         rites, against the days the Nepal Panchanga Nirnayak Samiti printed\n\
         for them on its VS 2083 muhurta sheet, and the Surya Panchanga's VS\n\
         2082 lists of the house's foundation, the house entry and the shop,\n\
         which its preface says the committee decided. Each printed day is\n\
         founded at Kathmandu (27.7172° N, 85.324° E, 1400 m) on Nepal's\n\
         clock, over the committee's own sky (`nepali-committee`, the Surya\n\
         Siddhanta) and the modern one (`nepali-drik`), and searched with each\n\
         of the rite's shipped rules. A day is **day-clean** when the rules'\n\
         grades of the tithi, nakshatra, yoga, karana, vara and month leave some\n\
         of its daylight unrejected, and **clean** when some window of that\n\
         daylight is also struck by no bar and rejected by no lagna, quarter or\n\
         bull of the house. Where a day is not, the clauses named are those\n\
         rejecting every window, or, where no one clause does, each window's.\n\n\
         ## 1. The lists\n\n",
    );
    let mut misprints = BTreeSet::new();
    for f in m.found.iter().filter(|f| f.misprinted()) {
        misprints.insert(format!(
            "{} {}-{}-{} (printed {}, the date's {})",
            f.printed.rite.name(),
            f.printed.year,
            f.printed.month,
            f.printed.day,
            f.printed.vara.map_or("none", |v| v.key()),
            f.vara.key()
        ));
    }
    let misprints: Vec<String> = misprints.into_iter().collect();
    let mut per_list: BTreeMap<(i32, Rite), usize> = BTreeMap::new();
    for printed in rows() {
        *per_list.entry((printed.year, printed.rite)).or_default() += 1;
    }
    let counted = |year: i32| -> String {
        per_list
            .iter()
            .filter(|((y, _), _)| *y == year)
            .map(|((_, rite), n)| format!("{n} {}", rite.name()))
            .collect::<Vec<_>>()
            .join(", ")
    };
    let weekdays = rows().filter(|p| p.vara.is_some()).count();
    let _ = writeln!(
        out,
        "The committee's sheet prints {} rows ({}), each a Bikram Sambat date\n\
         and its weekday; the Surya Panchanga's lists {} ({}), {} of them with\n\
         a weekday. All are read off the page images. Each date is converted by\n\
         the shipped Bikram Sambat calendar and a printed weekday compared with\n\
         its date's; a row whose two disagree contradicts itself and is read\n\
         neither way.\n",
        SHEET.len(),
        counted(YEAR),
        SURYA_2082.len(),
        counted(YEAR_BEFORE),
        SURYA_2082.iter().filter(|p| p.vara.is_some()).count(),
    );
    out.push_str(&table(&[Claim::counted(
        "every printed weekday is its date's",
        misprints.len(),
        weekdays,
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

/// The readings of a test's rows printed in a year under a sky.
fn readings(m: &Measured, test: usize, year: Option<i32>, sky: Sky) -> Vec<&Reading> {
    m.found
        .iter()
        .filter(|f| f.test == test && year.is_none_or(|y| f.printed.year == y))
        .filter_map(|f| f.reading(sky))
        .collect()
}

/// §2: each test's printed days and the year's open days, by list and
/// sky.
fn each_rite(out: &mut String, m: &Measured) {
    out.push_str(
        "\n## 2. Each rite against its printed days\n\n\
         A year is its Bikram Sambat year's days at Kathmandu; a day the rules'\n\
         season closes is not open. A printed day the day rules leave open is\n\
         necessarily one of the year's open days, so the year's count says how\n\
         much the rules narrow the year before the list chose.\n\n\
         | rite | rules | VS | sky | printed and read | day-clean | clean | open days of the year |\n\
         |---|---|---:|---|---:|---:|---:|---:|\n",
    );
    for (index, (rite, activity)) in TESTS.iter().enumerate() {
        for bs in YEARS {
            if !m
                .found
                .iter()
                .any(|f| f.test == index && f.printed.year == bs)
            {
                continue;
            }
            for sky in Sky::ALL {
                let read = readings(m, index, Some(bs), sky);
                let day_clean = read.iter().filter(|r| r.day == Verdict::Clean).count();
                let clean = read
                    .iter()
                    .filter(|r| r.instant == Some(Verdict::Clean))
                    .count();
                let (open, days) = m
                    .open
                    .iter()
                    .find(|(s, y, _, _)| *s == sky && *y == bs)
                    .map_or((0, 0), |(_, _, open, days)| {
                        (open.get(index).copied().unwrap_or(0), *days)
                    });
                let _ = writeln!(
                    out,
                    "| {} | `{}` | {bs} | {} | {} | {day_clean} | {clean} | {open} of {days} |",
                    rite.name(),
                    activity.key(),
                    sky.name(),
                    read.len(),
                );
            }
        }
    }
    out.push('\n');
    let claims: Vec<Claim> = TESTS
        .iter()
        .enumerate()
        .map(|(index, (rite, activity))| {
            let read = readings(m, index, None, Sky::Committee);
            let unclean = read
                .iter()
                .filter(|r| r.instant != Some(Verdict::Clean))
                .count();
            Claim::counted(
                format!(
                    "over the committee's sky, every printed {} day has a clean window under `{}`",
                    rite.name(),
                    activity.key()
                ),
                unclean,
                read.len(),
            )
        })
        .collect();
    out.push_str(&table(&claims));
}

/// §3: the clauses that part a printed day from the rules.
fn what_parts_them(out: &mut String, m: &Measured) {
    out.push_str(
        "\n## 3. What parts them\n\n\
         Over the committee's sky, each clause that rejects every daylight\n\
         window of a printed day, counted over the days it does: a day-level\n\
         clause on a day not day-clean, an instant one on a day-clean day with\n\
         no clean window. A day whose windows are each rejected by a different\n\
         clause is counted under \"no one clause\".\n\n\
         | rite | rules | clause | printed days |\n|---|---|---|---:|\n",
    );
    for (index, (rite, activity)) in TESTS.iter().enumerate() {
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for f in m.found.iter().filter(|f| f.test == index) {
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
            let _ = writeln!(
                out,
                "| {} | `{}` | {clause} | {n} |",
                rite.name(),
                activity.key()
            );
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

/// §4: every row under each of its rite's rules and both skies.
fn every_row(out: &mut String, m: &Measured) {
    out.push_str(
        "\n## 4. Every row\n\n\
         | rite | rules | list | BS | day | printed | committee's sky | modern sky |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for f in &m.found {
        let shown = |sky| {
            f.reading(sky)
                .map_or_else(|| String::from("—"), Reading::shown)
        };
        let printed = match f.printed.vara {
            None => String::from("—"),
            Some(vara) if vara == f.vara => vara.key().to_owned(),
            Some(vara) => format!("{} (the date's {})", vara.key(), f.vara.key()),
        };
        let _ = writeln!(
            out,
            "| {} | `{}` | {} | {}-{}-{:02} | {} | {printed} | {} | {} |",
            f.printed.rite.name(),
            test(f.test).1.key(),
            f.printed.list(),
            f.printed.year,
            f.printed.month,
            f.printed.day,
            iso(&f.date),
            shown(Sky::Committee),
            shown(Sky::Modern),
        );
    }
}
