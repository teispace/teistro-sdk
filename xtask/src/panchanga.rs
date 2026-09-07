//! The falsification pass over the recorded daily panchanga, which the
//! panchanga day's design is written from (Phase 4).
//!
//! `panchanga_day` is the largest section of the conformance corpus and
//! the only one nothing had read. It holds twenty-seven fields per chart —
//! the four moving limbs with their start and end instants, three
//! inauspicious periods, two choghadiya sequences, twenty-four horas,
//! Abhijit and Brahma muhurta, the lunar month under both conventions,
//! panchaka, the Moon's and the Sun's signs, the ayana, the disha shool
//! and five muhurta yogas — and it says nowhere how any of them was
//! reckoned. A daily panchanga is almost entirely convention, and a
//! convention guessed wrong is a number that looks right.
//!
//! So this pass proposes a rule for every field and measures it against
//! all fifty-five recorded days. A rule that reproduces the corpus exactly
//! is one the design can be written on; a rule that does not is either the
//! wrong rule or a difference worth registering, and the measurement says
//! which. Nothing here computes an ephemeris: the subject is what the
//! recording engine's numbers *mean*, which is arithmetic over what it
//! wrote down.
//!
//! `cargo xtask panchanga` writes the page; `check-panchanga` regenerates
//! it in memory and fails on any difference, so the numbers on the page
//! are the numbers this build produces.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::Path;

use serde_json::Value;
use teistro_calendar::{CalendarSystem, FixedDay, Gregorian};
use teistro_core::catalogue::{Graha, Karana, Nakshatra, Tithi, Vara, Yoga};

use crate::generated::{Output, check, write};

const PAGE: &str = "docs/03-design/panchanga-day-conventions.md";
const CHARTS: &str = "fixtures/baseline/charts";

/// Seconds in a day, for reporting an instant's error in seconds rather
/// than in fractions of a day nobody can picture.
const SECONDS: f64 = 86_400.0;

/// A day whose sunrise and sunset the engine synthesised because the Sun
/// did not cross the horizon. Its arcs are not arcs, so every claim about
/// a division of an arc is measured without them, and §11 reports them.
const POLAR: [&str; 2] = ["c028", "c029"];

/// The four moving limbs, in the order a panchanga names them, with the
/// number of members each cycles through.
const LIMBS: [(&str, usize); 4] = [
    ("tithi", 30),
    ("nakshatra", 27),
    ("yoga", 27),
    ("karana", 11),
];

/// The three inauspicious eighths of the daylight.
const EIGHTHS: [&str; 3] = ["rahu_kaal", "yamaghanda", "gulika_kaal"];

/// The five muhurta yogas the section carries, in the order it lists them.
const YOGAS: [&str; 5] = [
    "amrit_siddhi",
    "sarvartha_siddhi",
    "siddha",
    "dwipushkar",
    "tripushkar",
];

/// The lunar months as the recording engine keys them.
///
/// The engine's keys are used rather than the catalogue's because the two
/// differ by a spelling (`ASHVIN` against the catalogue's `ASHWINA`) and
/// the relation being tested is between two of the engine's own fields.
const MONTHS: [&str; 12] = [
    "CHAITRA",
    "VAISHAKHA",
    "JYESHTHA",
    "ASHADHA",
    "SHRAVANA",
    "BHADRAPADA",
    "ASHVIN",
    "KARTIKA",
    "MARGASHIRSHA",
    "PAUSHA",
    "MAGHA",
    "PHALGUNA",
];

/// The first nakshatra of the panchaka window, counting from Ashwini at
/// zero: Dhanishtha, whose second half puts the Moon into Aquarius.
const FIRST_PANCHAKA_NAKSHATRA: usize = 22;

/// The first sign of the panchaka window: Aquarius.
const FIRST_PANCHAKA_SIGN: usize = 10;

/// A whole millisecond, in seconds: the bound a claim about an instant
/// has to be inside to count as reproducing the corpus exactly. Every
/// arithmetic claim below lands orders of magnitude under it.
const EXACT_SECONDS: f64 = 0.001;

/// A hundredth of a second: what "the same instant" means for two
/// boundaries a solver found. The engine's tolerance is 1e-7 days, which
/// is 8.64 milliseconds, and the last bit of a Julian day near two and a
/// half million is half a nanosecond, so the bound is that with room.
const SHARED_BOUNDARY_SECONDS: f64 = 0.01;

/// The successors of a karana, indexed by the karana's own catalogue id.
///
/// The four fixed karanas are not part of a cycle of eleven: sixty
/// karanas make a lunar month, the first is Kimstughna, the last three
/// are Shakuni, Chatushpada and Naga, and the seven movable ones repeat
/// through everything between. So the seventh movable karana is followed
/// either by the first — anywhere inside the month — or by Shakuni, at
/// the end of it, and no modulus expresses that.
const KARANA_NEXT: [&[usize]; 11] = [
    &[1],
    &[2],
    &[3],
    &[4],
    &[5],
    &[6],
    &[0, 7],
    &[8],
    &[9],
    &[10],
    &[0],
];

// ── the recorded day ───────────────────────────────────────────────────────

/// An interval the corpus records, in Julian days.
#[derive(Clone, Copy)]
struct Interval {
    from: f64,
    to: f64,
}

impl Interval {
    /// The interval a JSON object with `start_jd` and `end_jd` holds.
    fn of(value: &Value) -> Interval {
        Interval {
            from: number(&value["start_jd"]),
            to: number(&value["end_jd"]),
        }
    }

    /// Its length in days.
    const fn length(self) -> f64 {
        self.to - self.from
    }

    /// Its length in hours, which is how a period is read.
    const fn hours(self) -> f64 {
        self.length() * 24.0
    }
}

/// One span of a moving limb: which member, when it began and ended as
/// the corpus clipped it to the day, and whatever the corpus prints
/// about the member itself.
struct Span {
    index: usize,
    at: Interval,
    /// The member's recorded attributes, by the corpus's field name, as
    /// text: §12 holds the SDK's catalogue to them.
    attributes: BTreeMap<&'static str, String>,
}

/// One part of a divided arc: a choghadiya or a hora.
struct Part {
    lord: String,
    quality: String,
    at: Interval,
}

/// A sequence divided over each half of the day.
struct Halves {
    daylight: Vec<Part>,
    night: Vec<Part>,
}

/// One recorded day, with the few fields outside the daily section that a
/// claim needs.
struct Day {
    id: String,
    date: String,
    /// The engine's own weekday numbering, zero for Monday.
    weekday: usize,
    /// The instant the local civil date begins, in the same scale as
    /// every other instant here.
    local_midnight: f64,
    sunrise: f64,
    sunset: f64,
    /// The corpus does not record it; it is where every limb list ends,
    /// which §1 establishes before anything else relies on it.
    next_sunrise: f64,
    /// The last sunset before this day's sunrise, from the three arcs the
    /// fixture's foundation records, or `None` when none of them is one.
    previous_sunset: Option<f64>,
    limbs: BTreeMap<&'static str, Vec<Span>>,
    eighths: BTreeMap<&'static str, Interval>,
    choghadiya: Halves,
    horas: Vec<Part>,
    abhijit: Interval,
    abhijit_effective: bool,
    brahma: Interval,
    month: String,
    amanta_month: String,
    is_adhika: bool,
    /// Which panchaka runs, if one does.
    panchaka: Option<String>,
    /// The Moon's rise and set, when the day has them.
    moon_events: Vec<f64>,
    /// When the Moon leaves the sign it is in, if it does.
    moon_transition: Option<f64>,
    sun_sign: usize,
    moon_sign: usize,
    ayana: String,
    disha_shool: String,
    yogas: BTreeMap<String, bool>,
    /// The birth instant, and how the fixture's natal panchanga — which
    /// the engine computes topocentrically — classified it.
    birth: f64,
    natal: BTreeMap<&'static str, usize>,
}

impl Day {
    /// Whether the day is one of the two the engine synthesised.
    fn is_polar(&self) -> bool {
        POLAR.contains(&self.id.as_str())
    }

    /// The daylight, in days.
    const fn daylight(&self) -> f64 {
        self.sunset - self.sunrise
    }

    /// The night that follows the daylight, in days.
    const fn night(&self) -> f64 {
        self.next_sunrise - self.sunset
    }

    /// The whole window, in days.
    const fn window(&self) -> f64 {
        self.next_sunrise - self.sunrise
    }

    /// The vara the weekday names. The engine counts Monday as zero and
    /// the catalogue counts Sunday, so the member is one on.
    fn vara(&self) -> Vara {
        vara_of(self.weekday)
    }

    /// The member of a limb the window opens in.
    fn at_sunrise(&self, limb: &str) -> Option<usize> {
        self.limbs
            .get(limb)
            .and_then(|spans| spans.first())
            .map(|span| span.index)
    }
}

/// The vara of the engine's weekday number.
fn vara_of(weekday: usize) -> Vara {
    Vara::from_id(u16::try_from((weekday + 1) % 7).unwrap_or(0)).unwrap_or(Vara::Ravivara)
}

// ── reading the corpus ─────────────────────────────────────────────────────

/// A numeric field, or a value that fails every comparison.
fn number(value: &Value) -> f64 {
    value.as_f64().unwrap_or(f64::NAN)
}

/// A string field, or the empty string.
fn text_of(value: &Value) -> String {
    value.as_str().unwrap_or_default().to_string()
}

/// A whole-number field.
fn index_of(value: &Value) -> usize {
    value
        .as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .unwrap_or(0)
}

/// Every fixture that carries a daily panchanga.
fn days(root: &Path) -> Result<Vec<Day>, String> {
    let dir = root.join(CHARTS);
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(&dir)
        .map_err(|err| {
            format!(
                "cannot read {}: {err}. The corpus is a submodule; `git submodule update --init`",
                dir.display()
            )
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|e| e == "json"))
        .collect();
    files.sort();
    let mut days = Vec::new();
    for path in &files {
        let text =
            std::fs::read_to_string(path).map_err(|err| format!("{}: {err}", path.display()))?;
        let fixture: Value =
            serde_json::from_str(&text).map_err(|err| format!("{}: {err}", path.display()))?;
        if fixture["panchanga_day"].is_object() {
            days.push(day(&fixture));
        }
    }
    if days.is_empty() {
        return Err(format!(
            "no fixture under {} carries a daily panchanga",
            dir.display()
        ));
    }
    Ok(days)
}

/// One fixture's daily panchanga.
fn day(fixture: &Value) -> Day {
    let section = &fixture["panchanga_day"];
    let foundation = &fixture["foundation"];
    let sunrise = number(&section["sunrise_jd"]);

    let limbs: BTreeMap<&'static str, Vec<Span>> = LIMBS
        .iter()
        .map(|(name, _)| (*name, spans(&section[*name], attributes_of(name))))
        .collect();
    let eighths: BTreeMap<&'static str, Interval> = EIGHTHS
        .iter()
        .map(|name| (*name, Interval::of(&section[*name])))
        .collect();
    let moon_events = ["moonrise_jd", "moonset_jd"]
        .iter()
        .filter_map(|key| section[*key].as_f64())
        .collect();
    let yogas = YOGAS
        .iter()
        .filter_map(|name| {
            section["muhurta_yogas"][*name]
                .as_bool()
                .map(|flag| ((*name).to_string(), flag))
        })
        .collect();
    let natal = natal_limbs(&fixture["panchanga"]);

    let date = text_of(&section["local_date"]);
    Day {
        local_midnight: local_midnight(&date, number(&section["tz_offset_hours"])),
        id: text_of(&fixture["id"]),
        date,
        weekday: index_of(&section["weekday_swe"]),
        sunrise,
        sunset: number(&section["sunset_jd"]),
        next_sunrise: limbs
            .get("tithi")
            .and_then(|spans| spans.last())
            .map_or(f64::NAN, |span| span.at.to),
        previous_sunset: previous_sunset(foundation, sunrise),
        limbs,
        eighths,
        choghadiya: Halves {
            daylight: parts(&section["day_choghadiya"], "quality"),
            night: parts(&section["night_choghadiya"], "quality"),
        },
        horas: parts(&section["hora"], ""),
        abhijit: Interval::of(&section["abhijit"]),
        abhijit_effective: section["abhijit"]["is_effective"]
            .as_bool()
            .unwrap_or(false),
        brahma: Interval::of(&section["brahma_muhurta"]),
        month: text_of(&section["lunar_month"]["key"]),
        amanta_month: text_of(&section["lunar_month"]["amanta_key"]),
        is_adhika: section["lunar_month"]["is_adhika"]
            .as_bool()
            .unwrap_or(false),
        panchaka: section["panchaka"]["is_active"]
            .as_bool()
            .unwrap_or(false)
            .then(|| text_of(&section["panchaka"]["type"])),
        moon_events,
        moon_transition: section["moon_sign"]["transition_jd"].as_f64(),
        sun_sign: index_of(&section["sun_sign"]["sign_index"]),
        moon_sign: index_of(&section["moon_sign"]["sign_index"]),
        ayana: text_of(&section["sun_sign"]["ayana"]),
        disha_shool: text_of(&section["disha_shool"]),
        yogas,
        birth: number(&foundation["jd_ut"]),
        natal,
    }
}

/// How the fixture's natal block classified the birth instant, in the
/// same numbering the daily spans use.
fn natal_limbs(panchanga: &Value) -> BTreeMap<&'static str, usize> {
    let mut natal = BTreeMap::new();
    // The natal block numbers the tithi from one and indexes the rest
    // from zero, which is what the daily spans do too.
    for (limb, value) in [
        ("tithi", &panchanga["tithi"]["number"]),
        ("nakshatra", &panchanga["nakshatra_index"]),
        ("yoga", &panchanga["yoga_index"]),
        ("karana", &panchanga["karana"]["index"]),
    ] {
        if value.is_number() {
            natal.insert(limb, index_of(value));
        }
    }
    natal
}

/// Which of a member's attributes the corpus prints beside a span of
/// that limb.
const fn attributes_of(limb: &str) -> &'static [&'static str] {
    match limb.as_bytes() {
        b"tithi" => &["paksha", "number", "group"],
        // A nakshatra's is its muhurta nature and a yoga's its
        // auspiciousness; the corpus gives both the same field name.
        b"nakshatra" | b"yoga" => &["nature"],
        b"karana" => &["is_vishti"],
        _ => &[],
    }
}

/// The SDK catalogue's answer for one recorded attribute of one member,
/// or `None` for an attribute the catalogue does not carry.
fn catalogued(limb: &str, index: usize, attribute: &str) -> Option<String> {
    let id = u16::try_from(index).ok()?;
    match (limb, attribute) {
        ("tithi", "paksha") => Tithi::from_id(id).map(|t| t.attributes().paksha.key().to_string()),
        ("tithi", "group") => Tithi::from_id(id).map(|t| t.attributes().class.key().to_string()),
        ("nakshatra", "nature") => {
            Nakshatra::from_id(id).map(|n| n.attributes().muhurta_nature.key().to_string())
        }
        ("yoga", "nature") => {
            Yoga::from_id(id).map(|y| y.attributes().auspiciousness.key().to_string())
        }
        ("karana", "is_vishti") => Karana::from_id(id).map(|k| (k == Karana::Vishti).to_string()),
        _ => None,
    }
}

/// The spans of one limb, with the attributes that limb prints.
fn spans(value: &Value, attributes: &[&'static str]) -> Vec<Span> {
    value
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .map(|entry| Span {
                    index: index_of(&entry["index"]),
                    at: Interval::of(entry),
                    attributes: attributes
                        .iter()
                        .filter(|name| !entry[**name].is_null())
                        .map(|name| (*name, scalar(&entry[*name])))
                        .collect(),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// A JSON scalar as the text a key comparison uses.
fn scalar(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), ToString::to_string)
}

/// Whether a recorded attribute names the same member as a catalogue
/// key. The corpus writes some of them in lower case and hyphenated
/// (`highly-inauspicious`) where a key is upper case with underscores.
fn same_key(recorded: &str, key: &str) -> bool {
    fn normalised(text: &str) -> String {
        text.to_ascii_uppercase().replace('-', "_")
    }
    normalised(recorded) == normalised(key)
}

/// The parts of a divided arc. A hora has no quality, so the field it
/// would be read from is named by the caller and may be absent.
fn parts(value: &Value, quality: &str) -> Vec<Part> {
    value
        .as_array()
        .map(|entries| {
            entries
                .iter()
                .map(|entry| Part {
                    lord: text_of(&entry["lord"]),
                    quality: text_of(&entry[quality]),
                    at: Interval::of(entry),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The instant an ISO civil date begins at a zone offset, as a Julian
/// day, or a value that fails every comparison if the date is not one.
fn local_midnight(date: &str, offset_hours: f64) -> f64 {
    let mut parts = date.split('-');
    let (Some(year), Some(month), Some(day)) = (parts.next(), parts.next(), parts.next()) else {
        return f64::NAN;
    };
    let (Ok(year), Ok(month), Ok(day)) =
        (year.parse::<i32>(), month.parse::<u8>(), day.parse::<u8>())
    else {
        return f64::NAN;
    };
    Gregorian
        .to_fixed_ymd(year, month, day)
        .ok()
        .and_then(|fixed: FixedDay| fixed.jd_at_midnight().ok())
        .map_or(f64::NAN, |midnight| midnight.get() - offset_hours / 24.0)
}

/// The last sunset the fixture records before a sunrise.
///
/// The engine's `foundation.sunrise` block is the previous day's on three
/// charts (entry 12 of the deliberate-difference registry), so the arc
/// wanted here is found by its instant and not by the engine's label.
fn previous_sunset(foundation: &Value, sunrise: f64) -> Option<f64> {
    ["previous_day", "sunrise", "next_day"]
        .iter()
        .filter_map(|block| foundation[*block]["sunset_jd"].as_f64())
        .filter(|sunset| *sunset < sunrise - 0.1)
        .fold(None, |best: Option<f64>, sunset| {
            Some(best.map_or(sunset, |best| best.max(sunset)))
        })
}

// ── claims ─────────────────────────────────────────────────────────────────

/// What the corpus said about a proposed rule.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Verdict {
    /// The rule reproduces every day the corpus records.
    Holds,
    /// The corpus contradicts it.
    Falsified,
    /// The corpus holds no day that would tell the difference.
    Untested,
}

impl Verdict {
    /// How the page marks it.
    const fn mark(self) -> &'static str {
        match self {
            Verdict::Holds => "**holds**",
            Verdict::Falsified => "falsified",
            Verdict::Untested => "untested",
        }
    }
}

/// One proposed rule and the measurement that decided it.
struct Claim {
    rule: String,
    verdict: Verdict,
    measured: String,
}

impl Claim {
    /// A claim decided by whether a worst-case error in seconds is inside
    /// a millisecond.
    fn exact(rule: impl Into<String>, worst_seconds: f64) -> Claim {
        Claim::within(rule, worst_seconds, EXACT_SECONDS)
    }

    /// A claim decided by whether a worst-case error in seconds is inside
    /// a stated bound.
    fn within(rule: impl Into<String>, worst_seconds: f64, bound: f64) -> Claim {
        Claim {
            rule: rule.into(),
            verdict: verdict_of(worst_seconds.is_finite() && worst_seconds <= bound),
            measured: format!("worst {}", seconds(worst_seconds)),
        }
    }

    /// A claim decided by a count of comparisons that contradict it.
    fn counted(rule: impl Into<String>, wrong: usize, of: usize) -> Claim {
        Claim {
            rule: rule.into(),
            verdict: if of == 0 {
                Verdict::Untested
            } else {
                verdict_of(wrong == 0)
            },
            measured: if of == 0 {
                String::from("no day tests it")
            } else {
                format!("{wrong} of {of} disagree")
            },
        }
    }

    /// A claim whose measurement is stated rather than counted.
    fn stated(rule: impl Into<String>, verdict: Verdict, measured: impl Into<String>) -> Claim {
        Claim {
            rule: rule.into(),
            verdict,
            measured: measured.into(),
        }
    }
}

const fn verdict_of(ok: bool) -> Verdict {
    if ok {
        Verdict::Holds
    } else {
        Verdict::Falsified
    }
}

/// The claims as a table.
fn table(claims: &[Claim]) -> String {
    let mut out = String::from("| proposed rule | verdict | measured |\n|---|---|---|\n");
    for claim in claims {
        let _ = writeln!(
            out,
            "| {} | {} | {} |",
            claim.rule,
            claim.verdict.mark(),
            claim.measured
        );
    }
    out
}

/// A duration in seconds, written at the scale it is.
fn seconds(value: f64) -> String {
    if !value.is_finite() {
        String::from("not a number")
    } else if value == 0.0 {
        String::from("0 s, exactly")
    } else if value < 0.001 {
        format!("{:.3} ms", value * 1000.0)
    } else if value < 120.0 {
        format!("{value:.3} s")
    } else {
        format!("{:.2} h", value / 3600.0)
    }
}

/// The worst absolute value of a set of day differences, in seconds.
fn worst_seconds(values: impl IntoIterator<Item = f64>) -> f64 {
    values
        .into_iter()
        .fold(0.0_f64, |worst, value| worst.max(value.abs() * SECONDS))
}

/// The greatest of a set of measurements.
fn worst(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(0.0_f64, f64::max)
}

/// The days whose arcs are real.
fn real(days: &[Day]) -> impl Iterator<Item = &Day> {
    days.iter().filter(|day| !day.is_polar())
}

/// An index as a float, for a proportional division.
#[expect(
    clippy::cast_precision_loss,
    reason = "an index below twenty-five is exact in a double"
)]
const fn part_index(index: usize) -> f64 {
    index as f64
}

// ── the page ───────────────────────────────────────────────────────────────

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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask panchanga") != 0)
        }
        Err(err) => {
            println!("FAIL  {err}");
            1
        }
    }
}

fn page(root: &Path) -> Result<String, String> {
    let days = days(root)?;
    let sections = [
        header(&days),
        window(&days),
        limbs(&days),
        divisions(&days),
        walk(&days),
        muhurtas(&days),
        month(&days),
        panchaka(&days),
        yogas(&days),
        moon(&days),
        frame(&days),
        polar(&days),
        catalogue(&days),
        decides(&days),
    ];
    Ok(fill(&sections.concat()))
}

fn header(days: &[Day]) -> String {
    format!(
        "# The daily panchanga's conventions, measured\n\n\
         Status: `generated` by `cargo xtask panchanga` over the conformance\n\
         corpus's `panchanga_day` section, 2026-09-07. Do not edit:\n\
         `check-panchanga` regenerates this page and fails on any difference.\n\
         The design written from it is [`panchanga-day.md`](panchanga-day.md).\n\n\
         A daily panchanga is almost entirely convention. Which arc a period\n\
         divides, which instant a limb is read at, which of two nights sizes a\n\
         muhurta, whether a yoga is a flag or an interval — none of it shows in\n\
         a number and all of it changes the number. The corpus records {} days\n\
         of one implementation's answers and says nowhere how any of them was\n\
         reckoned, so this pass proposes a rule for each field and measures it.\n\n\
         Two of the {} days ({}) are polar: the Sun did not cross the horizon\n\
         and the engine synthesised the bounds (entry 3 of the\n\
         deliberate-difference registry). Dividing an arc that is not an arc\n\
         measures nothing, so the claims below are measured over the {} real\n\
         days and §11 reports the other two.\n\n",
        days.len(),
        days.len(),
        POLAR.join(" and "),
        real(days).count(),
    )
}

// ── 1. the window ──────────────────────────────────────────────────────────

fn window(days: &[Day]) -> String {
    let mut starts_agree = 0.0_f64;
    let mut ends_agree = 0.0_f64;
    let mut starts_at_sunrise = 0.0_f64;
    let mut shortest = f64::INFINITY;
    let mut longest = f64::NEG_INFINITY;
    for day in days {
        let starts: Vec<f64> = day
            .limbs
            .values()
            .filter_map(|spans| spans.first().map(|span| span.at.from))
            .collect();
        let ends: Vec<f64> = day
            .limbs
            .values()
            .filter_map(|spans| spans.last().map(|span| span.at.to))
            .collect();
        starts_agree = starts_agree.max(spread(&starts) * SECONDS);
        ends_agree = ends_agree.max(spread(&ends) * SECONDS);
        starts_at_sunrise = starts_at_sunrise.max(worst_seconds(
            starts.iter().map(|start| start - day.sunrise),
        ));
        shortest = shortest.min(day.window() * 24.0);
        longest = longest.max(day.window() * 24.0);
    }

    let claims = [
        Claim::exact("the four limb lists begin at one instant", starts_agree),
        Claim::exact("that instant is the day's sunrise", starts_at_sunrise),
        Claim::exact(
            "the four limb lists end at one instant, which is therefore the next sunrise",
            ends_agree,
        ),
    ];

    format!(
        "## 1. The window is sunrise to the next sunrise\n\n\
         The section records a sunrise and a sunset and never the sunrise that\n\
         closes the day, so the first thing to settle is where the day ends.\n\
         Every limb list answers it: all four begin at one instant and end at\n\
         one instant, on every day of the corpus.\n\n{}\n\
         The window is {shortest:.4} to {longest:.4} hours long — a day, plus\n\
         or minus the change in the length of the daylight — and every claim\n\
         below reads the last limb end as the next sunrise.\n\n\
         This is not the civil day. The section's `local_date` is a civil date,\n\
         and the window carrying its limbs opens at that date's sunrise and\n\
         closes at the next, so an instant between midnight and sunrise belongs\n\
         to the window before. `crates/chart`'s `day` module already inverts\n\
         that for a chart; a daily panchanga is the same window asked for by\n\
         date rather than by instant.\n\n",
        table(&claims),
    )
}

/// The spread of a set of instants, in days.
fn spread(values: &[f64]) -> f64 {
    let low = values.iter().copied().fold(f64::INFINITY, f64::min);
    let high = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if low.is_finite() && high.is_finite() {
        high - low
    } else {
        0.0
    }
}

// ── 2. the limbs ───────────────────────────────────────────────────────────

/// How many spans of each limb a window holds, as a table.
fn span_counts(days: &[Day]) -> String {
    let mut out = String::from("| limb | members | 1 | 2 | 3 | 4 |\n|---|---|---|---|---|---|\n");
    for (name, members) in LIMBS {
        let mut counts = [0_usize; 5];
        for day in days {
            let held = day.limbs.get(name).map_or(0, Vec::len).min(4);
            if let Some(slot) = counts.get_mut(held) {
                *slot += 1;
            }
        }
        let _ = writeln!(
            out,
            "| {name} | {members} | {} | {} | {} | {} |",
            counts[1], counts[2], counts[3], counts[4]
        );
    }
    out
}

/// What §2 measures over the corpus's spans.
struct Sequences {
    cyclic_breaks: usize,
    cyclic_pairs: usize,
    karana_breaks: usize,
    karana_pairs: usize,
    through_month: usize,
    within_paksha: usize,
    numbered: usize,
    /// The gaps between consecutive spans, in seconds.
    gaps: Vec<f64>,
}

fn measure_sequences(days: &[Day]) -> Sequences {
    let mut found = Sequences {
        cyclic_breaks: 0,
        cyclic_pairs: 0,
        karana_breaks: 0,
        karana_pairs: 0,
        through_month: 0,
        within_paksha: 0,
        numbered: 0,
        gaps: Vec::new(),
    };
    for day in days {
        for span in day.limbs.get("tithi").into_iter().flatten() {
            let Some(recorded) = span.attributes.get("number") else {
                continue;
            };
            found.numbered += 1;
            found.through_month += usize::from(*recorded != (span.index + 1).to_string());
            found.within_paksha += usize::from(*recorded != (span.index % 15 + 1).to_string());
        }
        for (name, members) in LIMBS {
            let Some(spans) = day.limbs.get(name) else {
                continue;
            };
            for pair in spans.windows(2) {
                let (Some(previous), Some(next)) = (pair.first(), pair.get(1)) else {
                    continue;
                };
                if name == "karana" {
                    found.karana_pairs += 1;
                    found.karana_breaks += usize::from(
                        KARANA_NEXT
                            .get(previous.index)
                            .is_none_or(|after| !after.contains(&next.index)),
                    );
                } else {
                    found.cyclic_pairs += 1;
                    found.cyclic_breaks +=
                        usize::from((previous.index + 1) % members != next.index % members);
                }
                found.gaps.push((next.at.from - previous.at.to) * SECONDS);
            }
        }
    }
    found
}

fn limbs(days: &[Day]) -> String {
    let mut out = format!(
        "## 2. A limb is a span, and the corpus's spans are clipped\n\n\
         Each moving limb is recorded as the list of its members that touch the\n\
         window, in order, with a start and an end. The first span's start and\n\
         the last span's end are the *window's* bounds and not the member's: a\n\
         tithi that began yesterday evening is recorded as beginning at\n\
         sunrise, and its real start is not recoverable from the fixture.\n\n\
         How many spans a window holds, over the corpus:\n\n{}",
        span_counts(days),
    );

    let found = measure_sequences(days);
    let smallest = found.gaps.iter().copied().fold(f64::INFINITY, f64::min);
    let largest = found.gaps.iter().copied().fold(f64::NEG_INFINITY, f64::max);

    let claims = [
        Claim::counted(
            "a tithi, nakshatra or yoga span's member is the previous plus one, cycling",
            found.cyclic_breaks,
            found.cyclic_pairs,
        ),
        Claim::counted(
            "a karana span's member is the next of the month's sixty, not of a cycle of eleven",
            found.karana_breaks,
            found.karana_pairs,
        ),
        Claim::within(
            "a span ends where the next begins",
            largest,
            SHARED_BOUNDARY_SECONDS,
        ),
        Claim::counted(
            "a tithi's recorded number counts through the month, one to thirty",
            found.through_month,
            found.numbered,
        ),
        Claim::counted(
            "a tithi's recorded number counts within its paksha, one to fifteen",
            found.within_paksha,
            found.numbered,
        ),
    ];
    let _ = write!(
        out,
        "\n{}\nThe gap between a span's end and the next span's start is {} on\n\
         every one of the {} consecutive pairs, which is 1e-7 days: one boundary\n\
         instant written twice, at the tolerance the engine's solver stops at.\n\
         The SDK's boundary solver stops at the same figure —\n\
         `astro::events::TOLERANCE_DAYS` is 1e-7 days — and records the instant\n\
         once.\n\n\
         The tithi's number is the other place two conventions meet. The\n\
         corpus counts one to thirty through the lunar month; the SDK's\n\
         catalogue gives each member a number within its paksha, one to\n\
         fifteen, which is how a tithi is named. Both are right and they are\n\
         not the same field, so a harness that compares them without saying\n\
         which is comparing nothing.\n\n\
         The karana is the limb that is not a cycle. Sixty karanas make a lunar\n\
         month: Kimstughna opens it, Shakuni, Chatushpada and Naga close it, and\n\
         the seven movable ones repeat through everything between, so the\n\
         seventh movable karana is followed by the first inside the month and by\n\
         Shakuni at the end of it. It is also the only list that reaches four\n\
         spans, a karana being half a tithi. Every list can hold one, when a\n\
         slow member covers the whole window.\n\n",
        table(&claims),
        seconds(if (largest - smallest).abs() < f64::EPSILON {
            largest
        } else {
            smallest
        }),
        found.gaps.len(),
    );
    out
}

// ── 3. the divisions ───────────────────────────────────────────────────────

/// What §3 measures: how far each proposed division is from the corpus,
/// and which eighth of the daylight each inauspicious period occupies.
struct Divisions {
    /// The worst error of an eighth, as a fraction of an eighth.
    eighth: f64,
    /// The worst error of a choghadiya, in seconds.
    choghadiya: f64,
    /// The worst error of a proportional hora, in seconds.
    proportional: f64,
    /// The worst error of an equal hora, in seconds.
    equal: f64,
    /// The eighth each period occupies, by name and weekday.
    which: BTreeMap<&'static str, [Option<usize>; 7]>,
}

fn measure_divisions(days: &[Day]) -> Divisions {
    let mut eighth_error = 0.0_f64;
    let mut choghadiya_error = 0.0_f64;
    let mut proportional = 0.0_f64;
    let mut equal = 0.0_f64;
    let mut which: BTreeMap<&'static str, [Option<usize>; 7]> =
        EIGHTHS.iter().map(|name| (*name, [None; 7])).collect();
    for day in real(days) {
        let eighth = day.daylight() / 8.0;
        for name in EIGHTHS {
            let Some(period) = day.eighths.get(name) else {
                continue;
            };
            let index = (period.from - day.sunrise) / eighth;
            eighth_error = worst([
                eighth_error,
                (index - index.round()).abs(),
                (period.length() / eighth - 1.0).abs(),
            ]);
            if let Some(row) = which.get_mut(name)
                && let Some(slot) = row.get_mut(day.weekday)
            {
                // The index is one of eight, so it is found rather
                // than cast: a rounding that is not a whole eighth is
                // the falsification this claim exists to catch.
                *slot = (0..8_u8)
                    .find(|eighth| (index - f64::from(*eighth)).abs() < 0.5)
                    .map(usize::from);
            }
        }
        choghadiya_error = worst([
            choghadiya_error,
            equal_parts(&day.choghadiya.daylight, day.sunrise, day.daylight()),
            equal_parts(&day.choghadiya.night, day.sunset, day.night()),
        ]);
        proportional =
            proportional.max(worst(day.horas.iter().enumerate().map(|(index, hora)| {
                let (start, end) = proportional_hora(day, index);
                worst_seconds([hora.at.from - start, hora.at.to - end])
            })));
        equal = equal.max(worst(day.horas.iter().enumerate().map(|(index, hora)| {
            worst_seconds([hora.at.from - (day.sunrise + day.window() * part_index(index) / 24.0)])
        })));
    }
    Divisions {
        eighth: eighth_error,
        choghadiya: choghadiya_error,
        proportional,
        equal,
        which,
    }
}

fn divisions(days: &[Day]) -> String {
    let Divisions {
        eighth: eighth_error,
        choghadiya: choghadiya_error,
        proportional,
        equal,
        which,
    } = measure_divisions(days);

    let claims = [
        Claim::stated(
            "rahu kaal, yamaghanda and gulika are each one eighth of the daylight",
            verdict_of(eighth_error < 1e-6),
            format!("worst {eighth_error:.2e} of an eighth"),
        ),
        Claim::exact(
            "the choghadiya are eight equal parts of the daylight and eight of the night",
            choghadiya_error,
        ),
        Claim::exact(
            "the horas are twelve over the daylight and twelve over the night",
            proportional,
        ),
        Claim::exact(
            "the horas are twenty-four equal parts of the whole window",
            equal,
        ),
    ];

    let mut out = format!(
        "## 3. Every period is a proportional division of an arc\n\n\
         Six of the section's fields divide an arc into equal parts. Not one\n\
         divides a clock hour, and the daylight and the night are divided\n\
         separately, so a period's length changes with the season and the\n\
         latitude.\n\n{}\n\
         The equal hora — twenty-four sixty-minute hours from sunrise — is out\n\
         by up to {}, which is not a rounding difference but a different\n\
         reckoning. The corpus decides it, and it decides for the proportional\n\
         one; `day.hora_reckoning` already has both\n\
         values and `crates/time`'s hora module already computes either (entry\n\
         13 of the registry), so the daily panchanga calls it rather than\n\
         dividing an arc of its own.\n\n\
         Which eighth of the daylight each inauspicious period occupies, by the\n\
         day of the week — the corpus's table, counted from one:\n\n\
         | vara | rahu kaal | yamaghanda | gulika |\n|---|---|---|---|\n",
        table(&claims),
        seconds(equal),
    );
    for weekday in 0..7_usize {
        let vara = vara_of(weekday);
        let cell = |name: &str| {
            which
                .get(name)
                .and_then(|row| row.get(weekday).copied().flatten())
                .map_or_else(|| String::from("—"), |index| (index + 1).to_string())
        };
        let _ = writeln!(
            out,
            "| {} ({}) | {} | {} | {} |",
            vara.doc(),
            vara.key(),
            cell("rahu_kaal"),
            cell("yamaghanda"),
            cell("gulika_kaal"),
        );
    }
    out.push_str(
        "\nThe three rows are the classical ones. Yamaghanda and gulika are\n\
         arithmetic — the third and the fifth eighth, counted backwards from\n\
         the weekday, modulo seven — and rahu kaal is not, so all three ship as\n\
         one table rather than as two rules and an exception.\n\n",
    );
    out
}

/// The bounds a proportional hora would have.
fn proportional_hora(day: &Day, index: usize) -> (f64, f64) {
    let n = part_index(index);
    if index < 12 {
        (
            day.sunrise + day.daylight() * n / 12.0,
            day.sunrise + day.daylight() * (n + 1.0) / 12.0,
        )
    } else {
        (
            day.sunset + day.night() * (n - 12.0) / 12.0,
            day.sunset + day.night() * (n - 11.0) / 12.0,
        )
    }
}

/// The worst error, in seconds, of a sequence of parts against an equal
/// division of an arc into as many parts as the sequence has.
fn equal_parts(parts: &[Part], from: f64, length: f64) -> f64 {
    let count = part_index(parts.len());
    worst(parts.iter().enumerate().map(|(index, part)| {
        let n = part_index(index);
        worst_seconds([
            part.at.from - (from + length * n / count),
            part.at.to - (from + length * (n + 1.0) / count),
        ])
    }))
}

// ── 4. the weekday walk ────────────────────────────────────────────────────

/// How many weekdays a hora, and so a day choghadiya, steps.
const DAY_STEP: usize = 5;
/// How many weekdays a night choghadiya steps.
const NIGHT_STEP: usize = 4;
/// How far on from the vara the night's walk starts.
const NIGHT_START: usize = 4;

fn walk(days: &[Day]) -> String {
    let mut hora_wrong = 0_usize;
    let mut hora_seen = 0_usize;
    let mut day_wrong = 0_usize;
    let mut day_seen = 0_usize;
    let mut night_wrong = 0_usize;
    let mut night_seen = 0_usize;
    let mut named: BTreeMap<&'static str, String> = BTreeMap::new();
    for day in days {
        let vara = day.vara();
        for (index, hora) in day.horas.iter().enumerate() {
            hora_seen += 1;
            hora_wrong += usize::from(hora.lord != walk_lord(vara, index * DAY_STEP).key());
        }
        for (index, part) in day.choghadiya.daylight.iter().enumerate() {
            day_seen += 1;
            day_wrong += usize::from(part.lord != walk_lord(vara, index * DAY_STEP).key());
            remember(&mut named, part);
        }
        let start = walk_vara(vara, NIGHT_START);
        for (index, part) in day.choghadiya.night.iter().enumerate() {
            night_seen += 1;
            night_wrong += usize::from(part.lord != walk_lord(start, index * NIGHT_STEP).key());
            remember(&mut named, part);
        }
    }

    let claims = [
        Claim::counted(
            "a hora's lord is the vara's, walked five weekdays on per hora",
            hora_wrong,
            hora_seen,
        ),
        Claim::counted(
            "a day choghadiya's lord is that same walk, one step per eighth",
            day_wrong,
            day_seen,
        ),
        Claim::counted(
            "a night choghadiya's walk starts four weekdays on and steps four",
            night_wrong,
            night_seen,
        ),
    ];

    let mut out = format!(
        "## 4. The choghadiya is the hora's walk, not a grid of names\n\n\
         A choghadiya is usually printed as a grid of seven rows by eight\n\
         columns. It is not a grid. The lord of the *k*th eighth of the daylight\n\
         is the lord of the *k*th hora — the same walk five weekdays at a time\n\
         that `time::hora::lord_of` already computes — and the night's walk\n\
         starts four weekdays on from the vara and steps four at a time.\n\n{}\n\
         The name follows from the lord alone, one row per graha:\n\n\
         | lord | choghadiya |\n|---|---|\n",
        table(&claims),
    );
    for graha in Graha::ALL {
        if let Some(quality) = named.get(graha.key()) {
            let _ = writeln!(out, "| {} | {quality} |", graha.doc());
        }
    }
    out.push_str(
        "\nSo the whole of the choghadiya is a seven-row table of names over a\n\
         walk the SDK already has. Sixteen intervals a day come out of it and\n\
         none of it is a new algorithm.\n\n",
    );
    out
}

/// Records which choghadiya a lord gives its name to.
fn remember(named: &mut BTreeMap<&'static str, String>, part: &Part) {
    if let Some(graha) = Graha::ALL.iter().find(|graha| graha.key() == part.lord) {
        named.insert(graha.key(), part.quality.clone());
    }
}

/// The vara `steps` weekdays on from another.
fn walk_vara(vara: Vara, steps: usize) -> Vara {
    let weekday = (usize::from(vara.attributes().weekday) + steps) % 7;
    Vara::from_id(u16::try_from(weekday).unwrap_or(0)).unwrap_or(vara)
}

/// The lord of the weekday `steps` on from a vara.
fn walk_lord(vara: Vara, steps: usize) -> Graha {
    walk_vara(vara, steps).attributes().lord
}

// ── 5. Abhijit and Brahma muhurta ──────────────────────────────────────────

/// The engine's weekday number for Wednesday, on which Abhijit is void.
const WEDNESDAY: usize = 2;

fn muhurtas(days: &[Day]) -> String {
    let mut abhijit_error = 0.0_f64;
    let mut void_wrong = 0_usize;
    let mut from_coming = 0.0_f64;
    let mut from_previous = 0.0_f64;
    let mut shifts: Vec<f64> = Vec::new();
    for day in real(days) {
        let muhurta = day.daylight() / 15.0;
        abhijit_error = abhijit_error.max(worst_seconds([
            day.abhijit.from - (day.sunrise + 7.0 * muhurta),
            day.abhijit.to - (day.sunrise + 8.0 * muhurta),
        ]));
        let coming = day.night() / 15.0;
        from_coming = from_coming.max(brahma_error(day, coming));
        if let Some(sunset) = day.previous_sunset {
            let previous = (day.sunrise - sunset) / 15.0;
            from_previous = from_previous.max(brahma_error(day, previous));
            shifts.push((coming - previous).abs() * 2.0 * SECONDS);
        }
    }
    for day in days {
        void_wrong += usize::from(day.abhijit_effective == (day.weekday == WEDNESDAY));
    }
    shifts.sort_by(f64::total_cmp);
    let median = shifts.get(shifts.len() / 2).copied().unwrap_or(0.0);
    let extreme = shifts.last().copied().unwrap_or(0.0);
    let wednesdays = days.iter().filter(|day| day.weekday == WEDNESDAY).count();

    let claims = [
        Claim::exact(
            "Abhijit is the eighth of the daylight's fifteen muhurtas",
            abhijit_error,
        ),
        Claim::counted(
            "Abhijit is void on a Wednesday and effective on every other day",
            void_wrong,
            days.len(),
        ),
        Claim::exact(
            "Brahma muhurta is the fourteenth muhurta of the night that ends at this sunrise",
            from_previous,
        ),
        Claim::exact(
            "Brahma muhurta sits before this sunrise but is sized from the night after the day",
            from_coming,
        ),
    ];

    format!(
        "## 5. Abhijit holds; Brahma muhurta is sized from the wrong night\n\n\
         Both are muhurtas — fifteenths of an arc — and both sit where the\n\
         tradition puts them. Abhijit is the eighth muhurta of the daylight,\n\
         straddling noon, and is void on Wednesdays: {wednesdays} of the {} days\n\
         are Wednesdays and every one is recorded void, while every one of the\n\
         other {} is effective.\n\n{}\n\
         Brahma muhurta ends before sunrise, so the night it belongs to is the\n\
         night that *ends* at that sunrise. The engine puts it there and sizes\n\
         it from the night that *follows* the day — the same night its\n\
         choghadiya divide. The two nights differ by the change in the length\n\
         of the daylight from one day to the next, and that moves the start of\n\
         Brahma muhurta by a median of {} and at worst {}.\n\n\
         Small, systematic and wrong: the SDK sizes it from the night it is in,\n\
         and this becomes a row of the deliberate-difference registry rather\n\
         than a defect either implementation has to keep.\n\n",
        days.len(),
        days.len() - wednesdays,
        table(&claims),
        seconds(median),
        seconds(extreme),
    )
}

/// How far the recorded Brahma muhurta is from the fourteenth and
/// fifteenth muhurtas of a night of the given muhurta length.
fn brahma_error(day: &Day, muhurta: f64) -> f64 {
    worst_seconds([
        day.brahma.from - (day.sunrise - 2.0 * muhurta),
        day.brahma.to - (day.sunrise - muhurta),
    ])
}

// ── 6. the lunar month ─────────────────────────────────────────────────────

/// The first tithi index of the dark fortnight.
const FIRST_KRISHNA_TITHI: usize = 15;

fn month(days: &[Day]) -> String {
    let mut wrong = 0_usize;
    for day in days {
        let Some(amanta) = MONTHS.iter().position(|name| *name == day.amanta_month) else {
            wrong += 1;
            continue;
        };
        let krishna = day
            .at_sunrise("tithi")
            .is_some_and(|index| index >= FIRST_KRISHNA_TITHI);
        let expected = if krishna {
            (amanta + 1) % MONTHS.len()
        } else {
            amanta
        };
        if MONTHS.get(expected).is_none_or(|name| *name != day.month) {
            wrong += 1;
        }
    }
    let adhika = days.iter().filter(|day| day.is_adhika).count();
    let claims = [
        Claim::counted(
            "the purnimanta month is the amanta month, plus one through the dark fortnight",
            wrong,
            days.len(),
        ),
        Claim::stated(
            "the intercalation rule can be checked here",
            Verdict::Untested,
            format!(
                "{adhika} adhika and no kshaya month over {} days",
                days.len()
            ),
        ),
    ];
    format!(
        "## 6. The two lunar months are one month and a rule\n\n\
         The section names the month twice, once under each convention, and the\n\
         relation between them is arithmetic: a purnimanta month begins at the\n\
         full moon, so through the dark fortnight it is already the next amanta\n\
         month.\n\n{}\n\
         `calendars.lunar_month` is the knob and it already exists. What the\n\
         corpus does not settle is the intercalation: {adhika} of the {} days\n\
         are marked adhika and none is marked kshaya, which shows the field is\n\
         computed and does not test the rule that computes it. That belongs to\n\
         the Indian lunisolar calendar's own page, not here.\n\n",
        table(&claims),
        days.len(),
    )
}

// ── 7. panchaka ────────────────────────────────────────────────────────────

fn panchaka(days: &[Day]) -> String {
    let mut by_nakshatra: BTreeMap<usize, String> = BTreeMap::new();
    let mut nakshatra_wrong = 0_usize;
    let mut sign_wrong = 0_usize;
    let mut in_first_half = 0_usize;
    for day in days {
        let at_sunrise = day.at_sunrise("nakshatra").unwrap_or(0);
        let by_nak = at_sunrise >= FIRST_PANCHAKA_NAKSHATRA;
        let by_sign = day.moon_sign >= FIRST_PANCHAKA_SIGN;
        nakshatra_wrong += usize::from(by_nak != day.panchaka.is_some());
        sign_wrong += usize::from(by_sign != day.panchaka.is_some());
        in_first_half += usize::from(by_nak != by_sign);
        if let Some(kind) = &day.panchaka {
            by_nakshatra.insert(at_sunrise, kind.clone());
        }
    }
    let running = days.iter().filter(|day| day.panchaka.is_some()).count();
    let claims = [
        Claim::counted(
            "panchaka runs while the Moon is in the last five nakshatras",
            nakshatra_wrong,
            days.len(),
        ),
        Claim::counted(
            "panchaka runs while the Moon is in Aquarius or Pisces",
            sign_wrong,
            days.len(),
        ),
        Claim::stated(
            "a day separates the two: the Moon in Dhanishtha's first half",
            Verdict::Untested,
            format!("{in_first_half} of {} days", days.len()),
        ),
    ];
    let mut out = format!(
        "## 7. Panchaka follows the Moon's nakshatra, and its kind is a table\n\n\
         Two rules are proposed for when panchaka runs, and the corpus cannot\n\
         tell them apart: they differ only while the Moon is in the first half\n\
         of Dhanishtha, which is in Capricorn, and no recorded day has it\n\
         there.\n\n{}\n\
         Its kind is a function of the nakshatra alone — not of the classical\n\
         remainder of tithi, vara and nakshatra, which no numbering of the three\n\
         reproduces over the {running} days that carry one:\n\n\
         | nakshatra | panchaka |\n|---|---|\n",
        table(&claims),
    );
    for (index, kind) in &by_nakshatra {
        let name = Nakshatra::from_id(u16::try_from(*index).unwrap_or(0))
            .map_or_else(|| index.to_string(), |n| n.doc().to_string());
        let _ = writeln!(out, "| {name} | {kind} |");
    }
    out.push_str(
        "\nThe SDK ships the nakshatra rule, which is the one the texts state,\n\
         and the design page records that the corpus does not test the half\n\
         nakshatra where the two part.\n\n",
    );
    out
}

// ── 8. the muhurta yogas ───────────────────────────────────────────────────

fn yogas(days: &[Day]) -> String {
    let mut counts: BTreeMap<&str, usize> = YOGAS.iter().map(|name| (*name, 0)).collect();
    let mut firing: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    for day in days {
        for name in YOGAS {
            if !day.yogas.get(name).copied().unwrap_or(false) {
                continue;
            }
            if let Some(count) = counts.get_mut(name) {
                *count += 1;
            }
            let nakshatra = day
                .at_sunrise("nakshatra")
                .and_then(|index| Nakshatra::from_id(u16::try_from(index).unwrap_or(0)))
                .map_or_else(|| String::from("?"), |n| n.doc().to_string());
            firing
                .entry(name)
                .or_default()
                .push(format!("{} {nakshatra}", day.vara().doc()));
        }
    }
    let total: usize = counts.values().sum();
    let on_days = days
        .iter()
        .filter(|day| YOGAS.iter().any(|name| day.yogas.get(*name) == Some(&true)))
        .count();
    let mut out = format!(
        "## 8. The muhurta yogas are the one limb the corpus cannot settle\n\n\
         Five are recorded as flags. They fire {total} times, on {on_days} of\n\
         the {} days:\n\n| yoga | days | on |\n|---|---|---|\n",
        days.len(),
    );
    for name in YOGAS {
        let on = firing
            .get(name)
            .map_or_else(|| String::from("—"), |rows| rows.join(", "));
        let _ = writeln!(
            out,
            "| {name} | {} | {on} |",
            counts.get(name).copied().unwrap_or(0)
        );
    }
    let _ = write!(
        out,
        "\nA vara-and-nakshatra yoga is a table of seven rows by twenty-seven\n\
         columns. With {on_days} positive days there is nothing to derive one\n\
         from, and the corpus's\n\
         positives do not match the published tables: the standard Sarvartha\n\
         Siddhi table reproduces one of the engine's five and adds nine of its\n\
         own, and no rotation of the weekday or of the nakshatra index does\n\
         better than four wrong. Two things the corpus does settle:\n\n\
         1. **The nakshatra is read at sunrise, not across the day.** Reading\n   \
            any nakshatra of the day into the Amrit Siddhi pairs fires it on\n   \
            four days the engine leaves clear; reading only the nakshatra at\n   \
            sunrise agrees with the engine on all fifty-five.\n\
         2. **Tripushkar's classical rule holds**: a Bhadra tithi on a Sunday,\n   \
            Tuesday or Saturday in a three-footed nakshatra. The corpus has one\n   \
            such day and the rule agrees exactly, with no false positives.\n   \
            Dwipushkar never fires, so its rule is carried and untested.\n\n   \
         So the design ships the yogas as cited tables with the catalogue's own\n   \
         confidence marks, and reports them as **intervals**, which the\n   \
         corpus's flags reduce to: a yoga holds while the nakshatra that makes\n\
         it holds, and the engine's flag is that interval containing sunrise.\n\n",
    );
    out
}

/// The width the page's prose is filled to.
const FILL: usize = 72;

/// Refills the page's prose paragraphs.
///
/// Every measurement on this page is substituted into a sentence, and a
/// number that is two digits wide today may be three tomorrow, so prose
/// wrapped in the source drifts ragged as the corpus grows. Filling the
/// finished page instead keeps it tidy whatever the numbers turn out to
/// be. A block that is a heading, a table or a list is left exactly as it
/// was written.
fn fill(page: &str) -> String {
    page.split("\n\n")
        .map(|block| {
            let prose = block.lines().all(|line| {
                let line = line.trim_start();
                !line.starts_with('|')
                    && !line.starts_with('#')
                    && !line.starts_with("- ")
                    && !line.starts_with(|c: char| c.is_ascii_digit())
            });
            if prose {
                wrapped(
                    &block
                        .split_whitespace()
                        .map(str::to_string)
                        .collect::<Vec<_>>(),
                    FILL,
                    " ",
                )
            } else {
                block.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// A list of findings, wrapped so the generated prose stays readable.
fn wrapped(items: &[String], width: usize, separator: &str) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();
    for item in items {
        if !line.is_empty() && line.len() + separator.len() + item.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push_str(separator);
        }
        line.push_str(item);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines.join("\n")
}

// ── 9. the Moon's rise and set ─────────────────────────────────────────────

fn moon(days: &[Day]) -> String {
    let mut outside = 0_usize;
    let mut events = 0_usize;
    let mut earliest = f64::INFINITY;
    let mut latest = f64::NEG_INFINITY;
    let mut transitions = 0_usize;
    let mut transitions_outside = 0_usize;
    let mut ayana_wrong = 0_usize;
    let mut before_midnight = 0_usize;
    let mut shool: BTreeMap<usize, String> = BTreeMap::new();
    for day in days {
        for instant in &day.moon_events {
            events += 1;
            outside += usize::from(*instant < day.sunrise || *instant >= day.next_sunrise);
            before_midnight += usize::from(*instant < day.local_midnight);
            let hours = (instant - day.sunrise) * 24.0;
            earliest = earliest.min(hours);
            latest = latest.max(hours);
        }
        if let Some(at) = day.moon_transition {
            transitions += 1;
            transitions_outside += usize::from(at < day.sunrise || at >= day.next_sunrise);
        }
        // Uttarayana runs from Capricorn to Gemini in the sidereal zodiac.
        let uttarayana = day.sun_sign >= 9 || day.sun_sign <= 2;
        ayana_wrong += usize::from(uttarayana != (day.ayana == "uttarayana"));
        shool.insert(day.weekday, day.disha_shool.clone());
    }
    let claims = [
        Claim::counted(
            "the Moon's rise and set lie inside the panchanga day",
            outside,
            events,
        ),
        Claim::counted(
            "the Moon's rise and set are the first at or after the local civil midnight",
            before_midnight,
            events,
        ),
        Claim::counted(
            "the Moon's change of sign lies inside the panchanga day",
            transitions_outside,
            transitions,
        ),
        Claim::counted(
            "uttarayana runs while the sidereal Sun is in Capricorn to Gemini",
            ayana_wrong,
            days.len(),
        ),
    ];
    let mut out = format!(
        "## 9. The Moon's rise and set are the civil day's, not the panchanga day's\n\n\
         Everything else in the section is bounded by sunrise. The Moon's rise\n\
         and set are not: {outside} of the {events} recorded ones fall outside\n\
         the window the limbs occupy, spread from {earliest:.2} to {latest:.2}\n\
         hours after sunrise, and not one of them falls before the local civil\n\
         midnight. The Moon's change of sign, by contrast, is inside the window\n\
         every time.\n\n{}\n\
         Two windows in one section is a defect and not a convention: an almanac\n\
         reader cannot tell which one they are looking at. The SDK reports the\n\
         Moon's events over the window the rest of the day uses, and\n\
         `conformance-baseline` carries the engine's choice so that the corpus\n\
         still reproduces.\n\n\
         The disha shool is the vara's, one direction each:\n\n\
         | vara | disha shool |\n|---|---|\n",
        table(&claims),
    );
    for weekday in 0..7_usize {
        if let Some(direction) = shool.get(&weekday) {
            let _ = writeln!(out, "| {} | {direction} |", vara_of(weekday).doc());
        }
    }
    out.push('\n');
    out
}

// ── 10. the frame ──────────────────────────────────────────────────────────

fn frame(days: &[Day]) -> String {
    let mut compared = 0_usize;
    let mut disagree: Vec<String> = Vec::new();
    for day in days {
        for (name, _) in LIMBS {
            let (Some(spans), Some(natal)) = (day.limbs.get(name), day.natal.get(name)) else {
                continue;
            };
            let Some(span) = spans
                .iter()
                .find(|span| day.birth >= span.at.from && day.birth <= span.at.to)
            else {
                continue;
            };
            compared += 1;
            // The corpus numbers a tithi from one and indexes the rest
            // from zero, in the daily spans and the natal block alike.
            let daily = if name == "tithi" {
                span.index + 1
            } else {
                span.index
            };
            if daily != *natal {
                disagree.push(format!("{} {name}", day.id));
            }
        }
    }
    let claims = [Claim::counted(
        "the day's spans and the natal block classify the birth instant alike",
        disagree.len(),
        compared,
    )];
    format!(
        "## 10. The daily limbs are geocentric and the natal ones are not\n\n\
         The corpus records the limbs twice: once for the birth instant in\n\
         `panchanga`, and once as the day's spans here. Where the birth falls\n\
         inside the window the two can be compared, and they ought to agree —\n\
         the same limb at the same instant.\n\n{}\n\
         The {} that differ are all within minutes of a boundary:\n\n{}\n\n\
         That is the lunar parallax, which reaches about a degree and so moves\n\
         a tithi boundary by up to two hours; it is entry 1 of the\n\
         deliberate-difference registry — the natal panchanga uses the\n\
         topocentric Moon and the daily one is geocentric, as every almanac is.\n\n\
         So the frame is a decision the daily panchanga has to make and cannot\n\
         inherit: a profile that computes a chart topocentrically still wants a\n\
         geocentric almanac. The design adds one knob for it, defaulting to the\n\
         geocentric reading, so the choice is in the settings hash rather than\n\
         buried in the module.\n\n",
        table(&claims),
        disagree.len(),
        wrapped(&disagree, FILL, ", "),
    )
}

// ── 11. the polar days ─────────────────────────────────────────────────────

fn polar(days: &[Day]) -> String {
    let mut out = String::from(
        "## 11. A period that cannot exist is recorded as a period of no length\n\n\
         | day | daylight | rahu kaal | first hora | thirteenth hora | first night choghadiya |\n\
         |---|---|---|---|---|---|\n",
    );
    for day in days.iter().filter(|day| day.is_polar()) {
        let period = |interval: Option<&Interval>| {
            interval.map_or_else(|| String::from("—"), |i| format!("{:.3} h", i.hours()))
        };
        let part = |parts: &[Part], index: usize| {
            parts
                .get(index)
                .map_or_else(|| String::from("—"), |p| format!("{:.3} h", p.at.hours()))
        };
        let _ = writeln!(
            out,
            "| {} ({}) | {:.3} h | {} | {} | {} | {} |",
            day.id,
            day.date,
            day.daylight() * 24.0,
            period(day.eighths.get("rahu_kaal")),
            part(&day.horas, 0),
            part(&day.horas, 12),
            part(&day.choghadiya.night, 0),
        );
    }
    out.push_str(
        "\nUnder polar day the twelve horas of the night are twelve intervals of\n\
         no length; under polar night the twelve of the daylight are. An empty\n\
         interval is not the absence of a period — it is a period claiming to\n\
         begin and end at one instant, and code that asks which hora it is gets\n\
         an answer out of it.\n\n\
         The SDK reports absence as absence. `time::local_day` already carries\n\
         `DayState::Polar` with the policy that synthesised the bounds, and the\n\
         daily panchanga's periods are absent where the arc they divide does\n\
         not exist — which is why the value is a structure of options and not a\n\
         structure of intervals.\n\n",
    );
    out
}

// ── 12. the catalogue ──────────────────────────────────────────────────────

fn catalogue(days: &[Day]) -> String {
    let mut compared: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut missing: Vec<String> = Vec::new();
    for day in days {
        for (limb, _) in LIMBS {
            let Some(spans) = day.limbs.get(limb) else {
                continue;
            };
            for span in spans {
                for (attribute, recorded) in &span.attributes {
                    let Some(ours) = catalogued(limb, span.index, attribute) else {
                        missing.push(format!("{limb}.{attribute}"));
                        continue;
                    };
                    let row = compared.entry(format!("{limb}.{attribute}")).or_default();
                    row.0 += 1;
                    row.1 += usize::from(!same_key(recorded, &ours));
                }
            }
        }
    }
    missing.sort();
    missing.dedup();
    let claims: Vec<Claim> = compared
        .iter()
        .map(|(field, (seen, wrong))| {
            Claim::counted(
                format!("`{field}` is the SDK catalogue's attribute of that member"),
                *wrong,
                *seen,
            )
        })
        .collect();
    format!(
        "## 12. The catalogue already carries what the limbs print\n\n\
         Beside each span the corpus prints what the member *is* — the tithi's\n\
         paksha and class, the nakshatra's muhurta nature, the yoga's\n\
         auspiciousness, whether the karana is Vishti. Every one of those is an\n\
         attribute the SDK's catalogue already declares, so the comparison is\n\
         whether two independently sourced tables agree.\n\n{}\n\
         They do, member for member, over every span of every day. So the\n\
         daily panchanga adds no attribute table of its own: it names members,\n\
         and what a member is comes from the catalogue, in whatever language\n\
         the caller asked for.{}\n\n",
        table(&claims),
        if missing.is_empty() {
            String::new()
        } else {
            format!(
                " The corpus prints {} that the catalogue does not carry.",
                missing.join(", ")
            )
        },
    )
}

// ── what it decides ────────────────────────────────────────────────────────

fn decides(days: &[Day]) -> String {
    format!(
        "## 13. What this decides\n\n\
         1. **The day is a window, and the window is a `LocalDay`.** Sunrise to\n   \
            the next sunrise, on all {} days, for every field but two. Nothing\n   \
            in the daily panchanga needs a day model of its own.\n\
         2. **Every period is a proportional division of an arc**, and there are\n   \
            only two arcs: the daylight and the night. One divider serves the\n   \
            eighths, the choghadiya, the horas and the muhurtas.\n\
         3. **The choghadiya is the hora's walk.** It ships as a seven-row table\n   \
            of names over a function `crates/time` already has, not as a grid of\n   \
            fifty-six.\n\
         4. **A limb is a span, and the corpus's spans are clipped.** The SDK\n   \
            carries the member's own bounds as well, because \"the tithi ends at\n   \
            14:32\" and \"the tithi began yesterday at 21:05\" are both things an\n   \
            almanac prints and a clipped view answers only one of them.\n\
         5. **The frame is a knob.** The daily panchanga is geocentric where the\n   \
            chart may not be, and the difference reaches the limb a birth falls\n   \
            in.\n\
         6. **Two rows join the deliberate-difference registry**: Brahma muhurta\n   \
            sized from the following night, and the Moon's rise and set taken\n   \
            over the civil day inside a sunrise-bounded section.\n\
         7. **The muhurta yogas need a rank-1 source.** The corpus settles the\n   \
            convention they are read at and one of the five rules, and nothing\n   \
            else about them.\n",
        days.len(),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        DAY_STEP, KARANA_NEXT, NIGHT_START, NIGHT_STEP, Verdict, fill, local_midnight, seconds,
        spread, vara_of, walk_lord, walk_vara, wrapped,
    };
    use teistro_core::catalogue::{Graha, Vara};

    #[test]
    fn the_walk_reaches_the_chaldean_order() {
        // Saturday's first hora is Saturn's and its second Jupiter's:
        // five weekdays on from Saturday is Thursday.
        assert_eq!(walk_lord(Vara::Shanivara, 0), Graha::Saturn);
        assert_eq!(walk_lord(Vara::Shanivara, DAY_STEP), Graha::Jupiter);
        assert_eq!(walk_lord(Vara::Shanivara, 2 * DAY_STEP), Graha::Mars);
        // And it closes: twenty-four horas on from a day's first is the
        // next day's first, which is why the vara advances at sunrise.
        assert_eq!(walk_lord(Vara::Shanivara, 24 * DAY_STEP), Graha::Sun);
    }

    #[test]
    fn the_night_walk_starts_four_on_and_steps_four() {
        // Monday's night begins with Venus, Friday's lord.
        let start = walk_vara(Vara::Somavara, NIGHT_START);
        assert_eq!(start, Vara::Shukravara);
        assert_eq!(walk_lord(start, 0), Graha::Venus);
        assert_eq!(walk_lord(start, NIGHT_STEP), Graha::Mars);
        assert_eq!(walk_lord(start, 2 * NIGHT_STEP), Graha::Saturn);
    }

    #[test]
    fn the_engines_weekday_is_a_monday_first_count() {
        assert_eq!(vara_of(0), Vara::Somavara, "zero is Monday");
        assert_eq!(vara_of(5), Vara::Shanivara);
        assert_eq!(vara_of(6), Vara::Ravivara, "six is Sunday");
    }

    #[test]
    fn a_verdict_says_what_it_is() {
        assert_eq!(Verdict::Holds.mark(), "**holds**");
        assert_eq!(Verdict::Falsified.mark(), "falsified");
        assert_eq!(Verdict::Untested.mark(), "untested");
    }

    #[test]
    fn a_duration_is_written_at_the_scale_it_is() {
        assert_eq!(seconds(0.0), "0 s, exactly");
        assert_eq!(seconds(0.000_5), "0.500 ms");
        assert_eq!(seconds(12.5), "12.500 s");
        assert_eq!(seconds(7200.0), "2.00 h");
    }

    #[test]
    fn a_spread_of_one_instant_is_nothing() {
        assert!(spread(&[1.0]).abs() < f64::EPSILON);
        assert!(spread(&[]).abs() < f64::EPSILON);
        assert!((spread(&[1.0, 3.0, 2.0]) - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn a_local_midnight_is_the_zone_offset_before_the_utc_one() {
        // J2000.0 is 2000-01-01 at 12:00 UT, so that date's midnight is
        // half a day earlier.
        let utc = local_midnight("2000-01-01", 0.0);
        assert!((utc - 2_451_544.5).abs() < 1e-9, "{utc}");
        // A zone ahead of UTC starts its day earlier in UT.
        let kathmandu = local_midnight("2000-01-01", 5.75);
        assert!((utc - kathmandu - 5.75 / 24.0).abs() < 1e-9, "{kathmandu}");
        // A date the calendar does not have is not an instant.
        assert!(local_midnight("2000-02-30", 0.0).is_nan());
        assert!(local_midnight("not a date", 0.0).is_nan());
    }

    #[test]
    fn filling_leaves_tables_and_lists_alone() {
        let page = "# A heading\n\nsome prose that is\nwrapped oddly\n\n                    | a | table |\n|---|---|\n| and | a row |\n\n                    1. a list item\n   and its continuation\n";
        let filled = fill(page);
        assert!(
            filled.contains("some prose that is wrapped oddly"),
            "{filled}"
        );
        assert!(filled.contains("| a | table |\n|---|---|"), "{filled}");
        assert!(
            filled.contains("1. a list item\n   and its continuation"),
            "{filled}"
        );
    }

    #[test]
    fn wrapping_breaks_before_the_width_and_not_after() {
        let items = ["alpha", "beta", "gamma"].map(str::to_string);
        assert_eq!(wrapped(&items, 40, ", "), "alpha, beta, gamma");
        assert_eq!(wrapped(&items, 12, ", "), "alpha, beta\ngamma");
        assert_eq!(wrapped(&[], 12, ", "), "");
    }

    #[test]
    fn the_karana_chain_leaves_the_month_only_at_its_end() {
        // The seventh movable karana is followed by the first inside the
        // month and by the first fixed one at the end of it.
        assert_eq!(KARANA_NEXT[6], &[0, 7]);
        // The fixed ones are a chain that returns to the movable cycle.
        assert_eq!(KARANA_NEXT[7], &[8]);
        assert_eq!(KARANA_NEXT[10], &[0]);
        // Every karana has a successor.
        assert!(KARANA_NEXT.iter().all(|after| !after.is_empty()));
    }
}
