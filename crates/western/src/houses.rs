//! A Western chart's houses: the module's division, Placidus unless asked
//! otherwise (C249), each planet counted by the cusps, and Leo's
//! ascendant reaching one sidereal hour above the first house (C250;
//! `03-design/western-houses.md`).

use serde::{Deserialize, Serialize};
use teistro_astro::houses::{Obliquity, circle_point};
use teistro_core::catalogue::{Graha, HouseSystem};
use teistro_core::error::Error;
use teistro_core::house::{House, house_of};

use crate::aspects::{PlanetAt, refuse_unreadable};

/// The module's name under `houses.module_overrides`, where a profile
/// names its division.
pub const MODULE: &str = "western";

/// The division Leo's figures are cast in (C249): his p. 150 illustration
/// fits Placidus at every printed cusp within 0.5°.
pub const LEO_HOUSE_SYSTEM: HouseSystem = HouseSystem::Placidus;

/// Lilly's division (*Christian Astrology*), which his antiscia on the
/// cusps are read in unless asked otherwise.
pub const LILLY_HOUSE_SYSTEM: HouseSystem = HouseSystem::Regiomontanus;

/// How far above the ascendant Leo's first house reaches, in right
/// ascension of the meridian, degrees: one sidereal hour, "15° of
/// Oblique Ascension" (p. 90, C250).
pub const ASCENDANT_REACH_DEG: f64 = 15.0;

/// What a chart's Western houses are asked with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct HouseRequest {
    /// The division to read, or the module's own when `None`: the
    /// profile's `houses.module_overrides.western`, else
    /// [`LEO_HOUSE_SYSTEM`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<HouseSystem>,
}

impl HouseRequest {
    /// Reads the houses in `system` whatever the profile names.
    #[must_use]
    pub const fn with_system(mut self, system: HouseSystem) -> HouseRequest {
        self.system = Some(system);
        self
    }

    /// The request a binding sends, as JSON: `{}` for the module's
    /// division, or `{"system": "KOCH"}`.
    ///
    /// ```
    /// use teistro_core::catalogue::HouseSystem;
    /// use teistro_western::HouseRequest;
    ///
    /// assert_eq!(HouseRequest::from_json("{}")?.system, None);
    /// let koch = HouseRequest::from_json(r#"{"system": "KOCH"}"#)?;
    /// assert_eq!(koch.system, Some(HouseSystem::Koch));
    /// let typo = HouseRequest::from_json(r#"{"sistem": "KOCH"}"#).unwrap_err();
    /// assert_eq!(typo.field(), Some("westernHouses.sistem"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on text that is not the record or a key it does not
    /// read, named under `westernHouses`.
    pub fn from_json(text: &str) -> Result<HouseRequest, Error> {
        teistro_core::strict::read(text, ROOT)
    }
}

/// The record's name where a binding sends it, which a refusal is named
/// under.
const ROOT: &str = "westernHouses";

/// Where a planet is counted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HousePlacement {
    /// Which planet.
    pub graha: Graha,
    /// The house whose cusp it has passed and whose next cusp it has not.
    pub house: House,
    /// Whether Leo counts it with the ascendant (C250): in the first
    /// house, or above the ascendant no further than the degree that rose
    /// one sidereal hour before. The house is never moved for it.
    pub with_ascendant: bool,
}

/// What a chart's houses are read on: the cusps, the ascendant and the
/// limit of its reach, all in the chart's zodiac.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HouseFrame {
    /// The twelve cusps, first to twelfth, degrees.
    pub cusps_deg: [f64; 12],
    /// The ascendant, degrees; the first cusp in every quadrant division,
    /// and not in whole signs or equal houses from another point.
    pub ascendant_deg: f64,
    /// The degree that rose one sidereal hour before the birth, degrees.
    pub reach_deg: f64,
}

/// A chart's Western houses.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WesternHouses {
    /// The division the cusps are of: the one asked, or the one a polar
    /// policy fell back to.
    pub system: HouseSystem,
    /// What the planets were counted on.
    pub frame: HouseFrame,
    /// Each planet's house, in the order given.
    pub planets: Vec<HousePlacement>,
}

/// The degree rising one sidereal hour before a birth (C250), tropical
/// longitude: the ascendant of the meridian turned back by
/// [`ASCENDANT_REACH_DEG`], which is Leo's own method, "subtract one hour
/// from the Sidereal Time at Birth and see in the Table of Houses what
/// degree is then rising" (p. 90).
///
/// ```
/// use teistro_western::rising_an_hour_before;
///
/// // On the equator with the solstice culminating, 0° Libra rises; the
/// // hour before raised 16.3° of longitude, more than 15°, because the
/// // ecliptic crosses the horizon slantwise by the equinox.
/// let reach = rising_an_hour_before(90.0, 0.0, 23.4393);
/// assert!((180.0 - reach - 16.28).abs() < 0.01, "{reach}");
/// ```
#[must_use]
pub fn rising_an_hour_before(armc_deg: f64, latitude_deg: f64, obliquity_deg: f64) -> f64 {
    circle_point(
        armc_deg - ASCENDANT_REACH_DEG + 90.0,
        latitude_deg,
        &Obliquity::new(obliquity_deg),
    )
}

/// Counts each planet into a house of `frame` (decision 2) and says
/// whether Leo reads it with the ascendant (decision 3).
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_western::{HouseFrame, PlanetAt, place_in_houses};
///
/// // Equal houses from 0° Aries; the hour before raised 20° Pisces.
/// let frame = HouseFrame {
///     cusps_deg: std::array::from_fn(|k| k as f64 * 30.0),
///     ascendant_deg: 0.0,
///     reach_deg: 350.0,
/// };
/// let placed = place_in_houses(
///     &frame,
///     &[PlanetAt::new(Graha::Mars, 355.0), PlanetAt::new(Graha::Venus, 340.0)],
/// )?;
/// assert_eq!((placed[0].house.get(), placed[0].with_ascendant), (12, true));
/// assert_eq!((placed[1].house.get(), placed[1].with_ascendant), (12, false));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// A planet named twice or at a longitude that is not finite, named
/// under `planets`, and a frame whose degrees are not finite, named by
/// its field.
pub fn place_in_houses(
    frame: &HouseFrame,
    bodies: &[PlanetAt],
) -> Result<Vec<HousePlacement>, Error> {
    refuse_unreadable(bodies, "planets")?;
    if let Some(at) = frame.cusps_deg.iter().position(|cusp| !cusp.is_finite()) {
        return Err(Error::invalid_arg("a cusp is a finite number of degrees")
            .with_field(format!("cuspsDeg[{at}]")));
    }
    for (value, field) in [
        (frame.ascendant_deg, "ascendantDeg"),
        (frame.reach_deg, "reachDeg"),
    ] {
        if !value.is_finite() {
            return Err(Error::invalid_arg(format!("{value} is not finite")).with_field(field));
        }
    }
    let reach = (frame.ascendant_deg - frame.reach_deg).rem_euclid(360.0);
    Ok(bodies
        .iter()
        .map(|body| {
            let house = house_of(body.longitude_deg, &frame.cusps_deg, 0.0);
            let above = (body.longitude_deg - frame.reach_deg).rem_euclid(360.0) < reach;
            HousePlacement {
                graha: body.graha,
                house,
                with_ascendant: house.get() == 1 || above,
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, reason = "tests fail by panicking")]

    use super::*;

    const EPS: f64 = 23.4393;

    fn equal_from(start: f64) -> [f64; 12] {
        let mut cusp = start - 30.0;
        [(); 12].map(|()| {
            cusp = (cusp + 30.0).rem_euclid(360.0);
            cusp
        })
    }

    #[test]
    fn a_planet_is_counted_by_the_cusps_alone() {
        // Unequal cusps across 0° Aries; a planet just short of a cusp
        // stays in the house before it, which Lilly's 5° would not.
        let mut cusps = equal_from(300.0);
        cusps[1] = 335.0;
        let frame = HouseFrame {
            cusps_deg: cusps,
            ascendant_deg: 300.0,
            reach_deg: 290.0,
        };
        let placed = place_in_houses(
            &frame,
            &[
                PlanetAt::new(Graha::Sun, 334.0),
                PlanetAt::new(Graha::Moon, 335.0),
                PlanetAt::new(Graha::Mars, 299.0),
                PlanetAt::new(Graha::Jupiter, 5.0),
            ],
        )
        .unwrap();
        let houses: Vec<(u8, bool)> = placed
            .iter()
            .map(|one| (one.house.get(), one.with_ascendant))
            .collect();
        assert_eq!(houses, [(1, true), (2, false), (12, true), (3, false)]);
    }

    #[test]
    fn the_reach_stops_at_the_hour_before() {
        let frame = HouseFrame {
            cusps_deg: equal_from(0.0),
            ascendant_deg: 0.0,
            reach_deg: 346.0,
        };
        let with = |longitude: f64| {
            place_in_houses(&frame, &[PlanetAt::new(Graha::Saturn, longitude)])
                .unwrap()
                .first()
                .unwrap()
                .with_ascendant
        };
        assert!(with(346.0) && with(359.9) && with(0.0));
        assert!(!with(345.9) && !with(30.0));
    }

    #[test]
    fn an_hour_is_wider_where_signs_rise_quickly() {
        // At London, Aries rises short and Libra long: the hour before an
        // Aries ascendant raised more longitude than before a Libra one.
        let london = 51.5;
        let asc = |armc: f64| rising_an_hour_before(armc + ASCENDANT_REACH_DEG, london, EPS);
        let span =
            |armc: f64| (asc(armc) - rising_an_hour_before(armc, london, EPS)).rem_euclid(360.0);
        // Aries rises with the meridian near 270° of right ascension,
        // Libra near 90°.
        assert!(
            span(270.0) > 2.0 * span(90.0),
            "{} {}",
            span(270.0),
            span(90.0)
        );
        assert!(span(90.0) < ASCENDANT_REACH_DEG && span(270.0) > ASCENDANT_REACH_DEG);
    }

    #[test]
    fn a_repeated_or_unreadable_planet_is_refused_by_field() {
        let frame = HouseFrame {
            cusps_deg: equal_from(0.0),
            ascendant_deg: 0.0,
            reach_deg: 346.0,
        };
        let twice = [
            PlanetAt::new(Graha::Sun, 1.0),
            PlanetAt::new(Graha::Sun, 2.0),
        ];
        assert_eq!(
            place_in_houses(&frame, &twice).unwrap_err().field(),
            Some("planets")
        );
        let nan = [PlanetAt::new(Graha::Sun, f64::NAN)];
        assert_eq!(
            place_in_houses(&frame, &nan).unwrap_err().field(),
            Some("planets[0].longitudeDeg")
        );
        let mut broken = frame;
        broken.cusps_deg[4] = f64::INFINITY;
        assert_eq!(
            place_in_houses(&broken, &[]).unwrap_err().field(),
            Some("cuspsDeg[4]")
        );
    }
}
