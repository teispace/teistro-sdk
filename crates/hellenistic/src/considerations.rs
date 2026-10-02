//! Lilly's considerations before judgement (*Christian Astrology* I.19,
//! pp. 121–123; `03-design/hellenistic-considerations.md`): whether a
//! horary figure is "radicall and capable of judgment", and the
//! conditions that make judging it unsafe.
//!
//! They are reported as clauses, each with the facts it rests on, never
//! summed into a verdict: Lilly weighs them himself, and some he gives
//! only "as some say". Everything is read off a figure's [`Fortitudes`]
//! and the lord of the hour, so like the rest of the crate there is no
//! ephemeris here.
//!
//! ```
//! use teistro_core::catalogue::{Graha, Rashi};
//! use teistro_hellenistic::{Temperament, RadicalGround, radical_grounds};
//!
//! // Mars's hour with Leo rising: the Sun and Mars are both hot and dry
//! // (p. 122), so the figure is radical by their nature.
//! assert_eq!(
//!     radical_grounds(Graha::Mars, Rashi::Leo, teistro_hellenistic::Triplicities::Lilly),
//!     vec![RadicalGround::Nature]
//! );
//! assert_eq!(Temperament::of(Graha::Mars), Temperament::of(Graha::Sun));
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::accidental::Accident;
use crate::dignity::{CHALDEAN_ORDER, Sect, Triplicities};
use crate::fortitude::Fortitudes;

/// The degrees of a sign.
const SIGN_DEG: f64 = 30.0;

/// A planet's nature, its two qualities (each planet's own chapter).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Temperament {
    /// Mars and the Sun.
    HotDry,
    /// Jupiter, "temperately Hot and Moyst".
    HotMoist,
    /// Saturn, and Mercury "of his owne nature".
    ColdDry,
    /// Venus and the Moon.
    ColdMoist,
}

impl Temperament {
    /// A planet's nature as Lilly gives it; `None` outside the seven.
    #[must_use]
    pub const fn of(planet: Graha) -> Option<Temperament> {
        match planet {
            Graha::Mars | Graha::Sun => Some(Temperament::HotDry),
            Graha::Jupiter => Some(Temperament::HotMoist),
            Graha::Saturn | Graha::Mercury => Some(Temperament::ColdDry),
            Graha::Venus | Graha::Moon => Some(Temperament::ColdMoist),
            _ => None,
        }
    }
}

/// Why a figure is radical: the lord of the hour and the lord of the
/// Ascendant "are of one Triplicity, or be one, or of the same nature"
/// (p. 121).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum RadicalGround {
    /// The lord of the hour rules the ascending sign: Mars's hour with
    /// Aries rising.
    OneLord,
    /// The lord of the hour rules the ascending sign's triplicity, by day
    /// or by night: Mars's hour with Scorpio or Pisces rising, Mars being
    /// "of the Watry Triplicity".
    Triplicity,
    /// The two lords share a nature: Mars's hour with Leo rising, Mars
    /// and the Sun "both of one nature, viz. Hot and Dry".
    Nature,
}

/// The grounds on which the lord of the hour makes a figure with `rising`
/// ascending radical, in the order of [`RadicalGround`]; empty when it is
/// not radical. A lord of a triplicity is one by day or by night, as Lilly
/// calls the Sun "one of the Lords of the fiery Triplicity" with no sect.
#[must_use]
pub fn radical_grounds(
    hour_lord: Graha,
    rising: Rashi,
    triplicities: Triplicities,
) -> Vec<RadicalGround> {
    let ascendant_lord = rising.attributes().lord;
    let mut grounds = Vec::new();
    if hour_lord == ascendant_lord {
        grounds.push(RadicalGround::OneLord);
    }
    if [Sect::Day, Sect::Night]
        .into_iter()
        .any(|sect| triplicities.rules(hour_lord, rising, sect))
    {
        grounds.push(RadicalGround::Triplicity);
    }
    if hour_lord != ascendant_lord
        && Temperament::of(hour_lord).is_some()
        && Temperament::of(hour_lord) == Temperament::of(ascendant_lord)
    {
        grounds.push(RadicalGround::Nature);
    }
    grounds
}

/// A Ptolemaic aspect, which the Moon's course is read on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum PtolemaicAspect {
    /// 0°.
    Conjunction,
    /// 60°.
    Sextile,
    /// 90°.
    Square,
    /// 120°.
    Trine,
    /// 180°.
    Opposition,
}

impl PtolemaicAspect {
    /// The five, in order of their angle.
    pub const ALL: [PtolemaicAspect; 5] = [
        PtolemaicAspect::Conjunction,
        PtolemaicAspect::Sextile,
        PtolemaicAspect::Square,
        PtolemaicAspect::Trine,
        PtolemaicAspect::Opposition,
    ];

    /// Its angle, degrees.
    #[must_use]
    pub const fn degrees(self) -> f64 {
        match self {
            PtolemaicAspect::Conjunction => 0.0,
            PtolemaicAspect::Sextile => 60.0,
            PtolemaicAspect::Square => 90.0,
            PtolemaicAspect::Trine => 120.0,
            PtolemaicAspect::Opposition => 180.0,
        }
    }
}

/// An aspect the Moon perfects with a planet.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Perfection {
    /// The planet.
    pub planet: Graha,
    /// The aspect.
    pub aspect: PtolemaicAspect,
    /// Days until it is exact, at the motions of the moment.
    pub days: f64,
    /// How far it is from exact now, degrees: the arc the two close, which
    /// a reading by Lilly's moieties of orb weighs (C230).
    pub gap_deg: f64,
}

/// Where the Moon is going before she leaves her sign (p. 112).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonCourse {
    /// The first aspect she perfects with one of the other six while she
    /// is in her sign; `None` when she is void of course.
    pub next: Option<Perfection>,
    /// The first of those already within the two planets' moieties of
    /// orb (half of each one's orb, p. 107), which she "applies" to now;
    /// `None` when she is void by Lilly's moieties (C230).
    pub within_orb: Option<Perfection>,
    /// Days until she leaves her sign.
    pub days_in_sign: f64,
    /// Void in Taurus, Cancer, Sagittarius or Pisces, where "somewhat she
    /// performes" (p. 122).
    pub eased: bool,
}

impl MoonCourse {
    /// "Separated from a Planet, nor doth forthwith, during his being in
    /// that Signe, apply to any other" (p. 112): no aspect perfected
    /// before she leaves her sign (crux C230).
    #[must_use]
    pub const fn void(&self) -> bool {
        self.next.is_none()
    }

    /// Void by Lilly's moieties: no aspect ahead in her sign is yet within
    /// the two planets' half orbs, which is how his Presbytery figure
    /// (p. 439) reads "after a little being voyd of course" (C230).
    #[must_use]
    pub const fn void_by_moieties(&self) -> bool {
        self.within_orb.is_none()
    }
}

/// The Moon's course: the first Ptolemaic aspect she perfects with one of
/// the other six before she leaves her sign, each body carried on at its
/// motion of the moment. `planets` gives the seven's longitudes,
/// `speeds` their daily motions and `orbs_deg` their whole orbs, all in
/// the Chaldean order; the first perfection already within half of each
/// orb is reported too.
///
/// # Errors
///
/// A Moon that does not move forward, named `moon`; a longitude or speed
/// that is not finite, named `planets`.
pub fn moon_course(
    planets: &[f64; 7],
    speeds: &[f64; 7],
    orbs_deg: &[f64; 7],
) -> Result<MoonCourse, Error> {
    if planets.iter().chain(speeds).any(|value| !value.is_finite()) {
        return Err(
            Error::invalid_arg("a planet's place or motion is not finite").with_field("planets"),
        );
    }
    let at = |graha: Graha| {
        let index = CHALDEAN_ORDER.iter().position(|&each| each == graha);
        index.and_then(|index| {
            Some((
                *planets.get(index)?,
                *speeds.get(index)?,
                *orbs_deg.get(index)?,
            ))
        })
    };
    let Some((moon, moon_speed, moon_orb)) = at(Graha::Moon) else {
        return Err(Error::internal("the Chaldean order holds the Moon"));
    };
    if moon_speed <= 0.0 {
        return Err(Error::invalid_arg("the Moon must move forward").with_field("moon"));
    }
    let days_in_sign = (SIGN_DEG - moon.rem_euclid(SIGN_DEG)) / moon_speed;
    let mut next: Option<Perfection> = None;
    let mut within_orb: Option<Perfection> = None;
    let earlier =
        |found: Option<Perfection>, days: f64| found.is_none_or(|found| days < found.days);
    for planet in CHALDEAN_ORDER
        .into_iter()
        .filter(|&graha| graha != Graha::Moon)
    {
        let Some((place, speed, orb)) = at(planet) else {
            continue;
        };
        let moieties = f64::midpoint(moon_orb, orb);
        let closing = moon_speed - speed;
        if closing <= 0.0 {
            continue;
        }
        for aspect in PtolemaicAspect::ALL {
            // The Moon reaches the angle on either side of the planet.
            for side in [aspect.degrees(), 360.0 - aspect.degrees()] {
                let gap_deg = (place + side - moon).rem_euclid(360.0);
                let days = gap_deg / closing;
                if days > days_in_sign {
                    continue;
                }
                let perfection = Some(Perfection {
                    planet,
                    aspect,
                    days,
                    gap_deg,
                });
                if earlier(next, days) {
                    next = perfection;
                }
                if gap_deg <= moieties && earlier(within_orb, days) {
                    within_orb = perfection;
                }
            }
        }
    }
    let sign = Rashi::of_longitude(moon);
    Ok(MoonCourse {
        next,
        within_orb,
        days_in_sign,
        eased: matches!(
            sign,
            Rashi::Taurus | Rashi::Cancer | Rashi::Sagittarius | Rashi::Pisces
        ),
    })
}

/// The degrees Lilly names, as knobs where he leaves a number open.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct ConsiderationRules {
    /// From what degree of her sign the Moon is "in the later degrees"
    /// (crux C229): Lilly gives no number; 27, the degree he gives for a
    /// late Ascendant, unless said.
    pub moon_late_from_deg: f64,
    /// Each planet's whole orb in the Chaldean order, half of which counts
    /// toward an application (p. 107's first column: Saturn 10, Jupiter
    /// 12, Mars 7½, the Sun 17, Venus 8, Mercury 7, the Moon 12½).
    pub orbs_deg: [f64; 7],
}

impl ConsiderationRules {
    /// Lilly's, with C229 read by his late Ascendant.
    pub const LILLY: ConsiderationRules = ConsiderationRules {
        moon_late_from_deg: 27.0,
        orbs_deg: [10.0, 12.0, 7.5, 17.0, 8.0, 7.0, 12.5],
    };

    /// Reads the rules from JSON, every member optional and Lilly's when
    /// left out; a refusal is named from the root, as
    /// `considerations.moonLateFromDeg`.
    ///
    /// ```
    /// use teistro_hellenistic::ConsiderationRules;
    ///
    /// let rules = ConsiderationRules::from_json(r#"{"moonLateFromDeg": 25}"#).unwrap();
    /// assert_eq!(rules.orbs_deg, ConsiderationRules::LILLY.orbs_deg);
    /// let wide = ConsiderationRules::from_json(r#"{"moonLateFromDeg": 31}"#).unwrap_err();
    /// assert_eq!(wide.field(), Some("considerations.moonLateFromDeg"));
    /// ```
    ///
    /// # Errors
    ///
    /// Text that is not the record, a key it does not read, or a value
    /// out of range.
    pub fn from_json(text: &str) -> Result<ConsiderationRules, Error> {
        let rules: ConsiderationRules = teistro_core::strict::read(text, "considerations")?;
        rules.check().map_err(|why| why.under("considerations"))?;
        Ok(rules)
    }

    /// The late degree within a sign and every orb a finite number of
    /// degrees, zero or more, naming the field that is not.
    fn check(&self) -> Result<(), Error> {
        let late = self.moon_late_from_deg;
        if !(late.is_finite() && (0.0..=SIGN_DEG).contains(&late)) {
            return Err(Error::invalid_arg(format!("a late degree of {late}"))
                .with_field("moonLateFromDeg")
                .with_hint("a degree of the sign, 0 to 30"));
        }
        if let Some(orb) = self
            .orbs_deg
            .iter()
            .find(|orb| !(orb.is_finite() && **orb >= 0.0))
        {
            return Err(Error::invalid_arg(format!("an orb of {orb}"))
                .with_field("orbsDeg")
                .with_hint("an orb is a finite number of degrees, zero or more"));
        }
        Ok(())
    }
}

impl Default for ConsiderationRules {
    fn default() -> ConsiderationRules {
        ConsiderationRules::LILLY
    }
}

/// Whether the figure is radical (p. 121).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Radicality {
    /// The lord of the hour.
    pub hour_lord: Graha,
    /// The lord of the ascending sign.
    pub ascendant_lord: Graha,
    /// Why it is radical; empty when it is not.
    pub grounds: Vec<RadicalGround>,
}

impl Radicality {
    /// Radical on any ground.
    #[must_use]
    pub fn radical(&self) -> bool {
        !self.grounds.is_empty()
    }
}

/// The ascending degree (p. 122).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AscendantClause {
    /// The ascending sign.
    pub sign: Rashi,
    /// The degree within it, 0 to 30.
    pub degree: f64,
    /// "00. degrees, or the first or second": below 3°.
    pub early: bool,
    /// "27, 28, or 29 degrees": 27° and over.
    pub late: bool,
    /// A sign of short ascension, Capricorn to Gemini, where an early
    /// degree weighs the more.
    pub short_ascension: bool,
}

/// The Moon (p. 122).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct MoonClause {
    /// Her sign.
    pub sign: Rashi,
    /// Her degree within it, 0 to 30.
    pub degree: f64,
    /// In the later degrees of her sign, from
    /// [`ConsiderationRules::moon_late_from_deg`].
    pub late: bool,
    /// In Gemini, Scorpio or Capricorn, where a late Moon weighs the more.
    pub late_sign: bool,
    /// "As some say", in the via combusta: the last 15° of Libra or the
    /// first 15° of Scorpio.
    pub via_combusta: bool,
    /// Her course before she leaves her sign.
    pub course: MoonCourse,
}

/// The seventh house, which "hath signification of the Artist" (p. 122),
/// and the Arabians' rules on its lord (p. 123).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent conditions Lilly lists on the seventh's lord, any of which may hold at once; each is a clause a reader asks about by name"
)]
pub struct SeventhClause {
    /// Its cusp, degrees.
    pub cusp_deg: f64,
    /// The lord of the sign on the cusp.
    pub lord: Graha,
    /// Saturn or Mars counted in the seventh house: what afflicts the
    /// cusp, by one reading of C231.
    pub infortunes_in_house: Vec<Graha>,
    /// The lord retrograde.
    pub lord_retrograde: bool,
    /// The lord combust.
    pub lord_combust: bool,
    /// The lord in his fall.
    pub lord_in_fall: bool,
    /// The lord in the terms of Saturn or Mars.
    pub lord_in_infortune_term: bool,
    /// The lord's net strength over Lilly's table, which says whether he is
    /// "unfortunate".
    pub lord_net: i16,
}

/// Lilly's considerations before judgement, each a clause with what it
/// rests on.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Considerations {
    /// Whether the figure is radical.
    pub radicality: Radicality,
    /// The ascending degree.
    pub ascendant: AscendantClause,
    /// The Moon.
    pub moon: MoonClause,
    /// The seventh house and its lord.
    pub seventh: SeventhClause,
    /// The house Saturn is counted in.
    pub saturn_house: House,
    /// Saturn retrograde, which makes him in the Ascendant the worse.
    pub saturn_retrograde: bool,
    /// The lord of the Ascendant combust.
    pub ascendant_lord_combust: bool,
    /// The rules the open degrees were read under.
    pub rules: ConsiderationRules,
}

impl Considerations {
    /// Saturn in the Ascendant, "especially Retrograde" (p. 122).
    #[must_use]
    pub fn saturn_in_ascendant(&self) -> bool {
        self.saturn_house.get() == 1
    }

    /// Saturn in the seventh (p. 123).
    #[must_use]
    pub fn saturn_in_seventh(&self) -> bool {
        self.saturn_house.get() == 7
    }
}

/// Lilly's considerations before judgement of a figure whose fortitudes
/// are `fortitudes`, asked in the hour of `hour_lord`, under `rules`.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_hellenistic::{ConsiderationRules, Fortitudes, considerations};
///
/// fn judge(fortitudes: &Fortitudes) -> Result<bool, teistro_core::error::Error> {
///     let read = considerations(fortitudes, Graha::Mars, ConsiderationRules::LILLY)?;
///     Ok(read.radicality.radical() && !read.moon.course.void())
/// }
/// # let _ = judge;
/// ```
///
/// # Errors
///
/// An `hour_lord` outside the seven, named `hour_lord`; rules out of
/// range, named as [`ConsiderationRules::from_json`] names them without
/// the root; and as [`moon_course`].
pub fn considerations(
    fortitudes: &Fortitudes,
    hour_lord: Graha,
    rules: ConsiderationRules,
) -> Result<Considerations, Error> {
    if !CHALDEAN_ORDER.contains(&hour_lord) {
        return Err(Error::invalid_arg(format!(
            "the lord of an hour is one of the seven, not {hour_lord:?}"
        ))
        .with_field("hour_lord"));
    }
    rules.check()?;
    let dignities = &fortitudes.dignities;
    let sky = &fortitudes.sky;
    let planets = dignities
        .planets
        .each_ref()
        .map(|planet| planet.longitude_deg);
    let index = |graha: Graha| CHALDEAN_ORDER.iter().position(|&each| each == graha);
    let dignity = |graha: Graha| index(graha).and_then(|at| dignities.planets.get(at));
    let accidents = |graha: Graha| index(graha).and_then(|at| fortitudes.planets.get(at));
    let has = |graha: Graha, accident: Accident| {
        accidents(graha).is_some_and(|planet| planet.accidents.contains(&accident))
    };
    let missing = || Error::internal("the fortitudes read all seven");

    let rising = Rashi::of_longitude(sky.ascendant_deg);
    let ascendant_degree = sky.ascendant_deg.rem_euclid(SIGN_DEG);
    let ascendant_lord = rising.attributes().lord;

    let moon_deg = dignity(Graha::Moon).ok_or_else(missing)?.longitude_deg;
    let moon_sign = Rashi::of_longitude(moon_deg);
    let moon_degree = moon_deg.rem_euclid(SIGN_DEG);

    let cusp_deg = *sky.cusps_deg.get(6).ok_or_else(missing)?;
    let seventh_lord = Rashi::of_longitude(cusp_deg).attributes().lord;
    let lord = dignity(seventh_lord).ok_or_else(missing)?;
    let term_lord = dignities
        .rules
        .terms
        .table(dignities.sect)
        .lord_at(lord.longitude_deg);
    let infortunes_in_house = [Graha::Saturn, Graha::Mars]
        .into_iter()
        .filter(|&graha| accidents(graha).is_some_and(|planet| planet.house.get() == 7))
        .collect();

    Ok(Considerations {
        radicality: Radicality {
            hour_lord,
            ascendant_lord,
            grounds: radical_grounds(hour_lord, rising, dignities.rules.triplicities),
        },
        ascendant: AscendantClause {
            sign: rising,
            degree: ascendant_degree,
            early: ascendant_degree < 3.0,
            late: ascendant_degree >= 27.0,
            short_ascension: matches!(
                rising,
                Rashi::Capricorn
                    | Rashi::Aquarius
                    | Rashi::Pisces
                    | Rashi::Aries
                    | Rashi::Taurus
                    | Rashi::Gemini
            ),
        },
        moon: MoonClause {
            sign: moon_sign,
            degree: moon_degree,
            late: moon_degree >= rules.moon_late_from_deg,
            late_sign: matches!(moon_sign, Rashi::Gemini | Rashi::Scorpio | Rashi::Capricorn),
            via_combusta: (moon_sign == Rashi::Libra && moon_degree >= 15.0)
                || (moon_sign == Rashi::Scorpio && moon_degree < 15.0),
            course: moon_course(&planets, &sky.speeds_deg_per_day, &rules.orbs_deg)?,
        },
        seventh: SeventhClause {
            cusp_deg,
            lord: seventh_lord,
            infortunes_in_house,
            lord_retrograde: has(seventh_lord, Accident::Retrograde),
            lord_combust: has(seventh_lord, Accident::Combust),
            lord_in_fall: lord.dignity.fall,
            lord_in_infortune_term: matches!(term_lord, Some(Graha::Saturn | Graha::Mars)),
            lord_net: fortitudes.net(seventh_lord).ok_or_else(missing)?,
        },
        saturn_house: accidents(Graha::Saturn).ok_or_else(missing)?.house,
        saturn_retrograde: has(Graha::Saturn, Accident::Retrograde),
        ascendant_lord_combust: has(ascendant_lord, Accident::Combust),
        rules,
    })
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "tests fail by panicking and index what they built"
)]
mod tests {
    use teistro_core::catalogue::HouseSystem;

    use super::*;
    use crate::accidental::AccidentalSky;
    use crate::fortitude::FortitudeRequest;
    use crate::reading::ChartSky;

    /// p. 121–122: Mars's hour is radical with Scorpio or Pisces rising
    /// (Mars of the water triplicity), Aries (one lord) and Leo (one
    /// nature, hot and dry); not with Gemini, whose lord Mercury is cold
    /// and dry and whose triplicity Mars does not rule.
    #[test]
    fn lillys_examples_of_a_radical_figure() {
        let grounds = |rising| radical_grounds(Graha::Mars, rising, Triplicities::Lilly);
        assert_eq!(
            grounds(Rashi::Scorpio),
            [RadicalGround::OneLord, RadicalGround::Triplicity]
        );
        assert_eq!(grounds(Rashi::Pisces), [RadicalGround::Triplicity]);
        assert_eq!(grounds(Rashi::Aries), [RadicalGround::OneLord]);
        assert_eq!(grounds(Rashi::Leo), [RadicalGround::Nature]);
        assert_eq!(grounds(Rashi::Gemini), []);
        // Ptolemy's water triplicity has Mars too, "together with" Venus
        // and the Moon.
        assert_eq!(
            radical_grounds(Graha::Mars, Rashi::Cancer, Triplicities::Ptolemy),
            [RadicalGround::Triplicity]
        );
    }

    /// The seven with Saturn's and Mercury's cold and dry, Lilly's
    /// natures, and nothing for a node.
    #[test]
    fn the_natures() {
        assert_eq!(Temperament::of(Graha::Saturn), Some(Temperament::ColdDry));
        assert_eq!(Temperament::of(Graha::Mercury), Some(Temperament::ColdDry));
        assert_eq!(Temperament::of(Graha::Jupiter), Some(Temperament::HotMoist));
        assert_eq!(Temperament::of(Graha::Venus), Temperament::of(Graha::Moon));
        assert_eq!(Temperament::of(Graha::Rahu), None);
    }

    /// The seven at `planets` (Chaldean order) moving at `speeds`.
    fn course(planets: [f64; 7], speeds: [f64; 7]) -> MoonCourse {
        moon_course(&planets, &speeds, &ConsiderationRules::LILLY.orbs_deg).unwrap()
    }

    /// p. 112: a Moon 10° behind Mars in Aries perfects the conjunction
    /// in 0.8 days; at 29°30′ Aries with every planet at 15° Taurus she
    /// leaves the sign first, void.
    #[test]
    fn void_of_course() {
        let speeds = [0.03, 0.08, 0.5, 1.0, 1.2, 1.4, 13.0];
        let applying = course([210.0, 210.0, 20.0, 210.0, 210.0, 210.0, 10.0], speeds);
        let next = applying.next.unwrap();
        assert_eq!(
            (next.planet, next.aspect),
            (Graha::Mars, PtolemaicAspect::Conjunction)
        );
        assert!((next.days - 10.0 / 12.5).abs() < 1e-12);
        assert!(!applying.void());
        let void = course([45.0, 45.0, 45.0, 45.0, 45.0, 45.0, 29.5], speeds);
        assert!(void.void());
        assert!(!void.eased);
        assert!((void.days_in_sign - 0.5 / 13.0).abs() < 1e-12);
        // In Taurus she "performes" somewhat.
        let eased = course([75.0, 75.0, 75.0, 75.0, 75.0, 75.0, 59.5], speeds);
        assert!(eased.void() && eased.eased);
    }

    /// Lilly's ship at sea (p. 165, 9 March 1646/7 at 10h 15m in the
    /// Moon's hour, London), recast with the Swiss Ephemeris's Moshier
    /// series: the Moon at 10°29′ Virgo (his figure prints 10°44′) "lately
    /// separated from a □ of" Saturn, which perfected 1.74 days before. He
    /// calls her "voyd of course" at the question and has her "afterwards
    /// first" apply to Saturn's trine and Mercury's opposition, both of
    /// which perfect in Virgo within half a day. The shipped reading finds
    /// that trine 4.3° away, so it does not call her void: C230's evidence
    /// that his word is not the modern one, and why the gap is reported.
    #[test]
    fn lillys_ship_is_not_void_by_the_modern_reading() {
        let planets = [44.82, 118.62, 115.23, 358.68, 6.33, 346.28, 160.48];
        let speeds = [0.103, -0.018, 0.166, 0.990, 1.242, 1.795, 14.834];
        let course = course(planets, speeds);
        let next = course.next.unwrap();
        assert_eq!(
            (next.planet, next.aspect),
            (Graha::Saturn, PtolemaicAspect::Trine)
        );
        assert!((next.gap_deg - 4.34).abs() < 0.01, "{}", next.gap_deg);
        assert!((next.days - 0.294).abs() < 0.001, "{}", next.days);
        assert!(!course.void());
        // Nor by his moieties: 4.3° is inside the Moon's 6¼ and Saturn's 5.
        assert_eq!(course.within_orb, course.next);
        assert!(!course.void_by_moieties());
    }

    /// Lilly's "If Presbytery shall stand" (p. 439, 11 March 1646/7 at
    /// 4h 45m PM in Venus's hour, London), recast as the ship was: the Moon
    /// at 13°47′ Libra (his figure prints 13°37′), lately from Venus's
    /// opposition, "after a little being voyd of course" runs to the
    /// squares of Mars and Jupiter. Mars's is 11.9° ahead, beyond the
    /// Moon's and his moieties together (10°): void by the moieties, not by
    /// the sign's end — the reading this figure supports (C230).
    #[test]
    fn lillys_presbytery_is_void_by_the_moieties() {
        let planets = [45.07, 118.58, 115.65, 0.93, 9.15, 350.42, 193.78];
        let speeds = [0.105, -0.011, 0.187, 0.989, 1.241, 1.844, 14.386];
        let course = course(planets, speeds);
        let next = course.next.unwrap();
        assert_eq!(
            (next.planet, next.aspect),
            (Graha::Mars, PtolemaicAspect::Square)
        );
        assert!((next.gap_deg - 11.87).abs() < 0.01, "{}", next.gap_deg);
        assert!(!course.void());
        assert!(course.void_by_moieties());
    }

    /// Lilly's "A Lady, if marry the Gentleman desired?" (p. 385, Tuesday
    /// 16 June 1646 at 19h 26m after noon, astronomical reckoning, so
    /// 7:26 the next morning in Saturn's hour, London), recast as the ship
    /// was: the Moon at 27°17′ Sagittarius (his figure prints 28°09′),
    /// which the figure calls "a vac: ad ☍ ☉" and the text "voyd of
    /// course, and applying to" the Sun's opposition. That opposition is
    /// 8.2° ahead, inside the Moon's and the Sun's moieties together
    /// (14¾°), but perfects only after she leaves her sign: void by both
    /// readings, so a moiety reading must stop at the sign's end too (C230).
    #[test]
    fn lillys_lady_is_void_by_both_readings() {
        let planets = [44.61, 104.75, 47.72, 95.51, 50.35, 76.39, 267.29];
        let speeds = [0.097, 0.223, 0.714, 0.953, 0.860, 0.407, 11.842];
        let course = course(planets, speeds);
        assert!(course.void());
        assert!(course.void_by_moieties());
        let to_the_sun = (planets[3] + 180.0 - planets[6]).rem_euclid(360.0);
        assert!((to_the_sun - 8.22).abs() < 0.01, "{to_the_sun}");
        assert!(to_the_sun < f64::midpoint(12.5, 17.0));
        assert!(to_the_sun / (speeds[6] - speeds[3]) > course.days_in_sign);
    }

    /// Four more of Lilly's dated figures, recast as the ship was, each with
    /// the course he names: the horse lost near Henley (p. 467, 11 January
    /// 1646/7 at 2h 59m PM, "a □ ♄ ad Vac") is void by both readings, and
    /// three name the aspect she applies to, which both readings find
    /// first in her sign: the long-life question (p. 135, 14 March
    /// 1632/3 at 2h 15m PM) Jupiter's trine and then Mars's opposition,
    /// the Parsonage (p. 437, 6 August 1644 at 8h 24m PM) Mercury's
    /// opposition, and the escaped prisoner (p. 470, 6 June 1647 at 8 PM)
    /// the Sun's opposition (C230).
    #[test]
    fn lillys_other_figures_read_as_he_reads_them() {
        let figures = [
            (
                "horse",
                [41.16, 123.87, 124.18, 301.59, 295.32, 302.25, 132.08],
                [0.018, -0.133, -0.4, 1.016, 1.255, -1.255, 15.231],
                None,
            ),
            (
                "long life",
                [250.22, 54.1, 358.66, 4.18, 45.92, 344.93, 171.28],
                [-0.009, 0.189, 0.775, 0.987, 1.142, 1.641, 12.985],
                Some((Graha::Jupiter, PtolemaicAspect::Trine)),
            ),
            (
                "parsonage",
                [20.65, 59.61, 64.4, 144.32, 181.65, 129.32, 307.94],
                [-0.026, 0.103, 0.606, 0.963, 0.392, 1.728, 13.992],
                Some((Graha::Mercury, PtolemaicAspect::Opposition)),
            ),
            (
                "prisoner",
                [55.8, 127.67, 152.22, 85.28, 115.62, 62.79, 261.62],
                [0.119, 0.187, 0.553, 0.953, 1.198, 1.009, 12.496],
                Some((Graha::Sun, PtolemaicAspect::Opposition)),
            ),
        ];
        for (name, planets, speeds, applies) in figures {
            let course = course(planets, speeds);
            let next = course.next.map(|at| (at.planet, at.aspect));
            assert_eq!(next, applies, "{name}");
            assert_eq!(
                course.within_orb.map(|at| (at.planet, at.aspect)),
                applies,
                "{name}"
            );
        }
    }

    /// The rules read from JSON, every member optional, and refused out of
    /// range by name.
    #[test]
    fn the_rules_are_read_and_checked() {
        assert_eq!(
            ConsiderationRules::from_json("{}").unwrap(),
            ConsiderationRules::LILLY
        );
        for (text, field) in [
            (
                r#"{"moonLateFromDeg": -1}"#,
                "considerations.moonLateFromDeg",
            ),
            (
                r#"{"orbsDeg": [10, 12, 7.5, 17, 8, -7, 12.5]}"#,
                "considerations.orbsDeg",
            ),
            (r#"{"moonLate": 25}"#, "considerations.moonLate"),
        ] {
            let refused = ConsiderationRules::from_json(text).unwrap_err();
            assert_eq!(refused.field(), Some(field), "{text}");
        }
    }

    /// The first perfection is the one a step-by-step walk of the sky
    /// finds, on many skies.
    #[test]
    fn the_course_matches_a_walk() {
        let speeds = [-0.02, 0.1, 0.6, 0.98, -0.4, 1.6, 12.4];
        for seed in 0_u32..200 {
            let at = |k: u32| f64::from((seed * 7919 + k * 104_729) % 36_000) / 100.0;
            let planets = [at(1), at(2), at(3), at(4), at(5), at(6), at(7)];
            let found = course(planets, speeds);
            // Walk in steps of a thousandth of a day, watching each pair's
            // separation from every angle change sign.
            let step = 0.001;
            let mut walked = None;
            let mut t = 0.0;
            'walk: while t + step <= found.days_in_sign {
                for (index, planet) in CHALDEAN_ORDER.into_iter().enumerate().take(6) {
                    for aspect in PtolemaicAspect::ALL {
                        for side in [aspect.degrees(), -aspect.degrees()] {
                            let off = |time: f64| {
                                let moon = planets[6] + speeds[6] * time;
                                let other = planets[index] + speeds[index] * time;
                                (moon - other - side + 180.0).rem_euclid(360.0) - 180.0
                            };
                            if off(t) < 0.0 && off(t + step) >= 0.0 {
                                walked = Some(planet);
                                break 'walk;
                            }
                        }
                    }
                }
                t += step;
            }
            if let Some(planet) = walked {
                assert_eq!(
                    found.next.map(|next| next.planet),
                    Some(planet),
                    "sky {seed}"
                );
            }
        }
    }

    /// A figure with the Ascendant at `ascendant` and equal houses, the
    /// seven at `planets` (Chaldean order) with `speeds`, by day.
    fn fortitudes(ascendant: f64, planets: [f64; 7], speeds: [f64; 7]) -> Fortitudes {
        let [
            saturn_deg,
            jupiter_deg,
            mars_deg,
            sun_deg,
            venus_deg,
            mercury_deg,
            moon_deg,
        ] = planets;
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
        let cusps = core::array::from_fn(|house| {
            #[expect(clippy::cast_precision_loss, reason = "a house below 12")]
            let offset = house as f64 * SIGN_DEG;
            (ascendant + offset).rem_euclid(360.0)
        });
        let sky = AccidentalSky {
            speeds_deg_per_day: speeds,
            houses: HouseSystem::Equal,
            cusps_deg: cusps,
            ascendant_deg: ascendant,
            midheaven_deg: cusps[9],
            north_node_deg: 100.0,
            regulus_deg: 150.0,
            spica_deg: 204.0,
            algol_deg: 56.0,
        };
        FortitudeRequest::default().read(&chart, &sky).unwrap()
    }

    /// One figure read whole: Leo rising at 1°30′ in Mars's hour is
    /// radical by nature and early; the Moon at 20° Libra is in the via
    /// combusta; Saturn retrograde in the seventh, Aquarius, whose lord he
    /// is; the Sun, lord of the Ascendant, is not combust.
    #[test]
    fn a_figure_read_whole() {
        let planets = [305.0, 250.0, 100.0, 160.0, 175.0, 140.0, 200.0];
        let speeds = [-0.05, 0.1, 0.6, 0.98, 1.2, 1.5, 13.0];
        let read = considerations(
            &fortitudes(121.5, planets, speeds),
            Graha::Mars,
            ConsiderationRules::LILLY,
        )
        .unwrap();
        assert_eq!(read.radicality.ascendant_lord, Graha::Sun);
        assert_eq!(read.radicality.grounds, [RadicalGround::Nature]);
        assert!(read.radicality.radical());
        let ascendant = read.ascendant;
        assert_eq!(ascendant.sign, Rashi::Leo);
        assert!(ascendant.early && !ascendant.late && !ascendant.short_ascension);
        assert!(read.moon.via_combusta && !read.moon.late);
        assert_eq!(read.seventh.lord, Graha::Saturn);
        assert!(read.seventh.lord_retrograde);
        assert_eq!(read.seventh.infortunes_in_house, [Graha::Saturn]);
        assert!(read.saturn_in_seventh() && !read.saturn_in_ascendant());
        assert!(read.saturn_retrograde);
        assert!(!read.ascendant_lord_combust);
        assert_eq!(
            considerations(
                &fortitudes(121.5, planets, speeds),
                Graha::Rahu,
                ConsiderationRules::LILLY
            )
            .unwrap_err()
            .field(),
            Some("hour_lord")
        );
    }

    /// p. 122's degrees at their edges: 3° rising is not early nor 26°
    /// late; the via combusta is Libra 15° to the last of Scorpio 14°; the
    /// Moon's late degrees start where the rules say (C229).
    #[test]
    fn the_degrees_at_their_edges() {
        let speeds = [0.03, 0.08, 0.5, 1.0, 1.2, 1.4, 13.0];
        let read = |ascendant: f64, moon: f64, rules| {
            let planets = [10.0, 70.0, 130.0, 250.0, 260.0, 240.0, moon];
            considerations(&fortitudes(ascendant, planets, speeds), Graha::Sun, rules).unwrap()
        };
        let lilly = ConsiderationRules::LILLY;
        assert!(!read(3.0, 0.0, lilly).ascendant.early);
        assert!(read(2.99, 0.0, lilly).ascendant.early);
        assert!(!read(26.99, 0.0, lilly).ascendant.late);
        assert!(read(27.0, 0.0, lilly).ascendant.late);
        assert!(read(27.0, 0.0, lilly).ascendant.short_ascension);
        assert!(read(0.0, 195.0, lilly).moon.via_combusta);
        assert!(!read(0.0, 194.99, lilly).moon.via_combusta);
        assert!(read(0.0, 224.99, lilly).moon.via_combusta);
        assert!(!read(0.0, 225.0, lilly).moon.via_combusta);
        let late = read(0.0, 87.5, lilly).moon;
        assert!(late.late && late.late_sign);
        let wider = ConsiderationRules {
            moon_late_from_deg: 20.0,
            ..ConsiderationRules::LILLY
        };
        assert!(read(0.0, 82.0, wider).moon.late);
        assert!(!read(0.0, 82.0, lilly).moon.late);
    }
}
