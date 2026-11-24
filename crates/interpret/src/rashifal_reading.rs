//! The rashifal's reading voice (`03-design/rashifal-reading.md`, SR-152):
//! what a sign's period says in the register the dailies print, as a plan
//! of **intents** each naming the facts it rests on.
//!
//! The composer chooses everything a checker must trust: which facts are
//! said, how firmly ([`Hedge`]), how many cautions and who is named. A
//! language's pack maps each intent to its own patterns and never chooses a
//! fact (C2362). The wording is picked among a pack's variants by
//! [`ReadingItem::variant`], a seeded number (C2360).
//!
//! It reads a [`RashiReading`], the baseline's score of it and, for a week,
//! the daily scores of its days. It judges nothing the reading has not.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, LifeArea, Rashi, ScoreBand};
use teistro_core::settings::ScoreBands;
use teistro_gochar::Verdict;
use teistro_gochar::hits::HitEvent;
use teistro_gochar::sade_sati::Phase;
use teistro_intl::{Params, Value};
use teistro_rashifal::baseline::{BaselineScore, LuckyElements};
use teistro_rashifal::{Period, RashiReading};

/// The four areas a reader sees, each read from the baseline's life areas
/// (`03-design/rashifal-reading.md`, "The four areas"). Its name is the
/// reading pack's (`reading.rashifal.areaName.WORK`), since the grouping is
/// the reader's voice and not a kind of the texts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReadingArea {
    /// The baseline's CAREER: the 3rd, 6th, 10th and 11th houses.
    Work,
    /// The baseline's FINANCE: the 2nd and 11th houses.
    Money,
    /// The mean of RELATIONSHIPS and FAMILY: the 2nd, 4th, 5th and 7th
    /// houses (C2359).
    LoveFamily,
    /// The baseline's HEALTH: the 1st, 6th, 8th and 12th houses.
    Health,
}

impl ReadingArea {
    /// The four, in the order a page shows them.
    pub const ALL: [ReadingArea; 4] = [
        ReadingArea::Work,
        ReadingArea::Money,
        ReadingArea::LoveFamily,
        ReadingArea::Health,
    ];

    /// The member's key, as serde writes it.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            ReadingArea::Work => "WORK",
            ReadingArea::Money => "MONEY",
            ReadingArea::LoveFamily => "LOVE_FAMILY",
            ReadingArea::Health => "HEALTH",
        }
    }

    /// The baseline's areas it is read from.
    const fn baseline(self) -> &'static [LifeArea] {
        match self {
            ReadingArea::Work => &[LifeArea::Career],
            ReadingArea::Money => &[LifeArea::Finance],
            ReadingArea::LoveFamily => &[LifeArea::Relationships, LifeArea::Family],
            ReadingArea::Health => &[LifeArea::Health],
        }
    }

    /// The houses from the sign that feed it, as the baseline's table feeds
    /// its areas (`teistro-rashifal`'s `areas_of`).
    const fn houses(self) -> &'static [u8] {
        match self {
            ReadingArea::Work => &[3, 6, 10, 11],
            ReadingArea::Money => &[2, 11],
            ReadingArea::LoveFamily => &[2, 4, 5, 7],
            ReadingArea::Health => &[1, 6, 8, 12],
        }
    }
}

/// How firmly an item is said, from its facts (the voice guide's §5.2).
/// Nothing is said as certain.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Hedge {
    /// An excellent band with two facts agreeing.
    Firm,
    /// One good fact.
    Likely,
    /// A good fact obstructed, or a mixed band.
    Possible,
    /// A caution: a not-good fact or a band for care.
    May,
}

/// A fact an item rests on, as an index into the answer that carries it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "fact", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Fact {
    /// A graha's house and verdict from the sign (`reading.gochar.grahas`).
    Transit {
        /// The graha.
        graha: Graha,
    },
    /// Saturn's standing: its house, Sade Sati phase or spell.
    Saturn,
    /// The n-th event of the reading (`reading.events[index]`).
    Event {
        /// Its index.
        index: usize,
    },
    /// The n-th key influence of the score (`score.keyInfluences[index]`).
    Influence {
        /// Its index.
        index: usize,
    },
    /// The overall's band (`score.band`).
    Band,
    /// An area's band, read from the score's areas.
    AreaBand {
        /// The area.
        area: ReadingArea,
    },
    /// The n-th day's score of a week (`ReadingInput::days[index]`).
    Day {
        /// Its index, the week's first day 0.
        index: usize,
    },
    /// The Moon's change of sign within the day (`ReadingInput::moon_turn`).
    MoonTurn,
    /// The sign's lucky items (`score.lucky`).
    Lucky,
    /// The sign's name syllables.
    Syllables,
}

/// One thing said: an intent, its slots, how firmly, and the facts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ReadingItem {
    /// The intent, a message key of the reading pack
    /// (`reading.rashifal.area.WORK.GOOD`).
    pub intent: String,
    /// Its slots.
    #[cfg_attr(
        feature = "schema",
        schemars(with = "std::collections::BTreeMap<String, serde_json::Value>")
    )]
    pub params: Params,
    /// How firmly it is said.
    pub hedge: Hedge,
    /// Whether it is a caution, which the period's budget counts.
    pub caution: bool,
    /// Which of the pack's wordings of the intent to say; a pack with fewer
    /// reads it modulo its count (C2360).
    pub variant: u32,
    /// The facts it says; never empty.
    pub rests_on: Vec<Fact>,
}

/// Where in the period a part of the reading falls.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Frame {
    /// The whole period.
    Whole,
    /// A day before the Moon's change of sign.
    BeforeMoon,
    /// A day after it.
    AfterMoon,
    /// A week's first two days.
    Start,
    /// A week's middle three.
    Middle,
    /// A week's last two.
    End,
    /// A month before its largest event.
    FirstHalf,
    /// A month from it.
    SecondHalf,
    /// A year's slow grahas.
    Slow,
    /// A year's first quarter.
    Quarter1,
    /// Its second.
    Quarter2,
    /// Its third.
    Quarter3,
    /// Its fourth.
    Quarter4,
}

/// A part of the reading: a frame and what is said in it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ReadingPart {
    /// Where it falls.
    pub frame: Frame,
    /// What it says, in order.
    pub items: Vec<ReadingItem>,
}

/// One area as a page shows it: its band and its note.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AreaReading {
    /// The area.
    pub area: ReadingArea,
    /// Its score, 0 to 100, as the baseline scores it.
    pub score: u8,
    /// Its band under the overall's thresholds.
    pub band: ScoreBand,
    /// Its one-line note.
    pub note: ReadingItem,
}

/// A sign's period in the reading voice.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct ReadingPlan {
    /// The sign.
    pub rashi: Rashi,
    /// The period's kind.
    pub period: Period,
    /// The line over the reading.
    pub headline: ReadingItem,
    /// The reading, part by part.
    pub parts: Vec<ReadingPart>,
    /// The plain why: the strongest influence, and a period's events.
    pub why: Vec<ReadingItem>,
    /// The four areas.
    pub areas: Vec<AreaReading>,
    /// The week's best day, as an index into its days, when one day's own
    /// score is the strict highest (C2363).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub best_day: Option<usize>,
    /// The sign's lucky items.
    pub lucky: LuckyElements,
    /// The sign's nine name syllables, the padas' in order, in IAST as
    /// Muhurta Chintamani prints them (C299); a locale spells them.
    pub syllables: Vec<String>,
}

/// A day's score within a week.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayScore {
    /// The day's overall, 0 to 100.
    pub overall: u8,
}

/// The Moon's change of sign within a day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonTurn {
    /// The local time, minutes after midnight.
    pub minutes: u16,
    /// Her house from the sign before it.
    pub from_house: u8,
    /// Her house after it.
    pub to_house: u8,
    /// Whether that house is good for her (v. 2).
    pub good_after: bool,
}

/// What the composer reads.
#[derive(Clone, Copy, Debug)]
pub struct ReadingInput<'a> {
    /// The sign's reading.
    pub reading: &'a RashiReading,
    /// The baseline's score of it.
    pub score: &'a BaselineScore,
    /// The period's kind.
    pub period: Period,
    /// The period's first day, as a fixed day number: the seed's.
    pub first_day: i64,
    /// The period's first and last instant (Julian days, UT), which place
    /// its events in a half or a quarter.
    pub span: (f64, f64),
    /// The thresholds the overall was banded with.
    pub bands: ScoreBands,
    /// A day's Moon turn, when she changes sign that day.
    pub moon_turn: Option<MoonTurn>,
    /// A week's daily scores, its days in order.
    pub days: &'a [DayScore],
}

const NS: &str = "reading.rashifal";

/// The period's key, as serde writes it.
const fn period_key(period: Period) -> &'static str {
    match period {
        Period::Daily => "DAILY",
        Period::Weekly => "WEEKLY",
        Period::Monthly => "MONTHLY",
        Period::Yearly => "YEARLY",
    }
}

/// The order the voice guide weighs grahas in when it chooses the why:
/// the slow first (§8.4).
const WEIGHT: [Graha; 9] = [
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
    Graha::Sun,
    Graha::Mars,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
];

/// Composes a sign's period in the reading voice.
///
/// ```
/// use teistro_core::catalogue::{Graha, Rashi};
/// use teistro_core::settings::ScoreBands;
/// use teistro_gochar::Transit;
/// use teistro_interpret::rashifal_reading::{ReadingInput, reading};
/// use teistro_rashifal::baseline::{Panchanga, baseline_score};
/// use teistro_rashifal::{Period, RashifalRules, rashifal};
/// use teistro_core::catalogue::{Tithi, Yoga};
///
/// let mut transits = [Transit::new(Rashi::Aries, 5.0); 9];
/// transits[Graha::Saturn as usize] = Transit::new(Rashi::Pisces, 12.0);
/// transits[Graha::Rahu as usize] = Transit::new(Rashi::Virgo, 3.0);
/// transits[Graha::Ketu as usize] = Transit::new(Rashi::Pisces, 3.0);
/// let read = rashifal(&transits, &[], &RashifalRules::default());
/// let panchanga = Panchanga { tithi: Tithi::ShuklaPratipada, yoga: Yoga::Priti, muhurta_yogas: 1 };
/// let score = baseline_score(&read[0], &[], &[false; 9], panchanga, Period::Daily, ScoreBands::DEFAULT);
/// let plan = reading(&ReadingInput {
///     reading: &read[0],
///     score: &score,
///     period: Period::Daily,
///     first_day: 739_000,
///     span: (2_461_000.5, 2_461_001.5),
///     bands: ScoreBands::DEFAULT,
///     moon_turn: None,
///     days: &[],
/// });
/// // Every item names the facts it rests on.
/// assert!(plan.parts.iter().flat_map(|part| &part.items).all(|item| !item.rests_on.is_empty()));
/// assert_eq!(plan.areas.len(), 4);
/// assert_eq!(plan.syllables.len(), 9);
/// ```
#[must_use]
pub fn reading(input: &ReadingInput<'_>) -> ReadingPlan {
    let mut say = Sayer::new(input);
    let areas = area_readings(input, &mut say);
    let headline = headline(input, &areas, &mut say);
    let why = why(input, &mut say);
    let (parts, best_day) = parts(input, &areas, &mut say);
    ReadingPlan {
        rashi: input.reading.rashi,
        period: input.period,
        headline,
        parts,
        why,
        areas,
        best_day,
        lucky: input.score.lucky,
        syllables: syllables(input.reading.rashi),
    }
}

/// Builds items, numbering each intent's wordings so that no intent repeats
/// a wording within the plan and the same period always reads the same.
struct Sayer {
    seed: u64,
    used: Vec<(String, u32)>,
}

impl Sayer {
    fn new(input: &ReadingInput<'_>) -> Sayer {
        let span = match input.period {
            Period::Daily => 1,
            Period::Weekly => 7,
            Period::Monthly => 30,
            Period::Yearly => 365,
        };
        // Consecutive periods differ by one, so two consecutive days never
        // read one intent alike where a pack has two wordings of it.
        let ordinal = input.first_day.div_euclid(span);
        let mut seed = fnv(input.reading.rashi.key().as_bytes());
        seed = fnv_more(seed, period_key(input.period).as_bytes());
        seed = seed.wrapping_add(u64::from_le_bytes(ordinal.to_le_bytes()));
        Sayer {
            seed,
            used: Vec::new(),
        }
    }

    fn say(
        &mut self,
        intent: String,
        params: Params,
        hedge: Hedge,
        rests_on: Vec<Fact>,
    ) -> ReadingItem {
        let occurrence = self.used.iter().filter(|(key, _)| *key == intent).count();
        let base = fnv(intent.as_bytes()).wrapping_add(self.seed);
        let occurrence = u64::try_from(occurrence).unwrap_or_default();
        let variant =
            u32::try_from(base.wrapping_add(occurrence) % u64::from(u32::MAX)).unwrap_or_default();
        self.used.push((intent.clone(), variant));
        ReadingItem {
            caution: hedge == Hedge::May,
            intent,
            params,
            hedge,
            variant,
            rests_on,
        }
    }
}

/// FNV-1a, 64 bits: the same seed on every platform and in every binding.
fn fnv(bytes: &[u8]) -> u64 {
    fnv_more(0xcbf2_9ce4_8422_2325, bytes)
}

/// FNV-1a continued over more bytes.
fn fnv_more(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    hash
}

fn key(parts: &[&str]) -> String {
    let mut out = String::from(NS);
    for part in parts {
        out.push('.');
        out.push_str(part);
    }
    out
}

fn entity(kind: &str, member: &str) -> Value {
    Value::Entity(format!("{kind}.{member}"))
}

/// The hedge a band says an area or the whole with.
const fn hedge_of(band: ScoreBand, agreeing: usize) -> Hedge {
    match band {
        ScoreBand::Excellent if agreeing >= 2 => Hedge::Firm,
        ScoreBand::Excellent | ScoreBand::Good => Hedge::Likely,
        ScoreBand::Mixed => Hedge::Possible,
        _ => Hedge::May,
    }
}

/// Whether the Moon may be named in this period: on a day only (§7.2).
const fn moon_named(period: Period) -> bool {
    matches!(period, Period::Daily)
}

/// The transits feeding an area, strongest first in the guide's order,
/// keeping only those whose verdict is `wanted` (or any when `None`).
fn feeding(
    input: &ReadingInput<'_>,
    area: ReadingArea,
    wanted: Option<Verdict>,
) -> Vec<(Graha, u8, Verdict)> {
    WEIGHT
        .iter()
        .filter(|graha| **graha != Graha::Moon || moon_named(input.period))
        .filter_map(|graha| {
            let gochar = input
                .reading
                .gochar
                .grahas
                .iter()
                .find(|g| g.graha == *graha)?;
            let feeds = area.houses().contains(&gochar.house);
            let kept = wanted.is_none_or(|verdict| verdict == gochar.verdict);
            (feeds && kept).then_some((gochar.graha, gochar.house, gochar.verdict))
        })
        .collect()
}

/// Each area's score, band and note.
fn area_readings(input: &ReadingInput<'_>, say: &mut Sayer) -> Vec<AreaReading> {
    ReadingArea::ALL
        .iter()
        .map(|area| {
            let score = area_score(input.score, *area);
            let band = input.bands.band(score);
            let wanted = match band {
                ScoreBand::Excellent | ScoreBand::Good => Some(Verdict::Good),
                ScoreBand::Care => Some(Verdict::NotGood),
                _ => None,
            };
            let facts = feeding(input, *area, wanted);
            let mut rests_on = vec![Fact::AreaBand { area: *area }];
            let mut params = Params::new();
            if let Some((graha, house, _)) = facts.first() {
                rests_on.push(Fact::Transit { graha: *graha });
                params.insert(String::from("graha"), entity("graha", graha.key()));
                params.insert(String::from("house"), Value::Int(i64::from(*house)));
            }
            let note = say.say(
                key(&["area", area.key(), band.key()]),
                params,
                hedge_of(band, facts.len()),
                rests_on,
            );
            AreaReading {
                area: *area,
                score,
                band,
                note,
            }
        })
        .collect()
}

/// An area's score: its baseline area's, or the rounded mean of two
/// (C2359).
fn area_score(score: &BaselineScore, area: ReadingArea) -> u8 {
    let of = |wanted: LifeArea| {
        score
            .areas
            .iter()
            .find(|(life, _)| *life == wanted)
            .map_or(0, |(_, value)| u16::from(*value))
    };
    let from = area.baseline();
    let total: u16 = from.iter().map(|life| of(*life)).sum();
    let count = u16::try_from(from.len()).unwrap_or(1).max(1);
    // Rounded half up, as the overall is.
    u8::try_from((total * 2 + count) / (count * 2)).unwrap_or(u8::MAX)
}

/// The areas ranked best first, by score, ties in the page's order.
fn ranked(areas: &[AreaReading]) -> Vec<&AreaReading> {
    let mut sorted: Vec<&AreaReading> = areas.iter().collect();
    sorted.sort_by_key(|area| std::cmp::Reverse(area.score));
    sorted
}

/// The headline: the area split when the areas differ, else the band.
fn headline(input: &ReadingInput<'_>, areas: &[AreaReading], say: &mut Sayer) -> ReadingItem {
    let band = input.score.band();
    let ranked = ranked(areas);
    if let (Some(best), Some(worst)) = (ranked.first(), ranked.last())
        && best.band != worst.band
        && matches!(best.band, ScoreBand::Excellent | ScoreBand::Good)
    {
        let mut params = Params::new();
        params.insert(
            String::from("best"),
            Value::Str(String::from(best.area.key())),
        );
        params.insert(
            String::from("worst"),
            Value::Str(String::from(worst.area.key())),
        );
        return say.say(
            key(&["headline", "SPLIT"]),
            params,
            hedge_of(best.band, 1),
            vec![
                Fact::AreaBand { area: best.area },
                Fact::AreaBand { area: worst.area },
            ],
        );
    }
    say.say(
        key(&["headline", band.key()]),
        Params::new(),
        hedge_of(band, 1),
        vec![Fact::Band],
    )
}

/// The why: the strongest influence, with Saturn's phase when it changes
/// and, for a period longer than a day, its slow grahas' events.
fn why(input: &ReadingInput<'_>, say: &mut Sayer) -> Vec<ReadingItem> {
    let mut items = Vec::new();
    if let Some(turn) = input.moon_turn.filter(|_| moon_named(input.period)) {
        let mut params = Params::new();
        params.insert(String::from("minutes"), Value::Int(i64::from(turn.minutes)));
        params.insert(String::from("house"), Value::Int(i64::from(turn.to_house)));
        let hedge = if turn.good_after {
            Hedge::Likely
        } else {
            Hedge::May
        };
        items.push(say.say(
            key(&["why", "MOON_TURN"]),
            params,
            hedge,
            vec![Fact::MoonTurn],
        ));
    }
    let care = input.score.band() == ScoreBand::Care;
    let wanted = if care {
        Verdict::NotGood
    } else {
        Verdict::Good
    };
    let strongest = WEIGHT
        .iter()
        .filter(|graha| **graha != Graha::Moon || moon_named(input.period))
        .find_map(|graha| {
            input
                .reading
                .gochar
                .grahas
                .iter()
                .find(|g| g.graha == *graha && g.verdict == wanted)
        });
    if let Some(gochar) = strongest {
        let mut params = Params::new();
        params.insert(String::from("graha"), entity("graha", gochar.graha.key()));
        params.insert(String::from("house"), Value::Int(i64::from(gochar.house)));
        let hedge = if care { Hedge::May } else { Hedge::Likely };
        items.push(say.say(
            key(&["why", "TRANSIT", gochar.verdict.key()]),
            params,
            hedge,
            vec![Fact::Transit {
                graha: gochar.graha,
            }],
        ));
    }
    let saturn = input.reading.saturn;
    if saturn.sade_sati.is_some() || saturn.spell {
        let mut params = Params::new();
        params.insert(String::from("house"), Value::Int(i64::from(saturn.house)));
        let what = saturn.sade_sati.map_or("DHAIYA", phase_key);
        items.push(say.say(
            key(&["why", "SATURN", what]),
            params,
            Hedge::Possible,
            vec![Fact::Saturn],
        ));
    }
    if input.period != Period::Daily {
        for (index, from) in input.reading.events.iter().enumerate() {
            items.push(event_item(index, from, say));
        }
    }
    items
}

const fn phase_key(phase: Phase) -> &'static str {
    match phase {
        Phase::Rising => "RISING",
        Phase::Peak => "PEAK",
        Phase::Setting => "SETTING",
    }
}

/// An ingress or station, said with its house and whether it is good.
fn event_item(index: usize, from: &teistro_rashifal::EventFrom, say: &mut Sayer) -> ReadingItem {
    let hit = from.event.hit;
    let mut params = Params::new();
    params.insert(String::from("graha"), entity("graha", hit.graha.key()));
    params.insert(
        String::from("rashi"),
        entity("rashi", from.event.sign.key()),
    );
    params.insert(String::from("house"), Value::Int(i64::from(from.house)));
    let kind = match hit.event {
        HitEvent::Station { turns } => {
            if turns == teistro_gochar::hits::Motion::Retrograde {
                "STATION_RETROGRADE"
            } else {
                "STATION_DIRECT"
            }
        }
        _ => "INGRESS",
    };
    let good = if from.good_house { "GOOD" } else { "NOT_GOOD" };
    let hedge = if from.good_house {
        Hedge::Likely
    } else {
        Hedge::May
    };
    say.say(
        key(&["event", kind, good]),
        params,
        hedge,
        vec![Fact::Event { index }],
    )
}

/// The reading's parts, and a week's best day.
fn parts(
    input: &ReadingInput<'_>,
    areas: &[AreaReading],
    say: &mut Sayer,
) -> (Vec<ReadingPart>, Option<usize>) {
    let (mut parts, best) = match input.period {
        Period::Daily => (daily(input, areas, say), None),
        Period::Weekly => weekly(input, areas, say),
        Period::Monthly => (monthly(input, areas, say), None),
        Period::Yearly => (yearly(input, areas, say), None),
    };
    keep_the_budget(input.period, &mut parts);
    (parts, best)
}

/// Holds the period's cautions to its budget (§7.2): a day one, a week
/// two, a month and a year one in each part. A caution past the budget is
/// still said, as a possibility rather than a caution, and no part ends on
/// one (§6.3).
fn keep_the_budget(period: Period, parts: &mut [ReadingPart]) {
    let mut left = match period {
        Period::Daily => 1,
        Period::Weekly => 2,
        Period::Monthly | Period::Yearly => usize::MAX,
    };
    for part in parts.iter_mut() {
        let mut in_part = 0;
        for item in &mut part.items {
            if !item.caution {
                continue;
            }
            let per_part = matches!(period, Period::Monthly | Period::Yearly);
            if left == 0 || (per_part && in_part == 1) {
                item.caution = false;
                item.hedge = Hedge::Possible;
            } else {
                left = left.saturating_sub(1);
                in_part += 1;
            }
        }
        keep_items_caution_off_the_end(&mut part.items);
    }
}

/// The best area and, when it is good, the second, each from its strongest
/// fact; the worst, when it asks for care, said between them as the
/// caution, paired with what still works and never last (§6.3). The best
/// is never said as a caution: on a day for care it is what still works.
fn area_lines(
    input: &ReadingInput<'_>,
    areas: &[AreaReading],
    say: &mut Sayer,
) -> Vec<ReadingItem> {
    let ranked = ranked(areas);
    let line = |area: &AreaReading, floor: bool, say: &mut Sayer| {
        let good = matches!(area.band, ScoreBand::Excellent | ScoreBand::Good);
        let verdict = if good {
            Verdict::Good
        } else {
            Verdict::NotGood
        };
        let facts = feeding(input, area.area, Some(verdict));
        let mut rests_on = vec![Fact::AreaBand { area: area.area }];
        let mut params = Params::new();
        if let Some((graha, house, _)) = facts.first() {
            rests_on.push(Fact::Transit { graha: *graha });
            params.insert(String::from("house"), Value::Int(i64::from(*house)));
        }
        let mut hedge = hedge_of(area.band, facts.len());
        if floor && hedge == Hedge::May {
            hedge = Hedge::Possible;
        }
        say.say(
            key(&["line", area.area.key(), area.band.key()]),
            params,
            hedge,
            rests_on,
        )
    };
    let mut items = Vec::new();
    let (Some(best), Some(worst)) = (ranked.first(), ranked.last()) else {
        return items;
    };
    items.push(line(best, true, say));
    let second = ranked
        .get(1)
        .filter(|area| matches!(area.band, ScoreBand::Excellent | ScoreBand::Good));
    let cautions =
        matches!(worst.band, ScoreBand::Mixed | ScoreBand::Care) && worst.area != best.area;
    if cautions {
        let caution = line(worst, false, say);
        items.push(caution);
    }
    if let Some(second) = second.filter(|area| area.area != worst.area || !cautions) {
        items.push(line(second, true, say));
    }
    items
}

fn daily(input: &ReadingInput<'_>, areas: &[AreaReading], say: &mut Sayer) -> Vec<ReadingPart> {
    let lines = area_lines(input, areas, say);
    let Some(turn) = input.moon_turn else {
        return vec![ReadingPart {
            frame: Frame::Whole,
            items: lines,
        }];
    };
    // Split at the Moon's turn: the areas before, her new house after.
    let mut params = Params::new();
    params.insert(String::from("minutes"), Value::Int(i64::from(turn.minutes)));
    params.insert(String::from("house"), Value::Int(i64::from(turn.to_house)));
    let good = if turn.good_after { "GOOD" } else { "NOT_GOOD" };
    let hedge = if turn.good_after {
        Hedge::Likely
    } else {
        Hedge::May
    };
    let after = say.say(key(&["moon", good]), params, hedge, vec![Fact::MoonTurn]);
    vec![
        ReadingPart {
            frame: Frame::BeforeMoon,
            items: lines,
        },
        ReadingPart {
            frame: Frame::AfterMoon,
            items: vec![after],
        },
    ]
}

/// A week: start, middle and end from its days' scores, then the areas;
/// its best day when one day is the strict highest.
fn weekly(
    input: &ReadingInput<'_>,
    areas: &[AreaReading],
    say: &mut Sayer,
) -> (Vec<ReadingPart>, Option<usize>) {
    let mut parts = Vec::new();
    let days = input.days;
    let thirds: [(Frame, std::ops::Range<usize>); 3] = [
        (Frame::Start, 0..2),
        (Frame::Middle, 2..5),
        (Frame::End, 5..7),
    ];
    for (frame, range) in thirds {
        let range = range.start.min(days.len())..range.end.min(days.len());
        if range.is_empty() {
            continue;
        }
        let total: u32 = days
            .get(range.clone())
            .unwrap_or_default()
            .iter()
            .map(|day| u32::from(day.overall))
            .sum();
        let count = u32::try_from(range.len()).unwrap_or(1);
        let mean = u8::try_from((total * 2 + count) / (count * 2)).unwrap_or(u8::MAX);
        let band = input.bands.band(mean);
        let hedge = hedge_of(band, 1);
        let rests_on = range.clone().map(|index| Fact::Day { index }).collect();
        let item = say.say(key(&["week", band.key()]), Params::new(), hedge, rests_on);
        parts.push(ReadingPart {
            frame,
            items: vec![item],
        });
    }
    let lines = area_lines(input, areas, say);
    parts.push(ReadingPart {
        frame: Frame::Whole,
        items: lines,
    });
    let best = strict_best(days);
    if let Some(index) = best {
        let mut params = Params::new();
        params.insert(
            String::from("day"),
            Value::Int(i64::try_from(index).unwrap_or_default()),
        );
        let item = say.say(
            key(&["week", "BEST_DAY"]),
            params,
            Hedge::Likely,
            vec![Fact::Day { index }],
        );
        if let Some(whole) = parts.last_mut() {
            whole.items.push(item);
        }
    }
    (parts, best)
}

/// The day whose score is the week's strict highest (C2363).
fn strict_best(days: &[DayScore]) -> Option<usize> {
    let top = days.iter().map(|day| day.overall).max()?;
    let mut at = days
        .iter()
        .enumerate()
        .filter(|(_, day)| day.overall == top);
    let (index, _) = at.next()?;
    at.next().is_none().then_some(index)
}

/// A month: two halves, split at its largest event.
fn monthly(input: &ReadingInput<'_>, areas: &[AreaReading], say: &mut Sayer) -> Vec<ReadingPart> {
    let events = &input.reading.events;
    let largest = WEIGHT.iter().find_map(|graha| {
        events
            .iter()
            .position(|from| from.event.hit.graha == *graha)
    });
    let Some((split, from)) = largest.and_then(|at| Some((at, events.get(at)?))) else {
        let lines = area_lines(input, areas, say);
        return vec![ReadingPart {
            frame: Frame::Whole,
            items: lines,
        }];
    };
    let first = area_lines(input, areas, say);
    let second = vec![event_item(split, from, say)];
    vec![
        ReadingPart {
            frame: Frame::FirstHalf,
            items: first,
        },
        ReadingPart {
            frame: Frame::SecondHalf,
            items: second,
        },
    ]
}

/// A year: the slow grahas' standing, then each quarter's events.
fn yearly(input: &ReadingInput<'_>, areas: &[AreaReading], say: &mut Sayer) -> Vec<ReadingPart> {
    const SLOW: [Graha; 4] = [Graha::Jupiter, Graha::Saturn, Graha::Rahu, Graha::Ketu];
    let mut parts = Vec::new();
    let mut slow = Vec::new();
    for graha in SLOW {
        if let Some(gochar) = input
            .reading
            .gochar
            .grahas
            .iter()
            .find(|g| g.graha == graha)
        {
            let mut params = Params::new();
            params.insert(String::from("graha"), entity("graha", graha.key()));
            params.insert(String::from("house"), Value::Int(i64::from(gochar.house)));
            let hedge = match gochar.verdict {
                Verdict::Good => Hedge::Likely,
                Verdict::Obstructed => Hedge::Possible,
                _ => Hedge::May,
            };
            slow.push(say.say(
                key(&["year", "SLOW", gochar.verdict.key()]),
                params,
                hedge,
                vec![Fact::Transit { graha }],
            ));
        }
    }
    parts.push(ReadingPart {
        frame: Frame::Slow,
        items: slow,
    });
    let (start, end) = input.span;
    let length = (end - start).max(1.0);
    let frames = [
        Frame::Quarter1,
        Frame::Quarter2,
        Frame::Quarter3,
        Frame::Quarter4,
    ];
    let mut quarters: [Vec<ReadingItem>; 4] = Default::default();
    for (index, from) in input.reading.events.iter().enumerate() {
        let at = (from.event.hit.instant.get() - start) / length * 4.0;
        // An event at the period's very end belongs to its last quarter.
        let quarter = if at < 1.0 {
            0
        } else if at < 2.0 {
            1
        } else if at < 3.0 {
            2
        } else {
            3
        };
        if let Some(items) = quarters.get_mut(quarter) {
            items.push(event_item(index, from, say));
        }
    }
    for (frame, items) in frames.into_iter().zip(quarters) {
        if !items.is_empty() {
            parts.push(ReadingPart { frame, items });
        }
    }
    let lines = area_lines(input, areas, say);
    parts.push(ReadingPart {
        frame: Frame::Whole,
        items: lines,
    });
    parts
}

/// A caution is never last (§6.3): a part ending on one takes its last
/// non-caution after it.
fn keep_items_caution_off_the_end(items: &mut [ReadingItem]) {
    if !items.last().is_some_and(|item| item.caution) {
        return;
    }
    if let Some(at) = items.iter().rposition(|item| !item.caution) {
        if let Some(tail) = items.get_mut(at..) {
            tail.rotate_left(1);
        }
    }
}

/// The sign's nine name syllables: its padas' in order. A sign spans
/// nine padas, from pada `(sign × 9)` of the twenty-seven nakshatras'
/// hundred and eight.
fn syllables(rashi: Rashi) -> Vec<String> {
    use teistro_core::catalogue::Nakshatra;
    let first = rashi as usize * 9;
    (first..first + 9)
        .filter_map(|pada| {
            let nakshatra = Nakshatra::ALL.get(pada / 4)?;
            let pada = nakshatra.attributes().padas.get(pada % 4)?;
            Some(String::from(pada.akshara))
        })
        .collect()
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests;
