//! The Kuja dosha across two charts: Mars in the houses *Manasagari*'s
//! jāyābhāva v. 4 names, for each native, and the lifts BPHS ch. 80 and
//! the almanacs printed in Nepal give it (`03-design/matching.md`, C285
//! to C288, C755).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Rashi;
use teistro_core::error::Error;

/// The signs the Kuja dosha reads in one chart, all in the chart's own
/// zodiac: Mars's, and those of the three places its house is counted
/// from.
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::KujaNative;
///
/// let native = KujaNative::of_longitudes(5.0, 100.0, 200.0, 190.0)?;
/// assert_eq!((native.lagna, native.mars), (Rashi::Aries, Rashi::Libra));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KujaNative {
    /// The lagna's sign.
    pub lagna: Rashi,
    /// The Moon's sign.
    pub moon: Rashi,
    /// Venus's sign.
    pub venus: Rashi,
    /// Mars's sign.
    pub mars: Rashi,
    /// The other grahas' signs, which the lifts read; `None` where the
    /// native was given without them, and then no lift is read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sky: Option<KujaSky>,
}

impl KujaNative {
    /// The native whose lagna, Moon, Venus and Mars stand at these
    /// longitudes, in degrees of the chart's own zodiac.
    ///
    /// ```
    /// use teistro_matching::KujaNative;
    ///
    /// let refused = KujaNative::of_longitudes(5.0, 40.0, f64::NAN, 190.0).unwrap_err();
    /// assert_eq!(refused.field(), Some("venus"));
    /// ```
    ///
    /// # Errors
    ///
    /// A longitude that is not a finite number, named by what it places
    /// (`lagna`, `moon`, `venus` or `mars`).
    pub fn of_longitudes(
        lagna_deg: f64,
        moon_deg: f64,
        venus_deg: f64,
        mars_deg: f64,
    ) -> Result<KujaNative, Error> {
        Ok(KujaNative {
            lagna: sign("lagna", lagna_deg)?,
            moon: sign("moon", moon_deg)?,
            venus: sign("venus", venus_deg)?,
            mars: sign("mars", mars_deg)?,
            sky: None,
        })
    }

    /// The same native with the other grahas' signs, so its lifts are read.
    ///
    /// ```
    /// use teistro_matching::{KujaNative, KujaSky};
    ///
    /// let sky = KujaSky::of_longitudes(10.0, 40.0, 20.0, 250.0, 300.0, 150.0)?;
    /// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?.with_sky(sky);
    /// assert_eq!(native.sky, Some(sky));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn with_sky(self, sky: KujaSky) -> KujaNative {
        KujaNative {
            sky: Some(sky),
            ..self
        }
    }

    /// The sign a house is counted from.
    ///
    /// ```
    /// use teistro_matching::{KujaNative, KujaReference};
    ///
    /// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
    /// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
    /// use teistro_core::catalogue::Rashi;
    ///
    /// assert_eq!(native.sign_of(KujaReference::Moon), Rashi::Taurus);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn sign_of(self, reference: KujaReference) -> Rashi {
        match reference {
            KujaReference::Lagna => self.lagna,
            KujaReference::Moon => self.moon,
            KujaReference::Venus => self.venus,
        }
    }

    /// Mars's house from a reference, 1 to 12, counted by sign (C287).
    ///
    /// ```
    /// use teistro_matching::{KujaNative, KujaReference};
    ///
    /// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
    /// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
    /// // Libra is the 7th from Aries, the 6th from Taurus, the 5th from Gemini.
    /// let houses = [KujaReference::Lagna, KujaReference::Moon, KujaReference::Venus].map(|from| native.mars_house(from));
    /// assert_eq!(houses, [7, 6, 5]);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn mars_house(self, reference: KujaReference) -> u8 {
        house(self.sign_of(reference), self.mars)
    }
}

/// The house a sign is from another, 1 to 12, counted by sign.
fn house(from: Rashi, to: Rashi) -> u8 {
    u8::try_from((to.id() + 12 - from.id()) % 12 + 1).unwrap_or(1)
}

/// Fails a longitude that is not a finite number of degrees, naming the
/// field it places.
fn sign(field: &str, longitude_deg: f64) -> Result<Rashi, Error> {
    if longitude_deg.is_finite() {
        Ok(Rashi::of_longitude(longitude_deg.rem_euclid(360.0)))
    } else {
        Err(Error::invalid_arg(format!(
            "the {field}'s longitude is a finite number of degrees, not {longitude_deg}"
        ))
        .with_field(field))
    }
}

/// The signs of the grahas a lift reads beside the four the dosha does,
/// and whether the Moon waxes, which makes it a benefic (C755).
///
/// ```
/// use teistro_core::catalogue::Rashi;
/// use teistro_matching::KujaSky;
///
/// // The Sun at 10°, the Moon 30° ahead of it: a waxing Moon.
/// let sky = KujaSky::of_longitudes(10.0, 40.0, 20.0, 250.0, 300.0, 150.0)?;
/// assert_eq!((sky.jupiter, sky.rahu), (Rashi::Sagittarius, Rashi::Virgo));
/// assert!(sky.moon_waxing);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KujaSky {
    /// The Sun's sign.
    pub sun: Rashi,
    /// Mercury's sign.
    pub mercury: Rashi,
    /// Jupiter's sign.
    pub jupiter: Rashi,
    /// Saturn's sign.
    pub saturn: Rashi,
    /// Rahu's sign; Ketu stands in the seventh from it.
    pub rahu: Rashi,
    /// Whether the Moon is less than 180° ahead of the Sun.
    pub moon_waxing: bool,
}

impl KujaSky {
    /// The grahas at these longitudes, in degrees of the chart's own zodiac.
    ///
    /// ```
    /// use teistro_matching::KujaSky;
    ///
    /// let refused = KujaSky::of_longitudes(10.0, 40.0, 20.0, f64::NAN, 300.0, 150.0).unwrap_err();
    /// assert_eq!(refused.field(), Some("jupiter"));
    /// ```
    ///
    /// # Errors
    ///
    /// A longitude that is not a finite number, named by what it places
    /// (`sun`, `moon`, `mercury`, `jupiter`, `saturn` or `rahu`).
    pub fn of_longitudes(
        sun_deg: f64,
        moon_deg: f64,
        mercury_deg: f64,
        jupiter_deg: f64,
        saturn_deg: f64,
        rahu_deg: f64,
    ) -> Result<KujaSky, Error> {
        let sun = sign("sun", sun_deg)?;
        sign("moon", moon_deg)?;
        Ok(KujaSky {
            sun,
            mercury: sign("mercury", mercury_deg)?,
            jupiter: sign("jupiter", jupiter_deg)?,
            saturn: sign("saturn", saturn_deg)?,
            rahu: sign("rahu", rahu_deg)?,
            moon_waxing: (moon_deg - sun_deg).rem_euclid(360.0) < 180.0,
        })
    }
}

/// Which houses of Mars make the dosha (C285).
///
/// ```
/// use teistro_matching::{KujaHouses, KujaNative, KujaRules, kuja_side};
///
/// // Mars in the 2nd from an Aries lagna.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 40.0)?;
/// assert!(!kuja_side(native, KujaRules::default()).dosha);
/// let modern = KujaRules { houses: KujaHouses::WithSecond, ..KujaRules::default() };
/// assert!(kuja_side(native, modern).dosha);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KujaHouses {
    /// The 1st, 4th, 7th, 8th and 12th (*Manasagari*, jāyābhāva v. 4).
    #[default]
    Manasagari,
    /// The same with the 2nd, as the baseline engine and modern practice
    /// have it (rank 3).
    WithSecond,
}

impl KujaHouses {
    /// The houses, in order.
    ///
    /// ```
    /// use teistro_matching::KujaHouses;
    ///
    /// assert_eq!(KujaHouses::default().houses(), &[1, 4, 7, 8, 12]);
    /// assert!(KujaHouses::WithSecond.houses().contains(&2));
    /// ```
    #[must_use]
    pub fn houses(self) -> &'static [u8] {
        match self {
            KujaHouses::Manasagari => &[1, 4, 7, 8, 12],
            KujaHouses::WithSecond => &[1, 2, 4, 7, 8, 12],
        }
    }
}

/// From where Mars's house makes the dosha (C286).
///
/// ```
/// use teistro_matching::{KujaFrom, KujaNative, KujaRules, kuja_side};
///
/// // Mars in Cancer: the 4th from an Aries lagna, and the 12th from a Leo Moon.
/// let native = KujaNative::of_longitudes(5.0, 130.0, 130.0, 100.0)?;
/// let side = kuja_side(native, KujaRules::default());
/// assert_eq!(side.readings.map(|reading| reading.house), [4, 12, 12]);
/// // Mars in the 6th from Aquarius and with the Moon: only the Moon makes it.
/// let late = KujaNative::of_longitudes(305.0, 100.0, 100.0, 100.0)?;
/// let wider = KujaRules { from: KujaFrom::LagnaMoonVenus, ..KujaRules::default() };
/// assert!(!kuja_side(late, KujaRules::default()).dosha && kuja_side(late, wider).dosha);
/// # let _ = side;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KujaFrom {
    /// The lagna alone, as the verse counts.
    #[default]
    Lagna,
    /// The lagna, the Moon or Venus, as modern practice counts (rank 3).
    LagnaMoonVenus,
}

impl KujaFrom {
    /// Whether the dosha is read from this reference.
    ///
    /// ```
    /// use teistro_matching::{KujaFrom, KujaReference};
    ///
    /// assert!(KujaFrom::Lagna.counts(KujaReference::Lagna));
    /// assert!(!KujaFrom::Lagna.counts(KujaReference::Venus));
    /// assert!(KujaFrom::LagnaMoonVenus.counts(KujaReference::Venus));
    /// ```
    #[must_use]
    pub fn counts(self, reference: KujaReference) -> bool {
        match self {
            KujaFrom::Lagna => reference == KujaReference::Lagna,
            KujaFrom::LagnaMoonVenus => true,
        }
    }
}

/// A place Mars's house is counted from.
///
/// ```
/// use teistro_matching::{KujaNative, KujaReference};
///
/// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// assert_eq!(native.mars_house(KujaReference::Lagna), 7);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KujaReference {
    /// The lagna.
    Lagna,
    /// The Moon.
    Moon,
    /// Venus.
    Venus,
}

/// The three references, in the order every answer reports them.
///
/// ```
/// use teistro_matching::{KujaNative, KujaReference};
///
/// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// use teistro_matching::{KUJA_REFERENCES, KujaRules, kuja_side};
///
/// let side = kuja_side(native, KujaRules::default());
/// assert_eq!(side.readings.map(|reading| reading.from), KUJA_REFERENCES);
/// # let _ = KujaReference::Lagna;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub const KUJA_REFERENCES: [KujaReference; 3] = [
    KujaReference::Lagna,
    KujaReference::Moon,
    KujaReference::Venus,
];

/// The lifts of the Kuja dosha, each a clause: as a rule's choice, the
/// lifts it counts; as a side's answer, the lifts that hold (C288, C755).
///
/// BPHS ch. 80 v. 47 makes the dosha Mars "without a benefic's aspect or
/// conjunction", and vv. 48–49 lift it when both charts carry it: the
/// two at rank 1 ([`KujaLifts::PARASHARA`]). The almanacs printed in
/// Nepal add five from verses they do not attribute: Saturn in the 1st,
/// 4th, 7th, 9th or 12th; the Moon and Venus in the 2nd; Jupiter's aspect
/// on Mars; Rahu in a kendra; Mars with Rahu, all from the lagna
/// ([`KujaLifts::PRINTED_IN_NEPAL`], rank 4).
///
/// ```
/// use teistro_matching::KujaLifts;
///
/// assert_eq!(KujaLifts::default(), KujaLifts::NONE);
/// assert_eq!(KujaLifts::PARASHARA.count(), 2);
/// assert_eq!(KujaLifts::PRINTED_IN_NEPAL.count(), 5);
/// // What a rule counts and a side holds meet in `and`.
/// let held = KujaLifts { jupiter_aspect: true, ..KujaLifts::NONE };
/// assert!(held.and(KujaLifts::PRINTED_IN_NEPAL).any() && !held.and(KujaLifts::PARASHARA).any());
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "each is one clause a rule counts or a side holds, named as the sources name it"
)]
pub struct KujaLifts {
    /// A benefic aspects Mars or stands with it (BPHS ch. 80 v. 47).
    pub benefic: bool,
    /// The other chart carries the dosha too (BPHS ch. 80 vv. 48–49).
    pub mutual: bool,
    /// Saturn in the 1st, 4th, 7th, 9th or 12th from the lagna.
    pub saturn: bool,
    /// The Moon and Venus both in the 2nd from the lagna.
    pub moon_venus_second: bool,
    /// Jupiter aspects Mars.
    pub jupiter_aspect: bool,
    /// Rahu in a kendra from the lagna.
    pub rahu_kendra: bool,
    /// Mars with Rahu.
    pub with_rahu: bool,
}

impl KujaLifts {
    /// None: the verse's own reading, which nothing lifts.
    pub const NONE: KujaLifts = KujaLifts {
        benefic: false,
        mutual: false,
        saturn: false,
        moon_venus_second: false,
        jupiter_aspect: false,
        rahu_kendra: false,
        with_rahu: false,
    };
    /// BPHS ch. 80 vv. 47–49's two.
    pub const PARASHARA: KujaLifts = KujaLifts {
        benefic: true,
        mutual: true,
        ..KujaLifts::NONE
    };
    /// The five the almanacs printed in Nepal give.
    pub const PRINTED_IN_NEPAL: KujaLifts = KujaLifts {
        saturn: true,
        moon_venus_second: true,
        jupiter_aspect: true,
        rahu_kendra: true,
        with_rahu: true,
        ..KujaLifts::NONE
    };

    /// The clauses in a fixed order.
    const fn clauses(self) -> [bool; 7] {
        [
            self.benefic,
            self.mutual,
            self.saturn,
            self.moon_venus_second,
            self.jupiter_aspect,
            self.rahu_kendra,
            self.with_rahu,
        ]
    }

    /// The clauses as a bit set, bit 0 `benefic` to bit 6 `withRahu`, as
    /// the C boundary carries them.
    ///
    /// ```
    /// use teistro_matching::KujaLifts;
    ///
    /// assert_eq!(KujaLifts::PARASHARA.bits(), 0b11);
    /// assert_eq!(KujaLifts::from_bits(KujaLifts::PRINTED_IN_NEPAL.bits()), KujaLifts::PRINTED_IN_NEPAL);
    /// ```
    #[must_use]
    pub fn bits(self) -> u8 {
        self.clauses()
            .into_iter()
            .rev()
            .fold(0, |bits, set| (bits << 1) | u8::from(set))
    }

    /// The clauses a bit set holds, as [`KujaLifts::bits`] writes them;
    /// bit 7 is ignored.
    #[must_use]
    pub const fn from_bits(bits: u8) -> KujaLifts {
        KujaLifts {
            benefic: bits & 1 != 0,
            mutual: bits & (1 << 1) != 0,
            saturn: bits & (1 << 2) != 0,
            moon_venus_second: bits & (1 << 3) != 0,
            jupiter_aspect: bits & (1 << 4) != 0,
            rahu_kendra: bits & (1 << 5) != 0,
            with_rahu: bits & (1 << 6) != 0,
        }
    }

    /// How many clauses are set.
    #[must_use]
    pub fn count(self) -> usize {
        self.clauses().into_iter().filter(|set| *set).count()
    }

    /// Whether any clause is set.
    #[must_use]
    pub fn any(self) -> bool {
        self.count() > 0
    }

    /// The clauses set in both.
    #[must_use]
    pub const fn and(self, other: KujaLifts) -> KujaLifts {
        KujaLifts {
            benefic: self.benefic && other.benefic,
            mutual: self.mutual && other.mutual,
            saturn: self.saturn && other.saturn,
            moon_venus_second: self.moon_venus_second && other.moon_venus_second,
            jupiter_aspect: self.jupiter_aspect && other.jupiter_aspect,
            rahu_kendra: self.rahu_kendra && other.rahu_kendra,
            with_rahu: self.with_rahu && other.with_rahu,
        }
    }

    /// The clauses that hold for a native, `mutual` aside, which only the
    /// match reads; `None` where the native carries no sky.
    fn held(native: KujaNative) -> Option<KujaLifts> {
        let sky = native.sky?;
        let from_lagna = |sign: Rashi| house(native.lagna, sign);
        let aspects_mars = |sign: Rashi, houses: &[u8]| houses.contains(&house(sign, native.mars));
        // Mercury is a benefic unless a malefic stands with it, the Moon
        // while it waxes (BPHS ch. 3; C755).
        let ketu_with_mercury = house(sky.rahu, sky.mercury) == 7;
        let mercury = ![sky.sun, native.mars, sky.saturn, sky.rahu].contains(&sky.mercury)
            && !ketu_with_mercury;
        // Each benefic's sign and the houses from it it reaches: its own,
        // where it stands with Mars, and those it aspects.
        let (seventh, jupiter): (&[u8], &[u8]) = (&[1, 7], &[1, 5, 7, 9]);
        let benefics = [
            Some((sky.jupiter, jupiter)),
            Some((native.venus, seventh)),
            mercury.then_some((sky.mercury, seventh)),
            sky.moon_waxing.then_some((native.moon, seventh)),
        ];
        Some(KujaLifts {
            benefic: benefics
                .into_iter()
                .flatten()
                .any(|(sign, houses)| aspects_mars(sign, houses)),
            mutual: false,
            saturn: [1, 4, 7, 9, 12].contains(&from_lagna(sky.saturn)),
            moon_venus_second: from_lagna(native.moon) == 2 && from_lagna(native.venus) == 2,
            jupiter_aspect: aspects_mars(sky.jupiter, &[5, 7, 9]),
            rahu_kendra: [1, 4, 7, 10].contains(&from_lagna(sky.rahu)),
            with_rahu: sky.rahu == native.mars,
        })
    }
}

/// The readings the Kuja dosha is computed under; each default is the
/// verse's own.
///
/// ```
/// use teistro_matching::{KujaFrom, KujaHouses, KujaLifts, KujaRules};
///
/// assert_eq!(
///     KujaRules::default(),
///     KujaRules { houses: KujaHouses::Manasagari, from: KujaFrom::Lagna, lifts: KujaLifts::NONE }
/// );
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct KujaRules {
    /// Which houses make the dosha.
    pub houses: KujaHouses,
    /// From where they are counted.
    pub from: KujaFrom,
    /// The lifts that lift it; none by default, as the verse has none.
    pub lifts: KujaLifts,
}

/// Mars's house from one reference.
///
/// ```
/// use teistro_matching::{KujaNative, KujaReference};
///
/// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// use teistro_matching::{KujaReading, KujaRules, kuja_side};
///
/// let side = kuja_side(native, KujaRules::default());
/// assert_eq!(side.readings[0], KujaReading { from: KujaReference::Lagna, house: 7, in_houses: true });
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KujaReading {
    /// The place the house is counted from.
    pub from: KujaReference,
    /// Mars's house from it, 1 to 12.
    pub house: u8,
    /// Whether the house is one of the rules' houses. It makes the side's
    /// dosha only when the rules count from this reference.
    pub in_houses: bool,
}

/// One native's Kuja dosha.
///
/// ```
/// use teistro_matching::{KujaNative, KujaReference};
///
/// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// use teistro_matching::{KujaRules, kuja_side};
///
/// let side = kuja_side(native, KujaRules::default());
/// // The 7th from the lagna is the verse's own dosha.
/// assert!(side.dosha);
/// # let _ = KujaReference::Lagna;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct KujaSide {
    /// Mars's house from the lagna, the Moon and Venus, whatever the
    /// rules count.
    pub readings: [KujaReading; 3],
    /// Whether Mars stands in one of the rules' houses from a reference
    /// the rules count.
    pub dosha: bool,
    /// Every lift that holds, whatever the rules count; `None` where the
    /// native carries no sky, so no lift was read.
    pub held: Option<KujaLifts>,
    /// Whether the side carries the dosha and a lift the rules count holds.
    pub lifted: bool,
}

/// The Kuja dosha of both natives, as clauses, with the lifts that hold
/// on each side (C288, C755).
///
/// ```
/// use teistro_matching::{KujaNative, KujaReference};
///
/// // An Aries lagna, the Moon in Taurus, Venus in Gemini, Mars in Libra.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// use teistro_matching::{KujaRules, kuja};
///
/// // One Mars on both sides: both carry it.
/// let read = kuja(native, native, KujaRules::default());
/// assert!(read.both);
/// # let _ = KujaReference::Lagna;
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Kuja {
    /// The bride's, which the verse reads for her husband's ruin.
    pub bride: KujaSide,
    /// The groom's, which it reads for his wife's.
    pub groom: KujaSide,
    /// Whether both carry it, the fact the popular cancellation reads.
    pub both: bool,
}

/// One native's Kuja dosha under the rules.
///
/// ```
/// use teistro_matching::{KujaNative, KujaRules, kuja_side};
///
/// // Mars in the 3rd from the lagna.
/// let native = KujaNative::of_longitudes(5.0, 40.0, 70.0, 70.0)?;
/// assert!(!kuja_side(native, KujaRules::default()).dosha);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn kuja_side(native: KujaNative, rules: KujaRules) -> KujaSide {
    let houses = rules.houses.houses();
    let readings = KUJA_REFERENCES.map(|from| {
        let house = native.mars_house(from);
        KujaReading {
            from,
            house,
            in_houses: houses.contains(&house),
        }
    });
    let dosha = readings
        .iter()
        .any(|reading| reading.in_houses && rules.from.counts(reading.from));
    let held = KujaLifts::held(native);
    KujaSide {
        readings,
        dosha,
        held,
        lifted: dosha && held.is_some_and(|held| held.and(rules.lifts).any()),
    }
}

/// A side read beside the other: `mutual` holds where the other carries
/// the dosha too, and lifts it where the rules count it.
fn beside(side: KujaSide, other: KujaSide, rules: KujaRules) -> KujaSide {
    let mutual = side.dosha && other.dosha;
    KujaSide {
        held: side.held.map(|held| KujaLifts { mutual, ..held }),
        lifted: side.lifted || (mutual && rules.lifts.mutual),
        ..side
    }
}

/// The Kuja dosha of a bride and a groom under the rules (*Manasagari*,
/// jāyābhāva v. 4).
///
/// ```
/// use teistro_matching::{KujaNative, KujaRules, kuja};
///
/// // Mars in the 7th from an Aries lagna, and in the 2nd from another.
/// let bride = KujaNative::of_longitudes(5.0, 40.0, 70.0, 190.0)?;
/// let groom = KujaNative::of_longitudes(5.0, 40.0, 70.0, 40.0)?;
/// let read = kuja(bride, groom, KujaRules::default());
/// assert!(read.bride.dosha && !read.groom.dosha && !read.both);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[must_use]
pub fn kuja(bride: KujaNative, groom: KujaNative, rules: KujaRules) -> Kuja {
    let (bride, groom) = (kuja_side(bride, rules), kuja_side(groom, rules));
    Kuja {
        bride: beside(bride, groom, rules),
        groom: beside(groom, bride, rules),
        both: bride.dosha && groom.dosha,
    }
}
