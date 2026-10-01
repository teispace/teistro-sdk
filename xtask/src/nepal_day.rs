//! The measurement pass over **Nepal's day** (crux C187,
//! `03-design/nepal-day-measured.md`).
//!
//! Nepal's daily panchanga prints each day's tithi, nakshatra and yoga
//! with the instant each ends, the days one of them holds both sunrises
//! ("दिनरात", a vriddhi) or none (three in a day, a kshaya), and the
//! sunrise. This pass founds every recorded day at Kathmandu under the
//! committee's reading (`nepali-committee` over the Surya Siddhanta), the
//! same text without its bija, and the modern sky under Lahiri
//! (`nepali-default` over the built-in), and holds each to the print: the
//! ends paired member by member, the flags `span.sunrises` gives, and the
//! sunrise.
//!
//! A flag the committee's reading does not reproduce must carry a reason
//! the pass checks, or the pass fails: the day's own sunrise decides it,
//! the record names a member the text does not, or the record contradicts
//! itself.
//!
//! `cargo xtask nepal-day` writes the page; `check-nepal-day` regenerates
//! it in memory and fails on any difference.

use std::fmt::Write as _;
use std::path::Path;

use teistro::catalogue::Calendar;
use teistro::quantity::{Altitude, Latitude, Longitude, Place};
use teistro::{CalendarDate, Context, Ephemeris, Panchanga, Span, Sunrises, UtcOffset};

use crate::generated::{Output, check, write};
use crate::measure::{Claim, Verdict, fill, median, plural, table};

const PAGE: &str = "docs/03-design/nepal-day-measured.md";

/// Makalukhabar's daily "आजको पञ्चाङ्ग" posts, a day a line: the
/// Gregorian date, the printed sunrise, then for each of `T` (tithi),
/// `N` (nakshatra) and `Y` (yoga) the members and the instants they end,
/// alternating, in Nepal's clock (an hour past 24 is the next morning).
/// A member is its catalogue number from 1, a tithi counted through the
/// month from Shukla Pratipada; `*` is the print's "दिनरात", a member
/// holding both sunrises. Collected on 1 October 2026 from every post of
/// the series; the 8 posts written inline rather than as a table are not
/// read, and "तिष्य" is Pushya.
const RECORDS: &str = include_str!("nepal_day/records.txt");

/// How far a printed end may stand from the reading's, minutes: the print
/// is to the minute, and the SDK's instant is rounded by nothing.
const END_BOUND_MINUTES: f64 = 1.5;

/// How far a printed sunrise may stand from the reading's, minutes.
const SUNRISE_BOUND_MINUTES: f64 = 3.0;

/// A claim's misses are named day by day up to this many.
const LISTED: usize = 12;

/// Nepal's clock, minutes east of Greenwich.
const NPT_MINUTES: f64 = 345.0;

/// Minutes in a day.
const DAY_MINUTES: f64 = 1440.0;

/// The member the source prints for another: its 24th yoga, Shukla, it
/// prints as the 23rd, Shubha (which it also prints rightly).
const SHUBHA: u16 = 23;
const SHUKLA: u16 = 24;

/// A limb of the day, in the order the records write them.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Limb {
    Tithi,
    Nakshatra,
    Yoga,
}

impl Limb {
    const ALL: [Limb; 3] = [Limb::Tithi, Limb::Nakshatra, Limb::Yoga];

    const fn name(self) -> &'static str {
        match self {
            Limb::Tithi => "tithi",
            Limb::Nakshatra => "nakshatra",
            Limb::Yoga => "yoga",
        }
    }

    const fn tag(self) -> &'static str {
        match self {
            Limb::Tithi => "T",
            Limb::Nakshatra => "N",
            Limb::Yoga => "Y",
        }
    }

    /// The members on the wheel.
    const fn count(self) -> u16 {
        match self {
            Limb::Tithi => 30,
            Limb::Nakshatra | Limb::Yoga => 27,
        }
    }

    const fn index(self) -> usize {
        self as usize
    }

    /// The member before one, round the wheel.
    const fn before(self, member: u16) -> u16 {
        (member + self.count() - 2) % self.count() + 1
    }
}

/// What a day's print says of one limb.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Flag {
    /// One member gives way to the next.
    Plain,
    /// A member holds both sunrises.
    Vriddhi,
    /// A member holds neither: three in a day.
    Kshaya,
}

impl Flag {
    const fn name(self) -> &'static str {
        match self {
            Flag::Plain => "plain",
            Flag::Vriddhi => "vriddhi",
            Flag::Kshaya => "kshaya",
        }
    }

    /// The flag a count of ends between two sunrises makes.
    fn of_ends(count: usize) -> Option<Flag> {
        match count {
            0 => Some(Flag::Vriddhi),
            1 => Some(Flag::Plain),
            2 => Some(Flag::Kshaya),
            _ => None,
        }
    }
}

/// One limb as a day prints it.
struct Printed {
    members: Vec<u16>,
    /// The ends, minutes after the date's civil midnight.
    ends: Vec<f64>,
    flag: Flag,
}

impl Printed {
    /// Whether the members follow one another round the wheel, as a day's
    /// must: a record that skips or repeats one contradicts itself.
    fn consistent(&self, limb: Limb) -> bool {
        match self.flag {
            Flag::Vriddhi => self.members.windows(2).all(|pair| pair[0] == pair[1]),
            Flag::Plain | Flag::Kshaya => self
                .members
                .windows(2)
                .all(|pair| pair[1] == pair[0] % limb.count() + 1),
        }
    }
}

/// One recorded day.
struct Record {
    date: (i32, u8, u8),
    /// The printed sunrise, minutes after civil midnight.
    sunrise: f64,
    limbs: [Printed; 3],
}

impl Record {
    fn limb(&self, limb: Limb) -> &Printed {
        &self.limbs[limb.index()]
    }
}

/// A reading of Nepal's day, and what the page expects of it.
struct Reading {
    name: &'static str,
    profile: &'static str,
    patch: Option<&'static str>,
    ephemeris: fn() -> Ephemeris,
    /// The committee's reading, which every flag is held to.
    shipped: bool,
    /// Whether its ends are the print's within the bound.
    ends_agree: bool,
    /// Whether its sunrise is the print's within the bound.
    sunrise_agrees: bool,
}

const READINGS: [Reading; 3] = [
    Reading {
        name: "the Surya Siddhanta with the committee's bija, in its own zodiac (`nepali-committee`)",
        profile: "nepali-committee",
        patch: None,
        ephemeris: || Ephemeris::SuryaSiddhanta,
        shipped: true,
        ends_agree: true,
        sunrise_agrees: false,
    },
    Reading {
        name: "the text without a bija",
        profile: "nepali-committee",
        patch: Some(r#"{"frame": {"siddhanta": {"kind": "SURYA", "bija": {"kind": "NONE"}}}}"#),
        ephemeris: || Ephemeris::SuryaSiddhanta,
        shipped: false,
        ends_agree: false,
        sunrise_agrees: false,
    },
    Reading {
        name: "the modern sky under Lahiri (`nepali-default`)",
        profile: "nepali-default",
        patch: None,
        ephemeris: || Ephemeris::Builtin,
        shipped: false,
        ends_agree: false,
        sunrise_agrees: true,
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
            i32::from(check(root, &[Output::new(PAGE, text)], "cargo xtask nepal-day") != 0)
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

fn gregorian((y, m, d): (i32, u8, u8)) -> CalendarDate {
    CalendarDate::defined(Calendar::Gregorian, y, m, d)
}

fn iso((y, m, d): (i32, u8, u8)) -> String {
    format!("{y}-{m:02}-{d:02}")
}

/// `HH:MM` as minutes, the hour allowed past 24.
fn minutes(text: &str) -> Result<f64, String> {
    let (h, m) = text
        .split_once(':')
        .ok_or_else(|| format!("'{text}' is no time"))?;
    let h: u16 = h.parse().map_err(|_| format!("'{text}' is no time"))?;
    let m: u16 = m.parse().map_err(|_| format!("'{text}' is no time"))?;
    if m >= 60 {
        return Err(format!("'{text}' is no time"));
    }
    Ok(f64::from(h * 60 + m))
}

/// One limb's tokens: members and ends alternating, `*` for a vriddhi.
fn printed(limb: Limb, tokens: &[&str]) -> Result<Printed, String> {
    let mut members = Vec::new();
    let mut ends = Vec::new();
    let mut vriddhi = false;
    for (k, token) in tokens.iter().enumerate() {
        if k % 2 == 0 {
            let member: u16 = token
                .parse()
                .map_err(|_| format!("'{token}' is no member"))?;
            if !(1..=limb.count()).contains(&member) {
                return Err(format!("{member} is no {}", limb.name()));
            }
            members.push(member);
        } else if *token == "*" {
            vriddhi = true;
        } else {
            ends.push(minutes(token)?);
        }
    }
    if members.len() != tokens.len().div_ceil(2) || tokens.len() % 2 == 0 {
        return Err(format!("{} does not end on a member", limb.name()));
    }
    let flag = if vriddhi {
        if members.len() != 2 || !ends.is_empty() {
            return Err(format!("a {} vriddhi is 'X * X'", limb.name()));
        }
        Flag::Vriddhi
    } else {
        Flag::of_ends(ends.len())
            .filter(|flag| *flag != Flag::Vriddhi)
            .ok_or_else(|| format!("{} ends for one {}", ends.len(), limb.name()))?
    };
    Ok(Printed {
        members,
        ends,
        flag,
    })
}

/// The records, refusing a malformed line.
fn records() -> Result<Vec<Record>, String> {
    let mut out: Vec<Record> = Vec::new();
    for line in RECORDS.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        let [date, sunrise, rest @ ..] = fields.as_slice() else {
            return Err(format!("'{line}' is no record"));
        };
        let parts: Vec<u16> = date.split('-').filter_map(|p| p.parse().ok()).collect();
        let [year, month, day] = parts.as_slice() else {
            return Err(format!("'{date}' is no date"));
        };
        let date = (
            i32::from(*year),
            u8::try_from(*month).map_err(|e| e.to_string())?,
            u8::try_from(*day).map_err(|e| e.to_string())?,
        );
        if out.last().is_some_and(|last| last.date >= date) {
            return Err(format!("{} is out of order", iso(date)));
        }
        let at = |tag: &str| rest.iter().position(|t| *t == tag);
        let (Some(tithi), Some(star), Some(yoga)) = (
            at(Limb::Tithi.tag()),
            at(Limb::Nakshatra.tag()),
            at(Limb::Yoga.tag()),
        ) else {
            return Err(format!("{}: a limb is missing", iso(date)));
        };
        let limb = |limb: Limb, from: usize, to: usize| {
            printed(limb, rest.get(from + 1..to).unwrap_or_default())
                .map_err(|e| format!("{}: {e}", iso(date)))
        };
        out.push(Record {
            date,
            sunrise: minutes(sunrise)?,
            limbs: [
                limb(Limb::Tithi, tithi, star)?,
                limb(Limb::Nakshatra, star, yoga)?,
                limb(Limb::Yoga, yoga, rest.len())?,
            ],
        });
    }
    Ok(out)
}

/// The Julian day of minutes after a date's civil midnight in Nepal.
fn jd_of(date: (i32, u8, u8), minutes: f64) -> Result<f64, String> {
    let midnight = teistro_calendar::gregorian::fixed_from_gregorian(date.0, date.1, date.2)
        .jd_at_midnight()
        .map_err(|e| e.to_string())?
        .get();
    Ok(midnight + (minutes - NPT_MINUTES) / DAY_MINUTES)
}

/// A limb's spans on a founded day: the member's number, when it ended,
/// and which sunrises it held.
struct Turn {
    member: u16,
    from: f64,
    to: f64,
    sunrises: Sunrises,
}

fn turns_of<T: Copy>(day: &Panchanga, spans: &[Span<T>], number: fn(T) -> u16) -> Vec<Turn> {
    spans
        .iter()
        .map(|span| Turn {
            member: number(span.member) + 1,
            from: span.whole.from.get(),
            to: span.whole.to.get(),
            sunrises: day.sunrises(span),
        })
        .collect()
}

/// A founded day's turns for each limb.
fn turns(day: &Panchanga, limb: Limb) -> Vec<Turn> {
    match limb {
        Limb::Tithi => turns_of(day, &day.limbs.tithi, |m| m as u16),
        Limb::Nakshatra => turns_of(day, &day.limbs.nakshatra, |m| m as u16),
        Limb::Yoga => turns_of(day, &day.limbs.yoga, |m| m as u16),
    }
}

/// The flag a founded day gives a limb.
fn flag_of(turns: &[Turn]) -> Flag {
    if turns.iter().any(|turn| turn.sunrises.is_vriddhi()) {
        Flag::Vriddhi
    } else if turns.iter().any(|turn| turn.sunrises.is_kshaya()) {
        Flag::Kshaya
    } else {
        Flag::Plain
    }
}

fn context(reading: &Reading) -> Result<Context, String> {
    let builder = Context::builder()
        .profile(reading.profile)
        .ephemeris([(reading.ephemeris)()]);
    match reading.patch {
        Some(json) => builder.settings_json(json),
        None => builder,
    }
    .build()
    .map_err(|e| e.to_string())
}

/// Each record's founded day under a reading, a month at a time: a range
/// holds at most a year, and a month's is what a caller asks for.
fn days(reading: &Reading, records: &[Record]) -> Result<Vec<Panchanga>, String> {
    let context = context(reading)?;
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
                .ok_or_else(|| format!("no founded day for {}", iso(record.date)))?;
            out.push(day.clone());
        }
    }
    Ok(out)
}

/// Every reading's founded days, in `READINGS` order, a thread each: a
/// day founds the Moon's series, which is most of the pass's cost.
fn founded(records: &[Record]) -> Result<Vec<Vec<Panchanga>>, String> {
    std::thread::scope(|scope| {
        let handles: Vec<_> = READINGS
            .iter()
            .map(|reading| scope.spawn(move || days(reading, records)))
            .collect();
        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .map_err(|_| String::from("a founding thread panicked"))?
            })
            .collect()
    })
}

/// What the page's sections gather.
#[derive(Default)]
struct Findings {
    claims: Vec<Claim>,
    problems: Vec<String>,
}

impl Findings {
    /// A claim the page expects to hold or to fail, and a problem when it
    /// does the other.
    fn expect(&mut self, claim: Claim, holds: bool) {
        if (claim.verdict == Verdict::Holds) != holds {
            self.problems.push(format!(
                "'{}' is {:?} ({}) and the page expects it {}",
                claim.rule,
                claim.verdict,
                claim.measured,
                if holds { "to hold" } else { "not to" }
            ));
        }
        self.claims.push(claim);
    }
}

/// Minutes, signed, to one decimal.
fn signed(value: f64) -> String {
    format!("{value:+.1}")
}

/// §3: the printed ends against each reading's, member by member. A
/// record that contradicts itself is left out and counted.
fn ends(
    records: &[Record],
    founded: &[Vec<Panchanga>],
    found: &mut Findings,
) -> Result<String, String> {
    let mut out = String::from(
        "\n## 3. The ends\n\n\
         Each printed end against the end of the same member in the reading,\n\
         in minutes, the reading's less the print's. A member the reading\n\
         does not place within a day of the printed end is unpaired.\n\n\
         | reading | limb | ends | median | least | most | beyond 1.5 min | unpaired |\n\
         |---|---|---|---|---|---|---|---|\n",
    );
    for (reading, days) in READINGS.iter().zip(founded) {
        for limb in Limb::ALL {
            // Every span the reading founds for the limb, once.
            // Every end the reading places, once: each member's, and the
            // one before a day's first member, where that member began.
            let mut spans: Vec<(u16, f64)> = days
                .iter()
                .flat_map(|day| {
                    let turns = turns(day, limb);
                    let before = turns
                        .first()
                        .map(|first| (limb.before(first.member), first.from));
                    before
                        .into_iter()
                        .chain(turns.into_iter().map(|turn| (turn.member, turn.to)))
                        .collect::<Vec<_>>()
                })
                .collect();
            spans.sort_by(|a, b| a.1.total_cmp(&b.1));
            spans.dedup_by(|a, b| a.0 == b.0 && (a.1 - b.1).abs() < 1e-9);
            let mut differences = Vec::new();
            let mut unpaired = 0;
            let mut missed = Vec::new();
            for record in records {
                let printed = record.limb(limb);
                if !printed.consistent(limb) {
                    continue;
                }
                for (member, end) in printed.members.iter().zip(&printed.ends) {
                    let at = jd_of(record.date, *end)?;
                    let nearest = spans
                        .iter()
                        .filter(|(m, _)| m == member)
                        .map(|(_, to)| (to - at) * DAY_MINUTES)
                        .min_by(|a, b| a.abs().total_cmp(&b.abs()))
                        .filter(|minutes| minutes.abs() <= DAY_MINUTES);
                    match nearest {
                        Some(minutes) => differences.push(minutes),
                        None => unpaired += 1,
                    }
                    if nearest.is_none_or(|minutes| minutes.abs() > END_BOUND_MINUTES) {
                        missed.push(iso(record.date));
                    }
                }
            }
            let beyond = differences
                .iter()
                .filter(|m| m.abs() > END_BOUND_MINUTES)
                .count();
            let paired = differences.len();
            missed.dedup();
            let least = differences.iter().copied().fold(f64::INFINITY, f64::min);
            let most = differences
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max);
            let middle = median(differences.iter().copied());
            let _ = writeln!(
                out,
                "| {} | {} | {paired} | {} | {} | {} | {beyond} | {unpaired} |",
                reading.name,
                limb.name(),
                signed(middle),
                signed(least),
                signed(most),
            );
            found.expect(
                Claim::counted(
                    format!(
                        "{}: every printed {} end within {END_BOUND_MINUTES} minutes",
                        reading.name,
                        limb.name()
                    ),
                    beyond + unpaired,
                    paired + unpaired,
                )
                .with_note(if missed.is_empty() || missed.len() > LISTED {
                    format!("median {} min", signed(middle))
                } else {
                    format!("median {} min; {}", signed(middle), missed.join(", "))
                }),
                reading.ends_agree,
            );
        }
    }
    Ok(out)
}

/// §4: the printed sunrise against each reading's.
fn sunrises(
    records: &[Record],
    founded: &[Vec<Panchanga>],
    found: &mut Findings,
) -> Result<String, String> {
    let mut out = String::from(
        "\n## 4. The sunrise\n\n\
         Each printed sunrise against the reading's, in minutes, the\n\
         reading's less the print's.\n\n\
         | reading | days | median | least | most |\n|---|---|---|---|---|\n",
    );
    for (reading, days) in READINGS.iter().zip(founded) {
        let mut differences = Vec::with_capacity(records.len());
        for (record, day) in records.iter().zip(days) {
            differences
                .push((day.day.sunrise.get() - jd_of(record.date, record.sunrise)?) * DAY_MINUTES);
        }
        let beyond = differences
            .iter()
            .filter(|m| m.abs() > SUNRISE_BOUND_MINUTES)
            .count();
        let least = differences.iter().copied().fold(f64::INFINITY, f64::min);
        let most = differences
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let middle = median(differences.iter().copied());
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} |",
            reading.name,
            records.len(),
            signed(middle),
            signed(least),
            signed(most)
        );
        found.expect(
            Claim::counted(
                format!(
                    "{}: every printed sunrise within {SUNRISE_BOUND_MINUTES} minutes",
                    reading.name
                ),
                beyond,
                records.len(),
            )
            .with_note(format!("median {} min", signed(middle))),
            reading.sunrise_agrees,
        );
    }
    Ok(out)
}

/// Why the committee's reading gives a day's limb another flag than the
/// print, each checked rather than asserted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Reason {
    /// Counted between the printed sunrises, the reading's ends give the
    /// printed flag: an end falls between the text's sunrise and the
    /// print's.
    Sunrise,
    /// The print's vriddhi of Shubha is Shubha then Shukla, which the
    /// source names Shubha too.
    ShuklaAsShubha,
}

impl Reason {
    const fn says(self) -> &'static str {
        match self {
            Reason::Sunrise => "an end between the text's sunrise and the print's",
            Reason::ShuklaAsShubha => "Shubha then Shukla, which the source prints as Shubha",
        }
    }
}

/// The printed sunrise after a record: the next day's print where it is
/// recorded, otherwise this one's carried by the reading's day length.
fn next_printed_sunrise(records: &[Record], index: usize, day: &Panchanga) -> Result<f64, String> {
    let record = records.get(index).ok_or("no record")?;
    let today = jd_of(record.date, record.sunrise)?;
    let tomorrow = teistro_calendar::gregorian::fixed_from_gregorian(
        record.date.0,
        record.date.1,
        record.date.2,
    )
    .plus_days(1);
    let (y, m, d) = teistro_calendar::gregorian::gregorian_from_fixed(tomorrow);
    match records.get(index + 1).filter(|next| next.date == (y, m, d)) {
        Some(next) => jd_of(next.date, next.sunrise),
        None => Ok(today + (day.day.next_sunrise.get() - day.day.sunrise.get())),
    }
}

/// The reason a disagreeing flag has, or `None` when it has none the pass
/// can check.
fn reason(
    limb: Limb,
    printed: &Printed,
    turns: &[Turn],
    (sunrise, next): (f64, f64),
) -> Option<Reason> {
    // The boundaries the reading placed around the day: the first
    // member's start and every member's end.
    let boundaries = turns
        .first()
        .map(|turn| turn.from)
        .into_iter()
        .chain(turns.iter().map(|turn| turn.to));
    let between = boundaries.filter(|at| *at >= sunrise && *at < next).count();
    if Flag::of_ends(between) == Some(printed.flag) {
        return Some(Reason::Sunrise);
    }
    let members: Vec<u16> = turns.iter().map(|turn| turn.member).collect();
    if limb == Limb::Yoga
        && printed.flag == Flag::Vriddhi
        && printed.members == [SHUBHA, SHUBHA]
        && members == [SHUBHA, SHUKLA]
    {
        return Some(Reason::ShuklaAsShubha);
    }
    None
}

/// §5: every printed flag against the committee's reading; each
/// disagreement named with its reason, and one without a reason fails
/// the pass.
fn flags(
    records: &[Record],
    founded: &[Vec<Panchanga>],
    found: &mut Findings,
) -> Result<(String, usize), String> {
    let (reading, days) = READINGS
        .iter()
        .zip(founded)
        .find(|(reading, _)| reading.shipped)
        .ok_or("no shipped reading")?;
    let mut out = format!(
        "\n## 5. The flags\n\n\
         Each day's printed flag for each limb — plain, a vriddhi (\"दिनरात\")\n\
         or a kshaya (three in a day) — against the flag {}\n\
         gives through `span.sunrises`. A record that contradicts itself\n\
         (§6) is left out. Every disagreement is listed with the reason the\n\
         pass checked for it.\n\n\
         | limb | days | printed vriddhi | printed kshaya | agree | disagree |\n\
         |---|---|---|---|---|---|\n",
        reading.name
    );
    let mut misnamed = 0;
    let mut listed =
        String::from("\n| day | limb | printed | the reading's | why |\n|---|---|---|---|---|\n");
    for limb in Limb::ALL {
        let (mut days_read, mut vriddhi, mut kshaya, mut agree) = (0, 0, 0, 0);
        let mut disagree = 0;
        for (index, (record, day)) in records.iter().zip(days).enumerate() {
            let printed = record.limb(limb);
            if !printed.consistent(limb) {
                continue;
            }
            days_read += 1;
            vriddhi += usize::from(printed.flag == Flag::Vriddhi);
            kshaya += usize::from(printed.flag == Flag::Kshaya);
            let turns = turns(day, limb);
            let ours = flag_of(&turns);
            if ours == printed.flag {
                agree += 1;
                continue;
            }
            disagree += 1;
            let sunrise = jd_of(record.date, record.sunrise)?;
            let next = next_printed_sunrise(records, index, day)?;
            let why = reason(limb, printed, &turns, (sunrise, next));
            match why {
                Some(why) => {
                    misnamed += usize::from(why == Reason::ShuklaAsShubha);
                    let _ = writeln!(
                        listed,
                        "| {} | {} | {} | {} | {} |",
                        iso(record.date),
                        limb.name(),
                        printed.flag.name(),
                        ours.name(),
                        why.says()
                    );
                }
                None => found.problems.push(format!(
                    "{} {}: printed {}, the committee's reading {}, and no reason the pass can check",
                    iso(record.date),
                    limb.name(),
                    printed.flag.name(),
                    ours.name()
                )),
            }
        }
        let _ = writeln!(
            out,
            "| {} | {days_read} | {vriddhi} | {kshaya} | {agree} | {disagree} |",
            limb.name()
        );
        found.claims.push(Claim::stated(
            format!(
                "{}: every printed {} flag, or a reason the pass checks",
                reading.name,
                limb.name()
            ),
            Verdict::Holds,
            format!("{agree} of {days_read} agree; the {disagree} others each for a reason in §5"),
        ));
    }
    out.push_str(&listed);
    Ok((out, misnamed))
}

/// §6: the records that contradict themselves.
fn contradictions(records: &[Record]) -> String {
    let mut out = String::from(
        "\n## 6. Records that contradict themselves\n\n\
         A day's members follow one another round the wheel, so a record\n\
         that skips or repeats one says two things; it is left out of §3 and\n\
         §5.\n\n\
         | day | limb | printed members |\n|---|---|---|\n",
    );
    for record in records {
        for limb in Limb::ALL {
            let printed = record.limb(limb);
            if !printed.consistent(limb) {
                let _ = writeln!(
                    out,
                    "| {} | {} | {} |",
                    iso(record.date),
                    limb.name(),
                    printed
                        .members
                        .iter()
                        .map(u16::to_string)
                        .collect::<Vec<_>>()
                        .join(" → ")
                );
            }
        }
    }
    out
}

/// How often the source names Shukla rightly and how often as Shubha: a
/// Shubha a plain record follows with Brahma is a Shukla misnamed.
fn shukla_named(records: &[Record]) -> (usize, usize) {
    let mut right = 0;
    let mut misnamed = 0;
    for record in records {
        let members = &record.limb(Limb::Yoga).members;
        right += members.iter().filter(|m| **m == SHUKLA).count();
        misnamed += members
            .windows(2)
            .filter(|pair| pair[0] == SHUBHA && pair[1] == SHUKLA + 1)
            .count();
    }
    (right, misnamed)
}

fn page() -> Result<String, String> {
    let records = records()?;
    let founded = founded(&records)?;
    let mut found = Findings::default();
    let ends = ends(&records, &founded, &mut found)?;
    let sunrises = sunrises(&records, &founded, &mut found)?;
    let (flags, vriddhi_misnamed) = flags(&records, &founded, &mut found)?;
    let contradictions = contradictions(&records);
    let (right, misnamed) = shukla_named(&records);

    let mut out = String::new();
    out.push_str("# Nepal's day, measured\n\n");
    out.push_str(
        "Status: `generated` by `cargo xtask nepal-day`. Do not edit:\n\
         `check-nepal-day` regenerates this page and fails on any difference.\n\n\
         It measures the almanac's limbs at Kathmandu against Nepal's daily\n\
         panchanga: when each tithi, nakshatra and yoga ends, which days one\n\
         of them holds both sunrises or neither (`span.sunrises`), and the\n\
         sunrise. Nepal's national panchanga committee requires its makers\n\
         to compute by the Surya Siddhanta, and its printed Moon is the\n\
         text's with a bija on its apsis (`calendars/bikram-sambat.md`, R2;\n\
         C28); this page asks whether the daily print is that text, in the\n\
         text's own zodiac, and what the modern sky would print instead\n\
         (C187).\n",
    );
    out.push_str("\n## 1. The claims\n\n");
    out.push_str(&table(&found.claims));
    let _ = write!(
        out,
        "\n## 2. The records\n\n\
         Makalukhabar's daily \"आजको पञ्चाङ्ग\" posts, {} days from {} to {},\n\
         each read for its Gregorian date, its sunrise and its tithi,\n\
         nakshatra and yoga with their ends; the page holds them in\n\
         `xtask/src/nepal_day/records.txt`, a day a line. The source prints\n\
         Pushya as \"तिष्य\", and names the 24th yoga, Shukla, rightly {right}\n\
         times and as Shubha {as_shubha}: {misnamed} times before Brahma (§6),\n\
         and {vriddhi_misnamed} times as a Shubha lasting day and night where the text\n\
         has Shubha then Shukla (§5).\n",
        records.len(),
        records.first().map_or_else(String::new, |r| iso(r.date)),
        records.last().map_or_else(String::new, |r| iso(r.date)),
        as_shubha = plural(misnamed + vriddhi_misnamed, "time"),
    );
    out.push_str(&ends);
    out.push_str(&sunrises);
    out.push_str(&flags);
    out.push_str(&contradictions);
    out.push_str(
        "\n## 7. What the records decide\n\n\
         The daily print is the Surya Siddhanta with the committee's bija,\n\
         read in the text's own zodiac: its ends are the text's to the\n\
         minute the print is written in, where the text without the bija\n\
         parts from them by up to a quarter of an hour and the modern sky by\n\
         hours. So `nepali-committee` reads the text over `SURYA_SIDDHANTA`\n\
         with `SuryaBija::NepalCommittee` and the text's ayanamsha, and a\n\
         search over a provider that defines its zodiac reads the provider's\n\
         own longitudes (`ChartZodiac::searched`) rather than shifting the\n\
         tropical ones by the catalogue member of the same name, which stands\n\
         1.6° from the text today.\n\n\
         The print's sunrise is not the text's: the text's carries no\n\
         equation of time (C37), and the print's is a modern one (C39). A\n\
         flag decided within minutes of sunrise can differ for that alone,\n\
         and §5 names each such day; reproducing them needs the modern\n\
         Sun's horizon beside the text's limbs, the mixed provider C38 and\n\
         C39 record.\n",
    );
    if found.problems.is_empty() {
        Ok(fill(&out))
    } else {
        Err(found.problems.join("\n      "))
    }
}
