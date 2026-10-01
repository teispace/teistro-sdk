//! A planet's essential dignities at a longitude
//! (`03-design/essential-dignities.md`).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;

use crate::terms::Terms;

/// Whether a chart is of the day or of the night.
///
/// The dignities take it as given; [`SectRule`] says how a chart's is read
/// (crux C209).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Sect {
    /// A day chart.
    Day,
    /// A night chart.
    Night,
}

impl Sect {
    /// The sect by Valens's hemisphere (*Anthologies* I, 51K–52K): a day
    /// chart when the Sun's centre stands above the true horizon.
    ///
    /// Valens reckons the hemisphere in degrees from the Ascendant and the
    /// Descendant, which is this rule wherever the zodiac rises in order.
    /// Inside the polar circles part of it rises backwards, and only the
    /// altitude still says whether the Sun is up. The Sun exactly on the
    /// horizon is read as set.
    ///
    /// ```
    /// use teistro_hellenistic::Sect;
    ///
    /// assert_eq!(Sect::from_altitude(0.1), Sect::Day);
    /// assert_eq!(Sect::from_altitude(-0.1), Sect::Night);
    /// ```
    #[must_use]
    pub fn from_altitude(sun_altitude_deg: f64) -> Sect {
        if sun_altitude_deg > 0.0 {
            Sect::Day
        } else {
            Sect::Night
        }
    }
}

/// How a chart's sect is read (crux C209, `essential-dignities.md` §Sect).
///
/// Valens decides for the horizon, but his text cannot tell the geometric
/// horizon from the apparent one, where refraction and the Sun's limb move
/// sunrise by minutes. So both are rules, and a caller may also say the
/// sect outright.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum SectRule {
    /// Valens's hemisphere: the Sun's centre above the true horizon
    /// ([`Sect::from_altitude`]). The default.
    #[default]
    Horizon,
    /// The chart's own sunrise to sunset, under the sunrise convention the
    /// chart was founded with.
    Daylight,
    /// Every chart is read as a day chart.
    Day,
    /// Every chart is read as a night chart.
    Night,
}

impl SectRule {
    /// The sect this rule reads, from the Sun's geometric altitude and
    /// whether the chart's instant falls between its sunrise and sunset.
    ///
    /// ```
    /// use teistro_hellenistic::{Sect, SectRule};
    ///
    /// // Just after the geometric sunset, before the apparent one.
    /// assert_eq!(SectRule::Horizon.sect(-0.3, true), Sect::Night);
    /// assert_eq!(SectRule::Daylight.sect(-0.3, true), Sect::Day);
    /// ```
    #[must_use]
    pub fn sect(self, sun_altitude_deg: f64, daylight: bool) -> Sect {
        match self {
            SectRule::Horizon => Sect::from_altitude(sun_altitude_deg),
            SectRule::Daylight if daylight => Sect::Day,
            SectRule::Daylight | SectRule::Night => Sect::Night,
            SectRule::Day => Sect::Day,
        }
    }
}

/// The seven planets that hold essential dignity, in the Chaldean order,
/// slowest first.
pub const CHALDEAN_ORDER: [Graha; 7] = [
    Graha::Saturn,
    Graha::Jupiter,
    Graha::Mars,
    Graha::Sun,
    Graha::Venus,
    Graha::Mercury,
    Graha::Moon,
];

/// Who rules each triplicity.
///
/// A sign's triplicity is its element: fire (Aries, Leo, Sagittarius),
/// earth, air and water, in that order from Aries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Triplicities {
    /// *Tetrabiblos* I.XXI: fire the Sun by day and Jupiter by night;
    /// earth Venus and the Moon; air Saturn and Mercury; water Venus by day
    /// and the Moon by night, "together with Mars", who rules it in both.
    Ptolemy,
    /// Lilly, p. 104 and p. 102: Ptolemy's, but for water, which Mars
    /// "night and day ruleth" alone.
    Lilly,
}

impl Triplicities {
    /// Whether a planet rules the triplicity a sign belongs to, in a chart
    /// of the sect.
    #[must_use]
    pub fn rules(self, planet: Graha, sign: Rashi, sect: Sect) -> bool {
        use Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};
        let (day, night, both) = match (sign.id() % 4, self) {
            (0, _) => (Sun, Jupiter, None),
            (1, _) => (Venus, Moon, None),
            (2, _) => (Saturn, Mercury, None),
            (_, Triplicities::Ptolemy) => (Venus, Moon, Some(Mars)),
            (_, Triplicities::Lilly) => (Mars, Mars, None),
        };
        let by_sect = match sect {
            Sect::Day => day,
            Sect::Night => night,
        };
        planet == by_sect || both == Some(planet)
    }
}

/// The lord of the face a longitude falls in: the 36 ten-degree decans in
/// the Chaldean order, Mars first in Aries (Lilly, p. 104, every cell of
/// which this reproduces; `terms-measured.md`).
///
/// Not Ptolemy's "proper face" (I.XXVI), which is a planet's aspect to the
/// luminaries and no division of a sign.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "a longitude folded into 0..360 divides by ten into 0..36"
)]
pub fn face_lord(longitude_deg: f64) -> Graha {
    let decan = (longitude_deg.rem_euclid(360.0) / 10.0) as usize % 36;
    CHALDEAN_ORDER
        .iter()
        .cycle()
        .nth(decan + 2)
        .copied()
        .unwrap_or(Graha::Mars)
}

/// Lilly's exaltations with their degrees (p. 104).
///
/// Lilly counts exaltation through the whole sign (p. 102), so no dignity
/// reads the degree; it is here to be shown. It is **not** the catalogue's
/// `Graha::attributes().exaltation`, whose degrees are the Vedic deep
/// exaltations: the two differ for the Sun, Jupiter and Saturn.
#[must_use]
pub fn exaltation_degree(planet: Graha) -> Option<(Rashi, u8)> {
    match planet {
        Graha::Sun => Some((Rashi::Aries, 19)),
        Graha::Moon => Some((Rashi::Taurus, 3)),
        Graha::Jupiter => Some((Rashi::Cancer, 15)),
        Graha::Mercury => Some((Rashi::Virgo, 15)),
        Graha::Saturn => Some((Rashi::Libra, 21)),
        Graha::Mars => Some((Rashi::Capricorn, 28)),
        Graha::Venus => Some((Rashi::Pisces, 27)),
        _ => None,
    }
}

/// What a reading of the dignities chooses.
///
/// The default is [`DignityRules::LILLY`], and a record that names one
/// member takes the other from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
#[non_exhaustive]
pub struct DignityRules {
    /// The system of terms.
    pub terms: Terms,
    /// Who rules each triplicity.
    pub triplicities: Triplicities,
}

impl Default for DignityRules {
    fn default() -> DignityRules {
        DignityRules::LILLY
    }
}

impl DignityRules {
    /// Lilly's whole reading: his terms and his triplicities.
    pub const LILLY: DignityRules = DignityRules {
        terms: Terms::PtolemaicLilly,
        triplicities: Triplicities::Lilly,
    };

    /// Ptolemy's triplicities with the terms he transmits from the
    /// Egyptians, the only table his text certifies by its totals.
    pub const PTOLEMY_EGYPTIAN: DignityRules = DignityRules {
        terms: Terms::Egyptian,
        triplicities: Triplicities::Ptolemy,
    };

    /// These rules with another system of terms.
    #[must_use]
    pub const fn with_terms(mut self, terms: Terms) -> DignityRules {
        self.terms = terms;
        self
    }

    /// These rules with another triplicity scheme.
    #[must_use]
    pub const fn with_triplicities(mut self, triplicities: Triplicities) -> DignityRules {
        self.triplicities = triplicities;
        self
    }
}

/// What each dignity and debility is worth.
///
/// The default is [`Scores::LILLY`], and a record that names some scores
/// takes the rest from it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(default)]
#[non_exhaustive]
pub struct Scores {
    /// In its own house.
    pub house: i8,
    /// In its exaltation.
    pub exaltation: i8,
    /// In its triplicity.
    pub triplicity: i8,
    /// In its own term.
    pub term: i8,
    /// In its own face.
    pub face: i8,
    /// In its detriment.
    pub detriment: i8,
    /// In its fall.
    pub fall: i8,
    /// Peregrine: in none of its five dignities.
    pub peregrine: i8,
}

impl Default for Scores {
    fn default() -> Scores {
        Scores::LILLY
    }
}

impl Scores {
    /// Lilly's "ready Table ... of the Fortitudes and Debilities"
    /// (p. 115): house 5, exaltation 4, triplicity 3, term 2, face 1;
    /// detriment −5, fall −4, peregrine −5.
    pub const LILLY: Scores = Scores {
        house: 5,
        exaltation: 4,
        triplicity: 3,
        term: 2,
        face: 1,
        detriment: -5,
        fall: -4,
        peregrine: -5,
    };

    /// Scores of the caller's own.
    #[must_use]
    #[expect(
        clippy::too_many_arguments,
        reason = "one argument a dignity, named in order"
    )]
    pub const fn new(
        house: i8,
        exaltation: i8,
        triplicity: i8,
        term: i8,
        face: i8,
        detriment: i8,
        fall: i8,
        peregrine: i8,
    ) -> Scores {
        Scores {
            house,
            exaltation,
            triplicity,
            term,
            face,
            detriment,
            fall,
            peregrine,
        }
    }
}

/// The dignities and debilities a planet holds at a longitude.
///
/// Several can hold at once: Mars at 5° Aries is in its house and its
/// face. A score is one reading of these flags, by [`Scores`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
#[expect(
    clippy::struct_excessive_bools,
    reason = "one flag a dignity, each independent"
)]
pub struct EssentialDignity {
    /// The sign is its house.
    pub house: bool,
    /// The sign is its exaltation.
    pub exaltation: bool,
    /// It rules the sign's triplicity in a chart of this sect.
    pub triplicity: bool,
    /// The degree lies in its own term.
    pub term: bool,
    /// The degree lies in its own face.
    pub face: bool,
    /// The sign is opposite its house.
    pub detriment: bool,
    /// The sign is opposite its exaltation.
    pub fall: bool,
}

impl EssentialDignity {
    /// In none of its five dignities (Lilly's "peregrine"), whatever its
    /// debilities.
    #[must_use]
    pub const fn peregrine(&self) -> bool {
        !(self.house || self.exaltation || self.triplicity || self.term || self.face)
    }

    /// The dignities' worth by `scores`, each that holds counted once and
    /// peregrine counted as well as any debility.
    #[must_use]
    pub fn score(&self, scores: &Scores) -> i16 {
        [
            (self.house, scores.house),
            (self.exaltation, scores.exaltation),
            (self.triplicity, scores.triplicity),
            (self.term, scores.term),
            (self.face, scores.face),
            (self.detriment, scores.detriment),
            (self.fall, scores.fall),
            (self.peregrine(), scores.peregrine),
        ]
        .into_iter()
        .filter(|(holds, _)| *holds)
        .map(|(_, worth)| i16::from(worth))
        .sum()
    }
}

/// A planet's essential dignities at a longitude in a chart of the sect.
///
/// The longitude is in the chart's own zodiac, tropical or sidereal; the
/// tables do not care which. Only the seven planets hold dignity, and only
/// the five take terms: the Sun and the Moon are never in their own term.
///
/// # Errors
///
/// An invalid-argument error at `planet` for a body that is not one of the
/// seven, and at `longitude` for one that is not a number.
///
/// ```
/// use teistro_hellenistic::{DignityRules, Scores, Sect, essential_dignity};
/// use teistro_core::catalogue::Graha;
///
/// // Lilly, p. 102: the Sun in Aries by night has his exaltation, and no
/// // triplicity, "for by night the Sun ruleth not the fiery Triplicity".
/// // At 5° he is in Mars's face, so only the exaltation counts.
/// let sun = essential_dignity(Graha::Sun, 5.0, Sect::Night, &DignityRules::LILLY)?;
/// assert!(sun.exaltation && !sun.triplicity);
/// assert_eq!(sun.score(&Scores::LILLY), 4);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub fn essential_dignity(
    planet: Graha,
    longitude_deg: f64,
    sect: Sect,
    rules: &DignityRules,
) -> Result<EssentialDignity, Error> {
    if !CHALDEAN_ORDER.contains(&planet) {
        return Err(Error::invalid_arg(format!(
            "{planet:?} holds no essential dignity"
        ))
        .with_field("planet")
        .with_hint("the seven planets do: Saturn, Jupiter, Mars, the Sun, Venus, Mercury, the Moon"));
    }
    if !longitude_deg.is_finite() {
        return Err(
            Error::invalid_arg(format!("{longitude_deg} is not a longitude"))
                .with_field("longitude"),
        );
    }
    let sign = Rashi::of_longitude(longitude_deg);
    let exalted_in = planet.attributes().exaltation.map(|at| at.sign);
    Ok(EssentialDignity {
        house: sign.attributes().lord == planet,
        exaltation: exalted_in == Some(sign),
        triplicity: rules.triplicities.rules(planet, sign, sect),
        term: rules.terms.table(sect).lord_at(longitude_deg) == Some(planet),
        face: face_lord(longitude_deg) == planet,
        detriment: sign.opposite().attributes().lord == planet,
        fall: exalted_in == Some(sign.opposite()),
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{
        CHALDEAN_ORDER, DignityRules, Graha, Rashi, Scores, Sect, SectRule, Triplicities,
        essential_dignity, exaltation_degree, face_lord,
    };
    use crate::Terms;

    #[test]
    fn the_horizon_is_the_suns_centre() {
        assert_eq!(Sect::from_altitude(45.0), Sect::Day);
        assert_eq!(Sect::from_altitude(1e-9), Sect::Day);
        assert_eq!(Sect::from_altitude(0.0), Sect::Night);
        assert_eq!(Sect::from_altitude(-18.0), Sect::Night);
    }

    #[test]
    fn each_sect_rule_reads_what_it_names() {
        for (altitude, daylight) in [(30.0, true), (-30.0, false), (-0.3, true), (0.3, false)] {
            assert_eq!(
                SectRule::Horizon.sect(altitude, daylight),
                Sect::from_altitude(altitude)
            );
            let lit = if daylight { Sect::Day } else { Sect::Night };
            assert_eq!(SectRule::Daylight.sect(altitude, daylight), lit);
            assert_eq!(SectRule::Day.sect(altitude, daylight), Sect::Day);
            assert_eq!(SectRule::Night.sect(altitude, daylight), Sect::Night);
        }
        assert_eq!(SectRule::default(), SectRule::Horizon);
        assert_eq!(
            serde_json::to_string(&SectRule::Daylight).unwrap(),
            "\"DAYLIGHT\""
        );
    }

    fn at(planet: Graha, longitude: f64, sect: Sect) -> super::EssentialDignity {
        essential_dignity(planet, longitude, sect, &DignityRules::LILLY).unwrap()
    }

    /// Lilly's worked cases, p. 102 and p. 103.
    #[test]
    fn lillys_worked_cases_hold() {
        let sun = at(Graha::Sun, 10.0, Sect::Night);
        assert!(
            sun.exaltation && !sun.triplicity,
            "the Sun in Aries by night"
        );
        let jupiter = at(Graha::Jupiter, 3.0, Sect::Night);
        assert!(
            jupiter.term && jupiter.triplicity,
            "Jupiter in Aries 0-6 by night"
        );
        let mars = at(Graha::Mars, 5.0, Sect::Day);
        assert!(mars.house && mars.face, "Mars in his house and face");
        assert_eq!(mars.score(&Scores::LILLY), 6);
    }

    /// p. 102: Mars "night and day ruleth the watry Triplicity", in Lilly's
    /// scheme and not in Ptolemy's, where Venus rules it by day.
    #[test]
    fn the_triplicity_schemes_part_only_in_the_water_signs() {
        for sign in Rashi::ALL {
            for sect in [Sect::Day, Sect::Night] {
                for planet in CHALDEAN_ORDER {
                    let ptolemy = Triplicities::Ptolemy.rules(planet, sign, sect);
                    let lilly = Triplicities::Lilly.rules(planet, sign, sect);
                    let water = matches!(sign, Rashi::Cancer | Rashi::Scorpio | Rashi::Pisces);
                    if !water {
                        assert_eq!(ptolemy, lilly, "{planet:?} in {sign:?} by {sect:?}");
                    }
                }
            }
        }
        assert!(Triplicities::Lilly.rules(Graha::Mars, Rashi::Pisces, Sect::Night));
        assert!(!Triplicities::Lilly.rules(Graha::Venus, Rashi::Pisces, Sect::Day));
        assert!(Triplicities::Ptolemy.rules(Graha::Venus, Rashi::Pisces, Sect::Day));
        assert!(Triplicities::Ptolemy.rules(Graha::Mars, Rashi::Cancer, Sect::Day));
    }

    /// Lilly's faces column, p. 104, the first and last signs.
    #[test]
    fn the_faces_are_the_chaldean_decans() {
        assert_eq!(
            [5.0, 15.0, 25.0].map(face_lord),
            [Graha::Mars, Graha::Sun, Graha::Venus]
        );
        assert_eq!(
            [335.0, 345.0, 355.0].map(face_lord),
            [Graha::Saturn, Graha::Jupiter, Graha::Mars]
        );
        assert_eq!(face_lord(-5.0), Graha::Mars);
    }

    /// Every planet is in detriment exactly where it is opposite its own
    /// house, and falls exactly opposite its exaltation, over the whole
    /// zodiac.
    #[test]
    fn detriment_and_fall_are_the_opposites_of_house_and_exaltation() {
        for planet in CHALDEAN_ORDER {
            for sign in Rashi::ALL {
                let longitude = 30.0 * f64::from(sign.id()) + 15.0;
                let here = at(planet, longitude, Sect::Day);
                let there = at(planet, (longitude + 180.0) % 360.0, Sect::Day);
                assert_eq!(here.detriment, there.house, "{planet:?} in {sign:?}");
                assert_eq!(here.fall, there.exaltation, "{planet:?} in {sign:?}");
            }
            let (sign, _) = exaltation_degree(planet).unwrap();
            assert!(at(planet, 30.0 * f64::from(sign.id()) + 1.0, Sect::Day).exaltation);
        }
    }

    #[test]
    fn a_planet_with_no_dignity_is_peregrine_and_scored_for_it() {
        // The Sun at 25° Libra by day: Libra is Venus's, his fall, Saturn
        // rules air by day, the term is Mars's and the face Jupiter's.
        let sun = at(Graha::Sun, 205.0, Sect::Day);
        assert!(sun.fall && sun.peregrine());
        assert_eq!(sun.score(&Scores::LILLY), -9);
    }

    #[test]
    fn the_luminaries_take_no_term_under_any_system() {
        for terms in [Terms::Egyptian, Terms::PtolemaicAshmand, Terms::Chaldean] {
            let rules = DignityRules::LILLY.with_terms(terms);
            for degree in 0..360 {
                for planet in [Graha::Sun, Graha::Moon] {
                    let dignity =
                        essential_dignity(planet, f64::from(degree), Sect::Night, &rules).unwrap();
                    assert!(!dignity.term, "{planet:?} at {degree}");
                }
            }
        }
    }

    #[test]
    fn a_body_without_dignity_and_a_longitude_that_is_not_one_are_refused_by_name() {
        let rules = DignityRules::LILLY;
        let node = essential_dignity(Graha::Rahu, 10.0, Sect::Day, &rules).unwrap_err();
        assert_eq!(node.field(), Some("planet"));
        let nan = essential_dignity(Graha::Mars, f64::NAN, Sect::Day, &rules).unwrap_err();
        assert_eq!(nan.field(), Some("longitude"));
    }

    #[test]
    fn rules_read_and_write_by_name() {
        let json = serde_json::to_string(&DignityRules::LILLY).unwrap();
        assert_eq!(
            json,
            r#"{"terms":"PTOLEMAIC_LILLY","triplicities":"LILLY"}"#
        );
        let back: DignityRules = serde_json::from_str(&json).unwrap();
        assert_eq!(back, DignityRules::LILLY);
    }
}
