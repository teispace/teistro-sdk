//! Visibility and the heliacal phenomena: whether a body near the Sun can
//! be seen at dawn or dusk, and the days on which it appears and
//! disappears (`docs/03-design/astro-planetary-phenomena.md`, §4).
//!
//! Three criteria, each a convention the caller names ([`Criterion`]),
//! because the tradition and the astronomers disagree and both are
//! legitimate:
//!
//! - **Degrees of time**, the Surya Siddhanta's (IX.2 to 11 and X.1 in
//!   Burgess's 1860 translation): the interval in oblique ascension
//!   between the body's rising and the Sun's in the east, or between the
//!   Sun's setting and the body's in the west, one degree of time being
//!   four minutes of sidereal rotation, against the text's threshold for
//!   the body and its motion (Jupiter 11, Saturn 15, Mars 17; Venus 10
//!   direct and 8 retrograde; Mercury 14 direct and 12 retrograde; the
//!   Moon 12).
//! - **Longitude**, the tradition's combustion orb (asta): the difference
//!   of ecliptic longitude between the body and the Sun against the same
//!   numbers, which the tradition reads as degrees of longitude.
//! - **Arcus visionis**, the classical astronomers' measure: how far the
//!   Sun stands below the horizon at the moment the body stands on it,
//!   against a threshold per body; Ptolemy's (Almagest XIII.7 to 9, as
//!   Burgess quotes them under IX.9) by default.
//!
//! The body is visible on a day when its measure reaches the threshold,
//! in the morning sky (the east, the body west of the Sun in longitude)
//! or the evening sky (the west). The heliacal events are the days on
//! which that state changes: the first morning a body is seen (the
//! heliacal rising), the last morning, the first evening, the last
//! evening (the heliacal setting). The scan reads the state day by day
//! over the rise and set solver and the frame completion, so it runs over
//! any provider, a modern engine or the classical astronomy alike.
//!
//! ```
//! use teistro_astro::visibility::{Criterion, Heliacal, HeliacalKind};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
//! use teistro_core::settings::OverridePolicy;
//! use teistro_port_ephemeris::{Body, Horizon, TestProvider};
//!
//! let provider = TestProvider::new();
//! let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
//! let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
//! let heliacal = Heliacal::new(&sky, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, DeltaTModel::TableThenModel);
//! let from = JulianDay::literal(2_460_310.5);
//! let events = heliacal.events(Body::Mercury, from, from.plus_days(240.0).expect("finite")).expect("the test provider answers");
//! // The test provider's Mercury always outruns the Sun, so it leaves the
//! // morning sky and enters the evening sky in turn.
//! assert!(events.len() >= 3);
//! assert!(events.iter().all(|e| matches!(e.kind, HeliacalKind::MorningLast | HeliacalKind::EveningFirst)));
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::angle::difference_deg;
use teistro_core::catalogue::{Nakshatra, Star};
use teistro_core::error::Error;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_port_ephemeris::{Body, EphemerisProvider, Frame, Horizon, HorizonEventKind};

use crate::completion::Completion;
use crate::delta_t::DeltaTModel;
use crate::events::{Longitudes, check_window};
use crate::rise_set::{HOUR_ANGLE_RATE_DEG_PER_DAY, HorizonEvent, Solver};
use crate::sky::{ApparentPositions, local_mean_midnight};

/// Whether the body advances or retrogrades, which the Surya Siddhanta's
/// thresholds for Mercury and Venus turn on (IX.7 to 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Motion {
    /// Longitude increasing.
    Direct,
    /// Longitude decreasing.
    Retrograde,
}

impl Motion {
    /// The motion a rate of longitude shows.
    #[must_use]
    pub fn of_speed(speed_deg_per_day: f64) -> Motion {
        if speed_deg_per_day < 0.0 {
            Motion::Retrograde
        } else {
            Motion::Direct
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Motion::Direct => "DIRECT",
            Motion::Retrograde => "RETROGRADE",
        }
    }
}

/// Which sky the body is in: the morning sky, rising before the Sun, or
/// the evening sky, setting after it (IX.2 to 3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Side {
    /// The morning sky: the body's longitude is less than the Sun's, so it
    /// rises first.
    East,
    /// The evening sky: the body's longitude is greater, so it sets last.
    West,
}

impl Side {
    /// The side from the body's longitude less the Sun's, folded to a half
    /// turn either way.
    #[must_use]
    pub fn of_elongation(elongation_deg: f64) -> Side {
        if elongation_deg < 0.0 {
            Side::East
        } else {
            Side::West
        }
    }

    /// The horizon event the side's measure is read at: the rising in the
    /// east, the setting in the west.
    #[must_use]
    pub const fn event_kind(self) -> HorizonEventKind {
        match self {
            Side::East => HorizonEventKind::Rise,
            Side::West => HorizonEventKind::Set,
        }
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Side::East => "EAST",
            Side::West => "WEST",
        }
    }
}

/// A threshold for each motion of one body, degrees.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    /// The threshold while the body advances.
    pub direct: f64,
    /// The threshold while it retrogrades.
    pub retrograde: f64,
}

impl Pair {
    /// The same threshold for both motions.
    #[must_use]
    pub const fn same(deg: f64) -> Pair {
        Pair {
            direct: deg,
            retrograde: deg,
        }
    }

    /// The threshold for a motion.
    #[must_use]
    pub const fn of(self, motion: Motion) -> f64 {
        match motion {
            Motion::Direct => self.direct,
            Motion::Retrograde => self.retrograde,
        }
    }
}

/// A caller's own thresholds, degrees, per body; `None` for a body the
/// table does not place.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Table {
    /// The Moon's.
    pub moon: Option<Pair>,
    /// Mercury's.
    pub mercury: Option<Pair>,
    /// Venus's.
    pub venus: Option<Pair>,
    /// Mars's.
    pub mars: Option<Pair>,
    /// Jupiter's.
    pub jupiter: Option<Pair>,
    /// Saturn's.
    pub saturn: Option<Pair>,
    /// Uranus's.
    pub uranus: Option<Pair>,
    /// Neptune's.
    pub neptune: Option<Pair>,
    /// Pluto's.
    pub pluto: Option<Pair>,
}

impl Table {
    /// No body placed.
    pub const EMPTY: Table = Table {
        moon: None,
        mercury: None,
        venus: None,
        mars: None,
        jupiter: None,
        saturn: None,
        uranus: None,
        neptune: None,
        pluto: None,
    };

    /// The Surya Siddhanta's degrees of time (IX.6 to 8; the Moon's,
    /// X.1): Jupiter 11, Saturn 15, Mars 17; Venus 10 direct and 8
    /// retrograde; Mercury 14 direct and 12 retrograde; the Moon 12.
    pub const SURYA_SIDDHANTA: Table = Table {
        moon: Some(Pair::same(12.0)),
        mercury: Some(Pair {
            direct: 14.0,
            retrograde: 12.0,
        }),
        venus: Some(Pair {
            direct: 10.0,
            retrograde: 8.0,
        }),
        mars: Some(Pair::same(17.0)),
        jupiter: Some(Pair::same(11.0)),
        saturn: Some(Pair::same(15.0)),
        ..Table::EMPTY
    };

    /// Ptolemy's arcus visionis (Almagest XIII.7 to 9, as Burgess quotes
    /// them in his note to IX.9): Saturn 14°, Jupiter 12°45′, Mars 14°30′,
    /// Venus 5°40′, Mercury 11°30′, the same for both motions; nothing for
    /// the Moon.
    pub const PTOLEMY: Table = Table {
        mercury: Some(Pair::same(11.5)),
        venus: Some(Pair::same(5.0 + 40.0 / 60.0)),
        mars: Some(Pair::same(14.5)),
        jupiter: Some(Pair::same(12.75)),
        saturn: Some(Pair::same(14.0)),
        ..Table::EMPTY
    };

    /// The same threshold for the Moon and every planet, both motions.
    #[must_use]
    pub const fn uniform(deg: f64) -> Table {
        let pair = Some(Pair::same(deg));
        Table {
            moon: pair,
            mercury: pair,
            venus: pair,
            mars: pair,
            jupiter: pair,
            saturn: pair,
            uranus: pair,
            neptune: pair,
            pluto: pair,
        }
    }

    /// The threshold for a body and motion.
    #[must_use]
    pub fn of(&self, body: Body, motion: Motion) -> Option<f64> {
        let pair = match body {
            Body::Moon => self.moon,
            Body::Mercury => self.mercury,
            Body::Venus => self.venus,
            Body::Mars => self.mars,
            Body::Jupiter => self.jupiter,
            Body::Saturn => self.saturn,
            Body::Uranus => self.uranus,
            Body::Neptune => self.neptune,
            Body::Pluto => self.pluto,
            _ => None,
        };
        pair.map(|pair| pair.of(motion))
    }
}

/// The threshold table a criterion compares its measure against.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(
    clippy::large_enum_variant,
    reason = "a custom table is nine optional pairs, the size a criterion costs to pass by value once"
)]
pub enum Thresholds {
    /// The Surya Siddhanta's degrees of time ([`Table::SURYA_SIDDHANTA`]).
    /// The tradition's combustion orbs are these same numbers. The
    /// asterisms' junction stars and the named stars have their classes
    /// (IX.12 to 15, [`Thresholds::of_star`]).
    SuryaSiddhanta,
    /// Ptolemy's arcus visionis ([`Table::PTOLEMY`]).
    Ptolemy,
    /// A caller's own table.
    Custom(Table),
}

impl Thresholds {
    /// The table behind the name.
    #[must_use]
    pub const fn table(&self) -> &Table {
        match self {
            Thresholds::SuryaSiddhanta => &Table::SURYA_SIDDHANTA,
            Thresholds::Ptolemy => &Table::PTOLEMY,
            Thresholds::Custom(table) => table,
        }
    }

    /// The threshold for a body and motion, degrees; `None` for a body the
    /// table does not place.
    #[must_use]
    pub fn of(&self, body: Body, motion: Motion) -> Option<f64> {
        self.table().of(body, motion)
    }

    /// Whether the table places a body under either motion.
    #[must_use]
    pub fn places(&self, body: Body) -> bool {
        self.of(body, Motion::Direct).is_some() || self.of(body, Motion::Retrograde).is_some()
    }

    /// The threshold for a star, degrees: the Surya Siddhanta's classes of
    /// the asterisms' junction stars (IX.12 to 15) and of the named stars
    /// (Agastya, Mrgavyadha, Abhijit and Brahmahrdaya at thirteen); `None`
    /// under the other tables and for a star the text does not place.
    #[must_use]
    pub fn of_star(&self, star: Star) -> Option<f64> {
        match self {
            Thresholds::SuryaSiddhanta => surya_siddhanta_star_deg(star),
            Thresholds::Ptolemy | Thresholds::Custom(_) => None,
        }
    }

    /// The stars the Surya Siddhanta says the Sun's rays never extinguish
    /// (IX.18), owing to their northern situation at its latitude: Abhijit
    /// (Vega), Brahmahrdaya (Capella) and the junction stars of Svati,
    /// Shravana, Shravishtha and Uttara-Bhadrapada.
    #[must_use]
    pub fn never_sets_heliacally(star: Star) -> bool {
        matches!(star, Star::Vega | Star::Capella)
            || matches!(
                star.attributes().yogatara_of,
                Some(
                    Nakshatra::Swati
                        | Nakshatra::Shravana
                        | Nakshatra::Dhanishtha
                        | Nakshatra::UttaraBhadrapada
                )
            )
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(&self) -> &'static str {
        match self {
            Thresholds::SuryaSiddhanta => "SURYA_SIDDHANTA",
            Thresholds::Ptolemy => "PTOLEMY",
            Thresholds::Custom(_) => "CUSTOM",
        }
    }
}

/// IX.12 to 15: the classes of the junction stars by asterism, and the
/// named stars of the first class.
fn surya_siddhanta_star_deg(star: Star) -> Option<f64> {
    if matches!(
        star,
        Star::Vega | Star::Capella | Star::Canopus | Star::Sirius
    ) {
        return Some(13.0);
    }
    let nakshatra = star.attributes().yogatara_of?;
    let deg = match nakshatra {
        Nakshatra::Swati | Nakshatra::Chitra | Nakshatra::Jyeshtha | Nakshatra::Punarvasu => 13.0,
        Nakshatra::Hasta
        | Nakshatra::Shravana
        | Nakshatra::PurvaPhalguni
        | Nakshatra::UttaraPhalguni
        | Nakshatra::Dhanishtha
        | Nakshatra::Rohini
        | Nakshatra::Magha
        | Nakshatra::Vishakha
        | Nakshatra::Ashwini => 14.0,
        Nakshatra::Krittika
        | Nakshatra::Anuradha
        | Nakshatra::Mula
        | Nakshatra::Ashlesha
        | Nakshatra::Ardra
        | Nakshatra::PurvaAshadha
        | Nakshatra::UttaraAshadha => 15.0,
        Nakshatra::Bharani | Nakshatra::Pushya | Nakshatra::Mrigashira => 21.0,
        Nakshatra::Shatabhisha
        | Nakshatra::PurvaBhadrapada
        | Nakshatra::UttaraBhadrapada
        | Nakshatra::Revati => 17.0,
        // The catalogue kind may grow; a member the text does not name has
        // no class.
        _ => return None,
    };
    Some(deg)
}

/// How visibility is decided.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Criterion {
    /// The interval in oblique ascension between the body's rising and the
    /// Sun's (the east) or the Sun's setting and the body's (the west), in
    /// degrees of time, one being four minutes of sidereal rotation: the
    /// Surya Siddhanta's measure (IX.4 to 5), read at sunrise or sunset.
    TimeDegrees {
        /// The thresholds.
        thresholds: Thresholds,
    },
    /// The difference of ecliptic longitude between the body and the Sun,
    /// read at sunrise or sunset: the tradition's combustion orb.
    Longitude {
        /// The thresholds.
        thresholds: Thresholds,
    },
    /// The Sun's depression below the horizon at the deepest twilight the
    /// body is up in: its own rising (the east) or setting (the west), or
    /// the Sun's antitransit when the body is still up at that hour, so
    /// that a body far from the Sun, up at midnight, is seen. The arcus
    /// visionis.
    ArcusVisionis {
        /// The thresholds.
        thresholds: Thresholds,
    },
}

impl Criterion {
    /// The Surya Siddhanta's: degrees of time against the text's numbers.
    pub const SURYA_SIDDHANTA: Criterion = Criterion::TimeDegrees {
        thresholds: Thresholds::SuryaSiddhanta,
    };

    /// The tradition's combustion: the same numbers as degrees of
    /// longitude.
    pub const COMBUSTION_ORB: Criterion = Criterion::Longitude {
        thresholds: Thresholds::SuryaSiddhanta,
    };

    /// Ptolemy's arcus visionis.
    pub const PTOLEMY: Criterion = Criterion::ArcusVisionis {
        thresholds: Thresholds::Ptolemy,
    };

    /// The thresholds the criterion compares against.
    #[must_use]
    pub const fn thresholds(&self) -> &Thresholds {
        match self {
            Criterion::TimeDegrees { thresholds }
            | Criterion::Longitude { thresholds }
            | Criterion::ArcusVisionis { thresholds } => thresholds,
        }
    }

    /// The measure's name.
    #[must_use]
    pub const fn measure(&self) -> &'static str {
        match self {
            Criterion::TimeDegrees { .. } => "TIME_DEGREES",
            Criterion::Longitude { .. } => "LONGITUDE",
            Criterion::ArcusVisionis { .. } => "ARCUS_VISIONIS",
        }
    }

    /// The key stamped in provenance: the measure and the table.
    #[must_use]
    pub fn key(&self) -> String {
        format!("{}/{}", self.measure(), self.thresholds().key())
    }
}

impl fmt::Display for Criterion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.key())
    }
}

/// The state of a body's visibility on one local mean day.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Visibility {
    /// The body.
    pub body: Body,
    /// The local mean midnight the day begins at, UT1.
    pub day_start: JulianDay<Ut1>,
    /// The instant the measure was read at: the sunrise or the sunset for
    /// the degrees of time and the longitude; for the arcus visionis the
    /// body's own rising or setting, or the Sun's antitransit when the
    /// body is up at that hour.
    pub instant: JulianDay<Ut1>,
    /// Which sky the body is in.
    pub side: Side,
    /// The body's motion at the instant.
    pub motion: Motion,
    /// The criterion's measure, degrees.
    pub measure_deg: f64,
    /// The threshold it is held against, degrees.
    pub threshold_deg: f64,
    /// Whether the measure reaches the threshold.
    pub visible: bool,
    /// How many positions were read.
    pub evaluations: u32,
}

/// Which heliacal event.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HeliacalKind {
    /// The first morning the body is seen: its heliacal rising.
    MorningFirst,
    /// The last morning it is seen before the Sun overtakes it.
    MorningLast,
    /// The first evening it is seen after passing the Sun.
    EveningFirst,
    /// The last evening it is seen: its heliacal setting.
    EveningLast,
}

impl HeliacalKind {
    /// The event a change of state on a side is.
    #[must_use]
    pub const fn of(side: Side, appears: bool) -> HeliacalKind {
        match (side, appears) {
            (Side::East, true) => HeliacalKind::MorningFirst,
            (Side::East, false) => HeliacalKind::MorningLast,
            (Side::West, true) => HeliacalKind::EveningFirst,
            (Side::West, false) => HeliacalKind::EveningLast,
        }
    }

    /// The sky the event is in.
    #[must_use]
    pub const fn side(self) -> Side {
        match self {
            HeliacalKind::MorningFirst | HeliacalKind::MorningLast => Side::East,
            HeliacalKind::EveningFirst | HeliacalKind::EveningLast => Side::West,
        }
    }

    /// Whether the body appears (a first) rather than disappears (a last).
    #[must_use]
    pub const fn appears(self) -> bool {
        matches!(
            self,
            HeliacalKind::MorningFirst | HeliacalKind::EveningFirst
        )
    }

    /// The key stamped in provenance.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            HeliacalKind::MorningFirst => "MORNING_FIRST",
            HeliacalKind::MorningLast => "MORNING_LAST",
            HeliacalKind::EveningFirst => "EVENING_FIRST",
            HeliacalKind::EveningLast => "EVENING_LAST",
        }
    }
}

/// A heliacal event: the day a body is first seen, or the last day it is.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct HeliacalEvent {
    /// Which event.
    pub kind: HeliacalKind,
    /// The state on the day the event is dated: the first day seen for a
    /// first, the last day seen for a last.
    pub day: Visibility,
}

/// The visibility of bodies near the Sun at a place under a criterion,
/// over the frame completion: the state on a day, and the heliacal events
/// inside a window.
pub struct Heliacal<'a, P: EphemerisProvider + ?Sized> {
    completion: &'a Completion<'a, P>,
    place: Place,
    criterion: Criterion,
    horizon: Horizon,
    delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> fmt::Debug for Heliacal<'_, P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

impl<'a, P: EphemerisProvider + ?Sized> Heliacal<'a, P> {
    /// A visibility reckoner: the completion the positions come from, the
    /// place, the criterion, the horizon convention the risings and
    /// settings are taken under (the Sun's and the body's alike), and the
    /// Delta T model for the sidereal time.
    #[must_use]
    pub const fn new(
        completion: &'a Completion<'a, P>,
        place: Place,
        criterion: Criterion,
        horizon: Horizon,
        delta_t: DeltaTModel,
    ) -> Heliacal<'a, P> {
        Heliacal {
            completion,
            place,
            criterion,
            horizon,
            delta_t,
        }
    }

    /// The criterion.
    #[must_use]
    pub const fn criterion(&self) -> Criterion {
        self.criterion
    }

    /// The stamp: criterion, horizon convention, place and source.
    #[must_use]
    pub fn describe(&self) -> String {
        format!(
            "{} under {} at {} from {}",
            self.criterion,
            self.horizon,
            self.place,
            self.completion.describe()
        )
    }

    /// The local mean midnight that begins the day an instant falls in at
    /// the place.
    #[must_use]
    pub fn day_start(&self, at: JulianDay<Ut1>) -> JulianDay<Ut1> {
        local_mean_midnight(at, self.place.longitude)
    }

    /// The state of a body on the local mean day beginning at `day_start`.
    ///
    /// # Errors
    ///
    /// The Sun itself or a point without light (`INVALID_ARG`,
    /// `UNSUPPORTED`); a body the criterion's table does not place
    /// (`UNSUPPORTED`, naming the table); a day without a sunrise or a
    /// sunset, or a body that neither rises nor sets around it, where there
    /// is no dawn or dusk to read visibility at (`LIMIT`); the provider's
    /// refusal.
    pub fn state(&self, body: Body, day_start: JulianDay<Ut1>) -> Result<Visibility, Error> {
        self.check_body(body)?;
        let sun = self.solver(Body::Sun);
        let target = self.solver(body);
        let mut evaluations = 0u32;
        let sunrise = self.horizon_event(
            &sun,
            HorizonEventKind::Rise,
            day_start,
            1.0,
            &mut evaluations,
        )?;
        let sunset = self.horizon_event(
            &sun,
            HorizonEventKind::Set,
            day_start,
            1.0,
            &mut evaluations,
        )?;
        let longitudes = self.completion.longitudes(Frame::CANONICAL);
        let at_sunrise = elongation(&longitudes, body, sunrise, &mut evaluations)?;
        let side = Side::of_elongation(at_sunrise.0);
        let (reference, (elongation_deg, speed)) = match side {
            Side::East => (sunrise, at_sunrise),
            Side::West => (
                sunset,
                elongation(&longitudes, body, sunset, &mut evaluations)?,
            ),
        };
        let motion = Motion::of_speed(speed);
        let threshold_deg = self
            .criterion
            .thresholds()
            .of(body, motion)
            .ok_or_else(|| self.no_threshold(body, motion))?;
        let (instant, measure_deg) = match self.criterion {
            Criterion::Longitude { .. } => (reference, elongation_deg.abs()),
            Criterion::TimeDegrees { .. } => {
                let own = self.nearest(&target, side, reference, &mut evaluations)?;
                let interval_days = match side {
                    Side::East => reference.get() - own.get(),
                    Side::West => own.get() - reference.get(),
                };
                (reference, interval_days * HOUR_ANGLE_RATE_DEG_PER_DAY)
            }
            Criterion::ArcusVisionis { .. } => {
                let own = self.nearest(&target, side, reference, &mut evaluations)?;
                let deepest = deepest_night(&sun, side, reference, own, &mut evaluations)?;
                evaluations += 1;
                (deepest, -sun.altitude_deg(deepest)?)
            }
        };
        Ok(Visibility {
            body,
            day_start,
            instant,
            side,
            motion,
            measure_deg,
            threshold_deg,
            visible: measure_deg >= threshold_deg,
            evaluations,
        })
    }

    /// Every heliacal event of a body whose day begins inside `[from, to)`,
    /// in time order, reading the state day by day from the local mean day
    /// `from` falls in.
    ///
    /// # Errors
    ///
    /// A window that does not run forward (`INVALID_ARG`), and whatever
    /// [`Heliacal::state`] refuses on any day of it.
    pub fn events(
        &self,
        body: Body,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
    ) -> Result<Vec<HeliacalEvent>, Error> {
        let mut events = Vec::new();
        self.walk(body, from, to, |event| {
            events.push(event);
            true
        })?;
        Ok(events)
    }

    /// The first heliacal event of a body whose day begins inside a window
    /// of so many days from `from`, or `None` when there is none.
    ///
    /// # Errors
    ///
    /// A window that is not a positive number of days (`INVALID_ARG`), and
    /// whatever [`Heliacal::state`] refuses on any day of it.
    pub fn next(
        &self,
        body: Body,
        from: JulianDay<Ut1>,
        window_days: f64,
    ) -> Result<Option<HeliacalEvent>, Error> {
        if !(window_days.is_finite() && window_days > 0.0) {
            return Err(Error::invalid_arg(format!(
                "a search window is a positive number of days, not {window_days}"
            ))
            .with_field("window_days"));
        }
        let mut found = None;
        self.walk(body, from, from.plus_days(window_days)?, |event| {
            found = Some(event);
            false
        })?;
        Ok(found)
    }

    /// Reads the state day by day and hands each change to `sink`, which
    /// returns whether to go on.
    fn walk(
        &self,
        body: Body,
        from: JulianDay<Ut1>,
        to: JulianDay<Ut1>,
        mut sink: impl FnMut(HeliacalEvent) -> bool,
    ) -> Result<(), Error> {
        check_window("visibility", from, to)?;
        let mut day = self.day_start(from);
        let mut previous = self.state(body, day)?;
        loop {
            day = day.plus_days(1.0)?;
            if day.get() >= to.get() {
                return Ok(());
            }
            let current = self.state(body, day)?;
            if let Some(event) = transition(&previous, &current) {
                if !sink(event) {
                    return Ok(());
                }
            }
            previous = current;
        }
    }

    fn solver(&self, body: Body) -> Solver<'_> {
        Solver::new(
            self.completion,
            body,
            self.place,
            self.horizon,
            self.delta_t,
        )
    }

    /// A horizon event of the Sun inside a window, or the refusal that
    /// names the day without a dawn or dusk.
    fn horizon_event(
        &self,
        solver: &Solver<'_>,
        kind: HorizonEventKind,
        from: JulianDay<Ut1>,
        window_days: f64,
        evaluations: &mut u32,
    ) -> Result<JulianDay<Ut1>, Error> {
        match solver.event(kind, from, window_days)? {
            Some(HorizonEvent {
                instant,
                evaluations: read,
                ..
            }) => {
                *evaluations += read;
                Ok(instant)
            }
            None => Err(Error::limit(format!(
                "the Sun has no {} inside the day beginning at JD {} at latitude {}, so there is no dawn or dusk to read visibility at",
                kind.key().to_lowercase(),
                from.get(),
                self.place.latitude
            ))
            .with_field("day_start")
            .with_hint("visibility is a question for a day with a sunrise and a sunset")),
        }
    }

    /// The body's own rising (the east) or setting (the west) nearest the
    /// reference instant, within half a day either way.
    fn nearest(
        &self,
        solver: &Solver<'_>,
        side: Side,
        reference: JulianDay<Ut1>,
        evaluations: &mut u32,
    ) -> Result<JulianDay<Ut1>, Error> {
        let from = reference.plus_days(-0.5)?;
        match solver.event(side.event_kind(), from, 1.0)? {
            Some(HorizonEvent {
                instant,
                evaluations: read,
                ..
            }) => {
                *evaluations += read;
                Ok(instant)
            }
            None => Err(Error::limit(format!(
                "{} has no {} within half a day of JD {} at latitude {}",
                solver.describe(),
                side.event_kind().key().to_lowercase(),
                reference.get(),
                self.place.latitude
            ))
            .with_field("body")),
        }
    }

    fn check_body(&self, body: Body) -> Result<(), Error> {
        if body == Body::Sun {
            return Err(Error::invalid_arg(
                "the Sun's own visibility is not a question; name the body seen near it",
            )
            .with_field("body"));
        }
        if !body.has_distance() {
            return Err(Error::unsupported(format!(
                "{} is a point without light; visibility is a question for a body",
                body.key()
            ))
            .with_field("body"));
        }
        if !self.criterion.thresholds().places(body) {
            return Err(self.no_threshold(body, Motion::Direct));
        }
        Ok(())
    }

    fn no_threshold(&self, body: Body, motion: Motion) -> Error {
        Error::unsupported(format!(
            "the {} thresholds do not place {} ({}); supply Thresholds::Custom with a value for it",
            self.criterion.thresholds().key(),
            body.key(),
            motion.key()
        ))
        .with_field("criterion")
    }
}

/// The deepest twilight the body is up in around the reference: its own
/// rising or setting, unless the Sun's antitransit of that night falls
/// while the body is up, which is then the deeper moment. A body that
/// rises after the sunrise or sets before the sunset keeps its own
/// instant, where the Sun stands above the horizon and the measure says
/// so.
fn deepest_night(
    sun: &Solver<'_>,
    side: Side,
    reference: JulianDay<Ut1>,
    own: JulianDay<Ut1>,
    evaluations: &mut u32,
) -> Result<JulianDay<Ut1>, Error> {
    let from = match side {
        Side::East => reference.plus_days(-1.0)?,
        Side::West => reference,
    };
    let Some(HorizonEvent {
        instant: antitransit,
        evaluations: read,
        ..
    }) = sun.event(HorizonEventKind::Antitransit, from, 1.0)?
    else {
        return Ok(own);
    };
    *evaluations += read;
    Ok(match side {
        // The body rose before the night's deepest point: measure there.
        Side::East if own.get() < antitransit.get() && antitransit.get() < reference.get() => {
            antitransit
        }
        // The body sets after it: likewise.
        Side::West if own.get() > antitransit.get() && antitransit.get() > reference.get() => {
            antitransit
        }
        _ => own,
    })
}

/// The body's longitude less the Sun's, folded to a half turn either way,
/// and the body's rate, at an instant.
fn elongation(
    longitudes: &dyn Longitudes,
    body: Body,
    at: JulianDay<Ut1>,
    evaluations: &mut u32,
) -> Result<(f64, f64), Error> {
    let (body_deg, speed) = longitudes.longitude_and_speed(body, at)?;
    let (sun_deg, _) = longitudes.longitude_and_speed(Body::Sun, at)?;
    *evaluations += 2;
    Ok((difference_deg(body_deg, sun_deg), speed))
}

/// The event between two consecutive days' states, if any: the body seen
/// today and not yesterday is a first on today's side; seen yesterday and
/// not today, a last on yesterday's. A change of side alone is not an
/// event: seen on both days it is the opposition, seen on neither the
/// passage of the conjunction between the last and the first.
fn transition(previous: &Visibility, current: &Visibility) -> Option<HeliacalEvent> {
    match (previous.visible, current.visible) {
        (false, true) => Some(HeliacalEvent {
            kind: HeliacalKind::of(current.side, true),
            day: *current,
        }),
        (true, false) => Some(HeliacalEvent {
            kind: HeliacalKind::of(previous.side, false),
            day: *previous,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::panic,
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::float_cmp,
        clippy::indexing_slicing,
        reason = "tests fail by panicking"
    )]

    use teistro_core::quantity::{Altitude, Latitude, Longitude};
    use teistro_core::settings::OverridePolicy;
    use teistro_port_ephemeris::TestProvider;

    use super::*;

    fn kathmandu() -> Place {
        Place::new(
            Latitude::literal(27.7172),
            Longitude::literal(85.324),
            Altitude::literal(1400.0),
        )
    }

    #[test]
    fn the_texts_thresholds_are_its_verses() {
        let text = Thresholds::SuryaSiddhanta;
        assert_eq!(text.of(Body::Jupiter, Motion::Direct), Some(11.0));
        assert_eq!(text.of(Body::Saturn, Motion::Retrograde), Some(15.0));
        assert_eq!(text.of(Body::Mars, Motion::Direct), Some(17.0));
        assert_eq!(text.of(Body::Venus, Motion::Direct), Some(10.0));
        assert_eq!(text.of(Body::Venus, Motion::Retrograde), Some(8.0));
        assert_eq!(text.of(Body::Mercury, Motion::Direct), Some(14.0));
        assert_eq!(text.of(Body::Mercury, Motion::Retrograde), Some(12.0));
        assert_eq!(text.of(Body::Moon, Motion::Direct), Some(12.0));
        assert_eq!(text.of(Body::Uranus, Motion::Direct), None);
        assert!(!text.places(Body::MeanNode) && text.places(Body::Moon));
        let ptolemy = Thresholds::Ptolemy;
        assert_eq!(ptolemy.of(Body::Saturn, Motion::Direct), Some(14.0));
        assert_eq!(ptolemy.of(Body::Jupiter, Motion::Direct), Some(12.75));
        assert_eq!(ptolemy.of(Body::Mars, Motion::Retrograde), Some(14.5));
        assert!(
            (ptolemy.of(Body::Venus, Motion::Direct).unwrap() - 5.666_666_666_666_667).abs()
                < 1e-12
        );
        assert_eq!(ptolemy.of(Body::Mercury, Motion::Direct), Some(11.5));
        assert_eq!(ptolemy.of(Body::Moon, Motion::Direct), None);
        let custom = Thresholds::Custom(Table {
            uranus: Some(Pair::same(20.0)),
            ..Table::uniform(9.0)
        });
        assert_eq!(custom.of(Body::Uranus, Motion::Direct), Some(20.0));
        assert_eq!(custom.of(Body::Moon, Motion::Retrograde), Some(9.0));
        assert_eq!(custom.of(Body::TrueNode, Motion::Direct), None);
        assert_eq!(
            Criterion::SURYA_SIDDHANTA.key(),
            "TIME_DEGREES/SURYA_SIDDHANTA"
        );
        assert_eq!(
            Criterion::COMBUSTION_ORB.to_string(),
            "LONGITUDE/SURYA_SIDDHANTA"
        );
        assert_eq!(Criterion::PTOLEMY.key(), "ARCUS_VISIONIS/PTOLEMY");
        let json = serde_json::to_string(&Criterion::PTOLEMY).unwrap();
        assert!(json.contains("ARCUS_VISIONIS") && json.contains("PTOLEMY"));
    }

    #[test]
    fn the_texts_star_classes_cover_every_asterism_and_its_named_stars() {
        let text = Thresholds::SuryaSiddhanta;
        let mut by_class = [0usize; 5];
        let mut asterisms = 0;
        for star in Star::ALL {
            if let Some(deg) = text.of_star(star) {
                let class = [13.0, 14.0, 15.0, 17.0, 21.0]
                    .iter()
                    .position(|c| (c - deg).abs() < 1e-9)
                    .unwrap_or_else(|| panic!("{deg}"));
                by_class[class] += 1;
                if star.attributes().yogatara_of.is_some() {
                    asterisms += 1;
                }
            }
        }
        // Twenty-seven junction stars, four named stars at thirteen.
        assert_eq!(asterisms, 27);
        assert_eq!(by_class[0], 4 + 4);
        assert_eq!(by_class[1], 9);
        assert_eq!(by_class[2], 7);
        assert_eq!(by_class[3], 4);
        assert_eq!(by_class[4], 3);
        assert_eq!(text.of_star(Star::Sirius), Some(13.0));
        assert_eq!(Thresholds::Ptolemy.of_star(Star::Sirius), None);
        assert!(Thresholds::never_sets_heliacally(Star::Vega));
        assert!(Thresholds::never_sets_heliacally(Star::Capella));
        assert!(!Thresholds::never_sets_heliacally(Star::Sirius));
        assert_eq!(
            Star::ALL
                .iter()
                .filter(|s| Thresholds::never_sets_heliacally(**s))
                .count(),
            6
        );
    }

    #[test]
    fn sides_motions_and_kinds_read_their_signs() {
        assert_eq!(Side::of_elongation(-10.0), Side::East);
        assert_eq!(Side::of_elongation(10.0), Side::West);
        assert_eq!(Side::East.event_kind(), HorizonEventKind::Rise);
        assert_eq!(Motion::of_speed(-0.1), Motion::Retrograde);
        assert_eq!(Motion::of_speed(0.0), Motion::Direct);
        for kind in [
            HeliacalKind::MorningFirst,
            HeliacalKind::MorningLast,
            HeliacalKind::EveningFirst,
            HeliacalKind::EveningLast,
        ] {
            assert_eq!(HeliacalKind::of(kind.side(), kind.appears()), kind);
        }
        assert_eq!(HeliacalKind::EveningLast.key(), "EVENING_LAST");
    }

    #[test]
    fn the_scan_finds_alternating_events_over_the_test_sky() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let from = JulianDay::literal(2_460_310.5);
        let to = from.plus_days(240.0).unwrap();
        for criterion in [
            Criterion::SURYA_SIDDHANTA,
            Criterion::COMBUSTION_ORB,
            Criterion::PTOLEMY,
        ] {
            let heliacal = Heliacal::new(
                &sky,
                kathmandu(),
                criterion,
                Horizon::CENTRE_NO_REFRACTION,
                DeltaTModel::TableThenModel,
            );
            let events = heliacal.events(Body::Mercury, from, to).unwrap();
            assert!(events.len() >= 3, "{criterion}: {} events", events.len());
            // The test provider's Mercury always outruns the Sun: it leaves
            // the morning sky, then enters the evening sky, and so on.
            for pair in events.windows(2) {
                assert_ne!(pair[0].kind, pair[1].kind, "{criterion}");
                assert!(pair[0].day.day_start.get() < pair[1].day.day_start.get());
            }
            for event in &events {
                assert!(
                    matches!(
                        event.kind,
                        HeliacalKind::MorningLast | HeliacalKind::EveningFirst
                    ),
                    "{criterion}: {:?}",
                    event.kind
                );
                assert!(event.day.visible && event.day.measure_deg >= event.day.threshold_deg);
                assert_eq!(event.day.motion, Motion::Direct);
                assert_eq!(event.kind.side(), event.day.side);
                // The day before a first, or after a last, is not seen.
                let neighbour = if event.kind.appears() { -1.0 } else { 1.0 };
                let other = heliacal
                    .state(
                        Body::Mercury,
                        event.day.day_start.plus_days(neighbour).unwrap(),
                    )
                    .unwrap();
                assert!(
                    !other.visible || other.side != event.day.side,
                    "{criterion}"
                );
            }
            let first = heliacal.next(Body::Mercury, from, 240.0).unwrap().unwrap();
            assert_eq!(first, events[0]);
            assert!(heliacal.next(Body::Mercury, from, 0.5).unwrap().is_none());
        }
    }

    #[test]
    fn the_refusals_name_their_reasons() {
        let provider = TestProvider::new();
        let sky = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let heliacal = Heliacal::new(
            &sky,
            kathmandu(),
            Criterion::SURYA_SIDDHANTA,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let day = JulianDay::literal(2_460_310.5);
        assert_eq!(
            heliacal.state(Body::Sun, day).unwrap_err().field(),
            Some("body")
        );
        assert!(
            heliacal
                .state(Body::MeanNode, day)
                .unwrap_err()
                .to_string()
                .contains("point")
        );
        let refused = heliacal.state(Body::Uranus, day).unwrap_err();
        assert!(
            refused.to_string().contains("SURYA_SIDDHANTA")
                && refused.to_string().contains("URANUS")
        );
        assert!(
            heliacal
                .next(Body::Mercury, day, 0.0)
                .unwrap_err()
                .to_string()
                .contains("window")
        );
        assert!(heliacal.events(Body::Mercury, day, day).is_err());
        // Midsummer at 80° north: no sunset, so no dusk to read at.
        let svalbard = Place::new(
            Latitude::literal(80.0),
            Longitude::literal(15.0),
            Altitude::literal(0.0),
        );
        let polar = Heliacal::new(
            &sky,
            svalbard,
            Criterion::SURYA_SIDDHANTA,
            Horizon::CENTRE_NO_REFRACTION,
            DeltaTModel::TableThenModel,
        );
        let midsummer = JulianDay::literal(2_460_482.5);
        let error = polar.state(Body::Mercury, midsummer).unwrap_err();
        assert!(error.to_string().contains("dawn or dusk"), "{error}");
        assert!(heliacal.describe().contains("TIME_DEGREES/SURYA_SIDDHANTA"));
        assert_eq!(
            heliacal.day_start(day.plus_days(0.3).unwrap()),
            local_mean_midnight(day, Longitude::literal(85.324))
        );
    }
}
