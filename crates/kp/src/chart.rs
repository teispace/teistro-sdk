//! A chart as KP reads it: every cusp and every planet with its lords,
//! and each planet in the house whose cusp it follows.

use serde::{Deserialize, Serialize};
use teistro_core::Nas;
use teistro_core::catalogue::{Graha, HouseSystem};

use crate::chain::{Lords, lords};

/// Where a planet stands, what a [`KpChart`] is built from.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::Graha;
/// use teistro_kp::Position;
///
/// let saturn = Position::new(Graha::Saturn, Nas::new(140 * Nas::PER_DEGREE)).retrograde(true);
/// assert!(saturn.retrograde);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Position {
    /// The planet.
    pub graha: Graha,
    /// Its longitude in the chart's zodiac.
    pub longitude: Nas,
    /// Whether it is moving backwards through the zodiac.
    pub retrograde: bool,
}

impl Position {
    /// A planet at a longitude, moving forwards.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::Graha;
    /// use teistro_kp::Position;
    ///
    /// assert!(!Position::new(Graha::Sun, Nas::ZERO).retrograde);
    /// ```
    #[must_use]
    pub const fn new(graha: Graha, longitude: Nas) -> Position {
        Position {
            graha,
            longitude,
            retrograde: false,
        }
    }

    /// The same planet, retrograde or not.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::Graha;
    /// use teistro_kp::Position;
    ///
    /// let mars = Position::new(Graha::Mars, Nas::ZERO);
    /// assert_eq!(mars.retrograde(true).retrograde(false), mars);
    /// ```
    #[must_use]
    pub const fn retrograde(self, retrograde: bool) -> Position {
        Position { retrograde, ..self }
    }
}

/// A cusp and its lords.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position};
///
/// // Cusps 30° apart from 10° Aries; the Sun at 5° Aries, Saturn retrograde
/// // at 20° Leo.
/// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
/// let sun = Position::new(Graha::Sun, Nas::new(5 * Nas::PER_DEGREE));
/// let saturn = Position::new(Graha::Saturn, Nas::new(140 * Nas::PER_DEGREE)).retrograde(true);
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [sun, saturn]);
/// let first = chart.cusp(1).ok_or("twelve cusps")?;
/// // 10° Aries: Mars's sign, Ashwini, Ketu's star.
/// assert_eq!((first.house, first.lords.sign, first.lords.star.lord), (1, Graha::Mars, Graha::Ketu));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Cusp {
    /// The house the cusp opens, 1 to 12.
    pub house: u8,
    /// Its longitude in the chart's zodiac.
    pub longitude: Nas,
    /// Its sign, star, sub and sub-sub lords.
    pub lords: Lords,
}

/// A planet, its house and its lords.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position};
///
/// // Cusps 30° apart from 10° Aries; the Sun at 5° Aries, Saturn retrograde
/// // at 20° Leo.
/// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
/// let sun = Position::new(Graha::Sun, Nas::new(5 * Nas::PER_DEGREE));
/// let saturn = Position::new(Graha::Saturn, Nas::new(140 * Nas::PER_DEGREE)).retrograde(true);
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [sun, saturn]);
/// let saturn = chart.planet(Graha::Saturn).ok_or("Saturn was given")?;
/// // 20° Leo follows the 5th cusp at 10° Leo; Purva Phalguni, Venus's star.
/// assert_eq!((saturn.house, saturn.retrograde, saturn.lords.star.lord), (5, true, Graha::Venus));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Planet {
    /// The planet.
    pub graha: Graha,
    /// Its longitude in the chart's zodiac.
    pub longitude: Nas,
    /// Whether it is moving backwards through the zodiac.
    pub retrograde: bool,
    /// The house whose cusp it follows, 1 to 12.
    pub house: u8,
    /// Its sign, star, sub and sub-sub lords.
    pub lords: Lords,
}

/// A chart as KP reads it.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position};
///
/// // Twelve cusps 30° apart from 10° Aries, and the Sun at 5° Aries,
/// // which follows the 12th cusp at 10° Pisces.
/// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
/// let sun = Position::new(Graha::Sun, Nas::new(5 * Nas::PER_DEGREE));
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [sun]);
/// assert_eq!(chart.planet(Graha::Sun).map(|sun| sun.house), Some(12));
/// assert_eq!(chart.cusp(1).map(|cusp| cusp.lords.star.lord), Some(Graha::Ketu));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct KpChart {
    /// The house system the cusps were built by: Placidus unless the
    /// settings or the polar policy said otherwise.
    pub system: HouseSystem,
    /// The twelve cusps, the first house's first.
    pub cusps: [Cusp; 12],
    /// The planets, in the order given.
    pub planets: Vec<Planet>,
}

impl KpChart {
    /// Reads the cusps and the planets.
    #[must_use]
    pub fn new(
        system: HouseSystem,
        cusps: [Nas; 12],
        planets: impl IntoIterator<Item = Position>,
    ) -> KpChart {
        let planets = planets
            .into_iter()
            .map(|position| Planet {
                graha: position.graha,
                longitude: position.longitude,
                retrograde: position.retrograde,
                house: house_of(&cusps, position.longitude),
                lords: lords(position.longitude),
            })
            .collect();
        KpChart {
            system,
            cusps: std::array::from_fn(|index| {
                let longitude = cusps.get(index).copied().unwrap_or_default();
                Cusp {
                    house: house_number(index),
                    longitude,
                    lords: lords(longitude),
                }
            }),
            planets,
        }
    }

    /// A cusp by its house, 1 to 12.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_kp::{KpChart, Position};
    ///
    /// // Cusps 30° apart from 10° Aries; the Sun at 5° Aries, Saturn retrograde
    /// // at 20° Leo.
    /// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
    /// let sun = Position::new(Graha::Sun, Nas::new(5 * Nas::PER_DEGREE));
    /// let saturn = Position::new(Graha::Saturn, Nas::new(140 * Nas::PER_DEGREE)).retrograde(true);
    /// let chart = KpChart::new(HouseSystem::Placidus, cusps, [sun, saturn]);
    /// assert_eq!(chart.cusp(12).map(|cusp| cusp.longitude), Some(Nas::new(340 * Nas::PER_DEGREE)));
    /// assert!(chart.cusp(0).is_none() && chart.cusp(13).is_none());
    /// ```
    #[must_use]
    pub fn cusp(&self, house: u8) -> Option<&Cusp> {
        self.cusps.get(usize::from(house).checked_sub(1)?)
    }

    /// A planet, when the chart has it.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_kp::{KpChart, Position};
    ///
    /// // Cusps 30° apart from 10° Aries; the Sun at 5° Aries, Saturn retrograde
    /// // at 20° Leo.
    /// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
    /// let sun = Position::new(Graha::Sun, Nas::new(5 * Nas::PER_DEGREE));
    /// let saturn = Position::new(Graha::Saturn, Nas::new(140 * Nas::PER_DEGREE)).retrograde(true);
    /// let chart = KpChart::new(HouseSystem::Placidus, cusps, [sun, saturn]);
    /// assert_eq!(chart.planet(Graha::Sun).map(|sun| sun.house), Some(12));
    /// // The chart has only the planets it was given.
    /// assert!(chart.planet(Graha::Moon).is_none());
    /// ```
    #[must_use]
    pub fn planet(&self, graha: Graha) -> Option<&Planet> {
        self.planets.iter().find(|planet| planet.graha == graha)
    }
}

/// The house whose cusp a longitude follows: the cusp it is the shortest
/// arc past. KP takes a cusp as its house's start, so a planet a
/// nanoarcsecond short of the 2nd cusp is in the 1st.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_kp::house_of;
///
/// let cusps = std::array::from_fn(|h| Nas::new((10 + 30 * h as i64) * Nas::PER_DEGREE));
/// // On the 2nd cusp is the 2nd house; a nanoarcsecond short of it, the 1st.
/// let second = Nas::new(40 * Nas::PER_DEGREE);
/// assert_eq!(house_of(&cusps, second), 2);
/// assert_eq!(house_of(&cusps, Nas::new(second.get() - 1)), 1);
/// ```
#[must_use]
pub fn house_of(cusps: &[Nas; 12], at: Nas) -> u8 {
    let (index, _) = cusps
        .iter()
        .enumerate()
        .min_by_key(|(_, cusp)| cusp.arc_to(at))
        .unwrap_or((0, &Nas::ZERO));
    house_number(index)
}

/// Whether a graha is Rahu or Ketu.
pub(crate) const fn is_node(graha: Graha) -> bool {
    matches!(graha, Graha::Rahu | Graha::Ketu)
}

/// House `index + 1`.
#[allow(clippy::cast_possible_truncation, reason = "an index below twelve")]
const fn house_number(index: usize) -> u8 {
    index as u8 + 1
}
