//! The ruling planets at the moment of judgement (KP Reader VI).
//!
//! "Note the constellation in which the lagna at the moment of judgment
//! falls; then the lord of the sign where the ascendant is; find out in
//! which constellation Moon is passing and know the lord of the
//! constellation and lord of the rasi", and the lord of the day, taken
//! from sunrise to the next sunrise: "these five planets". A node in a
//! ruler's sign is taken as its agent, and a ruler deposited in the star
//! of a retrograde planet is rejected. Each fork is a setting, read here
//! as [`RulingRules`] (cruxes C150, C152, C153; the day, C151, is the
//! caller's, since only it knows the day).

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::settings::{NodeRulers, RetrogradeRejection, RulingCount, Settings};

use crate::chart::{KpChart, Planet, is_node};

/// The settings a reading of the ruling planets takes.
///
/// ```
/// use teistro_core::settings::{RulingCount, RetrogradeRejection};
/// use teistro_kp::RulingRules;
///
/// // The Reader with the subs counted and a retrograde planet's star or sub rejecting.
/// let mut rules = RulingRules::READER;
/// rules.count = RulingCount::WithSubs;
/// rules.retrograde_rejection = RetrogradeRejection::StarOrSub;
/// assert_ne!(rules, RulingRules::READER);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RulingRules {
    /// How many are counted (crux C150).
    pub count: RulingCount,
    /// When a node joins as an agent (crux C152).
    pub node_rulers: NodeRulers,
    /// Which a retrograde planet rejects (crux C153).
    pub retrograde_rejection: RetrogradeRejection,
}

impl RulingRules {
    /// KP Reader VI's reading: five, a node in a ruler's sign or beside
    /// one, and the rejection by a retrograde planet's star.
    pub const READER: RulingRules = RulingRules {
        count: RulingCount::Five,
        node_rulers: NodeRulers::SignOrConjoined,
        retrograde_rejection: RetrogradeRejection::Star,
    };

    /// The rules the settings' `kp` group gives.
    ///
    /// ```
    /// use teistro_core::settings::{DEFAULT_PROFILE, Profile, SettingsPatch};
    /// use teistro_kp::RulingRules;
    ///
    /// let profile = Profile::shipped(DEFAULT_PROFILE).ok_or("the default profile ships")?;
    /// let settings = profile.resolve(&SettingsPatch::default())?.settings;
    /// assert_eq!(RulingRules::of(&settings).count, settings.kp.ruling_count);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn of(settings: &Settings) -> RulingRules {
        RulingRules {
            count: settings.kp.ruling_count,
            node_rulers: settings.kp.node_rulers,
            retrograde_rejection: settings.kp.retrograde_rejection,
        }
    }
}

/// Why a planet is a ruler.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
///
/// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
/// // (its own star, Venus's sign), on a Thursday.
/// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let moon = Position::new(Graha::Moon, at(50));
/// let ruling = RulingPlanets::of(&KpChart::new(HouseSystem::Placidus, cusps, [moon]), Graha::Jupiter, RulingRules::READER);
/// use teistro_kp::Reason;
///
/// let ketu = ruling.ruler(Graha::Ketu).map(|ruler| ruler.reasons.clone());
/// assert_eq!(ketu, Some(vec![Reason::LagnaStar]));
/// // The Moon rules twice over: its own star, and nothing else.
/// assert_eq!(ruling.ruler(Graha::Moon).map(|ruler| ruler.reasons.clone()), Some(vec![Reason::MoonStar]));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[cfg_attr(feature = "schema", schemars(rename = "RulerReason"))]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Reason {
    /// The lord of the lagna's star.
    LagnaStar,
    /// The lord of the lagna's sign.
    LagnaSign,
    /// The lord of the lagna's sub, counted under `WITH_SUBS`.
    LagnaSub,
    /// The lord of the Moon's star.
    MoonStar,
    /// The lord of the Moon's sign.
    MoonSign,
    /// The lord of the Moon's sub, counted under `WITH_SUBS`.
    MoonSub,
    /// The lord of the day.
    DayLord,
    /// A node standing for a ruler.
    Agent {
        /// The ruler it stands for.
        of: Graha,
        /// In the ruler's sign, or beside it.
        by: Agency,
    },
}

/// How a node stands for a ruler.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
///
/// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
/// // (its own star, Venus's sign), on a Thursday.
/// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let moon = Position::new(Graha::Moon, at(50));
/// use teistro_kp::{Agency, Reason};
///
/// // Rahu in Aries, Mars's sign, stands for Mars.
/// let rahu = Position::new(Graha::Rahu, at(15)).retrograde(true);
/// let ketu = Position::new(Graha::Ketu, at(195)).retrograde(true);
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [moon, rahu, ketu]);
/// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
/// let reasons = ruling.ruler(Graha::Rahu).map(|ruler| ruler.reasons.clone()).unwrap_or_default();
/// assert!(reasons.contains(&Reason::Agent { of: Graha::Mars, by: Agency::InItsSign }));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Agency {
    /// In a sign the ruler owns.
    InItsSign,
    /// In the same sign as the ruler.
    Conjoined,
}

/// Why a ruler is rejected: it stands in the star, or the sub, of a
/// retrograde planet.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
///
/// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
/// // (its own star, Venus's sign), on a Thursday.
/// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let moon = Position::new(Graha::Moon, at(50));
/// // Mars in Purva Phalguni, the star of Venus, which is retrograde.
/// let mars = Position::new(Graha::Mars, at(140));
/// let venus = Position::new(Graha::Venus, at(10)).retrograde(true);
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [moon, mars, venus]);
/// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
/// use teistro_kp::Rejection;
///
/// let mars = ruling.ruler(Graha::Mars).ok_or("Mars rules the lagna's sign")?;
/// let rejection: Rejection = mars.rejected_by.ok_or("Venus is retrograde")?;
/// assert_eq!((rejection.retrograde, rejection.by_star), (Graha::Venus, true));
/// assert!(!ruling.accepted().contains(&Graha::Mars));
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Rejection {
    /// The retrograde planet.
    pub retrograde: Graha,
    /// Whether through its star (`true`) or its sub.
    pub by_star: bool,
}

/// One ruling planet, every reason it rules, and what rejects it.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
///
/// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
/// // (its own star, Venus's sign), on a Thursday.
/// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
/// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
/// let moon = Position::new(Graha::Moon, at(50));
/// // Mars in Purva Phalguni, the star of Venus, which is retrograde.
/// let mars = Position::new(Graha::Mars, at(140));
/// let venus = Position::new(Graha::Venus, at(10)).retrograde(true);
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [moon, mars, venus]);
/// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
/// // Venus is retrograde itself, which delays and does not reject.
/// let venus = ruling.ruler(Graha::Venus).ok_or("Venus rules the Moon's sign")?;
/// assert!(venus.retrograde);
/// # Ok::<(), &str>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Ruler {
    /// The planet.
    pub graha: Graha,
    /// Every reason it rules, the first the strongest.
    pub reasons: Vec<Reason>,
    /// Whether it is itself retrograde, which the Reader reads as delay
    /// and not as rejection.
    pub retrograde: bool,
    /// What rejects it, under the rules; `None` when it stands.
    pub rejected_by: Option<Rejection>,
    /// What would reject it under the other reading of crux C153, so the
    /// fork's reach is reported rather than hidden.
    pub rejected_by_sub: Option<Rejection>,
}

/// The ruling planets at a moment.
///
/// ```
/// use teistro_core::Nas;
/// use teistro_core::catalogue::{Graha, HouseSystem};
/// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
///
/// // The lagna at 5° Aries (Ashwini, Ketu's star; Mars's sign) and the
/// // Moon at 20° Taurus (Rohini, the Moon's star; Venus's sign), on a
/// // Thursday.
/// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
/// let moon = Position::new(Graha::Moon, Nas::new(50 * Nas::PER_DEGREE));
/// let chart = KpChart::new(HouseSystem::Placidus, cusps, [moon]);
/// let ruling = RulingPlanets::of(&chart, Graha::Jupiter, RulingRules::READER);
/// assert_eq!(
///     ruling.accepted(),
///     [Graha::Ketu, Graha::Mars, Graha::Moon, Graha::Venus, Graha::Jupiter],
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[non_exhaustive]
pub struct RulingPlanets {
    /// The rulers, each once, in the Reader's order: the lagna's star and
    /// sign lords, the Moon's, the day lord, then the nodes as agents.
    pub rulers: Vec<Ruler>,
    /// The rules they were read under.
    pub rules: RulingRules,
}

impl RulingPlanets {
    /// The ruling planets of a chart cast for the moment of judgement,
    /// with the day's lord.
    ///
    /// A chart without the Moon counts the lagna's lords and the day's.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
    ///
    /// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
    /// // (its own star, Venus's sign), on a Thursday.
    /// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
    /// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
    /// let moon = Position::new(Graha::Moon, at(50));
    /// // Without the Moon, the lagna's lords and the day's.
    /// let chart = KpChart::new(HouseSystem::Placidus, cusps, []);
    /// let ruling = RulingPlanets::of(&chart, Graha::Saturn, RulingRules::READER);
    /// assert_eq!(ruling.accepted(), [Graha::Ketu, Graha::Mars, Graha::Saturn]);
    /// # let _ = moon;
    /// ```
    #[must_use]
    pub fn of(chart: &KpChart, day_lord: Graha, rules: RulingRules) -> RulingPlanets {
        let with_subs = rules.count == RulingCount::WithSubs;
        let mut listed: Vec<Ruler> = Vec::new();
        let [first, ..] = &chart.cusps;
        let lagna = first.lords;
        add(&mut listed, lagna.star.lord, Reason::LagnaStar);
        add(&mut listed, lagna.sign, Reason::LagnaSign);
        if with_subs {
            add(&mut listed, lagna.sub.lord, Reason::LagnaSub);
        }
        if let Some(moon) = chart.planet(Graha::Moon) {
            add(&mut listed, moon.lords.star.lord, Reason::MoonStar);
            add(&mut listed, moon.lords.sign, Reason::MoonSign);
            if with_subs {
                add(&mut listed, moon.lords.sub.lord, Reason::MoonSub);
            }
        }
        add(&mut listed, day_lord, Reason::DayLord);

        // A node that is not already a ruler joins for one that is.
        let named: Vec<Graha> = listed.iter().map(|ruler| ruler.graha).collect();
        for node in chart.planets.iter().filter(|planet| is_node(planet.graha)) {
            if named.contains(&node.graha) {
                continue;
            }
            let agency = named.iter().find_map(|ruler| {
                if node.lords.sign == *ruler {
                    return Some((*ruler, Agency::InItsSign));
                }
                let beside = rules.node_rulers == NodeRulers::SignOrConjoined
                    && chart
                        .planet(*ruler)
                        .is_some_and(|planet| same_sign(planet, node));
                beside.then_some((*ruler, Agency::Conjoined))
            });
            if let Some((of, by)) = agency {
                add(&mut listed, node.graha, Reason::Agent { of, by });
            }
        }

        for ruler in &mut listed {
            let Some(planet) = chart.planet(ruler.graha) else {
                continue;
            };
            ruler.retrograde = planet.retrograde && !is_node(planet.graha);
            let by_star = rejecting(chart, planet.lords.star.lord).map(|retrograde| Rejection {
                retrograde,
                by_star: true,
            });
            let by_sub = rejecting(chart, planet.lords.sub.lord).map(|retrograde| Rejection {
                retrograde,
                by_star: false,
            });
            ruler.rejected_by = match rules.retrograde_rejection {
                RetrogradeRejection::StarOrSub => by_star.or(by_sub),
                RetrogradeRejection::Star | _ => by_star,
            };
            ruler.rejected_by_sub = by_sub;
        }
        RulingPlanets {
            rulers: listed,
            rules,
        }
    }

    /// The rulers that stand, in order.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
    ///
    /// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
    /// // (its own star, Venus's sign), on a Thursday.
    /// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
    /// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
    /// let moon = Position::new(Graha::Moon, at(50));
    /// let ruling = RulingPlanets::of(&KpChart::new(HouseSystem::Placidus, cusps, [moon]), Graha::Jupiter, RulingRules::READER);
    /// assert_eq!(ruling.accepted(), [Graha::Ketu, Graha::Mars, Graha::Moon, Graha::Venus, Graha::Jupiter]);
    /// ```
    #[must_use]
    pub fn accepted(&self) -> Vec<Graha> {
        self.rulers
            .iter()
            .filter(|ruler| ruler.rejected_by.is_none())
            .map(|ruler| ruler.graha)
            .collect()
    }

    /// One ruler, when it is one.
    ///
    /// ```
    /// use teistro_core::Nas;
    /// use teistro_core::catalogue::{Graha, HouseSystem};
    /// use teistro_kp::{KpChart, Position, RulingPlanets, RulingRules};
    ///
    /// // The lagna at 5° Aries (Ketu's star, Mars's sign), the Moon at 20° Taurus
    /// // (its own star, Venus's sign), on a Thursday.
    /// let cusps = std::array::from_fn(|h| Nas::new((5 + 30 * h as i64) * Nas::PER_DEGREE));
    /// let at = |deg: i64| Nas::new(deg * Nas::PER_DEGREE);
    /// let moon = Position::new(Graha::Moon, at(50));
    /// let ruling = RulingPlanets::of(&KpChart::new(HouseSystem::Placidus, cusps, [moon]), Graha::Jupiter, RulingRules::READER);
    /// assert!(ruling.ruler(Graha::Jupiter).is_some());
    /// assert!(ruling.ruler(Graha::Saturn).is_none());
    /// ```
    #[must_use]
    pub fn ruler(&self, graha: Graha) -> Option<&Ruler> {
        self.rulers.iter().find(|ruler| ruler.graha == graha)
    }
}

/// Adds a reason to a ruler, or the ruler with its first reason.
fn add(rulers: &mut Vec<Ruler>, graha: Graha, reason: Reason) {
    match rulers.iter_mut().find(|ruler| ruler.graha == graha) {
        Some(ruler) => ruler.reasons.push(reason),
        None => rulers.push(Ruler {
            graha,
            reasons: vec![reason],
            retrograde: false,
            rejected_by: None,
            rejected_by_sub: None,
        }),
    }
}

/// The lord that rejects, when it is a retrograde planet of the chart.
///
/// The nodes move backwards by nature, and a ruler in a node's star is not
/// rejected for it: the Reader's rejection is of a planet's retrogression,
/// which a node's motion is not.
fn rejecting(chart: &KpChart, lord: Graha) -> Option<Graha> {
    chart
        .planet(lord)
        .filter(|planet| planet.retrograde && !is_node(planet.graha))
        .map(|planet| planet.graha)
}

/// Whether two planets stand in one sign.
fn same_sign(a: &Planet, b: &Planet) -> bool {
    a.longitude.sign_index() == b.longitude.sign_index()
}
