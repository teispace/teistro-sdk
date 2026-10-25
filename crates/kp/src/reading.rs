//! A KP reading whole: the chart, its significators and the ruling
//! planets of its moment.

use serde::{Deserialize, Serialize};

use crate::chart::KpChart;
use crate::ruling::RulingPlanets;
use crate::significators::Significators;

/// A chart read as KP, with its significators and the ruling planets at
/// its moment: what a binding receives for each chart.
///
/// For a horary chart the cusps are the number's and the ruling planets
/// still the moment's own, which the Reader takes "at the moment of
/// judgment" (`03-design/kp.md`).
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_core::settings::NodeAspects;
/// use teistro_kp::{KpChart, Position, Significators};
///
/// // Equal cusps from 0° Aries; the Moon at 20° Taurus in its own Rohini,
/// // Mars at 25° in Mrigashira, Rahu with them at 22° and Ketu opposite.
/// let cusps = std::array::from_fn(|h| Nas::new(30 * h as i64 * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let chart = KpChart::new(
///     HouseSystem::Placidus,
///     cusps,
///     [
///         Position::new(Graha::Moon, at(50)),
///         Position::new(Graha::Mars, at(55)),
///         Position::new(Graha::Rahu, at(52)).retrograde(true),
///         Position::new(Graha::Ketu, at(232)).retrograde(true),
///     ],
/// );
/// let significators = Significators::of(&chart, NodeAspects::None);
/// use teistro_kp::{KpReading, RulingPlanets, RulingRules};
///
/// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
/// let reading = KpReading::new(chart, significators, ruling);
/// assert_eq!(reading.chart.planets.len(), 4);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct KpReading {
    /// The chart.
    pub chart: KpChart,
    /// Its significators.
    pub significators: Significators,
    /// The ruling planets at its moment.
    pub ruling: RulingPlanets,
}

impl KpReading {
    /// The three together.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_core::settings::NodeAspects;
    /// use teistro_kp::{KpChart, Position, Significators};
    ///
    /// // Equal cusps from 0° Aries; the Moon at 20° Taurus in its own Rohini,
    /// // Mars at 25° in Mrigashira, Rahu with them at 22° and Ketu opposite.
    /// let cusps = std::array::from_fn(|h| Nas::new(30 * h as i64 * Nas::PER_DEGREE));
    /// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
    /// let chart = KpChart::new(
    ///     HouseSystem::Placidus,
    ///     cusps,
    ///     [
    ///         Position::new(Graha::Moon, at(50)),
    ///         Position::new(Graha::Mars, at(55)),
    ///         Position::new(Graha::Rahu, at(52)).retrograde(true),
    ///         Position::new(Graha::Ketu, at(232)).retrograde(true),
    ///     ],
    /// );
    /// let significators = Significators::of(&chart, NodeAspects::None);
    /// use teistro_kp::{KpReading, RulingPlanets, RulingRules};
    ///
    /// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
    /// let reading = KpReading::new(chart.clone(), significators.clone(), ruling.clone());
    /// assert_eq!((reading.chart, reading.significators, reading.ruling), (chart, significators, ruling));
    /// ```
    #[must_use]
    pub const fn new(
        chart: KpChart,
        significators: Significators,
        ruling: RulingPlanets,
    ) -> KpReading {
        KpReading {
            chart,
            significators,
            ruling,
        }
    }
}
