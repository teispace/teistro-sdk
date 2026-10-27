//! The Ashta Koota: *Muhurta Chintamani* VI.21–34, with the points
//! *Daivajna-manohara* gives each koota as the *Piyushadhara* commentary
//! cites it (`03-design/matching.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Gana, Graha, Koota, Nadi, Rashi, Varna, Yoni};

use crate::native::Native;

/// The kootas in the verse's order, each worth one point more than the last
/// (VI.21).
///
/// ```
/// use teistro_matching::{ASHTA_KOOTA, KootaRules, Native, ashta_koota};
///
/// // Two Moons at 45°: Rohini's second pada, in Taurus.
/// let moon = Native::of_moon(45.0)?;
/// let koota = ashta_koota(moon, moon, KootaRules::default());
/// let read: Vec<_> = koota.kootas.iter().map(|row| row.reading.koota()).collect();
/// assert_eq!(read, ASHTA_KOOTA);
/// // Each worth one more than the last.
/// let most: Vec<f64> = koota.kootas.iter().map(|row| row.max_points).collect();
/// assert_eq!(most, [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const ASHTA_KOOTA: [Koota; 8] = [
    Koota::Varna,
    Koota::Vashya,
    Koota::Tara,
    Koota::Yoni,
    Koota::GrahaMaitri,
    Koota::Gana,
    Koota::Bhakoot,
    Koota::Nadi,
];

/// The most an Ashta Koota totals.
///
/// ```
/// use teistro_matching::{ASHTA_KOOTA_POINTS, KootaRules, Native, ashta_koota};
///
/// // Two Moons at 45°: Rohini's second pada, in Taurus.
/// let moon = Native::of_moon(45.0)?;
/// let koota = ashta_koota(moon, moon, KootaRules::default());
/// assert_eq!(koota.kootas.iter().map(|row| row.max_points).sum::<f64>(), ASHTA_KOOTA_POINTS);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const ASHTA_KOOTA_POINTS: f64 = 36.0;

/// The point an equal varna earns (C259).
///
/// ```
/// use teistro_matching::{EqualVarna, KootaRules, Native, ashta_koota};
///
/// // Two Moons at 45°: Rohini's second pada, in Taurus.
/// let moon = Native::of_moon(45.0)?;
/// // One sign, one varna: a whole point, or a half as "some say".
/// let half = KootaRules { equal_varna: EqualVarna::Half, ..KootaRules::default() };
/// assert_eq!(ashta_koota(moon, moon, KootaRules::default()).total, 28.0);
/// assert_eq!(ashta_koota(moon, moon, half).total, 27.5);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EqualVarna {
    /// One point, as *Daivajna-manohara* gives it.
    #[default]
    Whole,
    /// A half, as "some say" in the same verse.
    Half,
}

/// The points of a Deva bride and a Manushya groom, which
/// *Daivajna-manohara* gives as "four or three" (C262).
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{DevaBride, KootaRules, Native, ashta_koota};
///
/// // A Deva bride in Ashvini and a Manushya groom in Rohini.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(45.0)?);
/// let gana = |rules| ashta_koota(bride, groom, rules).row(Koota::Gana).map(|row| row.points);
/// assert_eq!(gana(KootaRules::default()), Some(4.0));
/// assert_eq!(gana(KootaRules { deva_bride: DevaBride::Three, ..KootaRules::default() }), Some(3.0));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DevaBride {
    /// Four.
    #[default]
    Four,
    /// Three.
    Three,
}

/// How many exceptions lift a bad Bhakoot (C263).
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{BhakootLift, KootaReading, KootaRules, Native, ashta_koota};
///
/// // Moons in Ashvini (Aries) and Anuradha (Scorpio): the 8th apart.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(215.0)?);
/// let lifted = |rules| match ashta_koota(bride, groom, rules).row(Koota::Bhakoot).map(|row| row.reading) {
///     Some(KootaReading::Bhakoot { lifted, .. }) => lifted,
///     _ => false,
/// };
/// // One lord rules both signs, which is one exception: enough for VI.32–33,
/// // and short of the three Garga asks of a 6/8.
/// assert!(lifted(KootaRules::default()));
/// assert!(!lifted(KootaRules { bhakoot_lift: BhakootLift::Garga, ..KootaRules::default() }));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BhakootLift {
    /// Any one of the five of VI.32–33, the nadi pure.
    #[default]
    AnyOne,
    /// Garga's count, in the commentary: three of graha maitri, a pure
    /// tara and vashya for the 6/8, two for the 2/12 and 5/9, the nadi
    /// pure.
    Garga,
}

/// Which shared nadi is a dosha (C264).
///
/// ```
/// use teistro_core::catalogue::{Koota, Nadi};
/// use teistro_matching::{KootaReading, KootaRules, NadiDosha, Native, ashta_koota};
///
/// // Rohini's nadi is the last, so the middle-only reading sees no dosha;
/// // the shared nadi earns nothing either way.
/// let moon = Native::of_moon(45.0)?;
/// let rules = KootaRules { nadi_dosha: NadiDosha::MiddleOnly, ..KootaRules::default() };
/// let row = ashta_koota(moon, moon, rules).row(Koota::Nadi).copied();
/// assert!(matches!(
///     row.map(|row| (row.points, row.reading)),
///     Some((0.0, KootaReading::Nadi { bride: Nadi::Antya, dosha: false, .. }))
/// ));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NadiDosha {
    /// Any of the three, as VI.34 reads it.
    #[default]
    Any,
    /// The middle one only, as *Jyotih-prakasha* reads it in the
    /// commentary (p. 260).
    MiddleOnly,
}

/// Which text's gana points (C262, C761).
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{GanaPoints, KootaRules, Native, ashta_koota};
///
/// // A Deva bride in Ashvini and a Manushya groom in Rohini.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(45.0)?);
/// let gana = |rules| ashta_koota(bride, groom, rules).row(Koota::Gana).map(|row| row.points);
/// assert_eq!(gana(KootaRules::default()), Some(4.0));
/// let martanda = KootaRules { gana: GanaPoints::MuhurtaMartanda, ..KootaRules::default() };
/// assert_eq!(gana(martanda), Some(5.0));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GanaPoints {
    /// *Daivajna-manohara*, as the commentary on VI.30 cites it: the same
    /// gana 6, groom Deva with bride Manushya 5, the reverse "four or
    /// three" (`deva_bride`), groom Rakshasa with bride Deva 2 and with
    /// bride Manushya 1, otherwise 0.
    #[default]
    DaivajnaManohara,
    /// *Muhurta Martanda*, vivaha v. 11: the same gana 6, groom Deva with
    /// bride Manushya 6, the reverse 5, groom Rakshasa with bride Deva 1,
    /// otherwise 0. `deva_bride` is not read.
    MuhurtaMartanda,
}

/// Which vashya scheme scores the signs (C260, C762).
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{KootaRules, Native, VashyaPoints, ashta_koota};
///
/// // A bride in Aries and a groom in Taurus: neither is vashya to the
/// // other in the table, and neither is human in the print.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(45.0)?);
/// let vashya = |rules| ashta_koota(bride, groom, rules).row(Koota::Vashya).map(|row| row.points);
/// assert_eq!(vashya(KootaRules::default()), Some(0.0));
/// let printed = KootaRules { vashya: VashyaPoints::PrintedInNepal, ..KootaRules::default() };
/// assert_eq!(vashya(printed), Some(2.0));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VashyaPoints {
    /// The verse's relations where it decides and Kalaprakasika's table
    /// for the rest, scored as [`VashyaRelation`] gives.
    #[default]
    Verse,
    /// The grades of *Muhurta Martanda* (vivaha v. 10) over the classes
    /// a guna grid printed in Nepal scores: a human sign (Gemini, Virgo,
    /// Libra, Sagittarius, Aquarius) with any other but Scorpio ½, with
    /// Scorpio 1, and every other pair 2. Whole signs: the print's
    /// half-nakshatra split of Sagittarius and Capricorn is not read.
    PrintedInNepal,
}

/// The readings an Ashta Koota is computed under; each default is the
/// source's own.
///
/// ```
/// use teistro_matching::{BhakootLift, DevaBride, EqualVarna, KootaRules, NadiDosha};
///
/// let rules = KootaRules::default();
/// assert_eq!((rules.equal_varna, rules.deva_bride), (EqualVarna::Whole, DevaBride::Four));
/// assert_eq!((rules.bhakoot_lift, rules.nadi_dosha), (BhakootLift::AnyOne, NadiDosha::Any));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct KootaRules {
    /// The point of an equal varna.
    pub equal_varna: EqualVarna,
    /// The points of a Deva bride and a Manushya groom.
    pub deva_bride: DevaBride,
    /// How a bad Bhakoot is lifted.
    pub bhakoot_lift: BhakootLift,
    /// Which shared nadi is a dosha.
    pub nadi_dosha: NadiDosha,
    /// Which text's gana points.
    pub gana: GanaPoints,
    /// Which vashya scheme scores the signs.
    pub vashya: VashyaPoints,
}

impl KootaRules {
    /// The readings a guna grid printed in Nepal follows where a text or
    /// the print decides them: [`GanaPoints::MuhurtaMartanda`] and
    /// [`VashyaPoints::PrintedInNepal`], every other knob the verse's own
    /// (C763). Its yoni grades are the print's and not built (C261).
    ///
    /// ```
    /// use teistro_matching::{GanaPoints, KootaRules, VashyaPoints};
    ///
    /// let nepal = KootaRules::PRINTED_IN_NEPAL;
    /// assert_eq!((nepal.gana, nepal.vashya), (GanaPoints::MuhurtaMartanda, VashyaPoints::PrintedInNepal));
    /// assert_eq!(KootaRules { gana: GanaPoints::default(), vashya: VashyaPoints::default(), ..nepal }, KootaRules::default());
    /// ```
    pub const PRINTED_IN_NEPAL: KootaRules = KootaRules {
        equal_varna: EqualVarna::Whole,
        deva_bride: DevaBride::Four,
        bhakoot_lift: BhakootLift::AnyOne,
        nadi_dosha: NadiDosha::Any,
        gana: GanaPoints::MuhurtaMartanda,
        vashya: VashyaPoints::PrintedInNepal,
    };
}

/// How one sign stands to another in Vashya (C260).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::{VashyaRelation, vashya_relation};
///
/// assert_eq!(vashya_relation(Rashi::Gemini, Rashi::Virgo), VashyaRelation::Mutual);
/// assert_eq!(vashya_relation(Rashi::Aries, Rashi::Scorpio), VashyaRelation::OneWay);
/// // A water sign to a human one is its food.
/// assert_eq!(vashya_relation(Rashi::Cancer, Rashi::Virgo), VashyaRelation::Food);
/// assert_eq!(vashya_relation(Rashi::Aries, Rashi::Taurus), VashyaRelation::Neither);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum VashyaRelation {
    /// Each is vashya to the other, the same sign among them: 2.
    Mutual,
    /// One is vashya to the other: 1.
    OneWay,
    /// One is vashya to the other and its food, a water sign to a human
    /// one: ½.
    Food,
    /// Neither: 0.
    Neither,
}

/// How one yoni stands to another (C261).
///
/// ```
/// use teistro_core::catalogue::Yoni;
/// use teistro_matching::{YoniRelation, yoni_relation};
///
/// assert_eq!(yoni_relation(Yoni::Horse, Yoni::Horse), YoniRelation::Same);
/// assert_eq!(yoni_relation(Yoni::Horse, Yoni::Elephant), YoniRelation::Neutral);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum YoniRelation {
    /// The same yoni: 4.
    Same,
    /// Neither the same nor one of the seven great enmities: 2.
    Neutral,
    /// One of the seven great enmities of VI.25–26: 0.
    GreatEnemy,
}

/// How two sign lords stand to each other, by the natural friendships of
/// VI.27–28.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_matching::{MaitriRelation, maitri_relation};
///
/// assert_eq!(maitri_relation(Graha::Mars, Graha::Mars), MaitriRelation::OneLord);
/// assert_eq!(maitri_relation(Graha::Sun, Graha::Moon), MaitriRelation::MutualFriends);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaitriRelation {
    /// One lord rules both signs: 5.
    OneLord,
    /// Each the other's friend: 5.
    MutualFriends,
    /// A friend one way, neutral the other: 4.
    FriendNeutral,
    /// Neutral both ways: 3.
    MutualNeutral,
    /// A friend one way, an enemy the other: 1.
    FriendEnemy,
    /// Neutral one way, an enemy the other: ½.
    NeutralEnemy,
    /// Each the other's enemy: 0.
    MutualEnemies,
}

/// A bad Bhakoot, by how far the signs stand apart (VI.31).
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{BhakootDosha, KootaReading, KootaRules, Native, ashta_koota};
///
/// // Moons in Ashvini (Aries) and Anuradha (Scorpio): the 8th apart.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(215.0)?);
/// let koota = ashta_koota(bride, groom, KootaRules::default());
/// let Some(KootaReading::Bhakoot { apart, dosha, exceptions, lifted }) = koota.row(Koota::Bhakoot).map(|row| row.reading) else {
///     unreachable!("the eight always read Bhakoot")
/// };
/// assert_eq!((apart, dosha), (8, Some(BhakootDosha::SixEight)));
/// // A bad Bhakoot earns nothing, lifted or not.
/// assert_eq!(koota.row(Koota::Bhakoot).map(|row| row.points), Some(0.0));
/// # let _ = (exceptions, lifted);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BhakootDosha {
    /// Sixth and eighth: death.
    SixEight,
    /// Fifth and ninth: loss of children.
    FiveNine,
    /// Second and twelfth: poverty.
    TwoTwelve,
}

/// The five exceptions of VI.32–33 that lift a bad Bhakoot, each as a
/// clause that holds or not.
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{BhakootExceptions, KootaReading, KootaRules, Native, ashta_koota};
///
/// // Moons in Ashvini (Aries) and Anuradha (Scorpio): the 8th apart.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(215.0)?);
/// let koota = ashta_koota(bride, groom, KootaRules::default());
/// let Some(KootaReading::Bhakoot { apart, dosha, exceptions, lifted }) = koota.row(Koota::Bhakoot).map(|row| row.reading) else {
///     unreachable!("the eight always read Bhakoot")
/// };
/// // Mars rules both; Scorpio is vashya to Aries; the taras are 8 and 3.
/// assert_eq!(exceptions, BhakootExceptions { one_lord: true, vashya: true, ..BhakootExceptions::default() });
/// # let _ = (apart, dosha, lifted);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is a clause the verse names, reported whether or not it holds"
)]
pub struct BhakootExceptions {
    /// One lord rules both signs.
    pub one_lord: bool,
    /// The sign lords are each other's friends.
    pub lords_friends: bool,
    /// The navamsha lords are one or each other's friends.
    pub navamsha_lords_friends: bool,
    /// The tara is pure both ways.
    pub tara_pure: bool,
    /// One sign is vashya to the other.
    pub vashya: bool,
}

/// What a koota read, by koota.
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{KootaReading, KootaRules, Native, ashta_koota};
///
/// // Moons in Ashvini (Aries) and Anuradha (Scorpio): the 8th apart.
/// let (bride, groom) = (Native::of_moon(5.0)?, Native::of_moon(215.0)?);
/// let koota = ashta_koota(bride, groom, KootaRules::default());
/// let Some(KootaReading::Tara { bride_to_groom, groom_to_bride }) = koota.row(Koota::Tara).map(|row| row.reading) else {
///     unreachable!("the eight always read Tara")
/// };
/// // Counted both ways: the 8th is pure, the 3rd is not, so half the points.
/// assert_eq!((bride_to_groom, groom_to_bride), (8, 3));
/// assert_eq!(koota.row(Koota::Tara).map(|row| row.points), Some(1.5));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(tag = "koota", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KootaReading {
    /// The varnas of the two signs (VI.22).
    Varna {
        /// The bride's.
        bride: Varna,
        /// The groom's.
        groom: Varna,
    },
    /// How the two signs stand in Vashya (VI.23).
    Vashya {
        /// The relation.
        relation: VashyaRelation,
    },
    /// The taras each way, 1 to 9 (VI.24).
    Tara {
        /// Counted from the bride's nakshatra to the groom's.
        bride_to_groom: u8,
        /// Counted from the groom's to the bride's.
        groom_to_bride: u8,
    },
    /// The two yonis (VI.25–26).
    Yoni {
        /// The bride's.
        bride: Yoni,
        /// The groom's.
        groom: Yoni,
        /// How they stand.
        relation: YoniRelation,
    },
    /// The two sign lords (VI.27–28).
    GrahaMaitri {
        /// The bride's sign lord.
        bride: Graha,
        /// The groom's.
        groom: Graha,
        /// How they stand.
        relation: MaitriRelation,
        /// Whether a good Bhakoot lifts an enmity between them (VI.33);
        /// `false` with none.
        lifted: bool,
    },
    /// The two ganas (VI.29–30).
    Gana {
        /// The bride's.
        bride: Gana,
        /// The groom's.
        groom: Gana,
        /// Whether they are a bad pair, a Rakshasa with another gana.
        dosha: bool,
        /// Whether the dosha is lifted: the sign lords or the navamsha
        /// lords friends (VI.33), or one sign or one star between them
        /// (VI.36); `false` with no dosha.
        lifted: bool,
    },
    /// How far the groom's sign stands from the bride's (VI.31–33).
    Bhakoot {
        /// The groom's sign counted from the bride's, 1 to 12.
        apart: u8,
        /// The dosha, when the signs stand badly.
        dosha: Option<BhakootDosha>,
        /// The exceptions, each a clause.
        exceptions: BhakootExceptions,
        /// Whether the dosha is lifted under the rules asked for; `false`
        /// with no dosha.
        lifted: bool,
    },
    /// The two nadis (VI.34).
    Nadi {
        /// The bride's.
        bride: Nadi,
        /// The groom's.
        groom: Nadi,
        /// Whether the shared nadi is a dosha under the rules asked for.
        dosha: bool,
        /// Whether the dosha is lifted by one sign with two stars, one star
        /// across two signs, or one star in two padas (VI.36); `false` with
        /// no dosha.
        lifted: bool,
    },
}

impl KootaReading {
    /// The koota this is the reading of.
    ///
    /// ```
    /// use teistro_core::catalogue::Koota;
    /// use teistro_matching::KootaReading;
    ///
    /// let tara = KootaReading::Tara { bride_to_groom: 2, groom_to_bride: 9 };
    /// assert_eq!(tara.koota(), Koota::Tara);
    /// ```
    #[must_use]
    pub const fn koota(&self) -> Koota {
        match self {
            KootaReading::Varna { .. } => Koota::Varna,
            KootaReading::Vashya { .. } => Koota::Vashya,
            KootaReading::Tara { .. } => Koota::Tara,
            KootaReading::Yoni { .. } => Koota::Yoni,
            KootaReading::GrahaMaitri { .. } => Koota::GrahaMaitri,
            KootaReading::Gana { .. } => Koota::Gana,
            KootaReading::Bhakoot { .. } => Koota::Bhakoot,
            KootaReading::Nadi { .. } => Koota::Nadi,
        }
    }
}

/// One koota's points and what it read.
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{KootaRules, Native, ashta_koota};
///
/// // Two Moons at 45°: Rohini's second pada, in Taurus.
/// let moon = Native::of_moon(45.0)?;
/// let koota = ashta_koota(moon, moon, KootaRules::default());
/// let nadi = koota.row(Koota::Nadi).copied();
/// assert_eq!(nadi.map(|row| (row.points, row.max_points)), Some((0.0, 8.0)));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KootaRow {
    /// The points, a multiple of a half.
    pub points: f64,
    /// The most it gives.
    pub max_points: f64,
    /// What it read.
    pub reading: KootaReading,
}

/// The Ashta Koota of a bride and a groom.
///
/// ```
/// use teistro_matching::{KootaRules, Native, ashta_koota};
///
/// // Two Moons at 45°: Rohini's second pada, in Taurus.
/// let moon = Native::of_moon(45.0)?;
/// let koota = ashta_koota(moon, moon, KootaRules::default());
/// assert_eq!(koota.kootas.len(), 8);
/// assert_eq!(koota.total, koota.kootas.iter().map(|row| row.points).sum::<f64>());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AshtaKoota {
    /// The eight, in the verse's order.
    pub kootas: Vec<KootaRow>,
    /// Their points, out of 36.
    pub total: f64,
}

impl AshtaKoota {
    /// One koota's row.
    ///
    /// ```
    /// use teistro_core::catalogue::Koota;
    /// use teistro_matching::{KootaRules, Native, ashta_koota};
    ///
    /// // Two Moons at 45°: Rohini's second pada, in Taurus.
    /// let moon = Native::of_moon(45.0)?;
    /// let koota = ashta_koota(moon, moon, KootaRules::default());
    /// assert_eq!(koota.row(Koota::Gana).map(|row| row.points), Some(6.0));
    /// // Rajju is one of the ten considerations, not the eight.
    /// assert!(koota.row(Koota::Rajju).is_none());
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn row(&self, koota: Koota) -> Option<&KootaRow> {
        let at = ASHTA_KOOTA.iter().position(|one| *one == koota)?;
        self.kootas.get(at)
    }
}

/// The Ashta Koota of a bride and a groom: each koota's points and what it
/// read, and their total out of 36 (`03-design/matching.md`). Never a
/// verdict: the doshas and their exceptions are clauses the answer carries.
///
/// ```
/// use teistro_core::catalogue::Koota;
/// use teistro_matching::{KootaReading, KootaRules, Native, ashta_koota};
///
/// // Moons in Ashvini (Aries) and Anuradha (Scorpio): a 6/8 lifted, as
/// // both signs are Mars's.
/// let bride = Native::of_moon(5.0)?;
/// let groom = Native::of_moon(215.0)?;
/// let koota = ashta_koota(bride, groom, KootaRules::default());
/// let Some(KootaReading::Bhakoot { apart, dosha, lifted, .. }) =
///     koota.row(Koota::Bhakoot).map(|row| row.reading)
/// else { unreachable!() };
/// assert_eq!((apart, dosha.is_some(), lifted), (8, true, true));
/// assert!(koota.total <= 36.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn ashta_koota(bride: Native, groom: Native, rules: KootaRules) -> AshtaKoota {
    let tara = tara(bride, groom);
    let vashya = vashya_relation(bride.rashi, groom.rashi);
    let nadi = nadi(bride, groom, rules);
    let bhakoot = bhakoot(bride, groom, rules, (tara.1, vashya, nadi.1));
    let good_bhakoot = matches!(bhakoot.reading, KootaReading::Bhakoot { dosha: None, .. });
    let kootas = vec![
        varna(bride, groom, rules),
        KootaRow {
            points: match rules.vashya {
                VashyaPoints::Verse => match vashya {
                    VashyaRelation::Mutual => 2.0,
                    VashyaRelation::OneWay => 1.0,
                    VashyaRelation::Food => 0.5,
                    VashyaRelation::Neither => 0.0,
                },
                VashyaPoints::PrintedInNepal => printed_vashya(bride.rashi, groom.rashi),
            },
            max_points: 2.0,
            reading: KootaReading::Vashya { relation: vashya },
        },
        tara.0,
        yoni(bride, groom),
        maitri(bride, groom, good_bhakoot),
        gana(bride, groom, rules),
        bhakoot,
        nadi.0,
    ];
    let total = kootas.iter().map(|row| row.points).sum();
    AshtaKoota { kootas, total }
}

/// A sign's varna (VI.22).
///
/// ```
/// use teistro_core::catalogue::{Rashi, Varna};
/// use teistro_matching::sign_varna;
///
/// assert_eq!(sign_varna(Rashi::Cancer), Varna::Brahmin);
/// assert_eq!(sign_varna(Rashi::Aries), Varna::Kshatriya);
/// assert_eq!(sign_varna(Rashi::Taurus), Varna::Vaishya);
/// assert_eq!(sign_varna(Rashi::Gemini), Varna::Shudra);
/// ```
#[must_use]
pub fn sign_varna(sign: Rashi) -> Varna {
    if matches!(sign, Rashi::Cancer | Rashi::Scorpio | Rashi::Pisces) {
        Varna::Brahmin
    } else if matches!(sign, Rashi::Aries | Rashi::Leo | Rashi::Sagittarius) {
        Varna::Kshatriya
    } else if matches!(sign, Rashi::Taurus | Rashi::Virgo | Rashi::Capricorn) {
        Varna::Vaishya
    } else {
        Varna::Shudra
    }
}

fn varna(bride: Native, groom: Native, rules: KootaRules) -> KootaRow {
    let (b, g) = (sign_varna(bride.rashi), sign_varna(groom.rashi));
    // The catalogue lists the varnas highest first.
    let points = match g.id().cmp(&b.id()) {
        std::cmp::Ordering::Less => 1.0,
        std::cmp::Ordering::Equal => match rules.equal_varna {
            EqualVarna::Whole => 1.0,
            EqualVarna::Half => 0.5,
        },
        std::cmp::Ordering::Greater => 0.0,
    };
    KootaRow {
        points,
        max_points: 1.0,
        reading: KootaReading::Varna { bride: b, groom: g },
    }
}

/// The human signs a guna grid printed in Nepal scores (C762).
const fn printed_human(sign: Rashi) -> bool {
    matches!(
        sign,
        Rashi::Gemini | Rashi::Virgo | Rashi::Libra | Rashi::Sagittarius | Rashi::Aquarius
    )
}

/// The print's vashya points: *Muhurta Martanda*'s ½ for a human sign
/// with an animal, 1 for Scorpio, its enemy, and 2 for every other pair
/// (C762).
fn printed_vashya(bride: Rashi, groom: Rashi) -> f64 {
    match (printed_human(bride), printed_human(groom)) {
        (true, true) | (false, false) => 2.0,
        _ if bride == Rashi::Scorpio || groom == Rashi::Scorpio => 1.0,
        _ => 0.5,
    }
}

/// The human signs of VI.23.
const fn human(sign: Rashi) -> bool {
    matches!(sign, Rashi::Gemini | Rashi::Virgo | Rashi::Libra)
}

/// The water signs the commentary names under VI.23.
const fn water(sign: Rashi) -> bool {
    matches!(
        sign,
        Rashi::Cancer | Rashi::Capricorn | Rashi::Aquarius | Rashi::Pisces
    )
}

/// Whether `sign` is vashya to `to`: the verse where it decides,
/// Kalaprakasika's table (XIII, p. 75) for the rest (C260).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::is_vashya;
///
/// // Every sign but Leo is vashya to a human one.
/// assert!(is_vashya(Rashi::Aries, Rashi::Gemini));
/// assert!(!is_vashya(Rashi::Leo, Rashi::Gemini));
/// // Directed: Scorpio to Aries, not Aries to Scorpio.
/// assert!(is_vashya(Rashi::Scorpio, Rashi::Aries) && !is_vashya(Rashi::Aries, Rashi::Scorpio));
/// ```
#[must_use]
pub fn is_vashya(sign: Rashi, to: Rashi) -> bool {
    if sign == to {
        return true;
    }
    if human(to) {
        return sign != Rashi::Leo;
    }
    if to == Rashi::Leo {
        return sign != Rashi::Scorpio;
    }
    let concordant: &[Rashi] = match to {
        Rashi::Aries => &[Rashi::Leo, Rashi::Scorpio],
        Rashi::Taurus => &[Rashi::Cancer, Rashi::Leo],
        Rashi::Cancer => &[Rashi::Scorpio, Rashi::Sagittarius],
        Rashi::Scorpio => &[Rashi::Virgo, Rashi::Cancer],
        Rashi::Sagittarius => &[Rashi::Pisces],
        Rashi::Capricorn => &[Rashi::Aquarius, Rashi::Aries],
        Rashi::Aquarius => &[Rashi::Aries],
        Rashi::Pisces => &[Rashi::Capricorn],
        // The human signs and Leo were answered above.
        _ => &[],
    };
    concordant.contains(&sign)
}

/// How two signs stand in Vashya (C260).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::{VashyaRelation, vashya_relation};
///
/// // One sign is vashya to itself both ways.
/// assert_eq!(vashya_relation(Rashi::Leo, Rashi::Leo), VashyaRelation::Mutual);
/// ```
#[must_use]
pub fn vashya_relation(bride: Rashi, groom: Rashi) -> VashyaRelation {
    let (to_groom, to_bride) = (is_vashya(bride, groom), is_vashya(groom, bride));
    let food = (water(bride) && human(groom)) || (water(groom) && human(bride));
    if food && (to_groom || to_bride) {
        VashyaRelation::Food
    } else if to_groom && to_bride {
        VashyaRelation::Mutual
    } else if to_groom || to_bride {
        VashyaRelation::OneWay
    } else {
        VashyaRelation::Neither
    }
}

/// The tara of counting from one nakshatra to another, both inclusive,
/// reduced to 1 to 9 (VI.24).
fn tara_of(from: Native, to: Native) -> u8 {
    let count = (to.star() + 27 - from.star()) % 27 + 1;
    u8::try_from((count - 1) % 9 + 1).unwrap_or(9)
}

/// The 3rd, 5th and 7th taras are bad.
const fn tara_pure(tara: u8) -> bool {
    !matches!(tara, 3 | 5 | 7)
}

/// The Tara row, and whether it is pure both ways.
fn tara(bride: Native, groom: Native) -> (KootaRow, bool) {
    let (there, back) = (tara_of(bride, groom), tara_of(groom, bride));
    let pure = u8::from(tara_pure(there)) + u8::from(tara_pure(back));
    let row = KootaRow {
        points: 1.5 * f64::from(pure),
        max_points: 3.0,
        reading: KootaReading::Tara {
            bride_to_groom: there,
            groom_to_bride: back,
        },
    };
    (row, pure == 2)
}

/// The seven great enmities of VI.25–26.
const GREAT_ENEMIES: [(Yoni, Yoni); 7] = [
    (Yoni::Horse, Yoni::Buffalo),
    (Yoni::Elephant, Yoni::Lion),
    (Yoni::Goat, Yoni::Monkey),
    (Yoni::Serpent, Yoni::Mongoose),
    (Yoni::Deer, Yoni::Dog),
    (Yoni::Cat, Yoni::Rat),
    (Yoni::Tiger, Yoni::Cow),
];

/// How two yonis stand (C261).
///
/// ```
/// use teistro_core::catalogue::Yoni;
/// use teistro_matching::{YoniRelation, yoni_relation};
///
/// // The horse and the buffalo, either way round.
/// assert_eq!(yoni_relation(Yoni::Horse, Yoni::Buffalo), YoniRelation::GreatEnemy);
/// assert_eq!(yoni_relation(Yoni::Buffalo, Yoni::Horse), YoniRelation::GreatEnemy);
/// ```
#[must_use]
pub fn yoni_relation(a: Yoni, b: Yoni) -> YoniRelation {
    if a == b {
        YoniRelation::Same
    } else if GREAT_ENEMIES.contains(&(a, b)) || GREAT_ENEMIES.contains(&(b, a)) {
        YoniRelation::GreatEnemy
    } else {
        YoniRelation::Neutral
    }
}

fn yoni(bride: Native, groom: Native) -> KootaRow {
    let (b, g) = (
        bride.nakshatra.attributes().yoni,
        groom.nakshatra.attributes().yoni,
    );
    let relation = yoni_relation(b, g);
    KootaRow {
        points: match relation {
            YoniRelation::Same => 4.0,
            YoniRelation::Neutral => 2.0,
            YoniRelation::GreatEnemy => 0.0,
        },
        max_points: 4.0,
        reading: KootaReading::Yoni {
            bride: b,
            groom: g,
            relation,
        },
    }
}

/// How one graha regards another by nature: a friend, neutral or an enemy,
/// as `+1`, `0` and `-1`.
fn regards(graha: Graha, other: Graha) -> i8 {
    let attributes = graha.attributes();
    if attributes.friends.contains(&other) {
        1
    } else if attributes.enemies.contains(&other) {
        -1
    } else {
        0
    }
}

/// How two sign lords stand by the natural friendships (VI.27–28).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_matching::{MaitriRelation, maitri_relation};
///
/// assert_eq!(maitri_relation(Graha::Sun, Graha::Saturn), MaitriRelation::MutualEnemies);
/// // The relation is of the pair, read the same either way round.
/// assert_eq!(maitri_relation(Graha::Sun, Graha::Mercury), maitri_relation(Graha::Mercury, Graha::Sun));
/// ```
#[must_use]
pub fn maitri_relation(a: Graha, b: Graha) -> MaitriRelation {
    if a == b {
        return MaitriRelation::OneLord;
    }
    match (regards(a, b), regards(b, a)) {
        (1, 1) => MaitriRelation::MutualFriends,
        (1, 0) | (0, 1) => MaitriRelation::FriendNeutral,
        (0, 0) => MaitriRelation::MutualNeutral,
        (1, -1) | (-1, 1) => MaitriRelation::FriendEnemy,
        (0, -1) | (-1, 0) => MaitriRelation::NeutralEnemy,
        _ => MaitriRelation::MutualEnemies,
    }
}

/// Whether two lords are friends as the exceptions read it: one lord, or
/// each the other's friend.
fn befriended(a: Graha, b: Graha) -> bool {
    matches!(
        maitri_relation(a, b),
        MaitriRelation::OneLord | MaitriRelation::MutualFriends
    )
}

/// One Moon's place shared as VI.36 lifts the nadi's and the gana's dosha:
/// one sign with two stars, one star across two signs, or one star and one
/// sign in two padas.
fn shared_apart(bride: Native, groom: Native) -> bool {
    match (
        bride.rashi == groom.rashi,
        bride.nakshatra == groom.nakshatra,
    ) {
        (true, false) | (false, true) => true,
        (true, true) => bride.pada != groom.pada,
        (false, false) => false,
    }
}

fn maitri(bride: Native, groom: Native, good_bhakoot: bool) -> KootaRow {
    let (b, g) = (bride.rashi.attributes().lord, groom.rashi.attributes().lord);
    let relation = maitri_relation(b, g);
    let enmity = matches!(
        relation,
        MaitriRelation::FriendEnemy | MaitriRelation::NeutralEnemy | MaitriRelation::MutualEnemies
    );
    KootaRow {
        points: match relation {
            MaitriRelation::OneLord | MaitriRelation::MutualFriends => 5.0,
            MaitriRelation::FriendNeutral => 4.0,
            MaitriRelation::MutualNeutral => 3.0,
            MaitriRelation::FriendEnemy => 1.0,
            MaitriRelation::NeutralEnemy => 0.5,
            MaitriRelation::MutualEnemies => 0.0,
        },
        max_points: 5.0,
        reading: KootaReading::GrahaMaitri {
            bride: b,
            groom: g,
            relation,
            lifted: enmity && good_bhakoot,
        },
    }
}

fn gana(bride: Native, groom: Native, rules: KootaRules) -> KootaRow {
    let (b, g) = (
        bride.nakshatra.attributes().gana,
        groom.nakshatra.attributes().gana,
    );
    let points = if b == g {
        6.0
    } else {
        match rules.gana {
            GanaPoints::DaivajnaManohara => match (b, g) {
                (Gana::Manushya, Gana::Deva) => 5.0,
                (Gana::Deva, Gana::Manushya) => match rules.deva_bride {
                    DevaBride::Four => 4.0,
                    DevaBride::Three => 3.0,
                },
                (Gana::Deva, Gana::Rakshasa) => 2.0,
                (Gana::Manushya, Gana::Rakshasa) => 1.0,
                _ => 0.0,
            },
            GanaPoints::MuhurtaMartanda => match (b, g) {
                (Gana::Manushya, Gana::Deva) => 6.0,
                (Gana::Deva, Gana::Manushya) => 5.0,
                (Gana::Deva, Gana::Rakshasa) => 1.0,
                _ => 0.0,
            },
        }
    };
    // The commentary's bad ganas: a Rakshasa beside either other.
    let dosha = b != g && (b == Gana::Rakshasa || g == Gana::Rakshasa);
    let lifted = dosha
        && (befriended(bride.rashi.attributes().lord, groom.rashi.attributes().lord)
            || befriended(
                bride.navamsha.attributes().lord,
                groom.navamsha.attributes().lord,
            )
            || shared_apart(bride, groom));
    KootaRow {
        points,
        max_points: 6.0,
        reading: KootaReading::Gana {
            bride: b,
            groom: g,
            dosha,
            lifted,
        },
    }
}

fn bhakoot(
    bride: Native,
    groom: Native,
    rules: KootaRules,
    (tara_pure, vashya, nadi_pure): (bool, VashyaRelation, bool),
) -> KootaRow {
    let apart = u8::try_from((groom.sign() + 12 - bride.sign()) % 12 + 1).unwrap_or(1);
    let dosha = match apart {
        6 | 8 => Some(BhakootDosha::SixEight),
        5 | 9 => Some(BhakootDosha::FiveNine),
        2 | 12 => Some(BhakootDosha::TwoTwelve),
        _ => None,
    };
    let lords = maitri_relation(bride.rashi.attributes().lord, groom.rashi.attributes().lord);
    let navamsha_lords = maitri_relation(
        bride.navamsha.attributes().lord,
        groom.navamsha.attributes().lord,
    );
    let exceptions = BhakootExceptions {
        one_lord: lords == MaitriRelation::OneLord,
        lords_friends: lords == MaitriRelation::MutualFriends,
        navamsha_lords_friends: matches!(
            navamsha_lords,
            MaitriRelation::OneLord | MaitriRelation::MutualFriends
        ),
        tara_pure,
        vashya: vashya != VashyaRelation::Neither,
    };
    let lifted = dosha.is_some_and(|dosha| {
        nadi_pure
            && match rules.bhakoot_lift {
                BhakootLift::AnyOne => {
                    exceptions.one_lord
                        || exceptions.lords_friends
                        || exceptions.navamsha_lords_friends
                        || exceptions.tara_pure
                        || exceptions.vashya
                }
                BhakootLift::Garga => {
                    let held = [
                        exceptions.one_lord || exceptions.lords_friends,
                        exceptions.tara_pure,
                        exceptions.vashya,
                    ]
                    .iter()
                    .filter(|held| **held)
                    .count();
                    held >= if dosha == BhakootDosha::SixEight {
                        3
                    } else {
                        2
                    }
                }
            }
    });
    KootaRow {
        points: if dosha.is_none() { 7.0 } else { 0.0 },
        max_points: 7.0,
        reading: KootaReading::Bhakoot {
            apart,
            dosha,
            exceptions,
            lifted,
        },
    }
}

/// The Nadi row, and whether the nadi is pure: no dosha, or one lifted.
fn nadi(bride: Native, groom: Native, rules: KootaRules) -> (KootaRow, bool) {
    let (b, g) = (
        bride.nakshatra.attributes().nadi,
        groom.nakshatra.attributes().nadi,
    );
    let dosha = b == g
        && match rules.nadi_dosha {
            NadiDosha::Any => true,
            NadiDosha::MiddleOnly => b == Nadi::Madhya,
        };
    let lifted = dosha && shared_apart(bride, groom);
    let row = KootaRow {
        points: if b == g { 0.0 } else { 8.0 },
        max_points: 8.0,
        reading: KootaReading::Nadi {
            bride: b,
            groom: g,
            dosha,
            lifted,
        },
    };
    // VI.36 says there is no nadi dosha, so a lifted one is pure for the
    // Bhakoot's exceptions too (C284).
    (row, !dosha || lifted)
}
