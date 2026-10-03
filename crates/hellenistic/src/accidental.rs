//! A planet's accidental fortitudes and debilities: Lilly's "ready Table"
//! (p. 115, second half), read from where it stands in the figure, how it
//! moves, and what it meets (`03-design/essential-dignities.md`
//! §Accidental fortitudes).
//!
//! Like the essential dignities this is a reading of rules, so there is no
//! ephemeris here: the caller gives each planet's longitude and daily
//! motion, the house cusps, the North Node and the three stars' places of
//! date, and every orb Lilly states is a knob of [`AccidentalRules`].

use core::cmp::Ordering;

use serde::{Deserialize, Serialize};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Graha, HouseSystem, Rashi};
use teistro_core::error::Error;
use teistro_core::house::{House, house_of};

use crate::dignity::CHALDEAN_ORDER;

/// One line of Lilly's table that a planet meets, apart from its house.
///
/// Whether a line is a fortitude or a debility, and its worth, is the
/// [`AccidentalScores`]' to say; orientality is a fortitude for Saturn,
/// Jupiter and Mars and a debility for Venus and Mercury.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Accident {
    /// Moving forward. Void for the Sun and the Moon, "alwayes so".
    Direct,
    /// Moving backward.
    Retrograde,
    /// Faster than its mean motion.
    Swift,
    /// Slower than its mean motion.
    Slow,
    /// Rising before the Sun: behind it in longitude, from their
    /// conjunction to the opposition (p. 114). Not read for the Moon.
    Oriental,
    /// Setting after the Sun: ahead of it in longitude.
    Occidental,
    /// The Moon from her conjunction with the Sun to the opposition:
    /// "encreasing, or when she is Occidentall".
    Increasing,
    /// The Moon from the opposition to the conjunction.
    Decreasing,
    /// Clear of the Sun: neither combust, under his beams nor in cazimi.
    FreeFromCombustion,
    /// "In the heart of the Sun", within [`AccidentalRules::cazimi_deg`].
    Cazimi,
    /// Within [`AccidentalRules::combustion_deg`] of the Sun.
    Combust,
    /// Within [`AccidentalRules::beams_deg`] of the Sun, not combust.
    UnderBeams,
    /// In partile conjunction with Jupiter or Venus.
    ConjunctBenefic,
    /// In partile conjunction with the North Node.
    ConjunctNorthNode,
    /// In partile trine to Jupiter or Venus.
    TrineBenefic,
    /// In partile sextile to Jupiter or Venus.
    SextileBenefic,
    /// In partile conjunction with Saturn or Mars.
    ConjunctMalefic,
    /// In partile conjunction with the South Node.
    ConjunctSouthNode,
    /// In partile opposition to Saturn or Mars.
    OpposedMalefic,
    /// In partile square to Saturn or Mars.
    SquareMalefic,
    /// Between the bodies of Saturn and Mars (p. 114).
    Besieged,
    /// With Cor Leonis (Regulus).
    Regulus,
    /// With Spica.
    Spica,
    /// With Caput Algol.
    Algol,
}

/// When two planets are in partile aspect (crux C216).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", rename_all_fields = "camelCase")]
#[non_exhaustive]
pub enum Partile {
    /// Lilly's (p. 106): "exactly so many degrees" apart, as Venus in 9
    /// Aries and Jupiter in 9 Leo, the same degree of signs the aspect
    /// apart.
    SameDegree,
    /// Within an orb of the exact aspect, in degrees.
    Within {
        /// The orb.
        orb_deg: f64,
    },
}

/// When a planet is besieged by Saturn and Mars (crux C215).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE", rename_all_fields = "camelCase")]
#[non_exhaustive]
pub enum Siege {
    /// Lilly's example (p. 114): all three in one sign, the planet between
    /// the two bodies (Saturn 15 Aries, Mars 10 Aries, Venus 13 Aries).
    SameSign,
    /// The planet on the shorter arc between the two bodies, the arc no
    /// wider than a span, in degrees.
    Within {
        /// The widest arc between Saturn and Mars that still besieges.
        span_deg: f64,
    },
}

/// The orbs and limits the accidental fortitudes are read with.
///
/// [`AccidentalRules::LILLY`] is Lilly's text; every field is a knob.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
#[non_exhaustive]
pub struct AccidentalRules {
    /// The Sun's half-orb for combustion, in degrees.
    pub combustion_deg: f64,
    /// Whether a planet is combust only "in the same Signe where the Sun
    /// is" (crux C211).
    pub combustion_in_sign: bool,
    /// How far from the Sun a planet is still under his beams, in degrees
    /// (crux C212).
    pub beams_deg: f64,
    /// How far from the Sun a planet is in cazimi, in degrees.
    pub cazimi_deg: f64,
    /// How far before or after a cusp a planet is counted in that house,
    /// the nearer cusp winning; zero places it by the cusps alone (crux
    /// C214).
    pub cusp_orb_deg: f64,
    /// How far from a fixed star a planet is with it, in degrees (crux
    /// C213).
    pub star_orb_deg: f64,
    /// When an aspect is partile.
    pub partile: Partile,
    /// When a planet is besieged.
    pub siege: Siege,
    /// Each planet's mean daily motion, in degrees, in the Chaldean order,
    /// which swift and slow are read against.
    pub mean_motion_deg: [f64; 7],
}

impl Default for AccidentalRules {
    fn default() -> AccidentalRules {
        AccidentalRules::LILLY
    }
}

/// Degrees from degrees, minutes and seconds.
const fn dms(degrees: f64, minutes: f64, seconds: f64) -> f64 {
    degrees + minutes / 60.0 + seconds / 3600.0
}

impl AccidentalRules {
    /// Lilly's (pp. 113–114, 115, and each planet's chapter): combust
    /// within 8°30′ of the Sun in his sign, under the beams to 17°, cazimi
    /// within 17′; a planet within five degrees of a cusp in the nearer
    /// house; five degrees about a fixed star; partile by the same degree;
    /// besieged in one sign; and his mean motions, Saturn 2′01″, Jupiter
    /// 4′59″, Mars 31′27″, the Sun, Venus and Mercury 59′08″, the Moon
    /// 13°10′36″.
    pub const LILLY: AccidentalRules = AccidentalRules {
        combustion_deg: 8.5,
        combustion_in_sign: true,
        beams_deg: 17.0,
        cazimi_deg: 17.0 / 60.0,
        cusp_orb_deg: 5.0,
        star_orb_deg: 5.0,
        partile: Partile::SameDegree,
        siege: Siege::SameSign,
        mean_motion_deg: [
            dms(0.0, 2.0, 1.0),
            dms(0.0, 4.0, 59.0),
            dms(0.0, 31.0, 27.0),
            dms(0.0, 59.0, 8.0),
            dms(0.0, 59.0, 8.0),
            dms(0.0, 59.0, 8.0),
            dms(13.0, 10.0, 36.0),
        ],
    };

    /// The same rules, the five-degree rule set aside or widened.
    #[must_use]
    pub const fn with_cusp_orb(mut self, cusp_orb_deg: f64) -> AccidentalRules {
        self.cusp_orb_deg = cusp_orb_deg;
        self
    }

    /// The same rules, with the Sun's orbs changed: combustion, beams and
    /// cazimi, in degrees.
    #[must_use]
    pub const fn with_solar_orbs(
        mut self,
        combustion_deg: f64,
        beams_deg: f64,
        cazimi_deg: f64,
    ) -> AccidentalRules {
        self.combustion_deg = combustion_deg;
        self.beams_deg = beams_deg;
        self.cazimi_deg = cazimi_deg;
        self
    }

    /// The same rules, partile read otherwise.
    #[must_use]
    pub const fn with_partile(mut self, partile: Partile) -> AccidentalRules {
        self.partile = partile;
        self
    }

    /// The same rules, besieging read otherwise.
    #[must_use]
    pub const fn with_siege(mut self, siege: Siege) -> AccidentalRules {
        self.siege = siege;
        self
    }

    /// Every orb a finite, non-negative number and every mean motion a
    /// positive one, naming the field that is not.
    pub(crate) fn check(&self) -> Result<(), Error> {
        let partile = match self.partile {
            Partile::SameDegree => 0.0,
            Partile::Within { orb_deg } => orb_deg,
        };
        let siege = match self.siege {
            Siege::SameSign => 0.0,
            Siege::Within { span_deg } => span_deg,
        };
        let orbs = [
            ("combustionDeg", self.combustion_deg),
            ("beamsDeg", self.beams_deg),
            ("cazimiDeg", self.cazimi_deg),
            ("cuspOrbDeg", self.cusp_orb_deg),
            ("starOrbDeg", self.star_orb_deg),
            ("partile.orbDeg", partile),
            ("siege.spanDeg", siege),
        ];
        for (field, orb) in orbs {
            if !(orb.is_finite() && orb >= 0.0) {
                return Err(Error::invalid_arg(format!("an orb of {orb}"))
                    .with_field(field)
                    .with_hint("an orb is a finite number of degrees, zero or more"));
            }
        }
        if let Some(mean) = self
            .mean_motion_deg
            .iter()
            .find(|mean| !(mean.is_finite() && **mean > 0.0))
        {
            return Err(Error::invalid_arg(format!("a mean motion of {mean}"))
                .with_field("meanMotionDeg")
                .with_hint("each planet's mean daily motion is a positive number of degrees"));
        }
        Ok(())
    }
}

/// What each of Lilly's lines is worth: positive for a fortitude,
/// negative for a debility.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
#[allow(missing_docs, reason = "each field is the line of the same name")]
pub struct AccidentalScores {
    /// The houses, first to twelfth.
    pub houses: [i8; 12],
    pub direct: i8,
    pub retrograde: i8,
    pub swift: i8,
    pub slow: i8,
    /// Saturn, Jupiter or Mars oriental.
    pub superior_oriental: i8,
    /// Saturn, Jupiter or Mars occidental.
    pub superior_occidental: i8,
    /// Venus or Mercury oriental.
    pub inferior_oriental: i8,
    /// Venus or Mercury occidental.
    pub inferior_occidental: i8,
    pub increasing: i8,
    pub decreasing: i8,
    pub free_from_combustion: i8,
    pub cazimi: i8,
    pub combust: i8,
    pub under_beams: i8,
    pub conjunct_benefic: i8,
    pub conjunct_north_node: i8,
    pub trine_benefic: i8,
    pub sextile_benefic: i8,
    pub conjunct_malefic: i8,
    pub conjunct_south_node: i8,
    pub opposed_malefic: i8,
    pub square_malefic: i8,
    pub besieged: i8,
    pub regulus: i8,
    pub spica: i8,
    pub algol: i8,
}

impl Default for AccidentalScores {
    fn default() -> AccidentalScores {
        AccidentalScores::LILLY
    }
}

impl AccidentalScores {
    /// Lilly's table (p. 115): the Midheaven or Ascendant 5, the 7th, 4th
    /// and 11th 4, the 2nd and 5th 3, the 9th 2, the 3rd 1, the 12th −5, the
    /// 8th and 6th −2; and each line as printed.
    pub const LILLY: AccidentalScores = AccidentalScores {
        houses: [5, 3, 1, 4, 3, -2, 4, -2, 2, 5, 4, -5],
        direct: 4,
        retrograde: -5,
        swift: 2,
        slow: -2,
        superior_oriental: 2,
        superior_occidental: -2,
        inferior_oriental: -2,
        inferior_occidental: 2,
        increasing: 2,
        decreasing: -2,
        free_from_combustion: 5,
        cazimi: 5,
        combust: -5,
        under_beams: -4,
        conjunct_benefic: 5,
        conjunct_north_node: 4,
        trine_benefic: 4,
        sextile_benefic: 3,
        conjunct_malefic: -5,
        conjunct_south_node: -4,
        opposed_malefic: -4,
        square_malefic: -3,
        besieged: -5,
        regulus: 6,
        spica: 5,
        algol: -5,
    };

    /// What a house is worth.
    #[must_use]
    pub fn house(&self, house: House) -> i8 {
        self.houses
            .get(usize::from(house.get() - 1))
            .copied()
            .unwrap_or(0)
    }

    /// What a line is worth to a planet.
    #[must_use]
    pub const fn points(&self, planet: Graha, accident: Accident) -> i8 {
        let superior = matches!(planet, Graha::Saturn | Graha::Jupiter | Graha::Mars);
        match accident {
            Accident::Direct => self.direct,
            Accident::Retrograde => self.retrograde,
            Accident::Swift => self.swift,
            Accident::Slow => self.slow,
            Accident::Oriental if superior => self.superior_oriental,
            Accident::Oriental => self.inferior_oriental,
            Accident::Occidental if superior => self.superior_occidental,
            Accident::Occidental => self.inferior_occidental,
            Accident::Increasing => self.increasing,
            Accident::Decreasing => self.decreasing,
            Accident::FreeFromCombustion => self.free_from_combustion,
            Accident::Cazimi => self.cazimi,
            Accident::Combust => self.combust,
            Accident::UnderBeams => self.under_beams,
            Accident::ConjunctBenefic => self.conjunct_benefic,
            Accident::ConjunctNorthNode => self.conjunct_north_node,
            Accident::TrineBenefic => self.trine_benefic,
            Accident::SextileBenefic => self.sextile_benefic,
            Accident::ConjunctMalefic => self.conjunct_malefic,
            Accident::ConjunctSouthNode => self.conjunct_south_node,
            Accident::OpposedMalefic => self.opposed_malefic,
            Accident::SquareMalefic => self.square_malefic,
            Accident::Besieged => self.besieged,
            Accident::Regulus => self.regulus,
            Accident::Spica => self.spica,
            Accident::Algol => self.algol,
        }
    }
}

/// Where a planet is and how fast it moves.
#[derive(Clone, Copy, Debug)]
struct Motion {
    longitude_deg: f64,
    speed_deg_per_day: f64,
}

/// What a chart's accidental fortitudes are read from beside the seven's
/// longitudes: how they move, the houses, the Nodes and three stars, all
/// in the chart's zodiac.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AccidentalSky {
    /// The seven's daily motions in longitude, in degrees, in the Chaldean
    /// order, Saturn first; negative when retrograde.
    pub speeds_deg_per_day: [f64; 7],
    /// The division the cusps are of: Lilly's is Regiomontanus.
    pub houses: HouseSystem,
    /// The twelve house cusps, first to twelfth, in degrees.
    pub cusps_deg: [f64; 12],
    /// The ascendant, from the chart's angles: whole-sign and equal houses
    /// do not put it on a cusp.
    pub ascendant_deg: f64,
    /// The midheaven, from the chart's angles: only a quadrant division
    /// puts it on the tenth cusp.
    pub midheaven_deg: f64,
    /// The North Node's longitude; the South Node is opposite.
    pub north_node_deg: f64,
    /// Regulus's longitude of date.
    pub regulus_deg: f64,
    /// Spica's longitude of date.
    pub spica_deg: f64,
    /// Algol's longitude of date.
    pub algol_deg: f64,
}

/// One planet's accidental fortitudes and debilities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PlanetAccidents {
    /// The planet.
    pub planet: Graha,
    /// The house it is counted in, by the five-degree rule.
    pub house: House,
    /// Every other line it meets, in the order of [`Accident`].
    pub accidents: Vec<Accident>,
    /// The sum of its fortitudes, house included.
    pub fortitude: i16,
    /// The sum of its debilities, house included, as a positive number,
    /// the way Lilly prints it.
    pub debility: i16,
}

/// Whether two longitudes are in partile aspect of `angle_deg`.
fn partile(rule: Partile, a: f64, b: f64, angle_deg: f64) -> bool {
    match rule {
        Partile::SameDegree => {
            let sign = |at: f64| (at.rem_euclid(360.0) / 30.0).floor();
            let degree = |at: f64| at.rem_euclid(30.0).floor();
            let signs = (sign(a) - sign(b)).abs();
            let apart = signs.min(12.0 - signs) * 30.0;
            degree(a).total_cmp(&degree(b)).is_eq() && apart.total_cmp(&angle_deg).is_eq()
        }
        Partile::Within { orb_deg } => (difference_deg(a, b).abs() - angle_deg).abs() <= orb_deg,
    }
}

/// Whether `at` lies between Saturn's and Mars's bodies.
fn besieged(rule: Siege, at: f64, saturn: f64, mars: f64) -> bool {
    match rule {
        Siege::SameSign => {
            let sign = Rashi::of_longitude(at);
            sign == Rashi::of_longitude(saturn)
                && sign == Rashi::of_longitude(mars)
                && saturn.min(mars) < at
                && at < saturn.max(mars)
        }
        Siege::Within { span_deg } => {
            let arc = difference_deg(mars, saturn);
            let from_saturn = difference_deg(at, saturn);
            arc.abs() <= span_deg && from_saturn * arc > 0.0 && from_saturn.abs() < arc.abs()
        }
    }
}

/// Every reading of a planet that is not its house.
struct Reading<'a> {
    rules: &'a AccidentalRules,
    sky: &'a AccidentalSky,
    sun: f64,
    saturn: f64,
    mars: f64,
    benefics: [(Graha, f64); 2],
}

impl Reading<'_> {
    fn accidents(&self, planet: Graha, at: Motion, mean_deg: f64) -> Vec<Accident> {
        let mut found = Vec::new();
        let luminary = matches!(planet, Graha::Sun | Graha::Moon);
        if !luminary {
            found.push(if at.speed_deg_per_day < 0.0 {
                Accident::Retrograde
            } else {
                Accident::Direct
            });
        }
        let pace = at.speed_deg_per_day.abs();
        if pace > mean_deg {
            found.push(Accident::Swift);
        } else if pace < mean_deg {
            found.push(Accident::Slow);
        }
        if planet != Graha::Sun {
            let from_sun = difference_deg(at.longitude_deg, self.sun);
            found.extend(Self::phase(planet, from_sun));
            found.push(self.solar(at.longitude_deg, from_sun.abs()));
        }
        found.extend(self.aspects(planet, at.longitude_deg));
        if planet != Graha::Saturn
            && planet != Graha::Mars
            && besieged(self.rules.siege, at.longitude_deg, self.saturn, self.mars)
        {
            found.push(Accident::Besieged);
        }
        let stars = [
            (Accident::Regulus, self.sky.regulus_deg),
            (Accident::Spica, self.sky.spica_deg),
            (Accident::Algol, self.sky.algol_deg),
        ];
        found.extend(stars.into_iter().filter_map(|(star, place)| {
            (difference_deg(at.longitude_deg, place).abs() <= self.rules.star_orb_deg)
                .then_some(star)
        }));
        found.sort_by_key(|&accident| accident as u8);
        found
    }

    /// Oriental or occidental, or for the Moon increasing or decreasing,
    /// from the planet's elongation east of the Sun.
    fn phase(planet: Graha, from_sun: f64) -> Option<Accident> {
        let moon = planet == Graha::Moon;
        match from_sun.total_cmp(&0.0) {
            Ordering::Greater if moon => Some(Accident::Increasing),
            Ordering::Greater => Some(Accident::Occidental),
            Ordering::Less if moon => Some(Accident::Decreasing),
            Ordering::Less => Some(Accident::Oriental),
            Ordering::Equal => None,
        }
    }

    /// Cazimi, combust, under the beams or free, from the arc to the Sun.
    fn solar(&self, at: f64, arc: f64) -> Accident {
        let rules = self.rules;
        let in_sign =
            !rules.combustion_in_sign || Rashi::of_longitude(at) == Rashi::of_longitude(self.sun);
        if arc <= rules.cazimi_deg {
            Accident::Cazimi
        } else if arc <= rules.combustion_deg && in_sign {
            Accident::Combust
        } else if arc <= rules.beams_deg {
            Accident::UnderBeams
        } else {
            Accident::FreeFromCombustion
        }
    }

    /// The partile lines: with Jupiter and Venus, Saturn and Mars, and the
    /// Nodes, each line once however many bodies hold it.
    fn aspects(&self, planet: Graha, at: f64) -> Vec<Accident> {
        let rule = self.rules.partile;
        let with = |bodies: &[(Graha, f64)], angle: f64| {
            bodies
                .iter()
                .any(|&(body, place)| body != planet && partile(rule, at, place, angle))
        };
        let malefics = [(Graha::Saturn, self.saturn), (Graha::Mars, self.mars)];
        let node = self.sky.north_node_deg;
        [
            (Accident::ConjunctBenefic, with(&self.benefics, 0.0)),
            (Accident::ConjunctNorthNode, partile(rule, at, node, 0.0)),
            (Accident::TrineBenefic, with(&self.benefics, 120.0)),
            (Accident::SextileBenefic, with(&self.benefics, 60.0)),
            (Accident::ConjunctMalefic, with(&malefics, 0.0)),
            (
                Accident::ConjunctSouthNode,
                partile(rule, at, node + 180.0, 0.0),
            ),
            (Accident::OpposedMalefic, with(&malefics, 180.0)),
            (Accident::SquareMalefic, with(&malefics, 90.0)),
        ]
        .into_iter()
        .filter_map(|(accident, holds)| holds.then_some(accident))
        .collect()
    }
}

/// The seven's accidental fortitudes and debilities, in the Chaldean
/// order, from their longitudes in that order and the rest of the sky.
///
/// ```
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_hellenistic::{
///     Accident, AccidentalRules, AccidentalScores, AccidentalSky, accidental_dignities,
/// };
///
/// // Saturn, Jupiter, Mars, the Sun, Venus, Mercury, the Moon.
/// let longitudes = [200.0, 100.0, 300.0, 15.5, 15.42, 40.0, 130.0];
/// let sky = AccidentalSky {
///     speeds_deg_per_day: [0.1, 0.1, 0.6, 1.0, 1.2, 1.5, 13.0],
///     houses: HouseSystem::Equal,
///     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
///     ascendant_deg: 0.0,
///     midheaven_deg: 270.0,
///     north_node_deg: 250.0,
///     regulus_deg: 150.0, spica_deg: 204.0, algol_deg: 56.0,
/// };
/// let read = accidental_dignities(
///     &longitudes, &sky, &AccidentalRules::LILLY, &AccidentalScores::LILLY,
/// )?;
/// let venus = &read[4];
/// assert_eq!(venus.planet, Graha::Venus);
/// assert!(venus.accidents.contains(&Accident::Cazimi));
/// // The first house 5, direct 4, swift 2, cazimi 5; oriental 2.
/// assert_eq!((venus.fortitude, venus.debility), (16, 2));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// `INVALID_ARG` on a longitude, motion or cusp that is not a finite
/// number, or an orb or mean motion out of range, naming its field.
pub fn accidental_dignities(
    longitudes_deg: &[f64; 7],
    sky: &AccidentalSky,
    rules: &AccidentalRules,
    scores: &AccidentalScores,
) -> Result<[PlanetAccidents; 7], Error> {
    rules.check()?;
    check_sky(longitudes_deg, sky)?;
    let [saturn, jupiter, mars, sun, venus, _, _] = *longitudes_deg;
    let reading = Reading {
        rules,
        sky,
        sun,
        saturn,
        mars,
        benefics: [(Graha::Jupiter, jupiter), (Graha::Venus, venus)],
    };
    let planets: Vec<PlanetAccidents> = CHALDEAN_ORDER
        .into_iter()
        .zip(longitudes_deg.iter().zip(sky.speeds_deg_per_day))
        .zip(rules.mean_motion_deg)
        .map(
            |((planet, (&longitude_deg, speed_deg_per_day)), mean_deg)| {
                let at = Motion {
                    longitude_deg,
                    speed_deg_per_day,
                };
                let house = house_of(at.longitude_deg, &sky.cusps_deg, rules.cusp_orb_deg);
                let accidents = reading.accidents(planet, at, mean_deg);
                let points = core::iter::once(scores.house(house))
                    .chain(accidents.iter().map(|&one| scores.points(planet, one)));
                let (fortitude, debility) = points.fold((0_i16, 0_i16), |(up, down), worth| {
                    let worth = i16::from(worth);
                    if worth > 0 {
                        (up + worth, down)
                    } else {
                        (up, down - worth)
                    }
                });
                PlanetAccidents {
                    planet,
                    house,
                    accidents,
                    fortitude,
                    debility,
                }
            },
        )
        .collect();
    planets
        .try_into()
        .map_err(|_| Error::internal("the seven are seven"))
}

/// Every longitude, motion and cusp a finite number, naming the one that
/// is not.
fn check_sky(longitudes_deg: &[f64; 7], sky: &AccidentalSky) -> Result<(), Error> {
    let names = [
        "saturn", "jupiter", "mars", "sun", "venus", "mercury", "moon",
    ];
    let motions = names
        .into_iter()
        .zip(longitudes_deg.iter().zip(sky.speeds_deg_per_day))
        .flat_map(|(name, (&longitude, speed))| {
            [
                (format!("longitudesDeg.{name}"), longitude),
                (format!("speedsDegPerDay.{name}"), speed),
            ]
        });
    let cusps = sky
        .cusps_deg
        .iter()
        .enumerate()
        .map(|(k, &cusp)| (format!("cuspsDeg.{}", k + 1), cusp));
    let points = [
        ("ascendantDeg", sky.ascendant_deg),
        ("midheavenDeg", sky.midheaven_deg),
        ("northNodeDeg", sky.north_node_deg),
        ("regulusDeg", sky.regulus_deg),
        ("spicaDeg", sky.spica_deg),
        ("algolDeg", sky.algol_deg),
    ]
    .map(|(name, value)| (String::from(name), value));
    for (field, value) in motions.chain(cusps).chain(points) {
        if !value.is_finite() {
            return Err(
                Error::invalid_arg(format!("{value} is not a number of degrees")).with_field(field),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own results"
    )]

    use super::{
        Accident, AccidentalRules, AccidentalScores, AccidentalSky, Partile, Siege,
        accidental_dignities, besieged, house_of, partile,
    };
    use crate::{
        CHALDEAN_ORDER, ChartSky, DignityRules, FortitudeRequest, Scores, Sect, essential_dignity,
    };
    use Accident::{
        Cazimi, Combust, ConjunctBenefic, Decreasing, Direct, FreeFromCombustion, Increasing,
        Occidental, Oriental, Regulus, Retrograde, Slow, Spica, Swift, UnderBeams,
    };
    use teistro_core::catalogue::Graha;
    use teistro_core::catalogue::HouseSystem;
    use teistro_core::house::House;

    /// A longitude from a sign (Aries 0) and degrees and minutes in it.
    fn at(sign: u8, degrees: f64, minutes: f64) -> f64 {
        f64::from(sign) * 30.0 + degrees + minutes / 60.0
    }

    /// Twelve houses of 30° from 0° Aries.
    fn equal() -> [f64; 12] {
        House::ALL.map(|house| f64::from(house.get() - 1) * 30.0)
    }

    /// A star's longitude of date from its J2000 ecliptic longitude, by
    /// general precession, 50.29″ a year: near enough for a five-degree
    /// orb, every margin below being more than half a degree.
    fn star_of(j2000_deg: f64, year: f64) -> f64 {
        j2000_deg - (2000.0 - year) * 50.29 / 3600.0
    }

    /// A figure: the seven's longitudes and their sky.
    struct Figure {
        longitudes: [f64; 7],
        sky: AccidentalSky,
    }

    fn figure(planets: [(f64, f64); 7], cusps: [f64; 12], node: f64, year: f64) -> Figure {
        Figure {
            longitudes: planets.map(|(longitude, _)| longitude),
            sky: AccidentalSky {
                speeds_deg_per_day: planets.map(|(_, speed)| speed),
                houses: HouseSystem::Regiomontanus,
                cusps_deg: cusps,
                ascendant_deg: cusps[0],
                midheaven_deg: cusps[9],
                north_node_deg: node,
                regulus_deg: star_of(149.8298, year),
                spica_deg: star_of(203.8410, year),
                algol_deg: star_of(56.1667, year),
            },
        }
    }

    /// One planet's tally as Lilly prints it, and each cell in which the
    /// stated rules (p. 115, pp. 113–114) read it otherwise, listed.
    struct Printed {
        house: u8,
        house_points: i8,
        accidents: &'static [Accident],
        /// Lines the rules give that the tally does not print.
        unprinted: &'static [Accident],
        /// Lines the tally prints that the rules do not give.
        unscored: &'static [Accident],
        /// The essential score the rules give less the tally's.
        essential_slip: i16,
        /// The tally's net, fortitudes less debilities.
        net: i16,
    }

    /// Reads a figure and holds each planet to its printed tally, cell by
    /// cell, the listed differences apart.
    fn holds(figure: &Figure, rules: &AccidentalRules, sect: Sect, printed: &[Printed; 7]) {
        let scores = AccidentalScores::LILLY;
        let read = accidental_dignities(&figure.longitudes, &figure.sky, rules, &scores).unwrap();
        for ((planet, at), lilly) in CHALDEAN_ORDER.into_iter().zip(&read).zip(printed) {
            assert_eq!(at.planet, planet);
            assert_eq!(at.house.get(), lilly.house, "{planet:?}'s house");
            let mut expected: Vec<Accident> = lilly
                .accidents
                .iter()
                .filter(|one| !lilly.unscored.contains(one))
                .chain(lilly.unprinted)
                .copied()
                .collect();
            expected.sort_by_key(|&one| one as u8);
            assert_eq!(at.accidents, expected, "{planet:?}");
            let worth = |lines: &[Accident]| -> i16 {
                lines
                    .iter()
                    .map(|&one| i16::from(scores.points(planet, one)))
                    .sum()
            };
            let longitude =
                figure.longitudes[CHALDEAN_ORDER.iter().position(|&p| p == planet).unwrap()];
            let essential = essential_dignity(planet, longitude, sect, &DignityRules::LILLY)
                .unwrap()
                .score(&Scores::LILLY);
            let net = essential + at.fortitude - at.debility;
            let house_slip = i16::from(scores.house(at.house)) - i16::from(lilly.house_points);
            assert_eq!(
                net - worth(lilly.unprinted) + worth(lilly.unscored)
                    - house_slip
                    - lilly.essential_slip,
                lilly.net,
                "{planet:?}'s net"
            );
        }
    }

    /// Chapter XXVIII, "If the Querent shall be Rich or Poore": the figure
    /// (p. 177), the motions (p. 178) and the tallies (pp. 178–180), all
    /// read off the page images. A day chart.
    ///
    /// The stated rules give every printed line but these: Jupiter,
    /// 15°39′ behind the Sun in Cancer, is oriental and under the beams;
    /// Mercury (14°35′) and the Moon (15°57′) are under the beams, which
    /// the tally prints as "free from combustion" (C212); and the Sun in
    /// Leo by day holds its fire triplicity, unprinted (C217). With the
    /// stars of date the Moon is 5°36′ short of Regulus, outside the five
    /// degrees, as the tally has it (C213).
    fn rich_or_poor() -> (Figure, [Printed; 7]) {
        let cusps = [
            at(6, 14.0, 13.0),
            at(7, 6.0, 35.0),
            at(8, 7.0, 12.0),
            at(9, 19.0, 0.0),
            at(10, 26.0, 52.0),
            at(11, 23.0, 9.0),
            at(0, 14.0, 13.0),
            at(1, 6.0, 35.0),
            at(2, 7.0, 12.0),
            at(3, 19.0, 0.0),
            at(4, 26.0, 52.0),
            at(5, 23.0, 9.0),
        ];
        let minutes = |m: f64| m / 60.0;
        let sky = figure(
            [
                (at(8, 15.0, 19.0), -minutes(2.0)),
                (at(3, 17.0, 31.0), minutes(13.0)),
                (at(6, 16.0, 12.0), minutes(35.0)),
                (at(4, 3.0, 10.0), minutes(57.0)),
                (at(4, 25.0, 34.0), minutes(73.0)),
                (at(4, 17.0, 45.0), minutes(104.0)),
                (at(4, 19.0, 7.0), minutes(714.0)),
            ],
            cusps,
            at(5, 22.0, 12.0),
            1634.5,
        );
        let printed = [
            Printed {
                house: 3,
                house_points: 1,
                accidents: &[Retrograde, Slow, Occidental, FreeFromCombustion],
                unprinted: &[],
                unscored: &[],
                essential_slip: 0,
                net: -8,
            },
            Printed {
                house: 10,
                house_points: 5,
                accidents: &[Direct, Swift, FreeFromCombustion],
                unprinted: &[Oriental, UnderBeams],
                unscored: &[FreeFromCombustion],
                essential_slip: 0,
                net: 20,
            },
            Printed {
                house: 1,
                house_points: 5,
                accidents: &[Direct, Swift, Occidental, FreeFromCombustion, Spica],
                unprinted: &[],
                unscored: &[],
                essential_slip: 0,
                net: 9,
            },
            Printed {
                house: 10,
                house_points: 5,
                accidents: &[Slow],
                unprinted: &[],
                unscored: &[],
                essential_slip: 3,
                net: 8,
            },
            Printed {
                house: 11,
                house_points: 4,
                accidents: &[Direct, Swift, Occidental, FreeFromCombustion, Regulus],
                unprinted: &[],
                unscored: &[],
                essential_slip: 0,
                net: 18,
            },
            Printed {
                house: 10,
                house_points: 5,
                accidents: &[Direct, Swift, Occidental, FreeFromCombustion],
                unprinted: &[UnderBeams],
                unscored: &[FreeFromCombustion],
                essential_slip: 0,
                net: 13,
            },
            Printed {
                house: 10,
                house_points: 5,
                accidents: &[Slow, Increasing, FreeFromCombustion],
                unprinted: &[UnderBeams],
                unscored: &[FreeFromCombustion],
                essential_slip: 0,
                net: 5,
            },
        ];
        (sky, printed)
    }

    #[test]
    fn the_rich_or_poor_figure_holds_to_its_tallies() {
        let (sky, printed) = rich_or_poor();
        holds(&sky, &AccidentalRules::LILLY, Sect::Day, &printed);
    }

    /// C212: with the beams no wider than combustion, the tally's three
    /// "free from combustion" cells hold, and only Jupiter's unprinted
    /// orientality is left.
    #[test]
    fn without_the_beams_the_rich_or_poor_tally_differs_only_by_an_omission() {
        let (sky, mut printed) = rich_or_poor();
        for lilly in &mut printed {
            lilly.unprinted = if lilly.unprinted.contains(&Oriental) {
                &[Oriental]
            } else {
                &[]
            };
            lilly.unscored = &[];
        }
        let rules = AccidentalRules::LILLY.with_solar_orbs(8.5, 8.5, 17.0 / 60.0);
        holds(&sky, &rules, Sect::Day, &printed);
    }

    /// Book III's English merchant (p. 742): Lilly prints no motions for
    /// it, so each is set to the verdict his table gives.
    fn merchant() -> Figure {
        let cusps = [
            at(9, 6.0, 37.0),
            at(10, 23.0, 30.0),
            at(0, 18.0, 34.0),
            at(1, 14.0, 39.0),
            at(2, 0.0, 0.0),
            at(2, 14.0, 49.0),
            at(3, 6.0, 37.0),
            at(4, 23.0, 30.0),
            at(6, 18.0, 34.0),
            at(7, 14.0, 39.0),
            at(8, 0.0, 0.0),
            at(8, 14.0, 49.0),
        ];
        figure(
            [
                (at(1, 9.0, 2.0), -0.02),
                (at(8, 21.0, 55.0), 0.1),
                (at(4, 0.0, 54.0), 0.6),
                (at(6, 6.0, 37.0), 1.0),
                (at(6, 6.0, 54.0), 1.2),
                (at(6, 3.0, 34.0), 1.5),
                (at(2, 1.0, 44.0), 14.0),
            ],
            cusps,
            at(11, 5.0, 50.0),
            1616.7,
        )
    }

    /// The merchant read through the request, both halves of the table:
    /// each net is the printed one but for the listed cells, the eighth
    /// house's 2 for the Sun and Venus.
    #[test]
    fn the_merchants_nets_through_the_request() {
        let figure = merchant();
        let [
            saturn_deg,
            jupiter_deg,
            mars_deg,
            sun_deg,
            venus_deg,
            mercury_deg,
            moon_deg,
        ] = figure.longitudes;
        let chart = ChartSky {
            saturn_deg,
            jupiter_deg,
            mars_deg,
            sun_deg,
            venus_deg,
            mercury_deg,
            moon_deg,
            sun_altitude_deg: 30.0,
            moon_altitude_deg: 10.0,
            daylight: true,
        };
        let read = FortitudeRequest::default()
            .read(&chart, &figure.sky)
            .unwrap();
        assert_eq!(read.dignities.sect, Sect::Day);
        assert_eq!(read.sky.houses, HouseSystem::Regiomontanus);
        assert_eq!(read.net(Graha::Venus), Some(16 + 2));
        assert_eq!(read.net(Graha::Sun), Some(-6 + 2));
        assert_eq!(read.net(Graha::Rahu), None);
        // Lilly names Venus "Almuten of the Geniture" in his judgment of
        // this nativity: the greatest net, alone.
        let almuten = read.almutens.figure;
        assert_eq!(almuten.almutens(), [Graha::Venus]);
        assert_eq!(almuten.total(Graha::Venus), Some(18));
    }

    /// Book III, Chapter CLXXV, an English merchant born 19 September 1616
    /// at 2h 04m 30s after noon, latitude 53 (p. 742), and the table of
    /// his planets' dignities (pp. 744–745), read off the page images. The
    /// figure prints no motions, so each is set to the verdict the table
    /// gives. A day chart.
    ///
    /// The stated rules give every printed line but these: the tally
    /// charges the eighth house 4 where the table (p. 115) charges 2, for
    /// the Sun, Venus and Mercury; and Mercury, combust at 3°34′ Libra, is
    /// oriental and peregrine by day, neither printed (C217). Venus in
    /// cazimi scores cazimi and not "free from combustion" as well, and the
    /// Sun scores his partile conjunction with her.
    #[test]
    fn the_merchants_nativity_holds_to_its_tallies() {
        let sky = merchant();
        holds(
            &sky,
            &AccidentalRules::LILLY,
            Sect::Day,
            &[
                Printed {
                    house: 3,
                    house_points: 1,
                    accidents: &[Retrograde, Slow, Oriental, FreeFromCombustion],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: -4,
                },
                Printed {
                    house: 12,
                    house_points: -5,
                    accidents: &[Direct, Swift, Occidental, FreeFromCombustion],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: 9,
                },
                Printed {
                    house: 7,
                    house_points: 4,
                    accidents: &[Direct, Swift, Oriental, FreeFromCombustion],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: 12,
                },
                Printed {
                    house: 8,
                    house_points: -4,
                    accidents: &[Swift, ConjunctBenefic],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: -6,
                },
                Printed {
                    house: 8,
                    house_points: -4,
                    accidents: &[Direct, Swift, Occidental, Cazimi],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: 16,
                },
                Printed {
                    house: 8,
                    house_points: -4,
                    accidents: &[Direct, Swift, Combust],
                    unprinted: &[Oriental],
                    unscored: &[],
                    essential_slip: -5,
                    net: -3,
                },
                Printed {
                    house: 5,
                    house_points: 3,
                    accidents: &[Swift, Decreasing, FreeFromCombustion],
                    unprinted: &[],
                    unscored: &[],
                    essential_slip: 0,
                    net: 3,
                },
            ],
        );
    }

    /// p. 113: Jupiter in 10 Aries and the Sun in 18 is combust. The second
    /// example, the Sun in 18 Aries and Jupiter in 28, "here Jupiter is
    /// combust", is ten degrees apart, outside the stated 8°30′: the rules
    /// read it under the beams (C211).
    #[test]
    fn lillys_combustion_examples() {
        let rules = AccidentalRules::LILLY;
        let solar = |planet: f64, sun: f64| {
            let mut planets = [(200.0, 0.1); 7];
            planets[1] = (planet, 0.1);
            planets[3] = (sun, 1.0);
            let sky = figure(planets, equal(), 300.0, 2000.0);
            accidental_dignities(&sky.longitudes, &sky.sky, &rules, &AccidentalScores::LILLY)
                .unwrap()[1]
                .accidents
                .clone()
        };
        assert!(solar(10.0, 18.0).contains(&Combust));
        assert!(solar(28.0, 18.0).contains(&UnderBeams));
        // In the next sign, 3° away: under the beams, not combust.
        assert!(solar(31.0, 28.0).contains(&UnderBeams));
        // p. 113's cazimi: the Sun in 15°30′ Taurus, Mercury in 15°25′.
        assert!(solar(at(1, 15.0, 25.0), at(1, 15.0, 30.0)).contains(&Cazimi));
    }

    /// p. 114: Saturn in 15 Aries, Mars in 10, Venus in 13 is besieged.
    #[test]
    fn lillys_siege() {
        assert!(besieged(Siege::SameSign, 13.0, 15.0, 10.0));
        assert!(!besieged(Siege::SameSign, 16.0, 15.0, 10.0));
        assert!(!besieged(Siege::SameSign, 31.0, 29.0, 35.0));
        let wide = Siege::Within { span_deg: 10.0 };
        assert!(besieged(wide, 31.0, 29.0, 35.0));
        assert!(besieged(wide, 359.0, 355.0, 3.0));
        assert!(!besieged(wide, 10.0, 355.0, 3.0));
    }

    /// p. 106: Venus in 9 Aries and Jupiter in 9 Leo is a partile trine;
    /// the Sun in 1 Taurus and the Moon in 1 Cancer a partile sextile.
    #[test]
    fn lillys_partile_aspects() {
        let same = Partile::SameDegree;
        assert!(partile(same, at(0, 9.0, 10.0), at(4, 9.0, 50.0), 120.0));
        assert!(partile(same, at(1, 1.0, 0.0), at(3, 1.0, 30.0), 60.0));
        assert!(!partile(same, at(0, 9.0, 50.0), at(4, 10.0, 10.0), 120.0));
        let near = Partile::Within { orb_deg: 1.0 };
        assert!(partile(near, at(0, 9.0, 50.0), at(4, 10.0, 10.0), 120.0));
        assert!(partile(same, at(11, 29.0, 0.0), at(5, 29.0, 30.0), 180.0));
    }

    /// p. 33: within five degrees of a cusp, the nearer cusp's house.
    #[test]
    fn the_five_degree_rule_takes_the_nearer_cusp() {
        let cusps: [f64; 12] = equal();
        let house = |at: f64, orb: f64| house_of(at, &cusps, orb).get();
        assert_eq!(house(26.0, 5.0), 2);
        assert_eq!(house(24.0, 5.0), 1);
        assert_eq!(house(2.0, 5.0), 1);
        assert_eq!(house(358.0, 5.0), 1);
        assert_eq!(house(358.0, 0.0), 12);
        // A second house of six degrees, 30° to 36°: a planet within five
        // degrees of both its cusps is in the house of the nearer.
        let mut narrow = cusps;
        narrow[2] = 36.0;
        assert_eq!(house_of(32.0, &narrow, 5.0), House::try_new(2).unwrap());
        assert_eq!(house_of(34.0, &narrow, 5.0), House::try_new(3).unwrap());
    }

    #[test]
    fn a_bad_orb_or_longitude_is_named() {
        let sky = figure([(10.0, 0.1); 7], [0.0; 12], 0.0, 2000.0);
        let bad = AccidentalRules::LILLY.with_solar_orbs(f64::NAN, 17.0, 0.3);
        let why = accidental_dignities(&sky.longitudes, &sky.sky, &bad, &AccidentalScores::LILLY)
            .unwrap_err();
        assert_eq!(why.field(), Some("combustionDeg"));
        let mut lost = sky.sky;
        lost.speeds_deg_per_day[5] = f64::INFINITY;
        let why = accidental_dignities(
            &sky.longitudes,
            &lost,
            &AccidentalRules::LILLY,
            &AccidentalScores::LILLY,
        )
        .unwrap_err();
        assert_eq!(why.field(), Some("speedsDegPerDay.mercury"));
    }
}
