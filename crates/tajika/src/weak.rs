//! A graha **unable to bring about the matter**: Tajika Nilakanthi's
//! Samjna Tantra v. 72 with its gloss (1893, leaf n66), which the verses
//! on the Moon (vv. 73–74, [`crate::moon_weakness`]) then narrow to her.
//!
//! The verse names four ways a graha is *nirbala*, each reported, none
//! weighed:
//!
//! - it does not look at the lagna (*anīkṣamāṇas tanum*);
//! - it stands in the Sun's **setting part** (*astabhāga*), which the gloss
//!   gives as "the seventh navāṁśa from the one the Sun is in", citing
//!   Samarasiṁha's *sūryāstabhāgasaṁstha* (crux C377);
//! - it is empty of the five offices, own sign, exaltation, hudda,
//!   drekkāṇa and navāṁśa (*svabhoccādipadaiḥ śūnyaḥ*);
//! - it is in Ishrafa with a malefic (*krūreśarāphī*).
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_tajika::{AnnualSky, GrahaClause, WeaknessRules, graha_weakness};
//!
//! // Aries rising; Saturn in Libra sees it from the 7th, Jupiter in
//! // Taurus does not.
//! let sky = AnnualSky {
//!     sun_deg: 100.0, moon_deg: 40.0, mars_deg: 300.0, mercury_deg: 110.0,
//!     jupiter_deg: 45.0, venus_deg: 130.0, saturn_deg: 190.0,
//! };
//! let jupiter = graha_weakness(Graha::Jupiter, 10.0, &sky, WeaknessRules::default())?;
//! assert!(jupiter.clauses.contains(&GrahaClause::UnseenLagna));
//! let saturn = graha_weakness(Graha::Saturn, 10.0, &sky, WeaknessRules::default())?;
//! assert!(!saturn.clauses.contains(&GrahaClause::UnseenLagna));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;

use crate::bala::{AnnualSky, SEVEN, finite_longitude, sign_of_longitude};
use crate::drishti::{Drishti, Yoga, between};
use crate::kamboola::{Adhikara, OfficeRules, held};
use crate::yoga::MALEFICS;

/// The span of a navāṁśa, degrees.
const NAVAMSHA_DEG: f64 = 30.0 / 9.0;

/// What the Sun's setting part is (crux C377).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{AnnualSky, GrahaClause, SettingPart, WeaknessRules, graha_weakness};
///
/// // The Sun at Cancer 10°, in the Aries navamsha's opposite: Venus at Leo 1°
/// // stands in the very navamsha seven on, the Moon at Aries 1° only in a
/// // navamsha of the seventh sign from the Sun's.
/// let sky = AnnualSky {
///     sun_deg: 100.0, moon_deg: 1.0, mars_deg: 300.0, mercury_deg: 110.0,
///     jupiter_deg: 45.0, venus_deg: 121.0, saturn_deg: 190.0,
/// };
/// let setting = |graha, setting| -> Result<bool, teistro_core::error::Error> {
///     let weak = graha_weakness(graha, 10.0, &sky, WeaknessRules { setting })?;
///     Ok(weak.clauses.contains(&GrahaClause::SettingPart))
/// };
///
/// assert!(setting(Graha::Venus, SettingPart::NavamshaSign)? && setting(Graha::Venus, SettingPart::NavamshaArc)?);
/// assert!(setting(Graha::Moon, SettingPart::NavamshaSign)?);
/// assert!(!setting(Graha::Moon, SettingPart::NavamshaArc)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SettingPart {
    /// Any navāṁśa whose sign is the seventh from the sign of the Sun's
    /// navāṁśa: the gloss counts from "the navāṁśa of the sign the Sun is
    /// in", and the seventh from a sign is where it sets.
    #[default]
    NavamshaSign,
    /// The one navāṁśa-arc seven on from the Sun's own, 20° to 23°20′
    /// past the start of the Sun's.
    NavamshaArc,
}

impl SettingPart {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `NAVAMSHA_SIGN`.
    ///
    /// ```
    /// use teistro_tajika::SettingPart;
    ///
    /// assert_eq!(SettingPart::NavamshaArc.key(), "NAVAMSHA_ARC");
    /// assert_eq!(serde_json::to_string(&SettingPart::default())?, "\"NAVAMSHA_SIGN\"");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            SettingPart::NavamshaSign => "NAVAMSHA_SIGN",
            SettingPart::NavamshaArc => "NAVAMSHA_ARC",
        }
    }

    /// Whether a graha at `longitude_deg` stands in the setting part of a
    /// Sun at `sun_deg`.
    fn holds(self, sun_deg: f64, longitude_deg: f64) -> bool {
        let division = |longitude: f64| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a folded longitude over a navāṁśa's span is 0 to 107"
            )]
            let index = (longitude.rem_euclid(360.0) / NAVAMSHA_DEG) as u32 % 108;
            index
        };
        let (sun, graha) = (division(sun_deg), division(longitude_deg));
        match self {
            // A navāṁśa's sign is its index mod 12 from Aries.
            SettingPart::NavamshaSign => (graha + 12 - sun % 12) % 12 == 6,
            SettingPart::NavamshaArc => graha == (sun + 6) % 108,
        }
    }
}

/// The readings v. 72 leaves to a reader.
///
/// ```
/// use teistro_tajika::{SettingPart, WeaknessRules};
///
/// assert_eq!(WeaknessRules::default().setting, SettingPart::NavamshaSign);
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct WeaknessRules {
    /// What the Sun's setting part is.
    pub setting: SettingPart,
}

/// A clause of v. 72, in the verse's order.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{AnnualSky, GrahaClause, SettingPart, WeaknessRules, graha_weakness};
///
/// // The Sun at Cancer 10°, in the Aries navamsha's opposite: Venus at Leo 1°
/// // stands in the very navamsha seven on, the Moon at Aries 1° only in a
/// // navamsha of the seventh sign from the Sun's.
/// let sky = AnnualSky {
///     sun_deg: 100.0, moon_deg: 1.0, mars_deg: 300.0, mercury_deg: 110.0,
///     jupiter_deg: 45.0, venus_deg: 121.0, saturn_deg: 190.0,
/// };
/// let setting = |graha, setting| -> Result<bool, teistro_core::error::Error> {
///     let weak = graha_weakness(graha, 10.0, &sky, WeaknessRules { setting })?;
///     Ok(weak.clauses.contains(&GrahaClause::SettingPart))
/// };
///
/// // The Sun never stands in his own setting part.
/// assert!(!setting(Graha::Sun, SettingPart::NavamshaSign)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "TajikaGrahaClause"))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GrahaClause {
    /// It does not aspect the lagna's sign, Tajika's sign aspect, the
    /// conjunction included, as C352 reads "unseen" for the Moon.
    UnseenLagna,
    /// It stands in the Sun's setting part, by [`SettingPart`]. Never held
    /// by the Sun.
    SettingPart,
    /// It holds none of its five offices.
    Officeless,
    /// It is in Ishrafa with Mars or Saturn, Tajika's malefics.
    MaleficIshrafa,
}

impl GrahaClause {
    /// The member's key, as serde writes it and every binding reads it
    /// back: `UNSEEN_LAGNA`.
    ///
    /// ```
    /// use teistro_tajika::GrahaClause;
    ///
    /// assert_eq!(GrahaClause::MaleficIshrafa.key(), "MALEFIC_ISHRAFA");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            GrahaClause::UnseenLagna => "UNSEEN_LAGNA",
            GrahaClause::SettingPart => "SETTING_PART",
            GrahaClause::Officeless => "OFFICELESS",
            GrahaClause::MaleficIshrafa => "MALEFIC_ISHRAFA",
        }
    }

    /// Every clause, in the verse's order.
    ///
    /// ```
    /// use teistro_tajika::GrahaClause;
    ///
    /// assert_eq!(GrahaClause::ALL.map(GrahaClause::key), ["UNSEEN_LAGNA", "SETTING_PART", "OFFICELESS", "MALEFIC_ISHRAFA"]);
    /// ```
    pub const ALL: [GrahaClause; 4] = [
        GrahaClause::UnseenLagna,
        GrahaClause::SettingPart,
        GrahaClause::Officeless,
        GrahaClause::MaleficIshrafa,
    ];
}

/// The clauses of v. 72 that hold for one graha.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{AnnualSky, GrahaClause, SettingPart, WeaknessRules, graha_weakness};
///
/// // The Sun at Cancer 10°, in the Aries navamsha's opposite: Venus at Leo 1°
/// // stands in the very navamsha seven on, the Moon at Aries 1° only in a
/// // navamsha of the seventh sign from the Sun's.
/// let sky = AnnualSky {
///     sun_deg: 100.0, moon_deg: 1.0, mars_deg: 300.0, mercury_deg: 110.0,
///     jupiter_deg: 45.0, venus_deg: 121.0, saturn_deg: 190.0,
/// };
/// let setting = |graha, setting| -> Result<bool, teistro_core::error::Error> {
///     let weak = graha_weakness(graha, 10.0, &sky, WeaknessRules { setting })?;
///     Ok(weak.clauses.contains(&GrahaClause::SettingPart))
/// };
///
/// let venus = graha_weakness(Graha::Venus, 10.0, &sky, WeaknessRules::default())?;
/// assert_eq!(venus.graha, Graha::Venus);
/// // Clauses come in the verse's order.
/// let order = |clause| GrahaClause::ALL.iter().position(|one| *one == clause);
/// assert!(venus.clauses.windows(2).all(|pair| order(pair[0]) < order(pair[1])));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct GrahaWeakness {
    /// Whose.
    pub graha: Graha,
    /// Each clause that holds, in the verse's order.
    pub clauses: Vec<GrahaClause>,
}

/// The clauses of v. 72 that hold for `graha`, one of the seven, with the
/// lagna at `lagna_deg`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{AnnualSky, GrahaClause, SettingPart, WeaknessRules, graha_weakness};
///
/// // The Sun at Cancer 10°, in the Aries navamsha's opposite: Venus at Leo 1°
/// // stands in the very navamsha seven on, the Moon at Aries 1° only in a
/// // navamsha of the seventh sign from the Sun's.
/// let sky = AnnualSky {
///     sun_deg: 100.0, moon_deg: 1.0, mars_deg: 300.0, mercury_deg: 110.0,
///     jupiter_deg: 45.0, venus_deg: 121.0, saturn_deg: 190.0,
/// };
/// let setting = |graha, setting| -> Result<bool, teistro_core::error::Error> {
///     let weak = graha_weakness(graha, 10.0, &sky, WeaknessRules { setting })?;
///     Ok(weak.clauses.contains(&GrahaClause::SettingPart))
/// };
///
/// assert!(setting(Graha::Venus, SettingPart::default())?);
/// assert_eq!(graha_weakness(Graha::Rahu, 10.0, &sky, WeaknessRules::default()).unwrap_err().field(), Some("graha"));
/// assert_eq!(graha_weakness(Graha::Venus, f64::NAN, &sky, WeaknessRules::default()).unwrap_err().field(), Some("lagna_deg"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A longitude that is not a number, named by its field (`lagna_deg` for
/// the lagna), or a graha outside the seven (`graha`).
pub fn graha_weakness(
    graha: Graha,
    lagna_deg: f64,
    sky: &AnnualSky,
    rules: WeaknessRules,
) -> Result<GrahaWeakness, Error> {
    sky.check()?;
    finite_longitude("lagna_deg", lagna_deg)?;
    if !SEVEN.contains(&graha) {
        return Err(
            Error::invalid_arg(format!("{graha:?} is not one of the seven v. 72 reads"))
                .with_field("graha".to_owned()),
        );
    }
    let lagna = sign_of_longitude(lagna_deg);
    let longitude = sky.longitude_of(graha);
    let mut clauses = Vec::new();
    for clause in GrahaClause::ALL {
        let holds = match clause {
            GrahaClause::UnseenLagna => {
                !Drishti::between_signs(sky.sign_of(graha), lagna).is_aspect()
            }
            GrahaClause::SettingPart => {
                graha != Graha::Sun && rules.setting.holds(sky.sun_deg, longitude)
            }
            // The five offices are the clauses that give one; a fault
            // (debilitation, an enemy's sign) is not an office.
            GrahaClause::Officeless => !held(graha, sky, OfficeRules::default())
                .iter()
                .any(|clause| clause.office() != Adhikara::Adhama),
            GrahaClause::MaleficIshrafa => {
                let mut ishrafa = false;
                for malefic in MALEFICS.into_iter().filter(|&malefic| malefic != graha) {
                    ishrafa |= between(graha, malefic, sky)?.yoga == Some(Yoga::Ishrafa);
                }
                ishrafa
            }
        };
        if holds {
            clauses.push(clause);
        }
    }
    Ok(GrahaWeakness { graha, clauses })
}

/// [`graha_weakness`] for each of the seven, the Sun first.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_tajika::{AnnualSky, GrahaClause, SettingPart, WeaknessRules, graha_weakness};
///
/// // The Sun at Cancer 10°, in the Aries navamsha's opposite: Venus at Leo 1°
/// // stands in the very navamsha seven on, the Moon at Aries 1° only in a
/// // navamsha of the seventh sign from the Sun's.
/// let sky = AnnualSky {
///     sun_deg: 100.0, moon_deg: 1.0, mars_deg: 300.0, mercury_deg: 110.0,
///     jupiter_deg: 45.0, venus_deg: 121.0, saturn_deg: 190.0,
/// };
/// let setting = |graha, setting| -> Result<bool, teistro_core::error::Error> {
///     let weak = graha_weakness(graha, 10.0, &sky, WeaknessRules { setting })?;
///     Ok(weak.clauses.contains(&GrahaClause::SettingPart))
/// };
/// use teistro_tajika::weaknesses;
///
/// let all = weaknesses(10.0, &sky, WeaknessRules::default())?;
/// assert_eq!(all.len(), 7);
/// assert_eq!(all[0].graha, Graha::Sun);
/// assert!(!all[0].clauses.contains(&GrahaClause::SettingPart));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`graha_weakness`].
pub fn weaknesses(
    lagna_deg: f64,
    sky: &AnnualSky,
    rules: WeaknessRules,
) -> Result<Vec<GrahaWeakness>, Error> {
    SEVEN
        .into_iter()
        .map(|graha| graha_weakness(graha, lagna_deg, sky, rules))
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests")]

    use super::*;

    /// Aries rising. The Sun at Cancer 5°, in Leo's navāṁśa (Cancer's
    /// second); Venus at Capricorn 5° is in Aquarius's navāṁśa (Capricorn's
    /// second), the seventh sign from Leo; Mercury at Cancer 25° is in the
    /// navāṁśa-arc seven on from the Sun's.
    fn sky() -> AnnualSky {
        AnnualSky {
            sun_deg: 95.0,
            moon_deg: 40.0,
            mars_deg: 92.0,
            mercury_deg: 115.0,
            jupiter_deg: 45.0,
            venus_deg: 275.0,
            saturn_deg: 190.0,
        }
    }

    fn clauses(graha: Graha, rules: WeaknessRules) -> Vec<GrahaClause> {
        graha_weakness(graha, 10.0, &sky(), rules).unwrap().clauses
    }

    #[test]
    fn the_setting_part_is_read_by_sign_or_by_arc() {
        let sign = WeaknessRules::default();
        let arc = WeaknessRules {
            setting: SettingPart::NavamshaArc,
        };
        assert!(clauses(Graha::Venus, sign).contains(&GrahaClause::SettingPart));
        assert!(!clauses(Graha::Venus, arc).contains(&GrahaClause::SettingPart));
        // The arc seven on is always in the seventh sign too.
        assert!(clauses(Graha::Mercury, arc).contains(&GrahaClause::SettingPart));
        assert!(clauses(Graha::Mercury, sign).contains(&GrahaClause::SettingPart));
        assert!(!clauses(Graha::Sun, sign).contains(&GrahaClause::SettingPart));
    }

    #[test]
    fn the_arc_reading_is_a_subset_of_the_sign_reading_everywhere() {
        let mut sign_count = 0;
        let mut arc_count = 0;
        for sun in 0..360 {
            for at in 0..1080 {
                let at = f64::from(at) / 3.0 + 0.1;
                let sun = f64::from(sun) + 0.5;
                let by_arc = SettingPart::NavamshaArc.holds(sun, at);
                let by_sign = SettingPart::NavamshaSign.holds(sun, at);
                assert!(!by_arc || by_sign, "Sun {sun}, graha {at}");
                sign_count += usize::from(by_sign);
                arc_count += usize::from(by_arc);
            }
        }
        // A twelfth of the circle by sign, a hundred-and-eighth by arc.
        assert_eq!(sign_count * 12, 360 * 1080);
        assert_eq!(arc_count * 108, 360 * 1080);
    }

    #[test]
    fn a_graha_unseen_by_the_lagna_officeless_or_parting_from_a_malefic_is_weak() {
        let rules = WeaknessRules::default();
        // Jupiter in Taurus, 2nd from Aries: no Tajika aspect.
        assert!(clauses(Graha::Jupiter, rules).contains(&GrahaClause::UnseenLagna));
        // Saturn in Libra, 7th, exalted: he looks and holds an office.
        let saturn = clauses(Graha::Saturn, rules);
        assert!(!saturn.contains(&GrahaClause::UnseenLagna));
        assert!(!saturn.contains(&GrahaClause::Officeless));
        // The Moon at Taurus 10° and Mars at Cancer 2°, a sextile: the
        // faster Moon eight degrees past him, inside their mean orb of
        // ten, is in Ishrafa with a malefic.
        assert!(clauses(Graha::Moon, rules).contains(&GrahaClause::MaleficIshrafa));
    }

    #[test]
    fn officeless_ignores_the_faults() {
        // Venus at Virgo 1°: debilitated, in Mercury's hudda, the Tajika
        // drekkāṇa (Virgo's first is the Sun's) and Capricorn's navāṁśa:
        // a fault and no office, so officeless.
        let sky = AnnualSky {
            venus_deg: 151.0,
            ..sky()
        };
        let venus = graha_weakness(Graha::Venus, 10.0, &sky, WeaknessRules::default()).unwrap();
        assert!(venus.clauses.contains(&GrahaClause::Officeless));
    }

    #[test]
    fn a_node_or_a_lagna_that_is_not_a_number_is_refused_by_its_field() {
        let rules = WeaknessRules::default();
        let field =
            |result: Result<GrahaWeakness, Error>| result.unwrap_err().field().map(str::to_owned);
        assert_eq!(
            field(graha_weakness(Graha::Rahu, 10.0, &sky(), rules)).as_deref(),
            Some("graha")
        );
        assert_eq!(
            field(graha_weakness(Graha::Sun, f64::NAN, &sky(), rules)).as_deref(),
            Some("lagna_deg")
        );
        assert_eq!(weaknesses(10.0, &sky(), rules).unwrap().len(), 7);
    }

    #[test]
    fn every_key_is_serdes() {
        for clause in GrahaClause::ALL {
            assert_eq!(serde_json::to_value(clause).unwrap(), clause.key());
        }
        for setting in [SettingPart::NavamshaSign, SettingPart::NavamshaArc] {
            assert_eq!(serde_json::to_value(setting).unwrap(), setting.key());
        }
    }
}
