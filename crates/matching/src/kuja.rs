//! The Kuja dosha across two charts: Mars in the houses *Manasagari*'s
//! jāyābhāva v. 4 names, for each native (`03-design/matching.md`,
//! C285 to C288).

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
}

impl KujaNative {
    /// The native whose lagna, Moon, Venus and Mars stand at these
    /// longitudes, in degrees of the chart's own zodiac.
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
        let sign = |field: &str, longitude_deg: f64| {
            if longitude_deg.is_finite() {
                Ok(Rashi::of_longitude(longitude_deg.rem_euclid(360.0)))
            } else {
                Err(Error::invalid_arg(format!(
                    "the {field}'s longitude is a finite number of degrees, not {longitude_deg}"
                ))
                .with_field(field))
            }
        };
        Ok(KujaNative {
            lagna: sign("lagna", lagna_deg)?,
            moon: sign("moon", moon_deg)?,
            venus: sign("venus", venus_deg)?,
            mars: sign("mars", mars_deg)?,
        })
    }

    /// The sign a house is counted from.
    #[must_use]
    pub fn sign_of(self, reference: KujaReference) -> Rashi {
        match reference {
            KujaReference::Lagna => self.lagna,
            KujaReference::Moon => self.moon,
            KujaReference::Venus => self.venus,
        }
    }

    /// Mars's house from a reference, 1 to 12, counted by sign (C287).
    #[must_use]
    pub fn mars_house(self, reference: KujaReference) -> u8 {
        let from = self.sign_of(reference).id();
        u8::try_from((self.mars.id() + 12 - from) % 12 + 1).unwrap_or(1)
    }
}

/// Which houses of Mars make the dosha (C285).
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
    #[must_use]
    pub fn houses(self) -> &'static [u8] {
        match self {
            KujaHouses::Manasagari => &[1, 4, 7, 8, 12],
            KujaHouses::WithSecond => &[1, 2, 4, 7, 8, 12],
        }
    }
}

/// From where Mars's house makes the dosha (C286).
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
    #[must_use]
    pub fn counts(self, reference: KujaReference) -> bool {
        match self {
            KujaFrom::Lagna => reference == KujaReference::Lagna,
            KujaFrom::LagnaMoonVenus => true,
        }
    }
}

/// A place Mars's house is counted from.
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
pub const KUJA_REFERENCES: [KujaReference; 3] = [
    KujaReference::Lagna,
    KujaReference::Moon,
    KujaReference::Venus,
];

/// The readings the Kuja dosha is computed under; each default is the
/// verse's own.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct KujaRules {
    /// Which houses make the dosha.
    pub houses: KujaHouses,
    /// From where they are counted.
    pub from: KujaFrom,
}

/// Mars's house from one reference.
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
}

/// The Kuja dosha of both natives, as clauses: no verse read lifts it
/// (C288).
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
    KujaSide { readings, dosha }
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
        bride,
        groom,
        both: bride.dosha && groom.dosha,
    }
}
