//! The baseline engine's rashifal score, `BASELINE` and unsourced (C361).
//!
//! No text scores a rashifal: Phaladeepika ch. 26 says which transits are
//! good, obstructed or not good, and stops. The baseline turns a reading
//! into a number from 0 to 100, eight life areas, five key influences and
//! a sign's lucky elements. This module reproduces that formula for
//! formula, its double-counted overall area and its key influences that
//! always pass over Mars included, because a consumer of its numbers sees
//! them. It reads them over the SDK's own reading — Phaladeepika's tables,
//! the events found exactly, the reference day's panchanga — so where the
//! baseline's own tables or its event scan differ, the numbers differ, and
//! `rashifal-measured.md` counts by how much.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Direction, Graha, Rashi, Tithi, TithiClass, Vara, Yoga};
use teistro_gochar::Verdict;
use teistro_gochar::hits::HitEvent;
use teistro_gochar::sade_sati::Phase;

use crate::RashiReading;

/// The period a score is weighed for, which sets the Moon's weight.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Period {
    /// A day: the Moon weighs 5.
    Daily,
    /// A week: the Moon weighs 2.
    Weekly,
    /// A month: the Moon weighs 1.
    Monthly,
    /// A year: the Moon weighs nothing and is left out.
    Yearly,
}

/// What the baseline reads of the reference day's panchanga: the tithi
/// running at sunrise, the yoga running then, and how many of the five
/// muhurta yogas (Amrita Siddhi, Sarvartha Siddhi, Siddha, Dvipushkara,
/// Tripushkara) hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Panchanga {
    /// The tithi at sunrise.
    pub tithi: Tithi,
    /// The yoga at sunrise.
    pub yoga: Yoga,
    /// The muhurta yogas that hold, 0 to 5.
    pub muhurta_yogas: u8,
}

/// The eight areas a score is split into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LifeArea {
    /// The whole, which every transit also feeds.
    Overall,
    /// Work.
    Career,
    /// Money.
    Finance,
    /// The body.
    Health,
    /// Partners.
    Relationships,
    /// Kin.
    Family,
    /// Learning.
    Education,
    /// The inner life.
    Spirituality,
}

impl LifeArea {
    /// The eight, in the baseline's order.
    pub const ALL: [LifeArea; 8] = [
        LifeArea::Overall,
        LifeArea::Career,
        LifeArea::Finance,
        LifeArea::Health,
        LifeArea::Relationships,
        LifeArea::Family,
        LifeArea::Education,
        LifeArea::Spirituality,
    ];
}

/// A colour the baseline names lucky.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Colour {
    /// Mars's.
    Red,
    /// Venus's and the Moon's.
    White,
    /// Mercury's.
    Green,
    /// The Sun's.
    Orange,
    /// Jupiter's.
    Yellow,
    /// Saturn's.
    Blue,
}

/// A sign's lucky elements, looked up by its lord: the same every day,
/// and the same for the two signs one graha rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LuckyElements {
    /// The colour.
    pub colour: Colour,
    /// The number, 1 to 9.
    pub number: u8,
    /// The direction.
    pub direction: Direction,
    /// The weekday.
    pub day: Vara,
}

/// One of the transits a score names as weighing most.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KeyInfluence {
    /// The graha.
    pub graha: Graha,
    /// Its house from the sign.
    pub house: u8,
    /// What its transit came to.
    pub verdict: Verdict,
}

/// The baseline's score of one sign's reading.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BaselineScore {
    /// The whole, 0 to 100.
    pub overall: u8,
    /// Each area, 0 to 100, in [`LifeArea::ALL`]'s order.
    pub areas: [(LifeArea, u8); 8],
    /// At most five transits, by weight.
    pub key_influences: Vec<KeyInfluence>,
    /// The sign's lucky elements.
    pub lucky: LuckyElements,
}

/// The order the baseline lists the grahas in, which breaks ties among the
/// key influences: the seven by their ephemeris numbers, then the nodes.
const ORDER: [Graha; 9] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mercury,
    Graha::Venus,
    Graha::Mars,
    Graha::Jupiter,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// A graha's weight for a period of the baseline's.
const fn weight(graha: Graha, period: Period) -> f64 {
    match graha {
        Graha::Jupiter | Graha::Saturn => 20.0,
        Graha::Rahu => 12.0,
        Graha::Ketu => 10.0,
        Graha::Sun | Graha::Mars => 8.0,
        Graha::Venus => 7.0,
        Graha::Mercury => 6.0,
        Graha::Moon => match period {
            Period::Daily => 5.0,
            Period::Weekly => 2.0,
            Period::Monthly => 1.0,
            Period::Yearly => 0.0,
        },
        _ => 0.0,
    }
}

/// What one transit adds to the score.
fn contribution(verdict: Verdict, retrograde: bool, weight: f64) -> f64 {
    match verdict {
        Verdict::Good => weight * if retrograde { 0.75 } else { 1.0 },
        Verdict::Obstructed => weight * 0.15,
        Verdict::NotGood => -weight * 0.6 * if retrograde { 1.25 } else { 1.0 },
    }
}

/// Saturn's standing's penalty: Sade Sati's three phases, and the 4th or
/// the 8th.
fn saturn_penalty(reading: &RashiReading) -> f64 {
    let phase = match reading.saturn.sade_sati {
        Some(Phase::Rising) => -8.0,
        Some(Phase::Peak) => -15.0,
        Some(Phase::Setting) => -5.0,
        None => 0.0,
    };
    let spell = if matches!(reading.saturn.house, 4 | 8) {
        -6.0
    } else {
        0.0
    };
    phase + spell
}

/// The panchanga's bonus: the tithi's class, the yoga's nature and two a
/// muhurta yoga.
fn panchanga_bonus(panchanga: Panchanga) -> f64 {
    let tithi = match panchanga.tithi.attributes().class {
        TithiClass::Nanda => 3.0,
        TithiClass::Bhadra => -2.0,
        TithiClass::Jaya => 2.0,
        TithiClass::Rikta => -3.0,
        _ => 4.0,
    };
    let yoga = match panchanga.yoga {
        Yoga::Vishkambha
        | Yoga::Atiganda
        | Yoga::Shoola
        | Yoga::Ganda
        | Yoga::Vyaghata
        | Yoga::Parigha => -2.0,
        Yoga::Vajra => 0.0,
        Yoga::Vyatipata | Yoga::Vaidhriti => -4.0,
        _ => 3.0,
    };
    tithi + yoga + 2.0 * f64::from(panchanga.muhurta_yogas)
}

/// The houses that feed each area besides the overall.
const fn areas_of(house: u8) -> &'static [LifeArea] {
    match house {
        1 => &[LifeArea::Health, LifeArea::Overall],
        2 => &[LifeArea::Finance, LifeArea::Family],
        3 => &[LifeArea::Education, LifeArea::Career],
        4 => &[LifeArea::Family, LifeArea::Overall],
        5 => &[LifeArea::Education, LifeArea::Relationships],
        6 => &[LifeArea::Health, LifeArea::Career],
        7 => &[LifeArea::Relationships],
        8 => &[LifeArea::Health, LifeArea::Spirituality],
        9 => &[LifeArea::Spirituality, LifeArea::Education],
        10 => &[LifeArea::Career],
        11 => &[LifeArea::Finance, LifeArea::Career],
        _ => &[LifeArea::Spirituality, LifeArea::Health],
    }
}

/// A sign's lucky elements, by its lord.
const fn lucky(lord: Graha) -> LuckyElements {
    let (colour, number, direction, day) = match lord {
        Graha::Mars => (Colour::Red, 9, Direction::South, Vara::Mangalavara),
        Graha::Venus => (Colour::White, 6, Direction::Southeast, Vara::Shukravara),
        Graha::Mercury => (Colour::Green, 5, Direction::North, Vara::Budhavara),
        Graha::Moon => (Colour::White, 2, Direction::Northwest, Vara::Somavara),
        Graha::Jupiter => (Colour::Yellow, 3, Direction::Northeast, Vara::Guruvara),
        Graha::Saturn => (Colour::Blue, 8, Direction::West, Vara::Shanivara),
        _ => (Colour::Orange, 1, Direction::East, Vara::Ravivara),
    };
    LuckyElements {
        colour,
        number,
        direction,
        day,
    }
}

/// A value mapped linearly from `[lo, hi]` onto 0 to 100 and clamped.
fn scaled(value: f64, lo: f64, hi: f64) -> f64 {
    if hi <= lo {
        return 50.0;
    }
    ((value - lo) / (hi - lo) * 100.0).clamp(0.0, 100.0)
}

/// A score rounded as the baseline rounds it, half up.
fn rounded(score: f64) -> u8 {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "a score clamped to 0..=100"
    )]
    let whole = (score + 0.5).floor().clamp(0.0, 100.0) as u8;
    whole
}

/// The baseline's score of `reading` for `period`, with each graha's
/// motion at the snapshot (`retrograde`, the Sun to Ketu: the mean nodes
/// always are) and the reference day's `panchanga`.
#[must_use]
pub fn baseline_score(
    reading: &RashiReading,
    retrograde: &[bool; 9],
    panchanga: Panchanga,
    period: Period,
) -> BaselineScore {
    let mut raw = 0.0;
    let mut most = 0.0;
    let mut area_raw = [0.0_f64; 8];
    let mut area_weight = [0.0_f64; 8];
    let mut weighed = Vec::new();
    for graha in ORDER {
        let Some(transit) = reading.gochar.grahas.get(graha as usize) else {
            continue;
        };
        let w = weight(graha, period);
        if w <= 0.0 {
            continue;
        }
        let moving_back = retrograde.get(graha as usize).copied().unwrap_or(false);
        let added = contribution(transit.verdict, moving_back, w);
        raw += added;
        most += w;
        for area in areas_of(transit.house).iter().chain(&[LifeArea::Overall]) {
            let at = *area as usize;
            if let (Some(sum), Some(of)) = (area_raw.get_mut(at), area_weight.get_mut(at)) {
                *sum += added;
                *of += w;
            }
        }
        if w >= 8.0 {
            weighed.push(KeyInfluence {
                graha,
                house: transit.house,
                verdict: transit.verdict,
            });
        }
    }
    let saturn = saturn_penalty(reading);
    let events: f64 = reading
        .events
        .iter()
        .map(|from| {
            let w = weight(from.event.hit.graha, Period::Daily);
            match from.event.hit.event {
                HitEvent::SignIngress { .. } if from.good_house => 0.5 * w,
                HitEvent::SignIngress { .. } => -0.3 * w,
                HitEvent::Station { .. } => -0.15 * w,
                _ => 0.0,
            }
        })
        .sum();
    let adjusted = raw + saturn + panchanga_bonus(panchanga) + events;
    let overall = scaled(adjusted, -0.6 * most - 25.0, most + 10.0);
    let areas = LifeArea::ALL.map(|area| {
        let at = area as usize;
        let of = area_weight.get(at).copied().unwrap_or(0.0);
        let score = if of == 0.0 {
            overall
        } else {
            let sum = area_raw.get(at).copied().unwrap_or(0.0);
            scaled(sum + 0.3 * saturn, -0.6 * of - 6.3, of)
        };
        (area, rounded(score))
    });
    // A stable sort by the weights alone, the Moon's at a day's: ties keep
    // the baseline's order, which puts the Sun before Mars.
    weighed
        .sort_by(|a, b| weight(b.graha, Period::Daily).total_cmp(&weight(a.graha, Period::Daily)));
    weighed.truncate(5);
    BaselineScore {
        overall: rounded(overall),
        areas,
        key_influences: weighed,
        lucky: lucky(lord_of(reading.rashi)),
    }
}

/// A sign's lord.
fn lord_of(rashi: Rashi) -> Graha {
    rashi.attributes().lord
}
