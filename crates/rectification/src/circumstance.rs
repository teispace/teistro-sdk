//! `CIRCUMSTANCE`: what *Brihat Jataka* ch. V says the birth moment shows,
//! set against what the family remembers (`03-design/rectification.md`,
//! step 5).
//!
//! Each verse is a clause over a fact the caller may know, and a fact not
//! given skips its clause rather than guessing it. Every clause that runs
//! is reported with what the sky foretold, and each fact given weighs for
//! or against the candidate by whether it agrees. None bars: the verses
//! are indications, and the stage only weighs.
//!
//! - **V.1–2**, the father away: the Moon not seeing the lagna; Saturn in
//!   the lagna, Mars in the 7th, or the Moon between Mercury and Venus.
//!   V.1 then places him by the Sun fallen from the 10th (X16, X17).
//! - **V.17**, the child's presentation: as the rising sign rises, head or
//!   feet first, Pisces the hands (the commentator); or by the lagna lord's
//!   motion, the reading Manittha supports (X18).
//! - **V.18**, the lamp: its oil by the Moon's degree in her sign, its wick
//!   by the rising degree, full at a sign's start and spent at its end.
//!   The only clauses that see inside a sign.
//! - **V.22**, the women attending: as many as the grahas between the
//!   lagna and the Moon, those in the visible half outside the room and
//!   the invisible inside; "some" reverse it (X19).
//!
//! The numbering is Iyer's 1885 translation, which follows Bhattotpala;
//! Vijnanananda's 1912 print agrees verse for verse in this chapter.

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti::{self, Strength};
use teistro_core::catalogue::{Graha, Modality, Rashi, Rising};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Utc};

use crate::conception::ConceptionSky;
use crate::purifier::house;

/// What the family remembers of the birth. Every fact is optional, and a
/// clause whose fact is absent is reported and not weighed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct Facts {
    /// Whether the father was there at the birth (V.1–2).
    pub father_present: Option<bool>,
    /// What came first (V.17).
    pub presentation: Option<Presentation>,
    /// How full the lamp's oil was (V.18).
    pub oil: Option<Level>,
    /// How much of the lamp's wick was left (V.18).
    pub wick: Option<Level>,
    /// The women who attended (V.22).
    pub attendants: Option<Attendants>,
}

/// What came first at the birth.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Presentation {
    /// The head: a sign that rises head first.
    Head,
    /// The feet: a sign that rises back first.
    Feet,
    /// The hands: Pisces, which rises both ways (the commentator's
    /// reading of V.17).
    Hands,
}

/// How much of the lamp's oil or wick was left, at the three points V.18's
/// gloss names: a sign's start, its middle and its end.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Level {
    /// Full oil, an unburnt wick.
    Full,
    /// Half.
    Half,
    /// No oil, a wick burnt down.
    Spent,
}

impl Level {
    /// The level nearest a fraction left, one at a sign's start and none
    /// at its end: the gloss's three points, each holding the third of the
    /// sign nearest it in the fraction.
    #[must_use]
    pub fn nearest(left: f64) -> Level {
        if left > 0.75 {
            Level::Full
        } else if left >= 0.25 {
            Level::Half
        } else {
            Level::Spent
        }
    }
}

/// The women who attended, as many as the family can say.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct Attendants {
    /// How many in all.
    pub total: Option<u8>,
    /// How many inside the room.
    pub inside: Option<u8>,
    /// How many outside it.
    pub outside: Option<u8>,
}

/// What "the Moon not seeing the lagna" asks of her aspect (X16).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonSees {
    /// Any aspect at all, a quarter or more: BJ II.13 gives every graha
    /// its quarter, half, three-quarter and full aspects, and V.1 says only
    /// "not seeing". The Moon in the lagna's own sign does not see it.
    #[default]
    AnyAspect,
    /// Only the full aspect, from the 7th.
    Full,
}

/// Which houses are "fallen from the 10th" for the Sun in V.1 (X17).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SunFallen {
    /// The 9th or the 8th: Iyer's 1885 translation, following Bhattotpala,
    /// the houses the Sun falls through after culminating.
    #[default]
    NinthOrEighth,
    /// The 8th, 9th, 11th or 12th: Vijnanananda's 1912 gloss of *madhyād
    /// bhraṣṭe*, fallen from the 10th either way.
    EitherSide,
}

impl SunFallen {
    /// Whether the Sun in a house from the lagna has fallen.
    #[must_use]
    pub fn holds(self, house: u8) -> bool {
        match self {
            SunFallen::NinthOrEighth => matches!(house, 8 | 9),
            SunFallen::EitherSide => matches!(house, 8 | 9 | 11 | 12),
        }
    }
}

/// Which reading of V.17's presentation is asked (X18).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PresentationBy {
    /// As the rising sign rises (BJ I.10): head first, back first, or
    /// Pisces both ways, which the commentator reads as the hands.
    #[default]
    RisingSign,
    /// By the lagna lord's motion: direct, a natural birth, head first;
    /// retrograde, an irregular one (the reading Manittha supports).
    LagnaLordMotion,
}

/// How "the grahas between the lagna and the Moon" are counted (X19).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BetweenBy {
    /// By degree: a graha past the rising degree and short of the Moon's,
    /// in the zodiac's order.
    #[default]
    Degree,
    /// By sign: a graha in a sign after the lagna's and before the Moon's.
    Sign,
}

/// Which half of the attendants V.22 puts outside the room.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OutsideHalf {
    /// The visible half outside, the invisible inside: Varahamihira's own,
    /// which the commentator says his *Swalpa Jataka* confirms.
    #[default]
    Visible,
    /// The reverse, which V.22 gives as some others'.
    Invisible,
}

/// The knobs `CIRCUMSTANCE` takes, one per crux.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct CircumstanceRules {
    /// What the Moon's not seeing the lagna asks (X16).
    pub moon_sees: MoonSees,
    /// Which houses the Sun has fallen to (X17).
    pub sun_fallen: SunFallen,
    /// Which reading of the presentation (X18).
    pub presentation_by: PresentationBy,
    /// How the grahas between the lagna and the Moon are counted (X19).
    pub between_by: BetweenBy,
    /// Which half of them is outside the room (V.22).
    pub outside: OutsideHalf,
}

/// The grahas V.2 and V.22 read: the seven, the Moon among them. The nodes
/// are not grahas Varahamihira counts here.
pub const SEVEN: [Graha; 7] = [
    Graha::Sun,
    Graha::Moon,
    Graha::Mars,
    Graha::Mercury,
    Graha::Jupiter,
    Graha::Venus,
    Graha::Saturn,
];

/// The sky at a candidate instant, as the clauses read it: the lagna, the
/// seven grahas and whether the lagna's lord is retrograde.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct BirthSky {
    /// The lagna, degrees.
    pub lagna_deg: f64,
    /// The seven grahas' longitudes, degrees, in [`SEVEN`]'s order.
    pub grahas_deg: [f64; 7],
    /// Whether the lagna's lord is retrograde, read only by
    /// [`PresentationBy::LagnaLordMotion`].
    pub lord_retrograde: bool,
}

impl BirthSky {
    /// A graha's longitude.
    #[must_use]
    pub fn of(&self, graha: Graha) -> f64 {
        SEVEN
            .iter()
            .zip(self.grahas_deg)
            .find_map(|(one, deg)| (*one == graha).then_some(deg))
            .unwrap_or(f64::NAN)
    }

    /// A graha's sign.
    fn sign(&self, graha: Graha) -> Rashi {
        Rashi::of_longitude(self.of(graha))
    }

    /// The lagna's sign.
    fn lagna(&self) -> Rashi {
        Rashi::of_longitude(self.lagna_deg)
    }
}

/// V.1–2: whether the father was away, and where.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is a clause the verse names, reported whether or not it holds"
)]
pub struct Father {
    /// The Moon's aspect on the lagna's sign (BJ II.13).
    pub moon_aspect: Strength,
    /// V.1: the Moon does not see the lagna, as [`MoonSees`] asks.
    pub unseen: bool,
    /// V.2: Saturn in the lagna.
    pub saturn_rising: bool,
    /// V.2: Mars in the 7th.
    pub mars_setting: bool,
    /// V.2: the Moon between Mercury and Venus, one in the 12th from her
    /// and the other in the 2nd, or all three in her sign with her degree
    /// between theirs (the 1912 print's note).
    pub moon_hemmed: bool,
    /// Whether any of them holds: the father away.
    pub away: bool,
    /// V.1's second half, where V.1 holds and the Sun has fallen from the
    /// 10th: where the father is.
    pub whereabouts: Option<Whereabouts>,
    /// The Sun's house from the lagna, by sign.
    pub sun_house: u8,
}

/// Where V.1 puts an absent father, by the Sun's sign.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Whereabouts {
    /// A movable sign: in a foreign country.
    Abroad,
    /// A fixed sign: in his own country, away from the house.
    OwnCountry,
    /// A dual sign: on his way back.
    Returning,
}

/// V.17's presentation as the sky foretells it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PresentationReading {
    /// The reading asked.
    pub by: PresentationBy,
    /// How the rising sign rises.
    pub rising: Rising,
    /// The lagna's lord.
    pub lord: Graha,
    /// Whether it is retrograde.
    pub lord_retrograde: bool,
    /// What the reading foretells: under [`PresentationBy::LagnaLordMotion`]
    /// the head for a natural birth and the feet for an irregular one,
    /// where any presentation but the head agrees.
    pub foretold: Presentation,
}

impl PresentationReading {
    /// Whether a remembered presentation agrees.
    #[must_use]
    pub fn agrees(&self, given: Presentation) -> bool {
        match self.by {
            PresentationBy::RisingSign => given == self.foretold,
            PresentationBy::LagnaLordMotion => {
                (given == Presentation::Head) == (self.foretold == Presentation::Head)
            }
        }
    }
}

/// V.18's lamp as the sky foretells it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Lamp {
    /// The oil left, one at the start of the Moon's sign and none at its end.
    pub oil: f64,
    /// Its nearest level.
    pub oil_level: Level,
    /// The wick left, one at the start of the rising sign and none at its end.
    pub wick: f64,
    /// Its nearest level.
    pub wick_level: Level,
}

/// V.22's attendants as the sky foretells them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Attending {
    /// The grahas between the lagna and the Moon, as [`BetweenBy`] counts.
    pub between: Vec<Graha>,
    /// Those of them in the visible half, the 7th house to the 12th by
    /// degree from the descendant up to the ascendant.
    pub visible: Vec<Graha>,
    /// How many inside the room.
    pub inside: u8,
    /// How many outside it.
    pub outside: u8,
}

/// The clause a weight comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Indication {
    /// The father away or present (V.1–2).
    Father,
    /// The presentation (V.17).
    Presentation,
    /// The oil (V.18).
    Oil,
    /// The wick (V.18).
    Wick,
    /// How many attended (V.22).
    AttendantsTotal,
    /// How many inside (V.22).
    AttendantsInside,
    /// How many outside (V.22).
    AttendantsOutside,
}

impl Indication {
    /// The verse it reads, in Iyer's numbering.
    #[must_use]
    pub const fn source(self) -> &'static str {
        match self {
            Indication::Father => "BJ V.1–2",
            Indication::Presentation => "BJ V.17, I.10",
            Indication::Oil | Indication::Wick => "BJ V.18",
            Indication::AttendantsTotal
            | Indication::AttendantsInside
            | Indication::AttendantsOutside => "BJ V.22",
        }
    }
}

/// One fact given, set against the clause that reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Weight {
    /// The clause.
    pub indication: Indication,
    /// Whether the fact agrees with what the sky foretold.
    pub agrees: bool,
}

/// What `CIRCUMSTANCE` reads at a candidate instant.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Circumstance {
    /// The sky the clauses read.
    pub sky: BirthSky,
    /// V.1–2.
    pub father: Father,
    /// V.17.
    pub presentation: PresentationReading,
    /// V.18.
    pub lamp: Lamp,
    /// V.22.
    pub attending: Attending,
    /// One per fact given, in [`Indication`]'s order; none for a fact
    /// absent.
    pub weights: Vec<Weight>,
}

impl Circumstance {
    /// How many facts agree.
    #[must_use]
    pub fn agreeing(&self) -> usize {
        self.weights.iter().filter(|weight| weight.agrees).count()
    }
}

/// The circumstances at a candidate instant, read from the sky.
///
/// # Errors
///
/// Whatever the [`ConceptionSky`] refuses.
pub fn circumstance(
    sky: &dyn ConceptionSky,
    at: JulianDay<Utc>,
    facts: &Facts,
    rules: CircumstanceRules,
) -> Result<Circumstance, Error> {
    let lagna_deg = sky.ascendant_deg(at)?;
    let mut grahas_deg = [0.0; 7];
    for (deg, graha) in grahas_deg.iter_mut().zip(SEVEN) {
        *deg = sky.graha_deg(graha, at)?;
    }
    let lord = Rashi::of_longitude(lagna_deg).attributes().lord;
    let lord_retrograde = match rules.presentation_by {
        PresentationBy::LagnaLordMotion => sky.graha_speed_deg(lord, at)? < 0.0,
        PresentationBy::RisingSign => false,
    };
    Ok(judged(
        BirthSky {
            lagna_deg,
            grahas_deg,
            lord_retrograde,
        },
        facts,
        rules,
    ))
}

/// The circumstances of a sky given whole, apart from the ephemeris, so a
/// clause can be asserted from placements written down.
#[must_use]
pub fn judged(sky: BirthSky, facts: &Facts, rules: CircumstanceRules) -> Circumstance {
    let father = father(&sky, rules);
    let presentation = presentation(&sky, rules.presentation_by);
    let lamp = lamp(&sky);
    let attending = attending(&sky, rules);
    let mut weights = Vec::new();
    let mut weigh = |indication, agrees| weights.push(Weight { indication, agrees });
    if let Some(present) = facts.father_present {
        weigh(Indication::Father, present != father.away);
    }
    if let Some(given) = facts.presentation {
        weigh(Indication::Presentation, presentation.agrees(given));
    }
    if let Some(given) = facts.oil {
        weigh(Indication::Oil, given == lamp.oil_level);
    }
    if let Some(given) = facts.wick {
        weigh(Indication::Wick, given == lamp.wick_level);
    }
    if let Some(given) = facts.attendants {
        let total = attending.inside + attending.outside;
        for (indication, count, foretold) in [
            (Indication::AttendantsTotal, given.total, total),
            (Indication::AttendantsInside, given.inside, attending.inside),
            (
                Indication::AttendantsOutside,
                given.outside,
                attending.outside,
            ),
        ] {
            if let Some(count) = count {
                weigh(indication, count == foretold);
            }
        }
    }
    Circumstance {
        sky,
        father,
        presentation,
        lamp,
        attending,
        weights,
    }
}

/// V.1–2.
fn father(sky: &BirthSky, rules: CircumstanceRules) -> Father {
    let lagna = sky.lagna();
    let moon = sky.sign(Graha::Moon);
    let moon_aspect = drishti::between(Graha::Moon, moon, lagna);
    let unseen = match rules.moon_sees {
        MoonSees::AnyAspect => moon_aspect == Strength::None,
        MoonSees::Full => moon_aspect != Strength::Full,
    };
    let saturn_rising = sky.sign(Graha::Saturn) == lagna;
    let mars_setting = house(lagna, sky.sign(Graha::Mars)) == 7;
    let moon_hemmed = hemmed(
        sky.of(Graha::Moon),
        sky.of(Graha::Mercury),
        sky.of(Graha::Venus),
    );
    let sun_house = house(lagna, sky.sign(Graha::Sun));
    let whereabouts = (unseen && rules.sun_fallen.holds(sun_house)).then(|| {
        match sky.sign(Graha::Sun).attributes().modality {
            Modality::Chara => Whereabouts::Abroad,
            Modality::Sthira => Whereabouts::OwnCountry,
            _ => Whereabouts::Returning,
        }
    });
    Father {
        moon_aspect,
        unseen,
        saturn_rising,
        mars_setting,
        moon_hemmed,
        away: unseen || saturn_rising || mars_setting || moon_hemmed,
        whereabouts,
        sun_house,
    }
}

/// Whether the Moon lies between two grahas: one in the 12th from her and
/// the other in the 2nd, or all three in one sign with her degree between
/// theirs (V.2 and the 1912 print's note).
#[must_use]
pub fn hemmed(moon_deg: f64, first_deg: f64, second_deg: f64) -> bool {
    let moon = Rashi::of_longitude(moon_deg);
    let [first, second] = [first_deg, second_deg].map(|deg| house(moon, Rashi::of_longitude(deg)));
    match (first, second) {
        (12, 2) | (2, 12) => true,
        (1, 1) => first_deg.min(second_deg) < moon_deg && moon_deg < first_deg.max(second_deg),
        _ => false,
    }
}

/// V.17.
fn presentation(sky: &BirthSky, by: PresentationBy) -> PresentationReading {
    let attributes = sky.lagna().attributes();
    let foretold = match by {
        PresentationBy::RisingSign => match attributes.rising {
            Rising::Sirshodaya => Presentation::Head,
            Rising::Prishtodaya => Presentation::Feet,
            _ => Presentation::Hands,
        },
        PresentationBy::LagnaLordMotion if sky.lord_retrograde => Presentation::Feet,
        PresentationBy::LagnaLordMotion => Presentation::Head,
    };
    PresentationReading {
        by,
        rising: attributes.rising,
        lord: attributes.lord,
        lord_retrograde: sky.lord_retrograde,
        foretold,
    }
}

/// V.18: what is left of a sign past a degree, one at its start.
fn left_in_sign(deg: f64) -> f64 {
    1.0 - deg.rem_euclid(30.0) / 30.0
}

/// V.18.
fn lamp(sky: &BirthSky) -> Lamp {
    let oil = left_in_sign(sky.of(Graha::Moon));
    let wick = left_in_sign(sky.lagna_deg);
    Lamp {
        oil,
        oil_level: Level::nearest(oil),
        wick,
        wick_level: Level::nearest(wick),
    }
}

/// V.22.
fn attending(sky: &BirthSky, rules: CircumstanceRules) -> Attending {
    let moon = sky.of(Graha::Moon);
    let between: Vec<Graha> = SEVEN
        .into_iter()
        .filter(|graha| *graha != Graha::Moon)
        .filter(|graha| {
            let deg = sky.of(*graha);
            match rules.between_by {
                BetweenBy::Degree => {
                    let past = (deg - sky.lagna_deg).rem_euclid(360.0);
                    0.0 < past && past < (moon - sky.lagna_deg).rem_euclid(360.0)
                }
                BetweenBy::Sign => {
                    let lagna = sky.lagna();
                    let far = house(lagna, Rashi::of_longitude(moon));
                    let at = house(lagna, Rashi::of_longitude(deg));
                    1 < at && at < far
                }
            }
        })
        .collect();
    let visible: Vec<Graha> = between
        .iter()
        .copied()
        .filter(|graha| (sky.of(*graha) - sky.lagna_deg).rem_euclid(360.0) >= 180.0)
        .collect();
    let seen = u8::try_from(visible.len()).unwrap_or(u8::MAX);
    let unseen = u8::try_from(between.len() - visible.len()).unwrap_or(u8::MAX);
    let (inside, outside) = match rules.outside {
        OutsideHalf::Visible => (unseen, seen),
        OutsideHalf::Invisible => (seen, unseen),
    };
    Attending {
        between,
        visible,
        inside,
        outside,
    }
}
