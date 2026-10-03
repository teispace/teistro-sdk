//! The ten considerations of the South: *Kalaprakasika* XIII (N. P.
//! Subramania Iyer's translation, 1917, pp. 69–77), each agreeing or not
//! (`03-design/matching.md`).
//!
//! Every table here is the chapter's own, never the Ashta Koota's (C266):
//! its yoni, its lords' friendships and its Vasyam differ from
//! *Muhurta Chintamani*'s.

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Gana, Graha, Koota, Nakshatra, Parity, Rashi, Yoni};

use crate::native::Native;

/// The ten in the chapter's order, each a catalogue koota (C282): Dhinam is
/// the tara count, Ganam the gana, Rasi the signs' distance (`BHAKOOT`),
/// Rasyadhipathi their lords (`GRAHA_MAITRI`) and Vasyam their vashya.
pub const PORUTHAM: [Koota; 10] = [
    Koota::Tara,
    Koota::Gana,
    Koota::Mahendra,
    Koota::StreeDeergha,
    Koota::Yoni,
    Koota::Bhakoot,
    Koota::GrahaMaitri,
    Koota::Vashya,
    Koota::Rajju,
    Koota::Vedha,
];

/// The chief five of p. 76: Dhinam, Ganam, Yoni, Rasi and Rajju.
pub const CHIEF_FIVE: [Koota; 5] = [
    Koota::Tara,
    Koota::Gana,
    Koota::Yoni,
    Koota::Bhakoot,
    Koota::Rajju,
];

/// Whose quarter comes first when one star, spanning two signs, is both
/// natives' (C270).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TwoSignStar {
    /// The groom's quarter is the earlier (p. 71), as the line for a star
    /// split two and two also has it.
    #[default]
    GroomEarlier,
    /// For a star with one quarter in the first sign, that quarter is the
    /// bride's (pp. 71–72); a star split otherwise as the default.
    BrideFirstSign,
}

/// How far the groom's star must stand from the bride's for
/// Sthree-Dheergham (C272).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeerghaBeyond {
    /// Beyond the 13th (p. 72).
    #[default]
    Thirteenth,
    /// Beyond the 7th, as "some writers hold".
    Seventh,
}

/// Which friendship of the lords agrees on Rasyadhipathi and lifts by the
/// p. 76 exception (C273).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LordsFriendship {
    /// Each lord calls the other a friend.
    #[default]
    Mutual,
    /// Either calls the other a friend.
    OneWay,
}

/// The readings the ten are computed under; each default is the source's
/// own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PoruthamRules {
    /// Whose quarter comes first in a star across two signs.
    pub two_sign_star: TwoSignStar,
    /// How far Sthree-Dheergham asks.
    pub deergha_beyond: DeerghaBeyond,
    /// Which friendship of the lords agrees.
    pub lords_friendship: LordsFriendship,
}

/// Which of the chapter's rules decided Dhinam.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DhinamRule {
    /// The count alone: the 3rd, 5th and 7th of the first nine disagree
    /// (p. 69, C267).
    Count,
    /// A quarter of the second nine: the 1st of the 12th, the 4th of the
    /// 14th, the 3rd of the 16th, the groom's star's (C268).
    SecondRoundQuarter,
    /// The 22nd, *Vadha-Vainasika*.
    VadhaVainasika,
    /// The 27th, in two signs.
    TwentySeventh,
    /// One star for both, among the excellent (p. 71).
    CommonExcellent,
    /// One star for both, among the neutral.
    CommonNeutral,
    /// One star for both, among those to avoid.
    CommonAvoid,
    /// One star for both across two signs, read by whose quarter comes
    /// first (C270).
    TwoSigns,
    /// Two stars in one sign: the groom's must be prior (p. 71).
    SameSign,
    /// Two stars in one sign, the groom's next after one the chapter
    /// names.
    NextStar,
    /// One of the four happy pairs of p. 70, either way round (C269).
    HappyPair,
}

/// A Rajju division, foot to head (p. 75).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Rajju {
    /// The foot: travel.
    Padha,
    /// The thigh: loss of property.
    Ooru,
    /// The navel: loss of offspring.
    Nabhi,
    /// The neck: the woman's death.
    Kanta,
    /// The head: the man's death.
    Siro,
}

impl Rajju {
    /// A star's division: its place from Ashvini, taken modulo nine and
    /// folded, which gives p. 75's lists star for star (C275).
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_matching::Rajju;
    ///
    /// assert_eq!(Rajju::of(Nakshatra::Ashwini), Rajju::Padha);
    /// assert_eq!(Rajju::of(Nakshatra::Chitra), Rajju::Siro);
    /// ```
    #[must_use]
    pub fn of(star: Nakshatra) -> Rajju {
        let k = star.id() % 9;
        match k.min(8 - k) {
            0 => Rajju::Padha,
            1 => Rajju::Ooru,
            2 => Rajju::Nabhi,
            3 => Rajju::Kanta,
            _ => Rajju::Siro,
        }
    }
}

/// The p. 76 exception's three clauses, any one of which lifts Rajju,
/// Vedhai, Ganam and Rasi (C277).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PoruthamException {
    /// One lord rules both signs.
    pub one_lord: bool,
    /// The lords are friendly, as the rules read friendship.
    pub lords_friendly: bool,
    /// The signs are opposite.
    pub opposite: bool,
}

impl PoruthamException {
    /// Whether any clause holds.
    #[must_use]
    pub const fn holds(self) -> bool {
        self.one_lord || self.lords_friendly || self.opposite
    }
}

/// What a consideration read, by koota.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "koota", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PoruthamReading {
    /// Dhinam: the count and the rule that decided it.
    Tara {
        /// The groom's star counted from the bride's, 1 to 27.
        count: u8,
        /// The rule.
        rule: DhinamRule,
    },
    /// Ganam: the two ganas.
    Gana {
        /// The bride's.
        bride: Gana,
        /// The groom's.
        groom: Gana,
        /// A Rakshasa beside another gana, the bride's star beyond the
        /// 14th from the groom's: the evil "diminishes" (C279).
        diminished: bool,
    },
    /// Mahendra: the count.
    Mahendra {
        /// The groom's star counted from the bride's, 1 to 27.
        count: u8,
    },
    /// Sthree-Dheergham: the count.
    StreeDeergha {
        /// The groom's star counted from the bride's, 1 to 27.
        count: u8,
    },
    /// Yoni, on the chapter's own table (C278).
    Yoni {
        /// The bride's.
        bride: Yoni,
        /// The groom's.
        groom: Yoni,
        /// Whether they are among the chapter's eight enmities.
        hostile: bool,
    },
    /// Rasi: how far the groom's sign stands from the bride's.
    Bhakoot {
        /// The groom's sign counted from the bride's, 1 to 12.
        apart: u8,
    },
    /// Rasyadhipathi, on the chapter's own friendships.
    GrahaMaitri {
        /// The bride's sign lord.
        bride: Graha,
        /// The groom's.
        groom: Graha,
        /// Whether the bride's lord calls the groom's a friend; a lord is
        /// its own.
        bride_calls_friend: bool,
        /// Whether the groom's lord calls the bride's a friend.
        groom_calls_friend: bool,
    },
    /// Vasyam, on p. 75's table.
    Vashya {
        /// Whether the bride's sign is concordant to the groom's.
        bride_to_groom: bool,
        /// Whether the groom's sign is concordant to the bride's.
        groom_to_bride: bool,
    },
    /// Rajju: the two divisions.
    Rajju {
        /// The bride's.
        bride: Rajju,
        /// The groom's.
        groom: Rajju,
    },
    /// Vedhai: whether the two stars pierce each other.
    Vedha {
        /// Whether they are a pair of p. 76.
        pierced: bool,
    },
}

impl PoruthamReading {
    /// The koota this is the reading of.
    #[must_use]
    pub const fn koota(&self) -> Koota {
        match self {
            PoruthamReading::Tara { .. } => Koota::Tara,
            PoruthamReading::Gana { .. } => Koota::Gana,
            PoruthamReading::Mahendra { .. } => Koota::Mahendra,
            PoruthamReading::StreeDeergha { .. } => Koota::StreeDeergha,
            PoruthamReading::Yoni { .. } => Koota::Yoni,
            PoruthamReading::Bhakoot { .. } => Koota::Bhakoot,
            PoruthamReading::GrahaMaitri { .. } => Koota::GrahaMaitri,
            PoruthamReading::Vashya { .. } => Koota::Vashya,
            PoruthamReading::Rajju { .. } => Koota::Rajju,
            PoruthamReading::Vedha { .. } => Koota::Vedha,
        }
    }
}

/// One consideration: whether it agrees, and what it read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct PoruthamRow {
    /// Whether it agrees, a lift included.
    pub agrees: bool,
    /// Whether it agrees only by the p. 76 exception.
    pub lifted: bool,
    /// What it read.
    pub reading: PoruthamReading,
}

/// The ten considerations of a bride and a groom.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Porutham {
    /// The ten, in the chapter's order.
    pub considerations: Vec<PoruthamRow>,
    /// How many agree; the chapter asks "at least five".
    pub agreeing: u8,
    /// How many of the chief five agree.
    pub chief_agreeing: u8,
    /// The p. 76 exception's clauses.
    pub exception: PoruthamException,
}

impl Porutham {
    /// One consideration's row.
    #[must_use]
    pub fn row(&self, koota: Koota) -> Option<&PoruthamRow> {
        let at = PORUTHAM.iter().position(|one| *one == koota)?;
        self.considerations.get(at)
    }
}

/// The ten considerations of a bride and a groom: whether each agrees and
/// what it read, how many agree, and the p. 76 exception's clauses
/// (`03-design/matching.md`). Never a verdict: "at least five" is the
/// reader's to apply.
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{PoruthamRules, Native, porutham};
///
/// // Moons in Ashvini and Bharani, both in Aries: the groom's star next
/// // after the bride's, which p. 71 allows for Ashvini.
/// let bride = Native::of_moon(5.0)?;
/// let groom = Native::of_moon(18.0)?;
/// let ten = porutham(bride, groom, PoruthamRules::default());
/// assert!(ten.row(Koota::Tara).is_some_and(|row| row.agrees));
/// assert!(ten.agreeing <= 10);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn porutham(bride: Native, groom: Native, rules: PoruthamRules) -> Porutham {
    let count = count(bride, groom);
    let apart = u8::try_from((groom.sign() + 12 - bride.sign()) % 12 + 1).unwrap_or(1);
    let (lord_b, lord_g) = (bride.rashi.attributes().lord, groom.rashi.attributes().lord);
    let (b_calls, g_calls) = (calls_friend(lord_b, lord_g), calls_friend(lord_g, lord_b));
    let friendly = match rules.lords_friendship {
        LordsFriendship::Mutual => b_calls && g_calls,
        LordsFriendship::OneWay => b_calls || g_calls,
    };
    let exception = PoruthamException {
        one_lord: lord_b == lord_g,
        lords_friendly: lord_b != lord_g && friendly,
        opposite: apart == 7,
    };
    let liftable = |own: bool, reading: PoruthamReading| PoruthamRow {
        agrees: own || exception.holds(),
        lifted: !own && exception.holds(),
        reading,
    };
    let plain = |agrees: bool, reading: PoruthamReading| PoruthamRow {
        agrees,
        lifted: false,
        reading,
    };
    let (gana, gana_agrees) = ganam(bride, groom);
    let (yoni_b, yoni_g) = (yoni_of(bride.nakshatra), yoni_of(groom.nakshatra));
    let hostile = HOSTILE.contains(&(yoni_b, yoni_g)) || HOSTILE.contains(&(yoni_g, yoni_b));
    let (to_groom, to_bride) = (
        concordant(bride.rashi, groom.rashi),
        concordant(groom.rashi, bride.rashi),
    );
    let (rajju_b, rajju_g) = (Rajju::of(bride.nakshatra), Rajju::of(groom.nakshatra));
    let pierced = pierce(bride.nakshatra, groom.nakshatra);
    let (rule, dhinam_agrees) = dhinam(bride, groom, count, rules);
    let considerations = vec![
        plain(dhinam_agrees, PoruthamReading::Tara { count, rule }),
        liftable(gana_agrees, gana),
        plain(
            count >= 4 && count % 3 == 1,
            PoruthamReading::Mahendra { count },
        ),
        plain(
            count
                > match rules.deergha_beyond {
                    DeerghaBeyond::Thirteenth => 13,
                    DeerghaBeyond::Seventh => 7,
                },
            PoruthamReading::StreeDeergha { count },
        ),
        plain(
            !hostile,
            PoruthamReading::Yoni {
                bride: yoni_b,
                groom: yoni_g,
                hostile,
            },
        ),
        liftable(rasi(bride.rashi, apart), PoruthamReading::Bhakoot { apart }),
        plain(
            lord_b == lord_g || friendly,
            PoruthamReading::GrahaMaitri {
                bride: lord_b,
                groom: lord_g,
                bride_calls_friend: b_calls,
                groom_calls_friend: g_calls,
            },
        ),
        plain(
            to_groom || to_bride,
            PoruthamReading::Vashya {
                bride_to_groom: to_groom,
                groom_to_bride: to_bride,
            },
        ),
        liftable(
            rajju_b != rajju_g,
            PoruthamReading::Rajju {
                bride: rajju_b,
                groom: rajju_g,
            },
        ),
        liftable(!pierced, PoruthamReading::Vedha { pierced }),
    ];
    let agreeing = considerations.iter().filter(|row| row.agrees).count();
    let chief_agreeing = considerations
        .iter()
        .filter(|row| row.agrees && CHIEF_FIVE.contains(&row.reading.koota()))
        .count();
    Porutham {
        considerations,
        agreeing: u8::try_from(agreeing).unwrap_or(u8::MAX),
        chief_agreeing: u8::try_from(chief_agreeing).unwrap_or(u8::MAX),
        exception,
    }
}

/// The count from one star to another, both inclusive, 1 to 27.
fn count_from(from: Nakshatra, to: Nakshatra) -> u8 {
    u8::try_from((to.id() + 27 - from.id()) % 27 + 1).unwrap_or(1)
}

/// The groom's star counted from the bride's.
fn count(bride: Native, groom: Native) -> u8 {
    count_from(bride.nakshatra, groom.nakshatra)
}

/// Whether an unordered pair of stars is among `pairs`.
fn paired(pairs: &[(Nakshatra, Nakshatra)], a: Nakshatra, b: Nakshatra) -> bool {
    pairs.contains(&(a, b)) || pairs.contains(&(b, a))
}

/// p. 71's common stars that are excellent.
const COMMON_EXCELLENT: [Nakshatra; 8] = [
    Nakshatra::Rohini,
    Nakshatra::Ardra,
    Nakshatra::Magha,
    Nakshatra::Hasta,
    Nakshatra::Vishakha,
    Nakshatra::Shravana,
    Nakshatra::UttaraBhadrapada,
    Nakshatra::Revati,
];

/// p. 71's common stars to avoid; the other eleven are neutral.
const COMMON_AVOID: [Nakshatra; 8] = [
    Nakshatra::Bharani,
    Nakshatra::Ashlesha,
    Nakshatra::Swati,
    Nakshatra::Jyeshtha,
    Nakshatra::Mula,
    Nakshatra::Dhanishtha,
    Nakshatra::Shatabhisha,
    Nakshatra::PurvaBhadrapada,
];

/// The bride's stars after which the groom's may be the next in one sign
/// (p. 71).
const NEXT_STAR: [Nakshatra; 8] = [
    Nakshatra::Ashwini,
    Nakshatra::Krittika,
    Nakshatra::Mrigashira,
    Nakshatra::Magha,
    Nakshatra::Hasta,
    Nakshatra::Swati,
    Nakshatra::PurvaAshadha,
    Nakshatra::Shatabhisha,
];

/// The pairs in one sign p. 71 leaves out of the same-sign rule.
const LEFT_OUT: [(Nakshatra, Nakshatra); 3] = [
    (Nakshatra::Bharani, Nakshatra::Krittika),
    (Nakshatra::Dhanishtha, Nakshatra::Shatabhisha),
    (Nakshatra::Pushya, Nakshatra::Ashlesha),
];

/// The four happy pairs of p. 70.
const HAPPY: [(Nakshatra, Nakshatra); 4] = [
    (Nakshatra::Ardra, Nakshatra::UttaraPhalguni),
    (Nakshatra::PurvaPhalguni, Nakshatra::Anuradha),
    (Nakshatra::Chitra, Nakshatra::Pushya),
    (Nakshatra::Punarvasu, Nakshatra::Hasta),
];

/// How many of a star's four quarters fall in its first sign.
fn first_sign_quarters(star: Nakshatra) -> u16 {
    let start = star.id() * 4;
    (9 - start % 9).min(4)
}

/// Dhinam's rule and whether it agrees (pp. 69–72).
fn dhinam(bride: Native, groom: Native, count: u8, rules: PoruthamRules) -> (DhinamRule, bool) {
    let one_sign = bride.rashi == groom.rashi;
    if count == 1 {
        if COMMON_AVOID.contains(&bride.nakshatra) {
            return (DhinamRule::CommonAvoid, false);
        }
        if !one_sign {
            let groom_first = groom.pada < bride.pada;
            let agrees = match rules.two_sign_star {
                TwoSignStar::BrideFirstSign if first_sign_quarters(bride.nakshatra) == 1 => {
                    !groom_first
                }
                _ => groom_first,
            };
            return (DhinamRule::TwoSigns, agrees);
        }
        return if COMMON_EXCELLENT.contains(&bride.nakshatra) {
            (DhinamRule::CommonExcellent, true)
        } else {
            (DhinamRule::CommonNeutral, true)
        };
    }
    if one_sign && !paired(&LEFT_OUT, bride.nakshatra, groom.nakshatra) {
        // One sign holds at most three stars, so the groom's is prior at
        // the 26th or 27th count and after at the 2nd or 3rd.
        return match count {
            26 | 27 => (DhinamRule::SameSign, true),
            2 if NEXT_STAR.contains(&bride.nakshatra) => (DhinamRule::NextStar, true),
            _ => (DhinamRule::SameSign, false),
        };
    }
    if paired(&HAPPY, bride.nakshatra, groom.nakshatra) {
        return (DhinamRule::HappyPair, true);
    }
    match count {
        1..=9 => (DhinamRule::Count, !matches!(count, 3 | 5 | 7)),
        12 if groom.pada == 1 => (DhinamRule::SecondRoundQuarter, false),
        14 if groom.pada == 4 => (DhinamRule::SecondRoundQuarter, false),
        16 if groom.pada == 3 => (DhinamRule::SecondRoundQuarter, false),
        22 => (DhinamRule::VadhaVainasika, false),
        27 if !one_sign => (DhinamRule::TwentySeventh, false),
        _ => (DhinamRule::Count, true),
    }
}

/// Ganam's reading and whether it agrees (p. 72, C271, C279).
fn ganam(bride: Native, groom: Native) -> (PoruthamReading, bool) {
    let (b, g) = (
        bride.nakshatra.attributes().gana,
        groom.nakshatra.attributes().gana,
    );
    let agrees = b == g
        || matches!(
            (b, g),
            (Gana::Deva, Gana::Manushya) | (Gana::Manushya, Gana::Deva)
        );
    let rakshasa = b != g && (b == Gana::Rakshasa || g == Gana::Rakshasa);
    let diminished = rakshasa && count_from(groom.nakshatra, bride.nakshatra) > 14;
    (
        PoruthamReading::Gana {
            bride: b,
            groom: g,
            diminished,
        },
        agrees,
    )
}

/// A star's yoni on the chapter's table: the catalogue's, but for
/// Uttarashadha the cow (p. 73, C278).
pub(crate) fn yoni_of(star: Nakshatra) -> Yoni {
    if star == Nakshatra::UttaraAshadha {
        Yoni::Cow
    } else {
        star.attributes().yoni
    }
}

/// The chapter's eight enmities (p. 73).
const HOSTILE: [(Yoni, Yoni); 8] = [
    (Yoni::Monkey, Yoni::Goat),
    (Yoni::Deer, Yoni::Elephant),
    (Yoni::Horse, Yoni::Buffalo),
    (Yoni::Cow, Yoni::Tiger),
    (Yoni::Rat, Yoni::Cat),
    (Yoni::Serpent, Yoni::Rat),
    (Yoni::Serpent, Yoni::Mongoose),
    (Yoni::Dog, Yoni::Deer),
];

/// Whether Rasi agrees, before the exception (pp. 73–74, C280, C281).
fn rasi(bride: Rashi, apart: u8) -> bool {
    match apart {
        // The groom's 2nd is even, and his 6th felicitous, when hers is odd.
        2 | 6 => bride.attributes().parity == Parity::Odd,
        3..=5 => false,
        _ => true,
    }
}

/// Whether a lord calls another a friend on the chapter's own table (pp.
/// 74–75, C273). A lord is its own friend; Saturn names only Jupiter an
/// enemy, so the others are his friends.
pub(crate) fn calls_friend(lord: Graha, other: Graha) -> bool {
    if lord == other {
        return true;
    }
    match lord {
        Graha::Sun => other == Graha::Jupiter,
        Graha::Moon => matches!(other, Graha::Mercury | Graha::Jupiter),
        Graha::Mars => matches!(other, Graha::Mercury | Graha::Venus),
        Graha::Mercury => other != Graha::Sun,
        Graha::Jupiter => other != Graha::Mars,
        Graha::Venus => !matches!(other, Graha::Sun | Graha::Moon),
        Graha::Saturn => other != Graha::Jupiter,
        // Only the seven rule signs.
        _ => false,
    }
}

/// Whether `sign` is concordant to `to` on p. 75's table (C274: never to
/// itself).
#[expect(
    clippy::match_same_arms,
    reason = "the table is transcribed sign by sign, as p. 75 prints it"
)]
pub(crate) fn concordant(sign: Rashi, to: Rashi) -> bool {
    let concordant: &[Rashi] = match to {
        Rashi::Aries => &[Rashi::Leo, Rashi::Scorpio],
        Rashi::Taurus => &[Rashi::Cancer, Rashi::Leo],
        Rashi::Gemini => &[Rashi::Virgo],
        Rashi::Cancer => &[Rashi::Scorpio, Rashi::Sagittarius],
        Rashi::Leo => &[Rashi::Libra],
        Rashi::Virgo => &[Rashi::Gemini, Rashi::Pisces],
        Rashi::Libra => &[Rashi::Capricorn],
        Rashi::Scorpio => &[Rashi::Virgo, Rashi::Cancer],
        Rashi::Sagittarius => &[Rashi::Pisces],
        Rashi::Capricorn => &[Rashi::Aquarius, Rashi::Aries],
        Rashi::Aquarius => &[Rashi::Aries],
        Rashi::Pisces => &[Rashi::Capricorn],
        // The catalogue's twelve are all above.
        _ => &[],
    };
    concordant.contains(&sign)
}

/// The twelve Vedhai pairs of p. 76; the triple is the Siro rajju.
const VEDHA: [(Nakshatra, Nakshatra); 12] = [
    (Nakshatra::Ashwini, Nakshatra::Jyeshtha),
    (Nakshatra::Bharani, Nakshatra::Anuradha),
    (Nakshatra::Krittika, Nakshatra::Vishakha),
    (Nakshatra::Rohini, Nakshatra::Swati),
    (Nakshatra::Ardra, Nakshatra::Shravana),
    (Nakshatra::Punarvasu, Nakshatra::UttaraAshadha),
    (Nakshatra::Pushya, Nakshatra::PurvaAshadha),
    (Nakshatra::Ashlesha, Nakshatra::Mula),
    (Nakshatra::Magha, Nakshatra::Revati),
    (Nakshatra::PurvaPhalguni, Nakshatra::UttaraBhadrapada),
    (Nakshatra::UttaraPhalguni, Nakshatra::PurvaBhadrapada),
    (Nakshatra::Hasta, Nakshatra::Shatabhisha),
];

/// Whether two stars pierce each other (p. 76, C276).
pub(crate) fn pierce(a: Nakshatra, b: Nakshatra) -> bool {
    let triple = [
        Nakshatra::Mrigashira,
        Nakshatra::Chitra,
        Nakshatra::Dhanishtha,
    ];
    paired(&VEDHA, a, b) || (a != b && triple.contains(&a) && triple.contains(&b))
}
