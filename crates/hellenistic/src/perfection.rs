//! Whether a horary matter is brought to pass: Lilly's relations between
//! two significators (*Christian Astrology*, pp. 107–113) and the ways of
//! perfection built from them (ch. XXI, pp. 125–127;
//! `03-design/hellenistic-perfection.md`).
//!
//! Every relation is a statement about the **order** in which aspects
//! perfect and planets station, so each is read off one [`AspectTimeline`]: the
//! contacts (exact Ptolemaic aspects) and stations ahead of the figure. The
//! SDK searches the ephemeris for it; [`AspectTimeline::projected`] carries each
//! planet on at its motion of the moment, which sees no station. Nothing
//! here is a verdict: [`Matter`] reports each relation that holds and the
//! facts it rests on.
//!
//! ```
//! use teistro_core::catalogue::Graha;
//! use teistro_core::house::House;
//! use teistro_hellenistic::{AspectTimeline, DignityRules, PerfectionRules, Sect, Standing, perfection};
//!
//! // Lilly p. 113: Mercury in 10° Aries, Mars 12°, Jupiter 13°. Mercury
//! // strives to come to Mars, but Mars first gets to Jupiter.
//! // The four others stand still where they meet none of the three.
//! let places = [42.0, 13.0, 12.0, 162.0, 222.0, 10.0, 342.0];
//! let speeds = [0.0, 0.2, 0.7, 0.0, 0.0, 1.4, 0.0];
//! let sky = AspectTimeline::projected(places, speeds, 3.0)?;
//! let standing = Standing { houses: [House::try_new(1)?; 7], sect: Sect::Day, dignities: &DignityRules::LILLY };
//! let matter = perfection(&sky, Graha::Mercury, Graha::Mars, &standing, &PerfectionRules::LILLY)?;
//! let first = matter.impediments.first().expect("Mars meets Jupiter first");
//! assert_eq!((first.significator, first.third), (Graha::Mars, Some(Graha::Jupiter)));
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use serde::{Deserialize, Serialize};
use teistro_core::catalogue::Graha;
use teistro_core::error::Error;
use teistro_core::house::House;

use crate::considerations::{LILLY_ORBS_DEG, PtolemaicAspect, check_orbs};
use crate::dignity::{CHALDEAN_ORDER, DignityRules, EssentialDignity, Sect, essential_dignity};
use crate::fortitude::Fortitudes;

/// The degrees of a sign.
const SIGN_DEG: f64 = 30.0;

/// The eight angles from one longitude to another at which the two stand
/// in a Ptolemaic aspect: the conjunction and the opposition once, the
/// other three on either side.
const SIDES: [(f64, PtolemaicAspect); 8] = [
    (0.0, PtolemaicAspect::Conjunction),
    (60.0, PtolemaicAspect::Sextile),
    (90.0, PtolemaicAspect::Square),
    (120.0, PtolemaicAspect::Trine),
    (180.0, PtolemaicAspect::Opposition),
    (240.0, PtolemaicAspect::Trine),
    (270.0, PtolemaicAspect::Square),
    (300.0, PtolemaicAspect::Sextile),
];

/// The Ptolemaic aspect two planets stand in when one's longitude less the
/// other's is `angle_deg`, to within a microdegree; `None` between them.
///
/// A search over the separation of two planets on a lattice of 30° finds
/// every aspect and the semisextiles and quincunxes between; this keeps
/// the five.
///
/// ```
/// use teistro_hellenistic::{PtolemaicAspect, aspect_at};
///
/// assert_eq!(aspect_at(270.0), Some(PtolemaicAspect::Square));
/// assert_eq!(aspect_at(-60.0), Some(PtolemaicAspect::Sextile));
/// assert_eq!(aspect_at(150.0), None);
/// ```
#[must_use]
pub fn aspect_at(angle_deg: f64) -> Option<PtolemaicAspect> {
    let angle = angle_deg.rem_euclid(360.0);
    SIDES
        .iter()
        .find(|(side, _)| {
            let off = (angle - side).rem_euclid(360.0);
            off.min(360.0 - off) < 1e-6
        })
        .map(|&(_, aspect)| aspect)
}

/// Where a planet stands in the Chaldean order: 0 for Saturn, the most
/// weighty, to 6 for the Moon, the lightest.
fn weight(planet: Graha) -> Option<usize> {
    CHALDEAN_ORDER.iter().position(|&each| each == planet)
}

/// A planet outside the seven, refused under `field`.
fn not_one_of_the_seven(planet: Graha, field: &str) -> Error {
    Error::invalid_arg(format!("{planet:?} is not one of the seven")).with_field(field)
}

/// Two planets at an exact Ptolemaic aspect.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    /// The two, the more weighty first.
    pub planets: [Graha; 2],
    /// The aspect.
    pub aspect: PtolemaicAspect,
    /// Days after the figure.
    pub days: f64,
}

impl Contact {
    /// The contact of two of the seven, put in the Chaldean order.
    ///
    /// # Errors
    ///
    /// A planet outside the seven, or the same planet twice, named
    /// `contacts`.
    pub fn new(
        first: Graha,
        second: Graha,
        aspect: PtolemaicAspect,
        days: f64,
    ) -> Result<Contact, Error> {
        let (Some(a), Some(b)) = (weight(first), weight(second)) else {
            let outside = if weight(first).is_none() {
                first
            } else {
                second
            };
            return Err(not_one_of_the_seven(outside, "contacts"));
        };
        if a == b {
            return Err(
                Error::invalid_arg(format!("{first:?} cannot aspect itself"))
                    .with_field("contacts"),
            );
        }
        let planets = if a < b {
            [first, second]
        } else {
            [second, first]
        };
        Ok(Contact {
            planets,
            aspect,
            days,
        })
    }

    /// Whether the planet is one of the two.
    #[must_use]
    pub fn involves(&self, planet: Graha) -> bool {
        self.planets.contains(&planet)
    }

    /// The other of the two, when the planet is one of them.
    #[must_use]
    pub fn other(&self, planet: Graha) -> Option<Graha> {
        match self.planets {
            [first, second] if first == planet => Some(second),
            [first, second] if second == planet => Some(first),
            _ => None,
        }
    }
}

/// A planet standing still before it turns.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Station {
    /// The planet.
    pub planet: Graha,
    /// Days after the figure.
    pub days: f64,
    /// Whether it turns retrograde, rather than direct.
    pub retrograde: bool,
}

impl Station {
    /// The station `days` after the figure.
    ///
    /// # Errors
    ///
    /// A planet outside the seven, or a time that is not a finite number
    /// of days, zero or more, named `stations`.
    pub fn new(planet: Graha, days: f64, retrograde: bool) -> Result<Station, Error> {
        if weight(planet).is_none() {
            return Err(not_one_of_the_seven(planet, "stations"));
        }
        if !(days.is_finite() && days >= 0.0) {
            return Err(
                Error::invalid_arg(format!("a station {days} days ahead")).with_field("stations")
            );
        }
        Ok(Station {
            planet,
            days,
            retrograde,
        })
    }
}

/// What lies ahead of a figure: its seven planets' places and motions, and
/// every contact and station between them up to a horizon, in time order.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct AspectTimeline {
    places: [f64; 7],
    speeds: [f64; 7],
    horizon_days: f64,
    contacts: Vec<Contact>,
    stations: Vec<Station>,
}

impl AspectTimeline {
    /// A timeline from a search: the seven's longitudes and daily motions
    /// in the Chaldean order at the figure, the horizon searched to, and
    /// what was found. Contacts and stations past the horizon are dropped,
    /// and the rest are put in time order.
    ///
    /// # Errors
    ///
    /// A place or motion that is not finite, named `planets`; a horizon
    /// that is not a finite number of days, zero or more, named
    /// `horizonDays`; a contact at a time that is not one, named
    /// `contacts`.
    pub fn new(
        places: [f64; 7],
        speeds: [f64; 7],
        horizon_days: f64,
        mut contacts: Vec<Contact>,
        mut stations: Vec<Station>,
    ) -> Result<AspectTimeline, Error> {
        if places.iter().chain(&speeds).any(|value| !value.is_finite()) {
            return Err(
                Error::invalid_arg("a planet's place or motion is not finite")
                    .with_field("planets"),
            );
        }
        if !(horizon_days.is_finite() && horizon_days >= 0.0) {
            return Err(
                Error::invalid_arg(format!("a horizon of {horizon_days} days"))
                    .with_field("horizonDays")
                    .with_hint("a finite number of days, zero or more"),
            );
        }
        if let Some(bad) = contacts
            .iter()
            .find(|contact| !(contact.days.is_finite() && contact.days >= 0.0))
        {
            return Err(
                Error::invalid_arg(format!("a contact {} days ahead", bad.days))
                    .with_field("contacts"),
            );
        }
        contacts.retain(|contact| contact.days <= horizon_days);
        contacts.sort_by(|a, b| a.days.total_cmp(&b.days));
        if let Some(bad) = stations
            .iter()
            .find(|station| !(station.days.is_finite() && station.days >= 0.0))
        {
            return Err(
                Error::invalid_arg(format!("a station {} days ahead", bad.days))
                    .with_field("stations"),
            );
        }
        stations.retain(|station| station.days <= horizon_days);
        stations.sort_by(|a, b| a.days.total_cmp(&b.days));
        Ok(AspectTimeline {
            places,
            speeds,
            horizon_days,
            contacts,
            stations,
        })
    }

    /// The timeline the motions of the moment give: each planet carried on
    /// at its present speed, so every pair perfects its aspects at even
    /// intervals and no planet ever stations.
    ///
    /// ```
    /// use teistro_core::catalogue::Graha;
    /// use teistro_hellenistic::{PtolemaicAspect, AspectTimeline};
    ///
    /// // Lilly p. 107: Mars in 10° Aries, Mercury in 5°, both direct.
    /// // The other five stand still where they meet neither.
    /// let places = [40.0, 160.0, 10.0, 220.0, 340.0, 5.0, 45.0];
    /// let sky = AspectTimeline::projected(places, [0.0, 0.0, 0.7, 0.0, 0.0, 1.5, 0.0], 7.0)?;
    /// let first = sky.contacts().first().expect("Mercury reaches Mars");
    /// assert_eq!((first.planets, first.aspect), ([Graha::Mars, Graha::Mercury], PtolemaicAspect::Conjunction));
    /// assert!((first.days - 5.0 / 0.8).abs() < 1e-9);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`AspectTimeline::new`].
    pub fn projected(
        places: [f64; 7],
        speeds: [f64; 7],
        horizon_days: f64,
    ) -> Result<AspectTimeline, Error> {
        // Checked before the walk, which a horizon or speed out of range
        // would never end.
        AspectTimeline::new(places, speeds, horizon_days, Vec::new(), Vec::new())?;
        let mut contacts = Vec::new();
        for (k, ((&first, &first_deg), &first_speed)) in
            CHALDEAN_ORDER.iter().zip(&places).zip(&speeds).enumerate()
        {
            for ((&second, &second_deg), &second_speed) in
                CHALDEAN_ORDER.iter().zip(&places).zip(&speeds).skip(k + 1)
            {
                let rate = first_speed - second_speed;
                for (days, aspect) in contacts_ahead(first_deg - second_deg, rate, horizon_days) {
                    contacts.push(Contact {
                        planets: [first, second],
                        aspect,
                        days,
                    });
                }
            }
        }
        AspectTimeline::new(places, speeds, horizon_days, contacts, Vec::new())
    }

    /// The contacts, in time order.
    #[must_use]
    pub fn contacts(&self) -> &[Contact] {
        &self.contacts
    }

    /// The stations, in time order.
    #[must_use]
    pub fn stations(&self) -> &[Station] {
        &self.stations
    }

    /// How many days ahead the timeline runs.
    #[must_use]
    pub const fn horizon_days(&self) -> f64 {
        self.horizon_days
    }

    /// A planet's longitude and daily motion at the figure.
    fn motion(&self, planet: Graha) -> Option<(f64, f64)> {
        let at = weight(planet)?;
        Some((*self.places.get(at)?, *self.speeds.get(at)?))
    }
}

/// Each time within the horizon at which an arc changing at `rate` degrees
/// a day reaches one of the eight angles, with the aspect there.
fn contacts_ahead(arc_deg: f64, rate: f64, horizon_days: f64) -> Vec<(f64, PtolemaicAspect)> {
    let mut found = Vec::new();
    if rate == 0.0 {
        return found;
    }
    let period = 360.0 / rate.abs();
    for (side, aspect) in SIDES {
        let mut days = closing_arc(arc_deg, side, rate) / rate.abs();
        while days <= horizon_days {
            found.push((days, aspect));
            days += period;
        }
    }
    found
}

/// The first time an arc changing at `rate` degrees a day reaches one of
/// the eight angles, with the aspect there; `None` for an arc standing
/// still.
fn first_ahead(arc_deg: f64, rate: f64) -> Option<(f64, PtolemaicAspect)> {
    if rate == 0.0 {
        return None;
    }
    SIDES
        .iter()
        .map(|&(side, aspect)| (closing_arc(arc_deg, side, rate) / rate.abs(), aspect))
        .min_by(|a, b| a.0.total_cmp(&b.0))
}

/// The arc an angle changing at `rate` must still run to reach `side`.
fn closing_arc(arc_deg: f64, side: f64, rate: f64) -> f64 {
    if rate > 0.0 {
        (side - arc_deg).rem_euclid(360.0)
    } else {
        (arc_deg - side).rem_euclid(360.0)
    }
}

/// Days until a planet leaves its sign at its motion of the moment, by
/// the end it is moving toward; infinite for a planet standing still.
#[must_use]
pub fn days_in_sign(longitude_deg: f64, speed: f64) -> f64 {
    let degree = longitude_deg.rem_euclid(SIGN_DEG);
    if speed > 0.0 {
        (SIGN_DEG - degree) / speed
    } else if speed < 0.0 {
        degree / -speed
    } else {
        f64::INFINITY
    }
}

/// How the significators' perfection is read.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct PerfectionRules {
    /// Each planet's whole orb in the Chaldean order, half of which counts
    /// toward an application and a separation: Lilly's first column of
    /// p. 107 unless said, as [`ConsiderationRules`](crate::ConsiderationRules) has it.
    pub orbs_deg: [f64; 7],
    /// How many days ahead to look; unset, until the swifter significator
    /// leaves its sign (crux C232).
    pub horizon_days: Option<f64>,
}

impl PerfectionRules {
    /// Lilly's orbs, and the swifter significator's sign as the horizon.
    pub const LILLY: PerfectionRules = PerfectionRules {
        orbs_deg: LILLY_ORBS_DEG,
        horizon_days: None,
    };

    /// Reads the rules from JSON, every member optional and Lilly's when
    /// left out; a refusal is named from the root, as
    /// `perfection.horizonDays`.
    ///
    /// ```
    /// use teistro_hellenistic::PerfectionRules;
    ///
    /// let rules = PerfectionRules::from_json(r#"{"horizonDays": 30}"#)?;
    /// assert_eq!(rules.horizon_days, Some(30.0));
    /// let back = PerfectionRules::from_json(r#"{"horizonDays": -1}"#).unwrap_err();
    /// assert_eq!(back.field(), Some("perfection.horizonDays"));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Text that is not the record, a key it does not read, or a value
    /// out of range.
    pub fn from_json(text: &str) -> Result<PerfectionRules, Error> {
        let rules: PerfectionRules = teistro_core::strict::read(text, "perfection")?;
        rules.check().map_err(|why| why.under("perfection"))?;
        Ok(rules)
    }

    /// The horizon a positive finite number of days and every orb one of
    /// degrees, naming the field that is not.
    fn check(&self) -> Result<(), Error> {
        let horizon = self.horizon_days.unwrap_or(1.0);
        if !(horizon.is_finite() && horizon > 0.0) {
            let days = horizon;
            return Err(Error::invalid_arg(format!("a horizon of {days} days"))
                .with_field("horizonDays")
                .with_hint("a finite number of days, more than zero"));
        }
        check_orbs(&self.orbs_deg)
    }

    /// The days to search ahead of a figure for two significators: the
    /// rules' horizon, or the days until the swifter of the two leaves its
    /// sign at its motion of the moment.
    ///
    /// # Errors
    ///
    /// A significator outside the seven, named `querent` or `quesited`.
    pub fn horizon_for(
        &self,
        places: &[f64; 7],
        speeds: &[f64; 7],
        querent: Graha,
        quesited: Graha,
    ) -> Result<f64, Error> {
        if let Some(days) = self.horizon_days {
            return Ok(days);
        }
        let motion = |planet: Graha, field: &str| {
            weight(planet)
                .and_then(|at| Some((*places.get(at)?, *speeds.get(at)?)))
                .ok_or_else(|| not_one_of_the_seven(planet, field))
        };
        let (a, b) = (motion(querent, "querent")?, motion(quesited, "quesited")?);
        let (place, speed) = if a.1.abs() >= b.1.abs() { a } else { b };
        Ok(days_in_sign(place, speed))
    }

    /// Half of each planet's orb, added: the arc within which two planets
    /// apply or separate (p. 110).
    fn moieties(&self, first: Graha, second: Graha) -> f64 {
        let orb = |planet: Graha| {
            weight(planet)
                .and_then(|at| self.orbs_deg.get(at))
                .copied()
                .unwrap_or(0.0)
        };
        f64::midpoint(orb(first), orb(second))
    }
}

impl Default for PerfectionRules {
    fn default() -> PerfectionRules {
        PerfectionRules::LILLY
    }
}

/// Whose perfection is asked: the two significators, named or found by
/// Lilly's rule, and the rules it is read under.
///
/// The querent's significator is the lord of the Ascendant unless named;
/// the quesited's is named, or the lord of the sign on the cusp of the
/// house of the matter (`house`), in the fortitudes' own house division.
///
/// ```
/// use teistro_core::catalogue::Graha;
/// use teistro_core::house::House;
/// use teistro_hellenistic::PerfectionRequest;
///
/// // "Shall I marry?": the seventh house's lord.
/// let marriage = PerfectionRequest::of_house(House::try_new(7)?);
/// assert_eq!(marriage.querent, None);
/// // Or both named, as a request writes them, bare or in full.
/// let named = PerfectionRequest::from_json(r#"{"querent": "VENUS", "quesited": "graha.MARS", "rules": {"horizonDays": 30}}"#)?;
/// assert_eq!(named, PerfectionRequest::between(Graha::Venus, Graha::Mars).with_rules(teistro_hellenistic::PerfectionRules {
///     horizon_days: Some(30.0),
///     ..teistro_hellenistic::PerfectionRules::LILLY
/// }));
/// // Neither the quesited nor its house is no question.
/// let empty = PerfectionRequest::from_json("{}").unwrap_err();
/// assert_eq!(empty.field(), Some("perfection.quesited"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase", default)]
pub struct PerfectionRequest {
    /// The querent's significator; unset, the lord of the Ascendant.
    pub querent: Option<Graha>,
    /// The quesited's significator, or unset for the lord of `house`.
    pub quesited: Option<Graha>,
    /// The house of the matter, whose cusp's lord signifies the quesited
    /// when `quesited` is unset.
    pub house: Option<House>,
    /// How the perfection is read.
    pub rules: PerfectionRules,
}

impl PerfectionRequest {
    /// A question of the matter of one house: the Ascendant's lord for
    /// the querent, the house's for the quesited.
    #[must_use]
    pub const fn of_house(house: House) -> PerfectionRequest {
        PerfectionRequest {
            querent: None,
            quesited: None,
            house: Some(house),
            rules: PerfectionRules::LILLY,
        }
    }

    /// A question between two named significators.
    #[must_use]
    pub const fn between(querent: Graha, quesited: Graha) -> PerfectionRequest {
        PerfectionRequest {
            querent: Some(querent),
            quesited: Some(quesited),
            house: None,
            rules: PerfectionRules::LILLY,
        }
    }

    /// The same question, the querent signified by another planet than
    /// the Ascendant's lord.
    #[must_use]
    pub const fn with_querent(mut self, querent: Graha) -> PerfectionRequest {
        self.querent = Some(querent);
        self
    }

    /// The same question, read under other rules.
    #[must_use]
    pub const fn with_rules(mut self, rules: PerfectionRules) -> PerfectionRequest {
        self.rules = rules;
        self
    }

    /// Reads the request from JSON, `{"querent", "quesited", "house",
    /// "rules"}`, a significator by its key bare or in full; a refusal is
    /// named from the root, as `perfection.rules.horizonDays`.
    ///
    /// # Errors
    ///
    /// Text that is not the record, a key it does not read, rules out of
    /// range, and a question that names neither the quesited nor its
    /// house, or both.
    pub fn from_json(text: &str) -> Result<PerfectionRequest, Error> {
        let request: PerfectionRequest = teistro_core::strict::read(text, "perfection")?;
        request.check().map_err(|why| why.under("perfection"))?;
        Ok(request)
    }

    /// The quesited named one way, and the rules in range.
    fn check(&self) -> Result<(), Error> {
        match (self.quesited, self.house) {
            (None, None) => {
                return Err(Error::invalid_arg(
                    "no quesited's significator, and no house of the matter",
                )
                .with_field("quesited")
                .with_hint(
                    "name the quesited's significator, or the house of the matter as `house`",
                ));
            }
            (Some(_), Some(_)) => {
                return Err(Error::invalid_arg(
                    "both a quesited's significator and a house of the matter",
                )
                .with_field("house")
                .with_hint("name one: `quesited` or `house`"));
            }
            _ => {}
        }
        self.rules.check().map_err(|why| why.under("rules"))
    }

    /// The two significators on a figure: the named ones, or the lords of
    /// the Ascendant and of the house of the matter's cusp.
    ///
    /// # Errors
    ///
    /// As [`PerfectionRequest::from_json`] for a request built otherwise.
    pub fn significators(&self, fortitudes: &Fortitudes) -> Result<(Graha, Graha), Error> {
        self.check()?;
        let sky = &fortitudes.sky;
        let lord = |deg: f64| {
            teistro_core::catalogue::Rashi::of_longitude(deg)
                .attributes()
                .lord
        };
        let querent = self.querent.unwrap_or_else(|| lord(sky.ascendant_deg));
        let quesited = match (self.quesited, self.house) {
            (Some(quesited), _) => quesited,
            (None, Some(house)) => {
                let cusp = sky
                    .cusps_deg
                    .get(usize::from(house.get()) - 1)
                    .ok_or_else(|| Error::internal("a figure has twelve cusps"))?;
                lord(*cusp)
            }
            (None, None) => return Err(Error::internal("checked above")),
        };
        Ok((querent, quesited))
    }
}

/// Lilly's three kinds of application (p. 107).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ApplicationKind {
    /// A swifter planet to a slower, both direct.
    BothDirect,
    /// Both retrograde, which he calls "an ill Application".
    BothRetrograde,
    /// One direct and one retrograde, meeting: also ill.
    AgainstRetrograde,
}

/// The next aspect the two significators perfect.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Application {
    /// The aspect.
    pub aspect: PtolemaicAspect,
    /// Days until it is exact.
    pub days: f64,
    /// The one whose motion closes the arc the faster.
    pub applying: Graha,
    /// Which of Lilly's three it is, by the two motions at the figure.
    pub kind: ApplicationKind,
    /// How far it is from exact now, degrees along the motions of the
    /// moment.
    pub gap_deg: f64,
    /// Whether the gap is already inside the two planets' moieties of
    /// orb.
    pub within_moieties: bool,
}

/// An aspect two planets have just perfected and are leaving, still inside
/// their moieties: "totally separated" only past them (p. 110).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Separation {
    /// The aspect.
    pub aspect: PtolemaicAspect,
    /// How far past exact they are, degrees.
    pub past_deg: f64,
}

/// What stops or hinders the significators' application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ImpedimentKind {
    /// A third planet comes to a significator first, bodily by a
    /// conjunction or by aspect (pp. 110–111).
    Prohibition,
    /// A significator comes to a third planet first (p. 112).
    Frustration,
    /// A significator stations before the perfection the motions of the
    /// moment promise (p. 111).
    Refranation,
}

/// One impediment, with what it rests on.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Impediment {
    /// Which impediment.
    pub kind: ImpedimentKind,
    /// The significator it falls on.
    pub significator: Graha,
    /// The third planet, for a prohibition or a frustration.
    pub third: Option<Graha>,
    /// The aspect the third perfects, or the one refrained from.
    pub aspect: PtolemaicAspect,
    /// Days after the figure: the contact, or the station.
    pub days: f64,
}

/// A lighter planet carrying the light of one significator to the other
/// (p. 111): it separates from the first and comes next to the second.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Translation {
    /// The planet that translates.
    pub translator: Graha,
    /// The significator it separates from.
    pub from: Graha,
    /// The significator it comes to next.
    pub to: Graha,
    /// The aspect it separates from.
    pub separating: Separation,
    /// The aspect it perfects next, with `to`.
    pub aspect: PtolemaicAspect,
    /// Days until then.
    pub days: f64,
    /// The dignities of `from` the translator stands in: how it is
    /// received, "by House, Triplicity or Terme" (p. 126).
    pub received: EssentialDignity,
}

/// A planet more weighty than both significators, to which both apply
/// while they do not behold each other (p. 126).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Collection {
    /// The planet that collects.
    pub collector: Graha,
    /// The querent's significator's contact with it.
    pub from_querent: Contact,
    /// The quesited's significator's contact with it.
    pub from_quesited: Contact,
    /// The querent's dignities the collector stands in: Lilly's
    /// direction, the significators "both receive him" (crux C233).
    pub collector_in_querent: EssentialDignity,
    /// The quesited's dignities the collector stands in.
    pub collector_in_quesited: EssentialDignity,
    /// The collector's dignities the querent's significator stands in,
    /// the direction later accounts read.
    pub querent_in_collector: EssentialDignity,
    /// The collector's dignities the quesited's significator stands in.
    pub quesited_in_collector: EssentialDignity,
}

/// What the ways of perfection weigh of a figure beside its timeline: the
/// house each of the seven is counted in, and whose dignities stand where.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Standing<'a> {
    /// Each planet's house in the Chaldean order, as the accidental
    /// fortitudes count it.
    pub houses: [House; 7],
    /// The figure's sect, for the triplicities.
    pub sect: Sect,
    /// How the essential dignities are read.
    pub dignities: &'a DignityRules,
}

impl<'a> Standing<'a> {
    /// The standing a figure's fortitudes give: their houses, sect and
    /// dignity rules.
    #[must_use]
    pub fn of(fortitudes: &'a Fortitudes) -> Standing<'a> {
        Standing {
            houses: fortitudes.planets.each_ref().map(|planet| planet.house),
            sect: fortitudes.dignities.sect,
            dignities: &fortitudes.dignities.rules,
        }
    }
}

/// One of Lilly's seven ways a matter is perfected (pp. 125–127).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum Way {
    /// The significators' conjunction, with no prohibition or refranation
    /// before it.
    Conjunction,
    /// Their sextile or trine, with no prohibition, refranation or
    /// infortune's aspect before it.
    SextileOrTrine,
    /// Their square, each in some dignity at its degree, with no
    /// prohibition or refranation before it.
    Square,
    /// Their opposition, with mutual reception by house and the Moon
    /// separating from the quesited's significator to the querent's.
    Opposition,
    /// A translation of light, the translator received by house,
    /// triplicity or term.
    Translation,
    /// A collection of light, the collector in some dignity of each.
    Collection,
    /// The quesited's significator in the Ascendant, the Moon translating
    /// the light as well.
    Dwelling,
}

impl Way {
    /// The seven, in Lilly's order.
    pub const ALL: [Way; 7] = [
        Way::Conjunction,
        Way::SextileOrTrine,
        Way::Square,
        Way::Opposition,
        Way::Translation,
        Way::Collection,
        Way::Dwelling,
    ];
}

/// A significator's place in the figure, as the ways weigh it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct SignificatorPlace {
    /// The planet.
    pub planet: Graha,
    /// Its house: angular, succedent or cadent is the house's own
    /// question ([`House::is_kendra`] and its kin).
    pub house: House,
    /// Its own dignities at its degree.
    pub dignity: EssentialDignity,
}

/// The facts Lilly's ways weigh beside the relations, and the ways whose
/// stated conditions hold.
///
/// "Out of good houses" and "well dignified" are judgements, so they are
/// left to the reader on [`SignificatorPlace`]; a way is held on the
/// conditions Lilly states as facts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Ways {
    /// The querent's significator.
    pub querent: SignificatorPlace,
    /// The quesited's significator.
    pub quesited: SignificatorPlace,
    /// Each stands in the other's house.
    pub mutual_by_house: bool,
    /// The infortunes, Saturn and Mars, among the thirds that come between
    /// the significators before they perfect.
    pub infortunes_between: Vec<Graha>,
    /// The Moon, neither significator, separating from the quesited's and
    /// coming next to the querent's (p. 126, the opposition).
    pub moon_relays: bool,
    /// The quesited's significator in the first house.
    pub quesited_in_ascendant: bool,
    /// The ways whose stated conditions hold, in Lilly's order.
    pub held: Vec<Way>,
}

/// The relations between two significators, each with the facts it rests
/// on; never a verdict.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Matter {
    /// The querent's significator.
    pub querent: Graha,
    /// The quesited's significator.
    pub quesited: Graha,
    /// The next aspect they perfect within the horizon.
    pub application: Option<Application>,
    /// The aspect they are leaving, while still inside their moieties.
    pub separation: Option<Separation>,
    /// Every impediment before the application, in time order; a
    /// refranation is reported with or without one.
    pub impediments: Vec<Impediment>,
    /// Every translation of light between them.
    pub translations: Vec<Translation>,
    /// Every collection of light, when they do not apply to each other.
    pub collections: Vec<Collection>,
    /// The ways of perfection: what they weigh, and which hold.
    pub ways: Ways,
    /// How many days ahead it was read.
    pub horizon_days: f64,
}

/// The arc between two planets at the figure and its rate of change: the
/// first's longitude less the second's.
fn arc_and_rate(timeline: &AspectTimeline, first: Graha, second: Graha) -> Option<(f64, f64)> {
    let (a, a_speed) = timeline.motion(first)?;
    let (b, b_speed) = timeline.motion(second)?;
    Some(((a - b).rem_euclid(360.0), a_speed - b_speed))
}

/// The one of two planets whose own motion closes the arc between them
/// the faster.
fn applier(timeline: &AspectTimeline, first: Graha, second: Graha) -> Graha {
    let speed = |planet| timeline.motion(planet).map_or(0.0, |(_, speed)| speed);
    let (a, b) = (speed(first), speed(second));
    let toward = if a - b >= 0.0 { 1.0 } else { -1.0 };
    if a * toward >= -b * toward {
        first
    } else {
        second
    }
}

/// The aspect two planets are leaving and how far past it they are, when
/// inside their moieties.
fn separation_of(
    timeline: &AspectTimeline,
    rules: &PerfectionRules,
    first: Graha,
    second: Graha,
) -> Option<Separation> {
    let (arc, rate) = arc_and_rate(timeline, first, second)?;
    if rate == 0.0 {
        return None;
    }
    let moieties = rules.moieties(first, second);
    SIDES
        .iter()
        .filter_map(|&(side, aspect)| {
            // Signed past the side, in [-180, 180): moving away when the
            // offset and the rate agree.
            let offset = (arc - side + 180.0).rem_euclid(360.0) - 180.0;
            (offset * rate > 0.0 && offset.abs() <= moieties).then_some(Separation {
                aspect,
                past_deg: offset.abs(),
            })
        })
        .min_by(|a, b| a.past_deg.total_cmp(&b.past_deg))
}

/// The two significators' relations, read off a figure's timeline.
///
/// `standing` gives the houses the ways weigh and whose dignities a planet
/// stands in, for the receptions a translation and a collection report.
///
/// # Errors
///
/// A significator outside the seven, or the two the same planet, named
/// `querent` or `quesited`; rules out of range, named as
/// [`PerfectionRules::from_json`] names them without the root; and a
/// dignity the rules cannot read.
pub fn perfection(
    timeline: &AspectTimeline,
    querent: Graha,
    quesited: Graha,
    standing: &Standing<'_>,
    rules: &PerfectionRules,
) -> Result<Matter, Error> {
    rules.check()?;
    for (planet, field) in [(querent, "querent"), (quesited, "quesited")] {
        if weight(planet).is_none() {
            return Err(not_one_of_the_seven(planet, field));
        }
    }
    if querent == quesited {
        return Err(Error::invalid_arg(format!(
            "{querent:?} cannot signify both the querent and the quesited"
        ))
        .with_field("quesited"));
    }
    let reader = Reader {
        timeline,
        querent,
        quesited,
        standing,
        rules,
    };
    let application = reader.application();
    let mut impediments = reader.impediments(application);
    impediments.extend(refranation(timeline, querent, quesited, application));
    impediments.sort_by(|a, b| a.days.total_cmp(&b.days));
    let mut translations = Vec::new();
    let mut collections = Vec::new();
    for third in CHALDEAN_ORDER {
        if third == querent || third == quesited {
            continue;
        }
        translations.extend(reader.translations(third)?);
        if application.is_none() {
            collections.extend(reader.collection(third)?);
        }
    }
    let ways = reader.ways(application, &impediments, &translations, &collections)?;
    Ok(Matter {
        querent,
        quesited,
        application,
        separation: separation_of(timeline, rules, querent, quesited),
        impediments,
        translations,
        collections,
        ways,
        horizon_days: timeline.horizon_days,
    })
}

/// One reading of two significators' relations, a method for each.
struct Reader<'r> {
    timeline: &'r AspectTimeline,
    querent: Graha,
    quesited: Graha,
    standing: &'r Standing<'r>,
    rules: &'r PerfectionRules,
}

impl Reader<'_> {
    /// Whether the planet is one of the two significators.
    fn signifies(&self, planet: Graha) -> bool {
        planet == self.querent || planet == self.quesited
    }

    /// The first contact between two planets.
    fn first_between(&self, first: Graha, second: Graha) -> Option<&Contact> {
        self.timeline
            .contacts
            .iter()
            .find(|contact| contact.involves(first) && contact.involves(second))
    }

    /// The host's dignities the guest stands in, at the guest's place.
    fn receives(&self, host: Graha, guest: Graha) -> Result<EssentialDignity, Error> {
        let (place, _) = self
            .timeline
            .motion(guest)
            .ok_or_else(|| not_one_of_the_seven(guest, "planets"))?;
        essential_dignity(host, place, self.standing.sect, self.standing.dignities)
    }

    /// A significator's house and its own dignities at its degree.
    fn place_of(&self, planet: Graha) -> Result<SignificatorPlace, Error> {
        let house = weight(planet)
            .and_then(|at| self.standing.houses.get(at))
            .copied()
            .ok_or_else(|| not_one_of_the_seven(planet, "planets"))?;
        Ok(SignificatorPlace {
            planet,
            house,
            dignity: self.receives(planet, planet)?,
        })
    }

    /// The facts the seven ways weigh, and the ways whose stated
    /// conditions hold.
    fn ways(
        &self,
        application: Option<Application>,
        impediments: &[Impediment],
        translations: &[Translation],
        collections: &[Collection],
    ) -> Result<Ways, Error> {
        let (querent, quesited) = (self.place_of(self.querent)?, self.place_of(self.quesited)?);
        let infortunes_between: Vec<Graha> = [Graha::Saturn, Graha::Mars]
            .into_iter()
            .filter(|&infortune| impediments.iter().any(|each| each.third == Some(infortune)))
            .collect();
        let stopped = impediments.iter().any(|each| {
            matches!(
                each.kind,
                ImpedimentKind::Prohibition | ImpedimentKind::Refranation
            )
        });
        let mutual_by_house = self.receives(self.querent, self.quesited)?.house
            && self.receives(self.quesited, self.querent)?.house;
        let moon_relays = !self.signifies(Graha::Moon)
            && separation_of(self.timeline, self.rules, Graha::Moon, self.quesited).is_some()
            && self
                .timeline
                .contacts
                .iter()
                .find(|contact| contact.involves(Graha::Moon))
                .is_some_and(|contact| contact.other(Graha::Moon) == Some(self.querent));
        let quesited_in_ascendant = quesited.house.get() == 1;
        let aspect = application.map(|found| found.aspect);
        let held = Way::ALL
            .into_iter()
            .filter(|way| match way {
                Way::Conjunction => aspect == Some(PtolemaicAspect::Conjunction) && !stopped,
                Way::SextileOrTrine => {
                    matches!(
                        aspect,
                        Some(PtolemaicAspect::Sextile | PtolemaicAspect::Trine)
                    ) && !stopped
                        && infortunes_between.is_empty()
                }
                Way::Square => {
                    aspect == Some(PtolemaicAspect::Square)
                        && !stopped
                        && !querent.dignity.peregrine()
                        && !quesited.dignity.peregrine()
                }
                Way::Opposition => {
                    aspect == Some(PtolemaicAspect::Opposition) && mutual_by_house && moon_relays
                }
                Way::Translation => translations.iter().any(|each| {
                    each.received.house || each.received.triplicity || each.received.term
                }),
                Way::Collection => collections.iter().any(|each| {
                    !each.collector_in_querent.peregrine()
                        && !each.collector_in_quesited.peregrine()
                }),
                Way::Dwelling => {
                    quesited_in_ascendant
                        && translations
                            .iter()
                            .any(|each| each.translator == Graha::Moon)
                }
            })
            .collect();
        Ok(Ways {
            querent,
            quesited,
            mutual_by_house,
            infortunes_between,
            moon_relays,
            quesited_in_ascendant,
            held,
        })
    }

    /// The significators' next perfection.
    fn application(&self) -> Option<Application> {
        let (querent, quesited) = (self.querent, self.quesited);
        let contact = self.first_between(querent, quesited)?;
        let gap_deg = arc_and_rate(self.timeline, querent, quesited)
            .map_or(0.0, |(arc, rate)| gap_to(arc, rate, contact.aspect));
        Some(Application {
            aspect: contact.aspect,
            days: contact.days,
            applying: applier(self.timeline, querent, quesited),
            kind: kind_of(self.timeline, querent, quesited),
            gap_deg,
            within_moieties: gap_deg <= self.rules.moieties(querent, quesited),
        })
    }

    /// Every contact of a significator with a third planet before the
    /// application: a prohibition when the third comes to it, a
    /// frustration when it goes to the third.
    fn impediments(&self, application: Option<Application>) -> Vec<Impediment> {
        let Some(application) = application else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for contact in self
            .timeline
            .contacts
            .iter()
            .take_while(|contact| contact.days < application.days)
        {
            for significator in [self.querent, self.quesited] {
                let Some(third) = contact.other(significator) else {
                    continue;
                };
                if self.signifies(third) {
                    continue;
                }
                let kind = if applier(self.timeline, significator, third) == third {
                    ImpedimentKind::Prohibition
                } else {
                    ImpedimentKind::Frustration
                };
                found.push(Impediment {
                    kind,
                    significator,
                    third: Some(third),
                    aspect: contact.aspect,
                    days: contact.days,
                });
            }
        }
        found
    }

    /// A lighter third planet separating from one significator whose next
    /// contact of all is with the other, either way round.
    fn translations(&self, third: Graha) -> Result<Vec<Translation>, Error> {
        let lighter = weight(third) > weight(self.querent).max(weight(self.quesited));
        let next = self
            .timeline
            .contacts
            .iter()
            .find(|contact| contact.involves(third));
        let mut found = Vec::new();
        for (from, to) in [(self.querent, self.quesited), (self.quesited, self.querent)] {
            let next = next.filter(|contact| contact.other(third) == Some(to));
            let (true, Some(separating), Some(next)) = (
                lighter,
                separation_of(self.timeline, self.rules, third, from),
                next,
            ) else {
                continue;
            };
            found.push(Translation {
                translator: third,
                from,
                to,
                separating,
                aspect: next.aspect,
                days: next.days,
                received: self.receives(from, third)?,
            });
        }
        Ok(found)
    }

    /// A third planet more weighty than both, which both come to.
    fn collection(&self, third: Graha) -> Result<Option<Collection>, Error> {
        let (querent, quesited) = (self.querent, self.quesited);
        let heavier = weight(third) < weight(querent).min(weight(quesited));
        let (true, Some(&from_querent), Some(&from_quesited)) = (
            heavier,
            self.first_between(querent, third),
            self.first_between(quesited, third),
        ) else {
            return Ok(None);
        };
        Ok(Some(Collection {
            collector: third,
            from_querent,
            from_quesited,
            collector_in_querent: self.receives(querent, third)?,
            collector_in_quesited: self.receives(quesited, third)?,
            querent_in_collector: self.receives(third, querent)?,
            quesited_in_collector: self.receives(third, quesited)?,
        }))
    }
}

/// The arc still to run, along the motions of the moment, to the nearer
/// side of an aspect.
fn gap_to(arc: f64, rate: f64, aspect: PtolemaicAspect) -> f64 {
    SIDES
        .iter()
        .filter(|&&(_, each)| each == aspect)
        .map(|&(side, _)| closing_arc(arc, side, rate))
        .fold(f64::INFINITY, f64::min)
}

/// Which of Lilly's three applications two planets' motions make.
fn kind_of(timeline: &AspectTimeline, first: Graha, second: Graha) -> ApplicationKind {
    let retrograde = |planet| {
        timeline
            .motion(planet)
            .is_some_and(|(_, speed)| speed < 0.0)
    };
    match (retrograde(first), retrograde(second)) {
        (false, false) => ApplicationKind::BothDirect,
        (true, true) => ApplicationKind::BothRetrograde,
        _ => ApplicationKind::AgainstRetrograde,
    }
}

/// A significator's station before the perfection the motions of the
/// moment promise, and before any the timeline finds.
fn refranation(
    timeline: &AspectTimeline,
    querent: Graha,
    quesited: Graha,
    application: Option<Application>,
) -> Option<Impediment> {
    let (arc, rate) = arc_and_rate(timeline, querent, quesited)?;
    let (promised, aspect) = first_ahead(arc, rate)?;
    let station = timeline.stations.iter().find(|station| {
        (station.planet == querent || station.planet == quesited)
            && station.days < promised
            && application.is_none_or(|found| station.days < found.days)
    })?;
    Some(Impediment {
        kind: ImpedimentKind::Refranation,
        significator: station.planet,
        third: None,
        aspect,
        days: station.days,
    })
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::panic,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and read small fixed lists"
    )]

    use super::*;
    use teistro_core::catalogue::Graha::{Jupiter, Mars, Mercury, Moon, Saturn, Sun, Venus};

    /// Lilly's second orb column of p. 107, which his p. 110 separation
    /// works in.
    const ORBS_ACCORDING_TO_OTHERS: [f64; 7] = [9.0, 9.0, 7.0, 15.0, 7.0, 7.0, 12.0];

    /// A sky where only the planets Lilly names move: each is placed and
    /// given its speed, and every other is parked, standing still, at the
    /// first longitude (in whole degrees) where it meets none of them
    /// within the horizon, so his examples are read without a stranger
    /// interfering.
    fn sky(movers: &[(Graha, f64, f64)], horizon_days: f64) -> AspectTimeline {
        let mut places = [0.0; 7];
        let mut speeds = [0.0; 7];
        let mut named = Vec::new();
        for &(planet, place, speed) in movers {
            let at = weight(planet).unwrap();
            places[at] = place;
            speeds[at] = speed;
            named.push(planet);
        }
        for planet in CHALDEAN_ORDER {
            if named.contains(&planet) {
                continue;
            }
            let at = weight(planet).unwrap();
            let quiet = (0..360).map(f64::from).find(|&place| {
                places[at] = place;
                let trial = AspectTimeline::projected(places, speeds, horizon_days).unwrap();
                !trial
                    .contacts()
                    .iter()
                    .any(|contact| contact.involves(planet))
            });
            places[at] = quiet.unwrap_or_else(|| panic!("nowhere quiet for {planet:?}"));
            named.push(planet);
        }
        AspectTimeline::projected(places, speeds, horizon_days).unwrap()
    }

    /// Every planet in the tenth, by day, under Lilly's dignities.
    fn standing() -> Standing<'static> {
        Standing {
            houses: [House::try_new(10).unwrap(); 7],
            sect: Sect::Day,
            dignities: &DignityRules::LILLY,
        }
    }

    fn read(timeline: &AspectTimeline, querent: Graha, quesited: Graha) -> Matter {
        read_under(timeline, querent, quesited, &PerfectionRules::LILLY)
    }

    fn read_under(
        timeline: &AspectTimeline,
        querent: Graha,
        quesited: Graha,
        rules: &PerfectionRules,
    ) -> Matter {
        perfection(timeline, querent, quesited, &standing(), rules).unwrap()
    }

    #[test]
    fn an_aspect_is_read_from_a_thirty_degree_lattice() {
        let kept: Vec<_> = (0..12)
            .filter_map(|k| aspect_at(f64::from(k) * 30.0))
            .collect();
        assert_eq!(kept.len(), 8);
        assert_eq!(aspect_at(359.999_999_9), Some(PtolemaicAspect::Conjunction));
        assert_eq!(aspect_at(90.001), None);
    }

    #[test]
    fn lillys_three_applications() {
        // p. 107: Mars in 10° Aries and Mercury in 5°, both direct.
        let first = read(
            &sky(&[(Mars, 10.0, 0.7), (Mercury, 5.0, 1.5)], 8.0),
            Mercury,
            Mars,
        );
        let applied = first.application.unwrap();
        assert_eq!(applied.kind, ApplicationKind::BothDirect);
        assert_eq!(applied.applying, Mercury);
        assert_eq!(applied.aspect, PtolemaicAspect::Conjunction);
        assert!((applied.gap_deg - 5.0).abs() < 1e-9 && applied.within_moieties);

        // Mercury in 10° and Mars in 9°, both retrograde.
        let second = read(
            &sky(&[(Mercury, 10.0, -1.0), (Mars, 9.0, -0.3)], 3.0),
            Mercury,
            Mars,
        );
        let applied = second.application.unwrap();
        assert_eq!(
            (applied.kind, applied.applying),
            (ApplicationKind::BothRetrograde, Mercury)
        );

        // Mars direct in 15° and Mercury retrograde in 17°.
        let third = read(
            &sky(&[(Mars, 15.0, 0.6), (Mercury, 17.0, -0.5)], 3.0),
            Mercury,
            Mars,
        );
        assert_eq!(
            third.application.unwrap().kind,
            ApplicationKind::AgainstRetrograde
        );
    }

    #[test]
    fn lillys_bodily_prohibition() {
        // p. 111: Mars in 7° Aries, Saturn in 12°, the Sun in 6°: the Sun
        // meets Mars and then Saturn before Mars comes to Saturn.
        let timeline = sky(
            &[(Mars, 7.0, 0.7), (Saturn, 12.0, 0.03), (Sun, 6.0, 0.98)],
            8.0,
        );
        let matter = read(&timeline, Mars, Saturn);
        assert_eq!(
            matter.application.unwrap().aspect,
            PtolemaicAspect::Conjunction
        );
        let found: Vec<_> = matter
            .impediments
            .iter()
            .map(|each| (each.kind, each.significator, each.third, each.aspect))
            .collect();
        assert_eq!(
            found,
            [
                (
                    ImpedimentKind::Prohibition,
                    Mars,
                    Some(Sun),
                    PtolemaicAspect::Conjunction
                ),
                (
                    ImpedimentKind::Prohibition,
                    Saturn,
                    Some(Sun),
                    PtolemaicAspect::Conjunction
                ),
            ]
        );
    }

    #[test]
    fn lillys_prohibition_by_aspect() {
        // p. 111: Mars in 7° Aries, Saturn in 15°, the Sun in 5° Gemini:
        // the Sun passes Mars's dexter sextile and reaches Saturn's before
        // Mars comes to Saturn.
        let timeline = sky(
            &[(Mars, 7.0, 0.7), (Saturn, 15.0, 0.03), (Sun, 65.0, 0.98)],
            12.0,
        );
        let matter = read(&timeline, Mars, Saturn);
        let by_the_sun: Vec<_> = matter
            .impediments
            .iter()
            .filter(|each| each.third == Some(Sun))
            .map(|each| (each.kind, each.significator, each.aspect))
            .collect();
        assert_eq!(
            by_the_sun,
            [
                (ImpedimentKind::Prohibition, Mars, PtolemaicAspect::Sextile),
                (
                    ImpedimentKind::Prohibition,
                    Saturn,
                    PtolemaicAspect::Sextile
                ),
            ]
        );
    }

    #[test]
    fn lillys_refranation() {
        // p. 111: Saturn in 12° Aries, Mars in 7°, who turns retrograde
        // before the tenth or eleventh degree and never comes to Saturn.
        // The ephemeris finds no conjunction; Mars stations on the fourth
        // day, at about 9° 30′.
        let quiet = sky(&[(Mars, 7.0, 0.6), (Saturn, 12.0, 0.03)], 1.0);
        let timeline = AspectTimeline::new(
            quiet.places,
            quiet.speeds,
            30.0,
            Vec::new(),
            vec![Station::new(Mars, 4.0, true).unwrap()],
        )
        .unwrap();
        let matter = read(&timeline, Mars, Saturn);
        assert!(matter.application.is_none());
        let refrained = matter.impediments.first().unwrap();
        assert_eq!(
            (refrained.kind, refrained.significator, refrained.aspect),
            (
                ImpedimentKind::Refranation,
                Mars,
                PtolemaicAspect::Conjunction
            )
        );
        // And a station after the perfection refrains from nothing.
        let late = AspectTimeline::projected(quiet.places, quiet.speeds, 30.0).unwrap();
        let late = AspectTimeline::new(
            late.places,
            late.speeds,
            30.0,
            late.contacts.clone(),
            vec![Station::new(Mars, 20.0, true).unwrap()],
        )
        .unwrap();
        assert!(
            read(&late, Mars, Saturn)
                .impediments
                .iter()
                .all(|each| each.kind != ImpedimentKind::Refranation)
        );
    }

    #[test]
    fn lillys_translation() {
        // p. 111: Saturn in 20° Aries, Mars in 15°, Mercury in 16°:
        // Mercury separates from Mars and carries his virtue to Saturn.
        let timeline = sky(
            &[
                (Saturn, 20.0, 0.03),
                (Mars, 15.0, 0.7),
                (Mercury, 16.0, 1.4),
            ],
            6.0,
        );
        let matter = read(&timeline, Mars, Saturn);
        let carried = matter.translations.first().unwrap();
        assert_eq!(
            (carried.translator, carried.from, carried.to, carried.aspect),
            (Mercury, Mars, Saturn, PtolemaicAspect::Conjunction)
        );
        assert_eq!(carried.separating.aspect, PtolemaicAspect::Conjunction);
        assert!((carried.separating.past_deg - 1.0).abs() < 1e-9);
        // Mercury in 16° Aries is in Mars's house.
        assert!(carried.received.house);
    }

    #[test]
    fn lillys_frustration() {
        // p. 113: Mercury in 10° Aries, Mars in 12°, Jupiter in 13°:
        // Mercury strives to come to Mars, but Mars gets to Jupiter first.
        let timeline = sky(
            &[
                (Mercury, 10.0, 1.4),
                (Mars, 12.0, 0.7),
                (Jupiter, 13.0, 0.2),
            ],
            4.0,
        );
        let matter = read(&timeline, Mercury, Mars);
        let first = matter.impediments.first().unwrap();
        assert_eq!(
            (first.kind, first.significator, first.third, first.aspect),
            (
                ImpedimentKind::Frustration,
                Mars,
                Some(Jupiter),
                PtolemaicAspect::Conjunction
            )
        );
        assert!((first.days - 2.0).abs() < 1e-9);
    }

    #[test]
    fn a_collection_is_reported_with_its_receptions_both_ways() {
        // Venus in 8° Cancer and Mars in 7° Gemini, 31° apart and closing
        // too slowly to meet, each coming to Saturn in 10° Virgo: Venus
        // to his sextile, Mars to his square.
        let timeline = sky(
            &[(Venus, 98.0, 1.2), (Mars, 67.0, 0.7), (Saturn, 160.0, 0.03)],
            10.0,
        );
        let matter = read(&timeline, Venus, Mars);
        assert!(matter.application.is_none());
        let collected = matter.collections.first().unwrap();
        assert_eq!(collected.collector, Saturn);
        assert_eq!(collected.from_querent.aspect, PtolemaicAspect::Sextile);
        assert_eq!(collected.from_quesited.aspect, PtolemaicAspect::Square);
        // Saturn stands in Venus's term and, by day, her earth
        // triplicity: Lilly's direction, Venus receives him (C233). He is
        // in no dignity of Mars's.
        assert!(collected.collector_in_querent.term && collected.collector_in_querent.triplicity);
        assert!(collected.collector_in_quesited.peregrine());
        // The other direction is reported too: Venus in Cancer stands in
        // none of Saturn's.
        assert!(collected.querent_in_collector.peregrine());
    }

    #[test]
    fn a_way_holds_on_the_conditions_lilly_states() {
        // p. 107's first application, met by nothing: a conjunction.
        let clear = read(
            &sky(&[(Mars, 10.0, 0.7), (Mercury, 5.0, 1.5)], 8.0),
            Mercury,
            Mars,
        );
        assert_eq!(clear.ways.held, [Way::Conjunction]);
        // p. 111's bodily prohibition: the conjunction is stopped.
        let prohibited = read(
            &sky(
                &[(Mars, 7.0, 0.7), (Saturn, 12.0, 0.03), (Sun, 6.0, 0.98)],
                8.0,
            ),
            Mars,
            Saturn,
        );
        assert_eq!(prohibited.ways.held, []);
        // p. 111's translation: Mercury in 16° Aries stands in Mars's
        // house, so the light is carried by a reception Lilly names.
        let carried = read(
            &sky(
                &[
                    (Saturn, 20.0, 0.03),
                    (Mars, 15.0, 0.7),
                    (Mercury, 16.0, 1.4),
                ],
                6.0,
            ),
            Mars,
            Saturn,
        );
        assert!(carried.ways.held.contains(&Way::Translation));
    }

    #[test]
    fn a_square_perfects_only_with_dignity_at_both_degrees() {
        // Mars in 10° Aries, his house, squaring Saturn in 14° Capricorn,
        // his house: both dignified.
        let dignified = read(
            &sky(&[(Mars, 10.0, 0.7), (Saturn, 284.0, 0.03)], 8.0),
            Mars,
            Saturn,
        );
        assert_eq!(
            dignified.application.unwrap().aspect,
            PtolemaicAspect::Square
        );
        assert_eq!(dignified.ways.held, [Way::Square]);
        // Saturn in 14° Cancer, his detriment and nothing else, is
        // peregrine.
        let peregrine = read(
            &sky(&[(Mars, 10.0, 0.7), (Saturn, 284.0 - 180.0, 0.03)], 8.0),
            Mars,
            Saturn,
        );
        assert!(peregrine.ways.quesited.dignity.peregrine());
        assert_eq!(peregrine.ways.held, []);
    }

    #[test]
    fn a_collection_needs_the_collector_in_both_significators_dignities() {
        // As above: Saturn is in Venus's term and triplicity but in none
        // of Mars's, so the light is collected and the way does not hold.
        let timeline = sky(
            &[(Venus, 98.0, 1.2), (Mars, 67.0, 0.7), (Saturn, 160.0, 0.03)],
            10.0,
        );
        let matter = read(&timeline, Venus, Mars);
        assert_eq!(matter.collections.len(), 1);
        assert!(!matter.ways.held.contains(&Way::Collection));
    }

    #[test]
    fn an_opposition_perfects_by_mutual_reception_and_the_moons_relay() {
        // Venus in Scorpio and Mars in Taurus, each in the other's house;
        // the Moon just past Mars, coming next to Venus's trine.
        let (venus, mars, moon) = (210.0, 52.0, 54.0);
        let places = [300.0, 140.0, mars, 170.0, venus, 20.0, moon];
        let speeds = [0.0, 0.0, 0.6, 0.0, 1.2, 0.0, 13.0];
        let contacts = vec![
            Contact::new(Moon, Venus, PtolemaicAspect::Trine, 3.05).unwrap(),
            Contact::new(Venus, Mars, PtolemaicAspect::Opposition, 36.7).unwrap(),
        ];
        let timeline = AspectTimeline::new(places, speeds, 40.0, contacts, Vec::new()).unwrap();
        let matter = read(&timeline, Venus, Mars);
        assert!(matter.ways.mutual_by_house && matter.ways.moon_relays);
        assert_eq!(matter.ways.held, [Way::Opposition]);
        // Without the reception the opposition does not perfect.
        let mut apart = places;
        apart[4] = venus + 30.0;
        let contacts = vec![
            Contact::new(Moon, Venus, PtolemaicAspect::Opposition, 3.05).unwrap(),
            Contact::new(Venus, Mars, PtolemaicAspect::Opposition, 36.7).unwrap(),
        ];
        let timeline = AspectTimeline::new(apart, speeds, 40.0, contacts, Vec::new()).unwrap();
        assert_eq!(read(&timeline, Venus, Mars).ways.held, []);
    }

    #[test]
    fn dwelling_holds_with_the_moon_translating() {
        // Saturn, the quesited's significator, in the Ascendant; the Moon
        // in 16° Aries carries Mars's light to him.
        let timeline = sky(
            &[(Saturn, 20.0, 0.03), (Mars, 15.0, 0.7), (Moon, 16.0, 13.0)],
            0.5,
        );
        let mut houses = [House::try_new(10).unwrap(); 7];
        houses[0] = House::try_new(1).unwrap();
        let standing = Standing {
            houses,
            ..standing()
        };
        let matter =
            perfection(&timeline, Mars, Saturn, &standing, &PerfectionRules::LILLY).unwrap();
        assert!(matter.ways.quesited_in_ascendant);
        assert_eq!(matter.ways.held, [Way::Translation, Way::Dwelling]);
    }

    #[test]
    fn lillys_separation_lasts_to_the_moieties() {
        // p. 110: Saturn in 10° 25′ Aries, Jupiter in 10° 25′, then 10°
        // 31′: separating, but not clear of Saturn's rays until 9° away
        // under the orbs he works there (4° 30′ each).
        let rules = PerfectionRules {
            orbs_deg: ORBS_ACCORDING_TO_OTHERS,
            horizon_days: None,
        };
        let near = sky(
            &[
                (Saturn, 10.0 + 25.0 / 60.0, 0.03),
                (Jupiter, 10.0 + 31.0 / 60.0, 0.2),
            ],
            1.0,
        );
        let parting = read_under(&near, Jupiter, Saturn, &rules)
            .separation
            .unwrap();
        assert_eq!(parting.aspect, PtolemaicAspect::Conjunction);
        assert!((parting.past_deg - 0.1).abs() < 1e-9);
        let clear = sky(&[(Saturn, 10.0, 0.03), (Jupiter, 19.1, 0.2)], 1.0);
        assert!(
            read_under(&clear, Jupiter, Saturn, &rules)
                .separation
                .is_none()
        );
    }

    #[test]
    fn the_horizon_is_the_swifter_significators_sign() {
        let places = [12.0, 0.0, 7.0, 0.0, 0.0, 0.0, 0.0];
        let speeds = [0.03, 0.0, 0.5, 0.0, 0.0, 0.0, 0.0];
        let horizon = PerfectionRules::LILLY
            .horizon_for(&places, &speeds, Mars, Saturn)
            .unwrap();
        assert!((horizon - 23.0 / 0.5).abs() < 1e-9);
        let retrograde = PerfectionRules::LILLY
            .horizon_for(
                &places,
                &[0.03, 0.0, -0.5, 0.0, 0.0, 0.0, 0.0],
                Mars,
                Saturn,
            )
            .unwrap();
        assert!((retrograde - 7.0 / 0.5).abs() < 1e-9);
        let set = PerfectionRules {
            horizon_days: Some(40.0),
            ..PerfectionRules::LILLY
        };
        assert_eq!(
            set.horizon_for(&places, &speeds, Mars, Saturn).unwrap(),
            40.0
        );
    }

    #[test]
    fn refusals_name_their_field() {
        let timeline = sky(&[(Mars, 7.0, 0.7)], 1.0);
        let same =
            perfection(&timeline, Mars, Mars, &standing(), &PerfectionRules::LILLY).unwrap_err();
        assert_eq!(same.field(), Some("quesited"));
        let node = perfection(
            &timeline,
            Graha::Rahu,
            Mars,
            &standing(),
            &PerfectionRules::LILLY,
        )
        .unwrap_err();
        assert_eq!(node.field(), Some("querent"));
        let wide =
            PerfectionRules::from_json(r#"{"orbsDeg": [1, 1, 1, 1, 1, 1, -1]}"#).unwrap_err();
        assert_eq!(wide.field(), Some("perfection.orbsDeg"));
        let never = AspectTimeline::projected([0.0; 7], [0.0; 7], f64::INFINITY).unwrap_err();
        assert_eq!(never.field(), Some("horizonDays"));
        assert_eq!(
            Contact::new(Mars, Mars, PtolemaicAspect::Trine, 1.0)
                .unwrap_err()
                .field(),
            Some("contacts")
        );
        assert_eq!(
            Station::new(Mars, f64::NAN, true).unwrap_err().field(),
            Some("stations")
        );
    }

    /// Lilly's "If the Querent should ever have Children?" (*Christian
    /// Astrology* p. 238), "Die ♃ 11 June 1635" counted from noon, so the
    /// morning of Friday 12 June (Julian) in London, recast by pyswisseph
    /// (Moshier) at the printed Ascendant, Virgo 24°33′: Saturn, Mars and
    /// the Sun stand within 5′ of the figure, and the houses are its
    /// Regiomontanus cusps'. The judgement reads Mercury, lord of the
    /// Ascendant, "going to ☍ of ♄", the fifth's lord, retrograde, and
    /// finds "no one promising testimony". The opposition applies against
    /// a retrograde, p. 107's third kind, before Mercury leaves Gemini,
    /// nothing comes between, and no way holds: an opposition perfects
    /// only with mutual reception by house and the Moon's relay (p. 126),
    /// and nothing is heavier than Saturn to collect.
    #[test]
    fn lillys_children_figure_applies_by_opposition_and_holds_no_way() {
        let places = [
            269.4527, 125.1334, 78.5604, 90.5781, 50.7822, 85.2252, 180.6684,
        ];
        let speeds = [-0.0736, 0.1985, 0.6821, 0.9533, 1.1433, 2.1614, 11.8363];
        let horizon = PerfectionRules::LILLY
            .horizon_for(&places, &speeds, Mercury, Saturn)
            .unwrap();
        let timeline = AspectTimeline::projected(places, speeds, horizon).unwrap();
        let house = |n| House::try_new(n).unwrap();
        let standing = Standing {
            houses: [
                house(4),
                house(11),
                house(9),
                house(10),
                house(9),
                house(10),
                house(1),
            ],
            sect: Sect::Day,
            dignities: &DignityRules::LILLY,
        };
        let matter = perfection(
            &timeline,
            Mercury,
            Saturn,
            &standing,
            &PerfectionRules::LILLY,
        )
        .unwrap();
        let application = matter.application.unwrap();
        assert_eq!(
            (application.aspect, application.applying, application.kind),
            (
                PtolemaicAspect::Opposition,
                Mercury,
                ApplicationKind::AgainstRetrograde
            )
        );
        assert!(
            (application.days - 1.89).abs() < 0.01,
            "{}",
            application.days
        );
        assert!(application.days < horizon);
        assert_eq!(matter.impediments, []);
        assert_eq!(matter.translations, []);
        assert_eq!(matter.collections, []);
        assert!(!matter.ways.mutual_by_house);
        assert_eq!(matter.ways.held, []);
    }
}
