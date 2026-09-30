//! The baseline engine's weights (`03-design/muhurta.md` §4.5, crux C162).
//!
//! The engine orders elected times by a number: five scorers stacked,
//! each clamped to 0–100 as it is added, then capped by the Mahadoshas
//! left uncancelled. No text weighs clauses so, and the SDK's own order is
//! [`Ranking::Texts`](crate::Ranking::Texts); this is the engine's, named,
//! because the roadmap's regression is stated in it.
//!
//! Each quantity is read **where the engine reads it** — the day's at its
//! sunrise, the window's at its start — from the SDK's own day, sunrise
//! and window. The score carries its factors, one per dimension with its
//! signed weight, so a reader sees why a window scored 55 and not 92.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{
    Auspiciousness, Choghadiya, Graha, Kaala, Karana, Nakshatra, Paksha, Rashi, Tithi, TithiClass,
    Vara, Yoga,
};
use teistro_core::interval::Interval;
use teistro_panchanga::Panchanga;
use teistro_panchanga::span::at as span_at;

use crate::day::Native;
use crate::instant::Sky;

/// What the engine's weights take for one activity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BaselineEvent {
    /// The stars the activity is matched to (+25 when the Moon stands in
    /// one at sunrise).
    pub stars: Vec<Nakshatra>,
    /// The weekdays the event favours (+8).
    pub favoured_varas: Vec<Vara>,
    /// The weekdays it avoids (−12).
    pub avoided_varas: Vec<Vara>,
    /// The tithis it favours (+6), both pakshas' named.
    pub favoured_tithis: Vec<Tithi>,
    /// The grahas that signify the event, judged at each window.
    pub karakas: Vec<Graha>,
    /// Whether the 7th house must be empty (+4, or −12).
    pub seventh_empty: bool,
    /// Whether Abhijit's own candidate is dropped. Its minutes are still
    /// offered through the choghadiya they fall in.
    pub abhijit_forbidden: bool,
}

impl BaselineEvent {
    /// The engine's marriage: its eleven stars; Monday, Wednesday,
    /// Thursday and Friday favoured, Tuesday avoided; the 2nd, 3rd, 5th,
    /// 7th, 10th, 11th and 13th of either paksha favoured; Venus and
    /// Jupiter the karakas; the 7th empty; Abhijit forbidden.
    #[must_use]
    pub fn marriage() -> BaselineEvent {
        BaselineEvent {
            stars: vec![
                Nakshatra::Rohini,
                Nakshatra::Mrigashira,
                Nakshatra::Magha,
                Nakshatra::UttaraPhalguni,
                Nakshatra::Hasta,
                Nakshatra::Swati,
                Nakshatra::Anuradha,
                Nakshatra::Mula,
                Nakshatra::UttaraAshadha,
                Nakshatra::UttaraBhadrapada,
                Nakshatra::Revati,
            ],
            favoured_varas: GENTLE_VARAS.to_vec(),
            avoided_varas: vec![Vara::Mangalavara],
            favoured_tithis: [2, 3, 5, 7, 10, 11, 13]
                .into_iter()
                .flat_map(|n: u16| [Tithi::from_id(n - 1), Tithi::from_id(n + 14)])
                .flatten()
                .collect(),
            karakas: vec![Graha::Venus, Graha::Jupiter],
            seventh_empty: true,
            abhijit_forbidden: true,
        }
    }
}

/// What a weight was given for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Dimension {
    /// The sunrise tithi: Nanda, Rikta or neither.
    TithiQuality,
    /// The sunrise star: the activity's, or a harsh one.
    NakshatraSuitability,
    /// The weekday.
    WeekdaySuitability,
    /// Solar noon inside Rahu kaala.
    RahuKaal,
    /// The sunrise yoga's nature.
    YogaShuddhi,
    /// A Vishti in the day.
    KaranaShuddhi,
    /// Panchaka at sunrise.
    Panchaka,
    /// The special yogas holding at sunrise.
    MuhurtaYoga,
    /// The Moon's brightness from the sunrise tithi.
    PakshaBala,
    /// The event's weekday.
    VaraEvent,
    /// The event's tithi.
    TithiEvent,
    /// The tara from the native's star.
    TaraBala,
    /// The Moon's house from the native's Moon.
    ChandraBala,
    /// The window's choghadiya.
    Choghadiya,
    /// Abhijit.
    Abhijit,
    /// The lagna's lord.
    LagnaLord,
    /// The grahas' houses from the lagna, summed.
    LagnaPlacement,
    /// A graha in the 8th.
    EighthHouse,
    /// The 7th empty or occupied.
    UdayastaShuddhi,
    /// Malefics either side of the lagna.
    Kartari,
    /// A benefic in the lagna cancelling a defect.
    DoshaBhanga,
    /// A karaka's dignity.
    KarakaStrength,
    /// A karaka too near the Sun.
    KarakaCombust,
    /// A karaka retrograde.
    KarakaRetrograde,
}

impl Dimension {
    /// Whether a weight against the time on this dimension is a Mahadosha,
    /// which caps the score.
    #[must_use]
    pub const fn mahadosha(self) -> bool {
        matches!(
            self,
            Dimension::YogaShuddhi
                | Dimension::KaranaShuddhi
                | Dimension::KarakaStrength
                | Dimension::KarakaCombust
        )
    }
}

/// One weight and what it was given for.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Factor {
    /// What for.
    pub dimension: Dimension,
    /// The weight, signed.
    pub weight: i32,
    /// The graha it concerns, for a karaka's or a cancelling benefic's.
    pub graha: Option<Graha>,
}

/// A score under the engine's weights.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Score {
    /// The score, 0 to 100.
    pub value: u8,
    /// Every weight that went into it, in the order added.
    pub factors: Vec<Factor>,
    /// The ceiling the Mahadoshas left uncancelled put on it, when they
    /// lowered it.
    pub capped_at: Option<u8>,
}

/// Monday, Wednesday, Thursday and Friday.
const GENTLE_VARAS: [Vara; 4] = [
    Vara::Somavara,
    Vara::Budhavara,
    Vara::Guruvara,
    Vara::Shukravara,
];

/// The harsh stars, the fierce and the sharp.
const HARSH_STARS: [Nakshatra; 9] = [
    Nakshatra::Bharani,
    Nakshatra::Magha,
    Nakshatra::PurvaPhalguni,
    Nakshatra::PurvaAshadha,
    Nakshatra::PurvaBhadrapada,
    Nakshatra::Ardra,
    Nakshatra::Ashlesha,
    Nakshatra::Jyeshtha,
    Nakshatra::Mula,
];

/// The engine's benefics and malefics for the lagna, unconditional.
const BENEFICS: [Graha; 4] = [Graha::Jupiter, Graha::Venus, Graha::Mercury, Graha::Moon];
const MALEFICS: [Graha; 5] = [
    Graha::Sun,
    Graha::Mars,
    Graha::Saturn,
    Graha::Rahu,
    Graha::Ketu,
];

/// The nine grahas in the engine's own order, which decides which
/// cancelling benefic it takes when several stand in the lagna.
const ENGINE_ORDER: [Graha; 9] = [
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

/// Kendras and trikonas, and the upachayas the engine counts.
const KENDRA_TRIKONA: [u8; 6] = [1, 4, 5, 7, 9, 10];
const UPACHAYA: [u8; 3] = [3, 6, 11];

/// The ceilings the uncancelled Mahadoshas put on a window.
const MAHADOSHA_CAP: i32 = 55;
const MAHADOSHA_SEVERE_CAP: i32 = 30;

/// A running score: its value and its factors.
#[derive(Default)]
struct Tally {
    value: i32,
    factors: Vec<Factor>,
}

impl Tally {
    fn add(&mut self, dimension: Dimension, weight: i32) {
        self.add_for(dimension, weight, None);
    }

    fn add_for(&mut self, dimension: Dimension, weight: i32, graha: Option<Graha>) {
        if weight != 0 {
            self.value += weight;
            self.factors.push(Factor {
                dimension,
                weight,
                graha,
            });
        }
    }

    fn clamp(&mut self) {
        self.value = self.value.clamp(0, 100);
    }

    fn score(self, capped_at: Option<i32>) -> Score {
        Score {
            value: u8::try_from(self.value).unwrap_or(0),
            factors: self.factors,
            capped_at: capped_at.and_then(|c| u8::try_from(c).ok()),
        }
    }
}

/// What the engine reads of a day: its limbs at sunrise, and three
/// facts of the whole day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct DayReading {
    /// The tithi at sunrise.
    pub tithi: Tithi,
    /// The Moon's star at sunrise.
    pub star: Nakshatra,
    /// The vara.
    pub vara: Vara,
    /// The yoga at sunrise.
    pub yoga: Option<Yoga>,
    /// The Moon's sign at sunrise.
    pub moon_sign: Option<Rashi>,
    /// Whether solar noon falls in Rahu kaala.
    pub noon_in_rahu_kaala: bool,
    /// Whether a Vishti runs at any time of the day.
    pub vishti: bool,
    /// Whether panchaka runs at sunrise.
    pub panchaka: bool,
    /// How many special yogas hold at sunrise.
    pub special_yogas: u8,
}

impl DayReading {
    /// A day as the engine reads it, or `None` for a day with no tithi or
    /// star at its sunrise, which the engine cannot score either.
    #[must_use]
    pub fn of(day: &Panchanga) -> Option<DayReading> {
        let sunrise = day.day.sunrise;
        let noon = f64::midpoint(sunrise.get(), day.day.sunset.get());
        let holds = |at: Interval, instant: f64| at.from.get() <= instant && instant < at.to.get();
        Some(DayReading {
            tithi: span_at(&day.limbs.tithi, sunrise)?.member,
            star: span_at(&day.limbs.nakshatra, sunrise)?.member,
            vara: day.vara(),
            yoga: span_at(&day.limbs.yoga, sunrise).map(|s| s.member),
            moon_sign: span_at(&day.moon.signs, sunrise).map(|s| s.member),
            noon_in_rahu_kaala: day
                .kaalas
                .iter()
                .any(|k| k.kaala == Kaala::RahuKaala && holds(k.at, noon)),
            vishti: day.limbs.karana.iter().any(|k| k.member == Karana::Vishti),
            panchaka: span_at(&day.omens.panchaka, sunrise).is_some(),
            special_yogas: u8::try_from(
                day.omens
                    .yogas
                    .iter()
                    .filter(|y| holds(y.at, sunrise.get()))
                    .count(),
            )
            .unwrap_or(u8::MAX),
        })
    }
}

/// The day's score: what the engine hands its window assembler.
#[must_use]
pub fn day(day: &DayReading, native: Option<&Native>, event: &BaselineEvent) -> Score {
    let mut tally = Tally::default();

    // The day, clamped on its own.
    tally.add(
        Dimension::TithiQuality,
        match day.tithi.attributes().class {
            TithiClass::Nanda => 20,
            TithiClass::Rikta => -10,
            _ => 10,
        },
    );
    if event.stars.contains(&day.star) {
        tally.add(Dimension::NakshatraSuitability, 25);
    } else if HARSH_STARS.contains(&day.star) {
        tally.add(Dimension::NakshatraSuitability, -15);
    }
    if GENTLE_VARAS.contains(&day.vara) {
        tally.add(Dimension::WeekdaySuitability, 15);
    } else if day.vara == Vara::Shanivara {
        tally.add(Dimension::WeekdaySuitability, -10);
    }
    if day.noon_in_rahu_kaala {
        tally.add(Dimension::RahuKaal, -30);
    }
    tally.clamp();

    // The shuddhi, the event and the native, clamped together.
    if let Some(yoga) = day.yoga {
        tally.add(
            Dimension::YogaShuddhi,
            match yoga.attributes().auspiciousness {
                Auspiciousness::HighlyInauspicious => -25,
                Auspiciousness::Inauspicious => -12,
                Auspiciousness::Auspicious => 8,
                _ => 0,
            },
        );
    }
    if day.vishti {
        tally.add(Dimension::KaranaShuddhi, -18);
    }
    if day.panchaka {
        tally.add(Dimension::Panchaka, -10);
    }
    tally.add(
        Dimension::MuhurtaYoga,
        (6 * i32::from(day.special_yogas)).min(15),
    );
    tally.add(Dimension::PakshaBala, paksha_bala(day.tithi));
    if event.avoided_varas.contains(&day.vara) {
        tally.add(Dimension::VaraEvent, -12);
    } else if event.favoured_varas.contains(&day.vara) {
        tally.add(Dimension::VaraEvent, 8);
    }
    if event.favoured_tithis.contains(&day.tithi) {
        tally.add(Dimension::TithiEvent, 6);
    }
    if let Some(native) = native {
        tally.add(Dimension::TaraBala, tara_bala(native.star, day.star));
        if let Some(moon) = day.moon_sign {
            tally.add(Dimension::ChandraBala, chandra_bala(native.moon_sign, moon));
        }
    }
    tally.clamp();
    tally.score(None)
}

/// What the engine reads of a window's time of day.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Period {
    /// The choghadiya the window falls in.
    pub choghadiya: Option<Choghadiya>,
    /// The daylight choghadiya, when it is one: Abhijit's candidate
    /// carries it.
    pub daytime: bool,
    /// Whether the window falls in an effective Abhijit.
    pub abhijit: bool,
}

impl Period {
    /// The period of a window, read at its middle.
    #[must_use]
    pub fn of(day: &Panchanga, at: Interval) -> Period {
        let middle = f64::midpoint(at.from.get(), at.to.get());
        let holds = |at: Interval| at.from.get() <= middle && middle < at.to.get();
        let part = day.choghadiya.iter().find(|p| holds(p.at));
        Period {
            choghadiya: part.map(|p| p.choghadiya),
            daytime: part.is_some_and(|p| p.is_daytime),
            abhijit: day.muhurtas.abhijit_effective && day.muhurtas.abhijit.is_some_and(holds),
        }
    }
}

/// A window's score over its day's, or `None` for a window the engine
/// would not have offered: one in no good choghadiya and not in an
/// Abhijit the event allows.
///
/// `sky` is the sky at the window's start, where the engine reads the
/// lagna and the karakas.
#[must_use]
pub fn window(day: &Score, period: Period, sky: &Sky, event: &BaselineEvent) -> Option<Score> {
    let mut tally = Tally {
        value: i32::from(day.value),
        factors: day.factors.clone(),
    };
    if period.abhijit && !event.abhijit_forbidden {
        // Abhijit's candidate carries the daylight choghadiya it falls
        // in, Shubha's weight when it falls in none, and outscores that
        // choghadiya's own.
        let cover = period
            .choghadiya
            .filter(|_| period.daytime)
            .map_or(14, bonus);
        tally.add(Dimension::Abhijit, 16);
        tally.add(Dimension::Choghadiya, cover);
    } else {
        let choghadiya = period.choghadiya.filter(|c| good(*c))?;
        tally.add(Dimension::Choghadiya, bonus(choghadiya));
    }
    tally.clamp();
    lagna(&mut tally, sky, event.seventh_empty);
    karakas(&mut tally, sky, &event.karakas);
    tally.clamp();
    let capped = cap(&tally.factors).filter(|cap| *cap < tally.value);
    if let Some(cap) = capped {
        tally.value = cap;
    }
    Some(tally.score(capped))
}

/// Whether the engine offers a choghadiya's windows.
const fn good(choghadiya: Choghadiya) -> bool {
    matches!(
        choghadiya,
        Choghadiya::Amrit | Choghadiya::Shubha | Choghadiya::Laabh
    )
}

/// A choghadiya's weight.
const fn bonus(choghadiya: Choghadiya) -> i32 {
    match choghadiya {
        Choghadiya::Amrit => 18,
        Choghadiya::Shubha => 14,
        Choghadiya::Laabh => 12,
        Choghadiya::Char => 4,
        _ => 0,
    }
}

/// The Moon's brightness from the tithi: waxing to full, waning to new;
/// +8 at 0.7 or more, −10 at 0.25 or less, in fifteenths.
fn paksha_bala(tithi: Tithi) -> i32 {
    let attributes = tithi.attributes();
    let number = i32::from(attributes.number);
    let bright = if attributes.paksha == Paksha::Shukla {
        number
    } else {
        15 - number
    };
    if 10 * bright >= 105 {
        8
    } else if 4 * bright <= 15 {
        -10
    } else {
        0
    }
}

/// The tara of the day's star from the native's, weighed by its cycle.
fn tara_bala(native: Nakshatra, day: Nakshatra) -> i32 {
    let count = (i32::from(day.id()) - i32::from(native.id())).rem_euclid(27) + 1;
    let tara = (count - 1) % 9 + 1;
    let cycle = (count - 1) / 9;
    match tara {
        2 | 4 | 6 | 8 | 9 => 12,
        // −10 in the first cycle, two thirds of it in the second and a
        // third in the third, rounded as the engine rounds.
        3 | 5 | 7 => [-10, -7, -3]
            .get(usize::try_from(cycle).unwrap_or(0))
            .copied()
            .unwrap_or(0),
        _ => 0,
    }
}

/// The Moon's house from the native's Moon.
fn chandra_bala(native: Rashi, day: Rashi) -> i32 {
    match (i32::from(day.id()) - i32::from(native.id())).rem_euclid(12) + 1 {
        1 | 3 | 6 | 7 | 10 | 11 => 10,
        8 => -14,
        _ => -6,
    }
}

/// Whether a graha is exalted in a sign.
fn exalted(graha: Graha, sign: Rashi) -> bool {
    graha
        .attributes()
        .exaltation
        .is_some_and(|e| e.sign == sign)
}

/// Whether a graha is debilitated in a sign.
fn debilitated(graha: Graha, sign: Rashi) -> bool {
    graha
        .attributes()
        .debilitation
        .is_some_and(|e| e.sign == sign)
}

/// The lagna's weights at the window.
fn lagna(tally: &mut Tally, sky: &Sky, seventh_empty: bool) {
    let rising = sky.lagna();
    let lord = rising.attributes().lord;
    let lord_sign = sky.sign(lord);
    if exalted(lord, lord_sign) || lord_sign.attributes().lord == lord {
        tally.add(Dimension::LagnaLord, 12);
    } else if debilitated(lord, lord_sign) {
        tally.add(Dimension::LagnaLord, -12);
    } else if KENDRA_TRIKONA.contains(&sky.house_of(lord)) {
        tally.add(Dimension::LagnaLord, 6);
    }
    let placement: i32 = ENGINE_ORDER
        .iter()
        .map(|g| {
            let house = sky.house_of(*g);
            if BENEFICS.contains(g) && KENDRA_TRIKONA.contains(&house) {
                3
            } else if MALEFICS.contains(g) && UPACHAYA.contains(&house) {
                2
            } else if MALEFICS.contains(g) && KENDRA_TRIKONA.contains(&house) {
                -3
            } else {
                0
            }
        })
        .sum();
    tally.add(Dimension::LagnaPlacement, placement.clamp(-12, 12));
    let occupied = |house: u8| ENGINE_ORDER.iter().any(|g| sky.house_of(*g) == house);
    if occupied(8) {
        tally.add(Dimension::EighthHouse, -8);
    }
    if seventh_empty {
        tally.add(
            Dimension::UdayastaShuddhi,
            if occupied(7) { -12 } else { 4 },
        );
    }
    let malefic_in = |house: u8| MALEFICS.iter().any(|g| sky.house_of(*g) == house);
    if malefic_in(2) && malefic_in(12) {
        tally.add(Dimension::Kartari, -8);
    }
    // The first cancelling benefic in the engine's order, not the
    // strongest: Mercury is taken before Jupiter.
    if let Some((graha, weight)) = ENGINE_ORDER
        .iter()
        .filter(|g| sky.house_of(**g) == 1)
        .find_map(|g| bhanga(*g).map(|w| (*g, w)))
    {
        tally.add_for(Dimension::DoshaBhanga, weight, Some(graha));
    }
}

/// What a benefic in the lagna weighs as a cancellor.
const fn bhanga(graha: Graha) -> Option<i32> {
    match graha {
        Graha::Jupiter => Some(16),
        Graha::Venus => Some(10),
        Graha::Mercury => Some(7),
        _ => None,
    }
}

/// How near the Sun a graha may come before the engine calls it combust,
/// degrees.
const fn combustion_orb(graha: Graha) -> Option<f64> {
    match graha {
        Graha::Moon => Some(12.0),
        Graha::Mars => Some(17.0),
        Graha::Mercury => Some(14.0),
        Graha::Jupiter => Some(11.0),
        Graha::Venus => Some(10.0),
        Graha::Saturn => Some(15.0),
        _ => None,
    }
}

/// The karakas' weights at the window.
fn karakas(tally: &mut Tally, sky: &Sky, karakas: &[Graha]) {
    let sun = sky.longitude(Graha::Sun);
    for &karaka in karakas {
        let sign = sky.sign(karaka);
        if exalted(karaka, sign) || karaka.attributes().own.contains(&sign) {
            tally.add_for(Dimension::KarakaStrength, 8, Some(karaka));
        } else if debilitated(karaka, sign) {
            tally.add_for(Dimension::KarakaStrength, -12, Some(karaka));
        }
        if let Some(orb) = combustion_orb(karaka) {
            let apart = (sky.longitude(karaka) - sun).rem_euclid(360.0);
            if apart.min(360.0 - apart) <= orb {
                tally.add_for(Dimension::KarakaCombust, -12, Some(karaka));
            }
        }
        if sky.speed(karaka) < 0.0 {
            tally.add_for(Dimension::KarakaRetrograde, -6, Some(karaka));
        }
    }
}

/// The ceiling the uncancelled Mahadoshas put on a score, if any.
///
/// One cancelling benefic lifts one Mahadosha, but never a combust
/// karaka: that is a condition of the season, which no arrangement of
/// houses answers.
fn cap(factors: &[Factor]) -> Option<i32> {
    let against = |f: &&Factor| f.dimension.mahadosha() && f.weight < 0;
    let mahadoshas = factors.iter().filter(against).count();
    let uncancellable = factors
        .iter()
        .filter(against)
        .filter(|f| f.dimension == Dimension::KarakaCombust)
        .count();
    let cancelled = usize::from(
        factors
            .iter()
            .any(|f| f.dimension == Dimension::DoshaBhanga),
    );
    let net = uncancellable + (mahadoshas - uncancellable).saturating_sub(cancelled);
    match net {
        0 => None,
        1 => Some(MAHADOSHA_CAP),
        _ => Some(MAHADOSHA_SEVERE_CAP),
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, reason = "tests index fixed lists")]
mod tests {
    use super::{
        BaselineEvent, DayReading, Dimension, HARSH_STARS, Period, Score, chandra_bala, day,
        paksha_bala, tara_bala, window,
    };
    use crate::instant::Sky;
    use teistro_core::catalogue::{
        Auspiciousness, Choghadiya, Nakshatra, Rashi, Tithi, TithiClass, Vara, Yoga,
    };

    /// A Friday on the bright eleventh in Rohini with Siddhi: the day
    /// part 60 (Nanda 20, the star 25, the weekday 15), then Siddhi 8, the
    /// Moon's brightness 8, the event's weekday 8 and tithi 6.
    fn friday() -> DayReading {
        DayReading {
            tithi: Tithi::ShuklaEkadashi,
            star: Nakshatra::Rohini,
            vara: Vara::Shukravara,
            yoga: Some(Yoga::Siddhi),
            moon_sign: Some(Rashi::Taurus),
            noon_in_rahu_kaala: false,
            vishti: false,
            panchaka: false,
            special_yogas: 0,
        }
    }

    fn score(reading: DayReading) -> u8 {
        day(&reading, None, &BaselineEvent::marriage()).value
    }

    #[test]
    fn the_engines_shuddhi_cases_move_the_day_by_its_own_weights() {
        // The engine's tests fix the day part at 70 and read 92 as the
        // base; here the day part is computed, so the base is 90 and each
        // case moves it by the weight the engine's own arithmetic states.
        assert_eq!(score(friday()), 90);
        // Vyatipata: −25 where Siddhi gave +8 (the engine's 92 → 59).
        let vyatipata = DayReading {
            yoga: Some(Yoga::Vyatipata),
            ..friday()
        };
        assert_eq!(score(vyatipata), 90 - 33);
        // Vishti −18 and panchaka −10 (92 → 74 and 82).
        let vishti = DayReading {
            vishti: true,
            ..friday()
        };
        assert_eq!(score(vishti), 72);
        let panchaka = DayReading {
            panchaka: true,
            ..friday()
        };
        assert_eq!(score(panchaka), 80);
        // Two special yogas +12, clamped at 100 (92 + 12 → 100).
        let yogas = DayReading {
            special_yogas: 2,
            ..friday()
        };
        assert_eq!(score(yogas), 100);
        // The dark fourteenth: a Rikta (−10 for the tithi's 20), the Moon
        // near new (−10 for +8), and not the event's tithi (0 for 6).
        let dark = DayReading {
            tithi: Tithi::KrishnaChaturdashi,
            ..friday()
        };
        assert_eq!(score(dark), 90 - 30 - 18 - 6);
    }

    #[test]
    fn the_day_part_is_clamped_before_the_rest_is_added() {
        // A Rikta Saturday in a harsh star with noon in Rahu kaala is
        // −65 on its own, and is clamped to 0 before Siddhi's 8 is added.
        let low = DayReading {
            tithi: Tithi::ShuklaChaturthi,
            star: Nakshatra::Bharani,
            vara: Vara::Shanivara,
            noon_in_rahu_kaala: true,
            ..friday()
        };
        let found = day(&low, None, &BaselineEvent::marriage());
        // 0, then Siddhi 8, then the Moon's brightness (4/15) nothing.
        assert_eq!(found.value, 8);
        assert_eq!(
            found.factors.iter().map(|f| f.weight).sum::<i32>(),
            -10 - 15 - 10 - 30 + 8
        );
    }

    #[test]
    fn the_tables_are_the_engines_by_index() {
        // Nanda 0, 5, 10, 15, 20, 25; Rikta 3, 8, 13, 18, 23, 28.
        let class = |c: TithiClass| -> Vec<u16> {
            (0..30)
                .filter(|i| Tithi::from_id(*i).is_some_and(|t| t.attributes().class == c))
                .collect()
        };
        assert_eq!(class(TithiClass::Nanda), [0, 5, 10, 15, 20, 25]);
        assert_eq!(class(TithiClass::Rikta), [3, 8, 13, 18, 23, 28]);
        let mut harsh: Vec<u16> = HARSH_STARS.iter().map(|n| n.id()).collect();
        harsh.sort_unstable();
        assert_eq!(harsh, [1, 5, 8, 9, 10, 17, 18, 19, 24]);
        let stars: Vec<u16> = BaselineEvent::marriage()
            .stars
            .iter()
            .map(|n| n.id())
            .collect();
        assert_eq!(stars, [3, 4, 9, 11, 12, 14, 16, 18, 20, 25, 26]);
        // The yogas' natures: two highly inauspicious, six inauspicious,
        // Vajra mixed, the rest auspicious.
        let of = |a: Auspiciousness| {
            (0..27)
                .filter_map(Yoga::from_id)
                .filter(|y| y.attributes().auspiciousness == a)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            of(Auspiciousness::HighlyInauspicious),
            [Yoga::Vyatipata, Yoga::Vaidhriti]
        );
        assert_eq!(of(Auspiciousness::Inauspicious).len(), 6);
        assert_eq!(of(Auspiciousness::Mixed), [Yoga::Vajra]);
        let favoured = BaselineEvent::marriage().favoured_tithis;
        assert_eq!(favoured.len(), 14);
        assert!(favoured.contains(&Tithi::KrishnaTrayodashi));
    }

    #[test]
    fn the_moons_brightness_turns_at_the_engines_thresholds() {
        let at = |id: u16| paksha_bala(Tithi::from_id(id).unwrap_or(Tithi::Purnima));
        // Bright: 11/15 is 0.73 (+8), 10/15 is 0.67 (0), 3/15 is 0.2
        // (−10), 4/15 is 0.27 (0); the Purnima is full.
        assert_eq!([at(10), at(9), at(2), at(3), at(14)], [8, 0, -10, 0, 8]);
        // Dark: the 4th is 11/15 (+8), the 5th 10/15 (0), the 12th 3/15
        // (−10), the 11th 4/15 (0); the new Moon is dark.
        assert_eq!(
            [at(18), at(19), at(26), at(25), at(29)],
            [8, 0, -10, 0, -10]
        );
    }

    #[test]
    fn a_tara_weighs_by_its_cycle_and_the_moon_by_its_house() {
        let from_ashwini = |id: u16| {
            tara_bala(
                Nakshatra::Ashwini,
                Nakshatra::from_id(id).unwrap_or(Nakshatra::Ashwini),
            )
        };
        // Janma 0; sampat +12; vipat −10, −7, −3 across the three cycles.
        assert_eq!(from_ashwini(0), 0);
        assert_eq!(from_ashwini(1), 12);
        assert_eq!(
            [from_ashwini(2), from_ashwini(11), from_ashwini(20)],
            [-10, -7, -3]
        );
        assert_eq!(chandra_bala(Rashi::Aries, Rashi::Aries), 10);
        assert_eq!(chandra_bala(Rashi::Aries, Rashi::Scorpio), -14);
        assert_eq!(chandra_bala(Rashi::Aries, Rashi::Taurus), -6);
    }

    /// Aries rising at 5°; every graha at `rest` unless placed.
    fn sky(rest: f64, placed: &[(usize, f64)]) -> Sky {
        let mut grahas = [rest; 9];
        for (i, deg) in placed {
            grahas[*i] = *deg;
        }
        Sky {
            lagna_deg: 5.0,
            grahas,
            speeds: [1.0; 9],
        }
    }

    fn day_of(value: u8, factors: Vec<super::Factor>) -> Score {
        Score {
            value,
            factors,
            capped_at: None,
        }
    }

    fn amrit() -> Period {
        Period {
            choghadiya: Some(Choghadiya::Amrit),
            daytime: true,
            abhijit: false,
        }
    }

    #[test]
    fn a_window_outside_a_good_choghadiya_and_abhijit_is_not_offered() {
        let event = BaselineEvent::marriage();
        let at_rest = sky(95.0, &[]);
        let rog = Period {
            choghadiya: Some(Choghadiya::Rog),
            daytime: true,
            abhijit: false,
        };
        assert!(window(&day_of(50, Vec::new()), rog, &at_rest, &event).is_none());
        // Abhijit in Rog: forbidden for a marriage, offered otherwise,
        // with Rog's weight of nothing beside Abhijit's 16.
        let abhijit = Period {
            abhijit: true,
            ..rog
        };
        assert!(window(&day_of(50, Vec::new()), abhijit, &at_rest, &event).is_none());
        let allowed = BaselineEvent {
            abhijit_forbidden: false,
            ..event
        };
        let found = window(&day_of(50, Vec::new()), abhijit, &at_rest, &allowed).map(|s| {
            s.factors
                .iter()
                .filter(|f| {
                    f.dimension == Dimension::Abhijit || f.dimension == Dimension::Choghadiya
                })
                .map(|f| f.weight)
                .collect::<Vec<_>>()
        });
        assert_eq!(found, Some(vec![16]));
    }

    #[test]
    fn the_lagna_and_the_karakas_weigh_as_the_engine_reads_them() {
        let event = BaselineEvent::marriage();
        // Aries rising, Mars (its lord) in Capricorn, exalted: +12. Every
        // other graha in Cancer, the 4th, a kendra: the benefics (Moon,
        // Mercury, Jupiter, Venus) +3 each, the malefics (Sun, Saturn,
        // Rahu, Ketu) −3 each, Mars in the 10th −3: −3 all told. Nothing
        // in the 7th (+4) or the 8th. Jupiter in Cancer exalted (+8);
        // Venus in Cancer with the Sun: combust (−12).
        let s = sky(100.0, &[(2, 280.0)]);
        let found = window(&day_of(50, Vec::new()), amrit(), &s, &event).map(|s| {
            s.factors
                .iter()
                .map(|f| (f.dimension, f.weight))
                .collect::<Vec<_>>()
        });
        assert_eq!(
            found,
            Some(vec![
                (Dimension::Choghadiya, 18),
                (Dimension::LagnaLord, 12),
                (Dimension::LagnaPlacement, -3),
                (Dimension::UdayastaShuddhi, 4),
                (Dimension::KarakaCombust, -12),
                (Dimension::KarakaStrength, 8),
                (Dimension::KarakaCombust, -12),
            ])
        );
    }

    #[test]
    fn the_mahadoshas_cap_and_one_benefic_lifts_one_but_never_a_combust_karaka() {
        let event = BaselineEvent::marriage();
        let vishti = vec![super::Factor {
            dimension: Dimension::KaranaShuddhi,
            weight: -18,
            graha: None,
        }];
        // Everything in the 11th from an Aries lagna: nothing in a kendra,
        // no karaka combust (the Sun elsewhere), Venus and Jupiter in
        // Aquarius, neither strong nor weak. A day at 90 with a Vishti.
        let quiet = sky(305.0, &[(0, 200.0)]);
        let capped = window(&day_of(90, vishti.clone()), amrit(), &quiet, &event);
        assert_eq!(
            capped.as_ref().map(|s| (s.value, s.capped_at)),
            Some((55, Some(55)))
        );
        // Mercury in the lagna cancels the one Mahadosha.
        let mercury = sky(305.0, &[(0, 200.0), (3, 10.0)]);
        let lifted = window(&day_of(90, vishti.clone()), amrit(), &mercury, &event);
        assert_eq!(lifted.as_ref().and_then(|s| s.capped_at), None);
        assert!(lifted.is_some_and(|s| s.value > 55));
        // Venus combust as well (Jupiter kept 25° clear of the Sun): two
        // Mahadoshas, one cancellable and one not, so one remains and the
        // cap is 55.
        let combust = sky(305.0, &[(0, 300.0), (3, 10.0), (4, 325.0)]);
        let one_left = window(&day_of(90, vishti.clone()), amrit(), &combust, &event);
        assert_eq!(one_left.and_then(|s| s.capped_at), Some(55));
        // Without the benefic both stand, and the cap is 30.
        let both = sky(305.0, &[(0, 300.0), (4, 325.0)]);
        let severe = window(&day_of(90, vishti), amrit(), &both, &event);
        assert_eq!(severe.map(|s| s.value), Some(30));
    }

    #[test]
    fn the_first_cancelling_benefic_in_the_engines_order_is_taken() {
        // Mercury and Jupiter both in the lagna: the engine meets Mercury
        // first and takes its 7, not Jupiter's 16.
        let s = sky(305.0, &[(0, 200.0), (3, 10.0), (4, 12.0)]);
        let found = window(
            &day_of(50, Vec::new()),
            amrit(),
            &s,
            &BaselineEvent::marriage(),
        );
        let bhanga: Vec<i32> = found
            .iter()
            .flat_map(|s| &s.factors)
            .filter(|f| f.dimension == Dimension::DoshaBhanga)
            .map(|f| f.weight)
            .collect();
        assert_eq!(bhanga, [7]);
    }
}
