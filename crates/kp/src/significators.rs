//! The significators: which planets signify a house, strongest first,
//! and whose results a node gives (KP Reader VI).
//!
//! A house is signified, in the Reader's order, by (a) the planets in the
//! stars of its occupants, (b) its occupants, (c) the planets in the star
//! of its lord and (d) its lord; then by (e) the planets conjoined with
//! those and (f) the planets they aspect. A node gives, first, the results
//! of the planets it is conjoined with, then of the planet in whose star
//! it stands, then of the planets aspecting it, and last of its sign's
//! lord (crux C155).
//!
//! Both passages take a conjunction as the same sign ("Rahu is conjoined
//! with Venus in Meena") and an aspect as a graha's drishti by sign
//! ("aspected by Venus by its 7th aspect"): the seventh from every graha
//! and the special houses of Mars, Jupiter and Saturn, a node's own read
//! from `aspect.node_aspects`.

use serde::{Deserialize, Serialize};
use teistro_aspect::drishti::{Strength, quarters_under};
use teistro_core::Nas;
use teistro_core::catalogue::{Graha, Rashi};
use teistro_core::settings::NodeAspects;

use crate::chart::{KpChart, Planet, house_of, is_node};

/// The planets signifying one house, level by level.
///
/// A planet may stand at more than one level: the Moon in its own Hasta
/// in the 5th is both (a) and (b) of the 5th. Within a level the planets
/// keep the chart's order.
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
/// let second = significators.house(2).ok_or("twelve houses")?;
/// assert_eq!(second.occupants, [Graha::Moon, Graha::Mars, Graha::Rahu]);
/// assert_eq!(second.lord, Graha::Venus);
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct HouseSignificators {
    /// The house, 1 to 12.
    pub house: u8,
    /// (a) The planets in the stars of its occupants.
    pub in_occupants_stars: Vec<Graha>,
    /// (b) Its occupants.
    pub occupants: Vec<Graha>,
    /// (c) The planets in the star of its lord.
    pub in_lords_star: Vec<Graha>,
    /// (d) Its lord, the lord of its cusp's sign.
    pub lord: Graha,
    /// (e) The planets conjoined with any of (a) to (d), and not already
    /// among them.
    pub conjoined: Vec<Graha>,
    /// (f) The planets any of (a) to (d) aspect, and not already among
    /// them or (e).
    pub aspected: Vec<Graha>,
    /// The signs lying wholly inside the house, touching neither of its
    /// cusps (Reader III): reported, and not a level (crux C154).
    pub intercepted: Vec<Rashi>,
}

impl HouseSignificators {
    /// The four levels, (a) first.
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
    /// let second = significators.house(2).ok_or("twelve houses")?;
    /// let [_, occupants, _, lord] = second.levels();
    /// assert_eq!(occupants, second.occupants);
    /// assert_eq!(lord, [Graha::Venus]);
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn levels(&self) -> [&[Graha]; 4] {
        [
            &self.in_occupants_stars,
            &self.occupants,
            &self.in_lords_star,
            std::slice::from_ref(&self.lord),
        ]
    }

    /// Every planet at the four levels, strongest first, each once.
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
    /// let second = significators.house(2).ok_or("twelve houses")?;
    /// let order = second.in_order();
    /// // Each once, the strongest level first, the lord last.
    /// assert_eq!(order.last(), Some(&Graha::Venus));
    /// assert!(order.iter().all(|graha| order.iter().filter(|other| *other == graha).count() == 1));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn in_order(&self) -> Vec<Graha> {
        once_each(self.levels().into_iter().flatten().copied())
    }

    /// The lords of the intercepted signs, the baseline engine's fifth
    /// level (crux C154).
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem, Rashi};
    /// use teistro_core::settings::NodeAspects;
    /// use teistro_kp::{KpChart, Significators};
    ///
    /// // A first house of 70°, from 0° Aries to 10° Gemini: Taurus lies inside it.
    /// let degrees = [0, 70, 90, 120, 150, 180, 210, 240, 270, 300, 320, 340];
    /// let chart = KpChart::new(HouseSystem::Placidus, degrees.map(|deg| Nas::new(deg * Nas::PER_DEGREE)), []);
    /// let first = Significators::of(&chart, NodeAspects::None).houses[0].clone();
    /// assert_eq!(first.intercepted, [Rashi::Taurus]);
    /// assert_eq!(first.intercepted_lords(), [Graha::Venus]);
    /// ```
    #[must_use]
    pub fn intercepted_lords(&self) -> Vec<Graha> {
        once_each(self.intercepted.iter().map(|sign| sign.attributes().lord))
    }
}

/// Whose results a node gives, in the Reader's order.
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
/// let rahu = significators.node(Graha::Rahu).ok_or("the chart has Rahu")?;
/// assert_eq!(rahu.conjoined, [Graha::Moon, Graha::Mars]);
/// // 22° Taurus is Rohini, the Moon's star, in Venus's sign.
/// assert_eq!((rahu.star_lord, rahu.sign_lord), (Graha::Moon, Graha::Venus));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct NodeAgency {
    /// Rahu or Ketu.
    pub node: Graha,
    /// The planets in its sign.
    pub conjoined: Vec<Graha>,
    /// The lord of the star it stands in.
    pub star_lord: Graha,
    /// The planets aspecting it, the other node aside: it always stands
    /// opposite.
    pub aspecting: Vec<Graha>,
    /// The lord of its sign.
    pub sign_lord: Graha,
}

impl NodeAgency {
    /// Every planet the node gives the results of, in the Reader's order,
    /// each once.
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
    /// let rahu = significators.node(Graha::Rahu).ok_or("the chart has Rahu")?;
    /// assert_eq!(&rahu.in_order()[..2], [Graha::Moon, Graha::Mars]);
    /// assert_eq!(rahu.in_order().last(), Some(&Graha::Venus));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn in_order(&self) -> Vec<Graha> {
        once_each(
            self.conjoined
                .iter()
                .copied()
                .chain([self.star_lord])
                .chain(self.aspecting.iter().copied())
                .chain([self.sign_lord]),
        )
    }
}

/// The houses one planet signifies at each level: the inverse of the
/// houses' table.
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
/// let moon = significators.signified_by(Graha::Moon);
/// // In its own star in the 2nd, and lord of the 4th.
/// assert_eq!((moon.by_star.clone(), moon.occupies.clone(), moon.owns.clone()), (vec![2], vec![2], vec![4]));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Signified {
    /// The planet.
    pub graha: Graha,
    /// (a) The houses its star lord occupies.
    pub by_star: Vec<u8>,
    /// (b) The house it occupies, when the chart has it.
    pub occupies: Vec<u8>,
    /// (c) The houses its star lord owns.
    pub by_lords_star: Vec<u8>,
    /// (d) The houses it owns.
    pub owns: Vec<u8>,
}

/// A chart's significators.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_core::settings::NodeAspects;
/// use teistro_kp::{KpChart, Position, Significators};
///
/// // Equal cusps from 0° Aries; the Moon at 20° Taurus in the Moon's own
/// // Rohini, and Mars at 25° Taurus in Mrigashira, Mars's.
/// let cusps = std::array::from_fn(|h| Nas::new(30 * h as i64 * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let chart = KpChart::new(
///     HouseSystem::Placidus,
///     cusps,
///     [Position::new(Graha::Moon, at(50)), Position::new(Graha::Mars, at(55))],
/// );
/// let significators = Significators::of(&chart, NodeAspects::None);
/// let second = significators.house(2).unwrap();
/// assert_eq!(second.occupants, [Graha::Moon, Graha::Mars]);
/// assert_eq!(second.in_occupants_stars, [Graha::Moon, Graha::Mars]);
/// assert_eq!(second.lord, Graha::Venus);
/// // Mars owns the 1st and the 8th; nothing stands in Venus's stars.
/// assert_eq!(significators.signified_by(Graha::Mars).owns, [1, 8]);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct Significators {
    /// The twelve houses, the 1st first.
    pub houses: [HouseSignificators; 12],
    /// Whose results each node the chart has gives.
    pub nodes: Vec<NodeAgency>,
}

impl Significators {
    /// Reads a chart's significators, a node's aspects under `node_aspects`.
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
    /// assert_eq!(significators.houses.len(), 12);
    /// assert_eq!(significators.nodes.len(), 2);
    /// ```
    #[must_use]
    pub fn of(chart: &KpChart, node_aspects: NodeAspects) -> Significators {
        let cusps = chart.cusps.map(|cusp| cusp.longitude);
        let planets = &chart.planets;
        let aspects = |from: &Planet, to: &Planet| {
            let houses = (sign(to) + 12 - sign(from)) % 12 + 1;
            quarters_under(from.graha, houses, node_aspects) == Strength::Full
        };
        let houses = chart.cusps.map(|cusp| {
            let (house, lord) = (cusp.house, cusp.lords.sign);
            let occupants = grahas(planets.iter().filter(|planet| planet.house == house));
            let in_occupants_stars = grahas(
                planets
                    .iter()
                    .filter(|planet| occupants.contains(&planet.lords.star.lord)),
            );
            let in_lords_star = grahas(
                planets
                    .iter()
                    .filter(|planet| planet.lords.star.lord == lord),
            );
            let levels: Vec<&Planet> = planets
                .iter()
                .filter(|planet| {
                    in_occupants_stars.contains(&planet.graha)
                        || occupants.contains(&planet.graha)
                        || in_lords_star.contains(&planet.graha)
                        || planet.graha == lord
                })
                .collect();
            let beyond = |planet: &&Planet| {
                !levels
                    .iter()
                    .any(|significator| significator.graha == planet.graha)
            };
            let conjoined = grahas(planets.iter().filter(beyond).filter(|planet| {
                levels
                    .iter()
                    .any(|significator| sign(significator) == sign(planet))
            }));
            let aspected = grahas(
                planets
                    .iter()
                    .filter(beyond)
                    .filter(|planet| !conjoined.contains(&planet.graha))
                    .filter(|planet| {
                        levels
                            .iter()
                            .any(|significator| aspects(significator, planet))
                    }),
            );
            HouseSignificators {
                house,
                in_occupants_stars,
                occupants,
                in_lords_star,
                lord,
                conjoined,
                aspected,
                intercepted: intercepted(&cusps, house),
            }
        });
        let nodes = planets
            .iter()
            .filter(|planet| is_node(planet.graha))
            .map(|node| NodeAgency {
                node: node.graha,
                conjoined: grahas(
                    planets
                        .iter()
                        .filter(|planet| planet.graha != node.graha && sign(planet) == sign(node)),
                ),
                star_lord: node.lords.star.lord,
                // The other node always stands opposite and so always
                // "aspects" it: a fact of geometry, not an agency.
                aspecting: grahas(
                    planets
                        .iter()
                        .filter(|planet| !is_node(planet.graha) && aspects(planet, node)),
                ),
                sign_lord: node.lords.sign,
            })
            .collect();
        Significators { houses, nodes }
    }

    /// One house's significators, 1 to 12.
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
    /// assert!(significators.house(12).is_some());
    /// assert!(significators.house(13).is_none());
    /// ```
    #[must_use]
    pub fn house(&self, house: u8) -> Option<&HouseSignificators> {
        self.houses.get(usize::from(house).checked_sub(1)?)
    }

    /// Whose results a node gives, when the chart has it.
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
    /// assert_eq!(significators.node(Graha::Ketu).map(|ketu| ketu.node), Some(Graha::Ketu));
    /// assert!(significators.node(Graha::Mars).is_none());
    /// ```
    #[must_use]
    pub fn node(&self, node: Graha) -> Option<&NodeAgency> {
        self.nodes.iter().find(|agency| agency.node == node)
    }

    /// The houses a planet signifies at each level.
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
    /// // Mars owns Aries and Scorpio, the 1st and the 8th.
    /// assert_eq!(significators.signified_by(Graha::Mars).owns, [1, 8]);
    /// ```
    #[must_use]
    pub fn signified_by(&self, graha: Graha) -> Signified {
        let houses = |planets: fn(&HouseSignificators) -> &[Graha]| -> Vec<u8> {
            self.houses
                .iter()
                .filter(|house| planets(house).contains(&graha))
                .map(|house| house.house)
                .collect()
        };
        Signified {
            graha,
            by_star: houses(|house| &house.in_occupants_stars),
            occupies: houses(|house| &house.occupants),
            by_lords_star: houses(|house| &house.in_lords_star),
            owns: houses(|house| std::slice::from_ref(&house.lord)),
        }
    }
}

/// The sign a planet stands in, 0 for Aries.
fn sign(planet: &Planet) -> u8 {
    planet.longitude.sign_index().get()
}

/// The planets, in order.
fn grahas<'p>(planets: impl Iterator<Item = &'p Planet>) -> Vec<Graha> {
    planets.map(|planet| planet.graha).collect()
}

/// Each graha once, at its first place.
fn once_each(grahas: impl Iterator<Item = Graha>) -> Vec<Graha> {
    let mut seen = Vec::new();
    for graha in grahas {
        if !seen.contains(&graha) {
            seen.push(graha);
        }
    }
    seen
}

/// The signs no cusp falls in that lie inside a house.
fn intercepted(cusps: &[Nas; 12], house: u8) -> Vec<Rashi> {
    Rashi::ALL
        .into_iter()
        .filter(|rashi| {
            let index = rashi.id();
            !cusps
                .iter()
                .any(|cusp| u16::from(cusp.sign_index().get()) == index)
        })
        .filter(|rashi| {
            let start = Nas::new(i64::from(rashi.id()) * Nas::PER_SIGN);
            house_of(cusps, start) == house
        })
        .collect()
}
