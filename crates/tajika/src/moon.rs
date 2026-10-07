//! The graha on an **empty road** and the Moon's weaknesses, as Tajika
//! Nilakanthi's Samjna Tantra prints them (1893, leaves n60 and n66).
//!
//! - **vv. 55–56, *śūnyamārga*:** a graha holds none of its five
//!   dignities (own sign, exaltation, hudda, drekkāṇa, navāṁśa), stands
//!   neither in its debilitation nor in an enemy's sign, and no graha
//!   aspects it. The Moon on such a road can carry no ithasala.
//! - **v. 73:** the Moon is *pādona* in the 12th from the Sun, in the
//!   first half of Scorpio or, "especially", the last half of Libra (the
//!   gloss names the halves), unseen by her sign's lord, unseen by every
//!   graha, or on an empty road.
//! - **v. 74:** she is not auspicious when *kṣīṇa*, at a sign's end
//!   (*bhānte*, the gloss's last navāṁśa), or under the hungry aspect of
//!   Mars in the bright half or Saturn in the dark, "at birth or in a
//!   query".
//!
//! Each clause is reported, none is weighed, and no verdict is drawn.
//!
//! ```
//! use teistro_tajika::{AnnualSky, MoonClause, MoonRules, moon_weakness};
//!
//! // The Moon at 28° Libra, the Sun in Scorpio, Saturn in Capricorn.
//! let sky = AnnualSky {
//!     sun_deg: 220.0, moon_deg: 208.0, mars_deg: 100.0, mercury_deg: 230.0,
//!     jupiter_deg: 10.0, venus_deg: 200.0, saturn_deg: 280.0,
//! };
//! let weak = moon_weakness(&sky, MoonRules::default())?;
//! assert!(weak.clauses.contains(&MoonClause::LibraLastHalf));
//! assert!(weak.clauses.contains(&MoonClause::HungryAspect));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::bala::{AnnualSky, Relation, SEVEN, drekkana_lord, hudda_lord, navamsha_lord};
use crate::drishti::Drishti;

/// The span of a navāṁśa, degrees.
const NAVAMSHA_DEG: f64 = 30.0 / 9.0;

/// Where the dark half's waning Moon is counted from and to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum KshinaRule {
    /// The gloss's own reading: from the dark 8th tithi through the bright
    /// 8th, an elongation from 264° round to 96°.
    #[default]
    DarkEighthToBrightEighth,
    /// The reading the gloss gives as "some say": from the dark 11th to
    /// the new Moon, an elongation of 300° or more.
    DarkEleventhToNewMoon,
}

impl KshinaRule {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `DARK_EIGHTH_TO_BRIGHT_EIGHTH`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            KshinaRule::DarkEighthToBrightEighth => "DARK_EIGHTH_TO_BRIGHT_EIGHTH",
            KshinaRule::DarkEleventhToNewMoon => "DARK_ELEVENTH_TO_NEW_MOON",
        }
    }

    /// Whether a Moon `elongation_deg` past the Sun is *kṣīṇa*.
    fn holds(self, elongation_deg: f64) -> bool {
        match self {
            KshinaRule::DarkEighthToBrightEighth => !(96.0..264.0).contains(&elongation_deg),
            KshinaRule::DarkEleventhToNewMoon => elongation_deg >= 300.0,
        }
    }
}

/// The choices the verses leave to a reader.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MoonRules {
    /// Which span of the waning Moon is *kṣīṇa*.
    pub kshina: KshinaRule,
}

/// A clause of vv. 73–74, in the verses' order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MoonClause {
    /// v. 73: in the 12th sign from the Sun's.
    TwelfthFromSun,
    /// v. 73: in the first half of Scorpio.
    ScorpioFirstHalf,
    /// v. 73: in the last half of Libra, "especially".
    LibraLastHalf,
    /// v. 73: her sign's lord does not aspect her. Never held in Cancer,
    /// where the lord is herself.
    UnseenByLord,
    /// v. 73: no graha aspects her.
    UnseenByAll,
    /// v. 73: on an empty road (vv. 55–56).
    ShunyaMarga,
    /// v. 74: waning, by [`KshinaRule`].
    Kshina,
    /// v. 74: in her sign's last navāṁśa.
    Bhante,
    /// v. 74: Mars in the bright half, or Saturn in the dark, aspects her
    /// from the 1st, 4th, 7th or 10th.
    HungryAspect,
}

impl MoonClause {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `TWELFTH_FROM_SUN`.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            MoonClause::TwelfthFromSun => "TWELFTH_FROM_SUN",
            MoonClause::ScorpioFirstHalf => "SCORPIO_FIRST_HALF",
            MoonClause::LibraLastHalf => "LIBRA_LAST_HALF",
            MoonClause::UnseenByLord => "UNSEEN_BY_LORD",
            MoonClause::UnseenByAll => "UNSEEN_BY_ALL",
            MoonClause::ShunyaMarga => "SHUNYA_MARGA",
            MoonClause::Kshina => "KSHINA",
            MoonClause::Bhante => "BHANTE",
            MoonClause::HungryAspect => "HUNGRY_ASPECT",
        }
    }

    /// Every clause, in the verses' order.
    pub const ALL: [MoonClause; 9] = [
        MoonClause::TwelfthFromSun,
        MoonClause::ScorpioFirstHalf,
        MoonClause::LibraLastHalf,
        MoonClause::UnseenByLord,
        MoonClause::UnseenByAll,
        MoonClause::ShunyaMarga,
        MoonClause::Kshina,
        MoonClause::Bhante,
        MoonClause::HungryAspect,
    ];

    /// The verse that states it.
    #[must_use]
    pub const fn verse(self) -> u8 {
        match self {
            MoonClause::Kshina | MoonClause::Bhante | MoonClause::HungryAspect => 74,
            _ => 73,
        }
    }
}

/// The clauses of vv. 73–74 that hold for the Moon.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonWeakness {
    /// The rules they were read under.
    pub rules: MoonRules,
    /// Each clause that holds, in the verses' order.
    pub clauses: Vec<MoonClause>,
}

/// Whether `graha`, one of the seven, travels an empty road (vv. 55–56).
///
/// An aspect is Tajika's by sign, the conjunction included, as
/// [`Drishti::between_signs`] reads it, and an enemy's sign is one whose
/// lord stands to `graha` as [`Relation::Enemy`].
///
/// # Errors
///
/// A longitude that is not a number, named by its field, or a graha
/// outside the seven.
pub fn shunya_marga(graha: Graha, sky: &AnnualSky) -> Result<bool, Error> {
    sky.check()?;
    if !SEVEN.contains(&graha) {
        return Err(Error::invalid_arg(format!(
            "{graha:?} is not one of the seven a road is read for"
        ))
        .with_field("graha".to_owned()));
    }
    Ok(empty_road(graha, sky))
}

/// vv. 55–56 on a checked sky, for one of the seven.
fn empty_road(graha: Graha, sky: &AnnualSky) -> bool {
    let longitude = sky.longitude_of(graha);
    let sign = sky.sign_of(graha);
    let lord = sign.attributes().lord;
    let attributes = graha.attributes();
    let in_sign = |fixed: Option<teistro_core::catalogue::SignDegree>| {
        fixed.is_some_and(|fixed| fixed.sign == sign)
    };
    let dignified = lord == graha
        || in_sign(attributes.exaltation)
        || hudda_lord(longitude) == graha
        || drekkana_lord(longitude) == graha
        || navamsha_lord(longitude) == graha;
    let afflicted =
        in_sign(attributes.debilitation) || Relation::between(graha, lord, sky) == Relation::Enemy;
    !dignified && !afflicted && !aspected(graha, sky, |_| true)
}

/// Whether any of the seven but `graha` that `by` admits aspects it.
fn aspected(graha: Graha, sky: &AnnualSky, by: impl Fn(Graha) -> bool) -> bool {
    let sign = sky.sign_of(graha);
    SEVEN
        .into_iter()
        .filter(|&other| other != graha && by(other))
        .any(|other| Drishti::between_signs(sky.sign_of(other), sign).is_aspect())
}

/// The clauses of vv. 73–74 that hold for the Moon of `sky`.
///
/// # Errors
///
/// A longitude that is not a number, named by its field.
pub fn moon_weakness(sky: &AnnualSky, rules: MoonRules) -> Result<MoonWeakness, Error> {
    sky.check()?;
    let moon = sky.longitude_of(Graha::Moon);
    let sign = sky.sign_of(Graha::Moon);
    let lord = sign.attributes().lord;
    let elongation = (moon - sky.longitude_of(Graha::Sun)).rem_euclid(360.0);
    let hungry_from = if elongation < 180.0 {
        Graha::Mars
    } else {
        Graha::Saturn
    };
    let clauses = MoonClause::ALL
        .into_iter()
        .filter(|&clause| match clause {
            MoonClause::TwelfthFromSun => House::between(sky.sign_of(Graha::Sun), sign).get() == 12,
            MoonClause::ScorpioFirstHalf => (210.0..225.0).contains(&moon),
            MoonClause::LibraLastHalf => (195.0..210.0).contains(&moon),
            MoonClause::UnseenByLord => {
                lord != Graha::Moon && !aspected(Graha::Moon, sky, |other| other == lord)
            }
            MoonClause::UnseenByAll => !aspected(Graha::Moon, sky, |_| true),
            MoonClause::ShunyaMarga => empty_road(Graha::Moon, sky),
            MoonClause::Kshina => rules.kshina.holds(elongation),
            MoonClause::Bhante => moon % 30.0 >= 30.0 - NAVAMSHA_DEG,
            MoonClause::HungryAspect => matches!(
                Drishti::between_signs(sky.sign_of(hungry_from), sign),
                Drishti::Inimical | Drishti::SecretlyInimical
            ),
        })
        .collect();
    Ok(MoonWeakness { rules, clauses })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        reason = "tests fail by panicking on what they asked for"
    )]

    use teistro_core::catalogue::Graha;

    use super::{AnnualSky, KshinaRule, MoonClause, MoonRules, moon_weakness, shunya_marga};

    /// The Moon at 2° Gemini on an empty road: Mercury, her sign's lord,
    /// and the Sun in Taurus, Venus and Saturn in Cancer, Mars in
    /// Scorpio, Jupiter in Capricorn, each 2nd, 6th, 8th or 12th from her.
    /// Her hudda is Mercury's, her drekkāṇa Jupiter's, her navāṁśa
    /// Libra's.
    fn empty() -> AnnualSky {
        AnnualSky {
            sun_deg: 40.0,
            moon_deg: 62.0,
            mars_deg: 215.0,
            mercury_deg: 50.0,
            jupiter_deg: 280.0,
            venus_deg: 100.0,
            saturn_deg: 110.0,
        }
    }

    #[test]
    fn the_moon_on_an_empty_road_meets_every_clause_of_it() {
        let read = moon_weakness(&empty(), MoonRules::default()).unwrap();
        assert_eq!(
            read.clauses,
            [
                MoonClause::UnseenByLord,
                MoonClause::UnseenByAll,
                MoonClause::ShunyaMarga,
                MoonClause::Kshina,
            ]
        );
        // Jupiter moved to Libra, her 5th, aspects her: the road is not
        // empty and she is seen, though not by her lord.
        let seen = AnnualSky {
            jupiter_deg: 190.0,
            ..empty()
        };
        assert!(!shunya_marga(Graha::Moon, &seen).unwrap());
        assert_eq!(
            moon_weakness(&seen, MoonRules::default()).unwrap().clauses,
            [MoonClause::UnseenByLord, MoonClause::Kshina]
        );
        // So does her own exaltation, or an enemy's sign: Mercury beside
        // her in Gemini is at her 1st.
        let mercury_beside = AnnualSky {
            mercury_deg: 70.0,
            ..empty()
        };
        assert!(!shunya_marga(Graha::Moon, &mercury_beside).unwrap());
        assert!(shunya_marga(Graha::Rahu, &empty()).is_err());
    }

    #[test]
    fn the_end_of_libra_carries_the_placements_of_both_verses() {
        // The Moon at 28° Libra, 12th from the Sun in Scorpio, in the
        // dark half with Saturn in Capricorn at her 10th; Venus beside her.
        let sky = AnnualSky {
            sun_deg: 220.0,
            moon_deg: 208.0,
            mars_deg: 100.0,
            mercury_deg: 230.0,
            jupiter_deg: 10.0,
            venus_deg: 200.0,
            saturn_deg: 280.0,
        };
        assert_eq!(
            moon_weakness(&sky, MoonRules::default()).unwrap().clauses,
            [
                MoonClause::TwelfthFromSun,
                MoonClause::LibraLastHalf,
                MoonClause::Kshina,
                MoonClause::Bhante,
                MoonClause::HungryAspect,
            ]
        );
        // The halves meet at Scorpio's 15th degree.
        let at = |moon_deg: f64| {
            moon_weakness(&AnnualSky { moon_deg, ..sky }, MoonRules::default())
                .unwrap()
                .clauses
        };
        assert!(at(224.9).contains(&MoonClause::ScorpioFirstHalf));
        assert!(!at(225.0).contains(&MoonClause::ScorpioFirstHalf));
        assert!(!at(194.9).contains(&MoonClause::LibraLastHalf));
    }

    #[test]
    fn the_two_readings_of_the_waning_moon_part_between_the_8th_and_11th() {
        let at = |elongation: f64, kshina: KshinaRule| {
            let sky = AnnualSky {
                moon_deg: 40.0 + elongation,
                ..empty()
            };
            moon_weakness(&sky, MoonRules { kshina })
                .unwrap()
                .clauses
                .contains(&MoonClause::Kshina)
        };
        let (gloss, some) = (
            KshinaRule::DarkEighthToBrightEighth,
            KshinaRule::DarkEleventhToNewMoon,
        );
        assert!(at(270.0, gloss) && !at(270.0, some));
        assert!(at(310.0, gloss) && at(310.0, some));
        assert!(at(95.0, gloss) && !at(96.0, gloss) && !at(95.0, some));
        assert_eq!(gloss, KshinaRule::default());
    }

    #[test]
    fn the_keys_are_what_serde_writes() {
        for clause in MoonClause::ALL {
            assert_eq!(
                serde_json::to_value(clause).unwrap(),
                serde_json::json!(clause.key())
            );
        }
        for rule in [
            KshinaRule::DarkEighthToBrightEighth,
            KshinaRule::DarkEleventhToNewMoon,
        ] {
            assert_eq!(
                serde_json::to_value(rule).unwrap(),
                serde_json::json!(rule.key())
            );
        }
    }
}
