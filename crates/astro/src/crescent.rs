//! The new crescent on an evening: whether it can be seen after the
//! sunset at a place, by Odeh's criterion or Yallop's q-test
//! (`docs/03-design/hijri-sighted.md` §2).
//!
//! Both criteria read the sky at one instant, the **best time**, which
//! Yallop takes from Bruin's curves as the sunset plus four ninths of the
//! Moon's lag, the interval from the sunset to the moonset (NAO Technical
//! Note 69, 1997, eq. 4.1, p. 4); Odeh adopts it (*Experimental
//! Astronomy* 18, 2004, eq. 1, p. 43). At that instant:
//!
//! - **ARCV**, the arc of vision, is the Moon's altitude less the Sun's,
//!   their centres, with no refraction: geocentric for Yallop (p. 1),
//!   topocentric for Odeh (p. 61);
//! - **ARCL**, the arc of light, is the angle between their centres, and
//!   **DAZ** the Sun's azimuth less the Moon's;
//! - **W**, the crescent's width in arcminutes, is the Moon's
//!   semi-diameter times `1 − cos ARCL`: Yallop's `W′` from the
//!   topocentric semi-diameter and the geocentric ARCL (eqs. 3.8 to 3.10,
//!   p. 3), Odeh's from the topocentric semi-diameter and ARCL.
//!
//! Odeh's value is `V = ARCV − (−0.1018 W³ + 0.7319 W² − 6.3226 W +
//! 7.1651)`, read in zones A (V ≥ 5.65) to D (V < −0.96) (eq. 2, p. 61);
//! Yallop's is `q = (ARCV − (11.8371 − 6.3226 W′ + 0.7319 W′² −
//! 0.1018 W′³)) / 10`, in zones A (q > +0.216) to F (q ≤ −0.293)
//! (eq. 6.1 and Table 5, pp. 11 and 12). A Moon that sets before the Sun,
//! or that has not yet passed the Sun in longitude, is not looked for.
//!
//! The sunset and the moonset are the upper limbs' under the standard
//! refraction ([`Horizon::UPPER_LIMB_REFRACTION`]), as both authors
//! reckon the lag.
//!
//! ```
//! use teistro_astro::crescent::{Crescent, Sighting};
//! use teistro_astro::{Completion, DeltaTModel};
//! use teistro_core::quantity::{JulianDay, Place, Ut1};
//! use teistro_core::settings::{CrescentCriterion, CrescentZone, OverridePolicy};
//! use teistro_port_ephemeris::TestProvider;
//!
//! let provider = TestProvider::new();
//! let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
//! let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
//! let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
//! let sighting = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::B)?;
//! // An evening's reading, from the local noon it follows.
//! let evening = crescent.evening(JulianDay::<Ut1>::literal(2_460_310.0))?;
//! assert!(evening.lag_minutes.is_finite());
//! // Seen or not, by the criterion and the zone asked.
//! let _ = sighting.seen(&evening);
//! # Ok::<(), teistro_core::error::Error>(())
//! ```

use core::fmt;

use serde::{Deserialize, Serialize};
use teistro_core::angle::difference_deg;
use teistro_core::error::Error;
use teistro_core::math;
use teistro_core::quantity::{JulianDay, Place, Ut1};
use teistro_core::settings::{CrescentCriterion, CrescentZone};
use teistro_port_ephemeris::{Body, Horizon, HorizonEventKind};

use crate::day_sky::horizontal;
use crate::delta_t::DeltaTModel;
use crate::iau::vector::{Vector3, pdp, pm, pxp};
use crate::iau::{DEG2RAD, RAD2DEG};
use crate::rise_set::{AU_KM, EARTH_EQUATORIAL_RADIUS_KM, Solver};
use crate::scale::tt_of;
use crate::sky::{self, Apparent, ApparentPositions, Spherical};

/// The Moon's radius in Earth radii, from which Yallop takes its
/// semi-diameter (`SD = 0.27245 π`, eq. 3.8, p. 3).
pub const MOON_EARTH_RATIO: f64 = 0.272_45;

/// The share of the lag the best time falls at: four ninths (Yallop,
/// eq. 4.1, p. 4).
pub const BEST_TIME_SHARE: f64 = 4.0 / 9.0;

/// What one evening's sky gives the criteria at the best time.
///
/// ```
/// # use teistro_astro::crescent::Crescent;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// # let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
/// let evening = crescent.evening(JulianDay::<Ut1>::literal(2_460_310.0))?;
/// // The best time lies four ninths into the lag when the Moon sets last.
/// if let Some(geometry) = evening.geometry {
///     let share = (geometry.best.get() - evening.sunset.get()) * 1440.0 / evening.lag_minutes;
///     assert!((share - 4.0 / 9.0).abs() < 1e-6);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Evening {
    /// The sunset, UT1.
    pub sunset: JulianDay<Ut1>,
    /// The moonset nearest after half a day before the sunset, UT1:
    /// before the sunset when the Moon sets first.
    pub moonset: Option<JulianDay<Ut1>>,
    /// The moonset less the sunset, minutes; negative when the Moon sets
    /// first, and negative infinity when it does not set near the
    /// evening at all.
    pub lag_minutes: f64,
    /// The sky at the best time, when the Moon sets after the Sun and has
    /// passed it in longitude; `None` when there is no crescent to look
    /// for.
    pub geometry: Option<Geometry>,
}

/// The crescent's geometry at the best time: every quantity either
/// criterion reads.
///
/// ```
/// # use teistro_astro::crescent::Crescent;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// # let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
/// let at = crescent.geometry(JulianDay::<Ut1>::literal(2_460_310.6))?;
/// if let Some(at) = at {
///     // Yallop's width is never more than the Moon's whole diameter.
///     assert!(at.width_yallop_arcmin >= 0.0 && at.width_yallop_arcmin < 35.0);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Geometry {
    /// The best time, UT1.
    pub best: JulianDay<Ut1>,
    /// The Moon's centre's altitude less the Sun's, airless, from the
    /// Earth's centre, degrees (Yallop's ARCV).
    pub arcv_geocentric_deg: f64,
    /// The same from the place, degrees (Odeh's ARCV).
    pub arcv_topocentric_deg: f64,
    /// The angle between the centres from the Earth's centre, degrees.
    pub arcl_geocentric_deg: f64,
    /// The same from the place, degrees.
    pub arcl_topocentric_deg: f64,
    /// The Sun's azimuth less the Moon's, from the place, degrees.
    pub daz_deg: f64,
    /// Yallop's topocentric width `W′`, arcminutes.
    pub width_yallop_arcmin: f64,
    /// Odeh's topocentric width `W`, arcminutes.
    pub width_odeh_arcmin: f64,
    /// Odeh's value `V`.
    pub odeh_v: f64,
    /// Yallop's value `q`.
    pub yallop_q: f64,
}

impl Geometry {
    /// The zone a criterion puts the crescent in.
    ///
    /// ```
    /// use teistro_astro::crescent::{Geometry, Zone};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::CrescentCriterion;
    ///
    /// let mut at = Geometry {
    ///     best: JulianDay::literal(2_460_310.5),
    ///     arcv_geocentric_deg: 0.0, arcv_topocentric_deg: 0.0,
    ///     arcl_geocentric_deg: 0.0, arcl_topocentric_deg: 0.0, daz_deg: 0.0,
    ///     width_yallop_arcmin: 0.0, width_odeh_arcmin: 0.0,
    ///     odeh_v: 5.65, yallop_q: 0.216,
    /// };
    /// // Odeh's A opens at 5.65; Yallop's needs more than +0.216.
    /// assert_eq!(at.zone(CrescentCriterion::Odeh2006), Zone::A);
    /// assert_eq!(at.zone(CrescentCriterion::Yallop1997), Zone::B);
    /// at.yallop_q = -0.3;
    /// assert_eq!(at.zone(CrescentCriterion::Yallop1997), Zone::F);
    /// ```
    #[must_use]
    pub fn zone(&self, criterion: CrescentCriterion) -> Zone {
        match criterion {
            CrescentCriterion::Yallop1997 => yallop_zone(self.yallop_q),
            _ => odeh_zone(self.odeh_v),
        }
    }
}

/// A zone of either criterion, best first: Odeh names A to D, Yallop
/// A to F.
///
/// ```
/// use teistro_astro::crescent::Zone;
///
/// assert!(Zone::A < Zone::D);
/// assert_eq!(Zone::C.key(), "C");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub enum Zone {
    /// The best.
    A,
    /// The second.
    B,
    /// The third.
    C,
    /// The fourth: Odeh's last, not visible even by optical aid.
    D,
    /// Yallop's "not visible with a telescope".
    E,
    /// Yallop's "not visible, below Danjon limit".
    F,
}

impl Zone {
    /// The zone's letter.
    ///
    /// ```
    /// use teistro_astro::crescent::Zone;
    ///
    /// assert_eq!(Zone::F.key(), "F");
    /// ```
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Zone::A => "A",
            Zone::B => "B",
            Zone::C => "C",
            Zone::D => "D",
            Zone::E => "E",
            Zone::F => "F",
        }
    }

    /// The zone a knob names.
    ///
    /// ```
    /// use teistro_astro::crescent::Zone;
    /// use teistro_core::settings::CrescentZone;
    ///
    /// assert_eq!(Zone::of(CrescentZone::C), Zone::C);
    /// ```
    #[must_use]
    pub fn of(knob: CrescentZone) -> Zone {
        match knob {
            CrescentZone::A => Zone::A,
            CrescentZone::C => Zone::C,
            CrescentZone::D => Zone::D,
            _ => Zone::B,
        }
    }
}

impl fmt::Display for Zone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key())
    }
}

/// Odeh's zone of a value `V` (eq. 2, p. 61).
///
/// ```
/// use teistro_astro::crescent::{Zone, odeh_zone};
///
/// assert_eq!(odeh_zone(5.65), Zone::A);
/// assert_eq!(odeh_zone(2.0), Zone::B);
/// assert_eq!(odeh_zone(-0.96), Zone::C);
/// assert_eq!(odeh_zone(-0.97), Zone::D);
/// ```
#[must_use]
pub fn odeh_zone(v: f64) -> Zone {
    if v >= 5.65 {
        Zone::A
    } else if v >= 2.0 {
        Zone::B
    } else if v >= -0.96 {
        Zone::C
    } else {
        Zone::D
    }
}

/// Yallop's zone of a value `q` (Table 5, p. 12).
///
/// ```
/// use teistro_astro::crescent::{Zone, yallop_zone};
///
/// assert_eq!(yallop_zone(0.217), Zone::A);
/// assert_eq!(yallop_zone(0.216), Zone::B);
/// assert_eq!(yallop_zone(-0.014), Zone::C);
/// assert_eq!(yallop_zone(-0.160), Zone::D);
/// assert_eq!(yallop_zone(-0.232), Zone::E);
/// assert_eq!(yallop_zone(-0.293), Zone::F);
/// ```
#[must_use]
pub fn yallop_zone(q: f64) -> Zone {
    if q > 0.216 {
        Zone::A
    } else if q > -0.014 {
        Zone::B
    } else if q > -0.160 {
        Zone::C
    } else if q > -0.232 {
        Zone::D
    } else if q > -0.293 {
        Zone::E
    } else {
        Zone::F
    }
}

/// Odeh's value `V` of a topocentric arc of vision, degrees, and width,
/// arcminutes (eq. 2, p. 61).
///
/// ```
/// use teistro_astro::crescent::odeh_v;
///
/// // A crescent of no width needs 7.1651° of arc of vision for V = 0.
/// assert!(odeh_v(7.1651, 0.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn odeh_v(arcv_deg: f64, width_arcmin: f64) -> f64 {
    let w = width_arcmin;
    arcv_deg - (-0.1018 * w * w * w + 0.7319 * w * w - 6.3226 * w + 7.1651)
}

/// Yallop's value `q` of a geocentric arc of vision, degrees, and
/// topocentric width, arcminutes (eq. 6.1, p. 11).
///
/// ```
/// use teistro_astro::crescent::yallop_q;
///
/// assert!(yallop_q(11.8371, 0.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn yallop_q(arcv_deg: f64, width_arcmin: f64) -> f64 {
    let w = width_arcmin;
    (arcv_deg - (11.8371 - 6.3226 * w + 0.7319 * w * w - 0.1018 * w * w * w)) / 10.0
}

/// A criterion and the weakest zone it counts a crescent seen in.
///
/// ```
/// use teistro_astro::crescent::{Sighting, Zone};
/// use teistro_core::settings::{CrescentCriterion, CrescentZone};
///
/// let odeh = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::C)?;
/// assert_eq!(odeh.weakest(), Zone::C);
/// // Odeh's zone D is a crescent not seen even by optical aid.
/// let refused = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::D).unwrap_err();
/// assert_eq!(refused.field(), Some("calendars.hijri_zone"));
/// assert!(Sighting::new(CrescentCriterion::Yallop1997, CrescentZone::D).is_ok());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Sighting {
    criterion: CrescentCriterion,
    weakest: Zone,
}

impl Sighting {
    /// A sighting by `criterion` down to `zone`.
    ///
    /// ```
    /// use teistro_astro::crescent::{Sighting, Zone};
    /// use teistro_core::settings::{CrescentCriterion, CrescentZone};
    ///
    /// let yallop = Sighting::new(CrescentCriterion::Yallop1997, CrescentZone::D)?;
    /// assert_eq!(yallop.weakest(), Zone::D);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `INVALID_ARG` on `calendars.hijri_zone` for a zone the criterion
    /// calls unseen: Odeh's D.
    pub fn new(criterion: CrescentCriterion, zone: CrescentZone) -> Result<Sighting, Error> {
        let weakest = Zone::of(zone);
        if criterion == CrescentCriterion::Odeh2006 && weakest > Zone::C {
            return Err(Error::invalid_arg(format!(
                "Odeh's zone {weakest} is a crescent not visible even by optical aid, so it begins no month"
            ))
            .with_field("calendars.hijri_zone")
            .with_hint("name A, B or C under ODEH_2006, or YALLOP_1997 for its zone D"));
        }
        Ok(Sighting { criterion, weakest })
    }

    /// The criterion.
    ///
    /// ```
    /// use teistro_astro::crescent::Sighting;
    /// use teistro_core::settings::{CrescentCriterion, CrescentZone};
    ///
    /// let yallop = Sighting::new(CrescentCriterion::Yallop1997, CrescentZone::A)?;
    /// assert_eq!(yallop.criterion(), CrescentCriterion::Yallop1997);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn criterion(&self) -> CrescentCriterion {
        self.criterion
    }

    /// The weakest zone a crescent counts as seen in.
    ///
    /// ```
    /// use teistro_astro::crescent::{Sighting, Zone};
    /// use teistro_core::settings::{CrescentCriterion, CrescentZone};
    ///
    /// let yallop = Sighting::new(CrescentCriterion::Yallop1997, CrescentZone::A)?;
    /// assert_eq!(yallop.weakest(), Zone::A);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub const fn weakest(&self) -> Zone {
        self.weakest
    }

    /// Whether the crescent is seen on an evening: the Moon sets after
    /// the Sun, has passed it, and stands in the weakest zone or a better.
    ///
    /// ```
    /// use teistro_astro::crescent::{Evening, Sighting};
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_core::settings::{CrescentCriterion, CrescentZone};
    ///
    /// let sighting = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::B)?;
    /// // A Moon that set first leaves nothing to see.
    /// let first = Evening { sunset: JulianDay::literal(2_460_310.5), moonset: None, lag_minutes: -20.0, geometry: None };
    /// assert!(!sighting.seen(&first));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn seen(&self, evening: &Evening) -> bool {
        evening
            .geometry
            .is_some_and(|geometry| geometry.zone(self.criterion) <= self.weakest)
    }

    /// The stamp: `ODEH_2006 to zone B`.
    ///
    /// ```
    /// use teistro_astro::crescent::Sighting;
    /// use teistro_core::settings::{CrescentCriterion, CrescentZone};
    ///
    /// let odeh = Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::B)?;
    /// assert_eq!(odeh.to_string(), "ODEH_2006 to zone B");
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn describe(&self) -> String {
        format!("{} to zone {}", self.criterion.key(), self.weakest)
    }
}

impl fmt::Display for Sighting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.describe())
    }
}

/// The crescent reckoner: the sky, the place and the Delta T model.
///
/// ```
/// # use teistro_astro::crescent::Crescent;
/// # use teistro_astro::{Completion, DeltaTModel};
/// # use teistro_core::quantity::{JulianDay, Place, Ut1};
/// # use teistro_core::settings::OverridePolicy;
/// # use teistro_port_ephemeris::TestProvider;
/// #
/// # let provider = TestProvider::new();
/// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
/// # let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
/// // Each evening is read from the local noon before it.
/// let evening = crescent.evening(JulianDay::<Ut1>::literal(2_460_310.25))?;
/// assert!(evening.sunset.get() > 2_460_310.25);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
pub struct Crescent<'a> {
    sky: &'a dyn ApparentPositions,
    place: Place,
    delta_t: DeltaTModel,
}

impl fmt::Debug for Crescent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "crescent at {} from {}", self.place, self.sky.describe())
    }
}

impl<'a> Crescent<'a> {
    /// A reckoner over `sky` at `place`.
    ///
    /// ```
    /// # use teistro_astro::crescent::Crescent;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::Place;
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    /// let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
    /// assert!(format!("{crescent:?}").starts_with("crescent at "));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn new(sky: &'a dyn ApparentPositions, place: Place, delta_t: DeltaTModel) -> Crescent<'a> {
        Crescent {
            sky,
            place,
            delta_t,
        }
    }

    /// The evening that follows `noon`: the first sunset after it, the
    /// moonset that goes with it and, when the Moon sets after the Sun and
    /// has passed it, the geometry at the best time.
    ///
    /// ```
    /// # use teistro_astro::crescent::Crescent;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    /// # let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
    /// let evening = crescent.evening(JulianDay::<Ut1>::literal(2_460_310.25))?;
    /// // The sunset falls within a day of the noon it follows.
    /// assert!(evening.sunset.get() - 2_460_310.25 < 1.0);
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// `LIMIT` when the Sun does not set within a day of `noon`, so there
    /// is no evening; the source's refusal.
    pub fn evening(&self, noon: JulianDay<Ut1>) -> Result<Evening, Error> {
        let horizon = Horizon::UPPER_LIMB_REFRACTION;
        let sun = Solver::new(self.sky, Body::Sun, self.place, horizon, self.delta_t);
        let Some(sunset) = sun.event(HorizonEventKind::Set, noon, 1.0)? else {
            return Err(Error::limit(format!(
                "the Sun does not set within a day of JD {} at latitude {}, so there is no evening to look for the crescent in",
                noon.get(),
                self.place.latitude
            ))
            .with_hint("a sighted month is reckoned where the Sun sets every day"));
        };
        let sunset = sunset.instant;
        let moon = Solver::new(self.sky, Body::Moon, self.place, horizon, self.delta_t);
        let moonset = moon
            .event(HorizonEventKind::Set, sunset.plus_days(-0.5)?, 1.0)?
            .map(|event| event.instant);
        let Some(set) = moonset else {
            return Ok(Evening {
                sunset,
                moonset,
                lag_minutes: f64::NEG_INFINITY,
                geometry: None,
            });
        };
        let lag_days = set.get() - sunset.get();
        let lag_minutes = lag_days * 1440.0;
        let geometry = if lag_days > 0.0 {
            self.geometry(sunset.plus_days(BEST_TIME_SHARE * lag_days)?)?
        } else {
            None
        };
        Ok(Evening {
            sunset,
            moonset,
            lag_minutes,
            geometry,
        })
    }

    /// The geometry at `best`, or `None` when the Moon has not yet passed
    /// the Sun in longitude.
    ///
    /// ```
    /// # use teistro_astro::crescent::Crescent;
    /// # use teistro_astro::{Completion, DeltaTModel};
    /// # use teistro_core::quantity::{JulianDay, Place, Ut1};
    /// # use teistro_core::settings::OverridePolicy;
    /// # use teistro_port_ephemeris::TestProvider;
    /// #
    /// # let provider = TestProvider::new();
    /// # let sky = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let kathmandu = Place::try_from_degrees(27.7172, 85.324, 1_400.0)?;
    /// # let crescent = Crescent::new(&sky, kathmandu, DeltaTModel::TableThenModel);
    /// if let Some(at) = crescent.geometry(JulianDay::<Ut1>::literal(2_460_310.6))? {
    ///     // The arc of light is an angle between two centres.
    ///     assert!((0.0..=180.0).contains(&at.arcl_topocentric_deg));
    /// }
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The source's refusal, or a Delta T model that cannot answer.
    pub fn geometry(&self, best: JulianDay<Ut1>) -> Result<Option<Geometry>, Error> {
        let (tt, _) = tt_of(best, self.delta_t)?;
        let sun = self.sky.apparent(Body::Sun, best)?;
        let moon = self.sky.apparent(Body::Moon, best)?;
        let obliquity = sky::obliquity(tt).mean_deg;
        let longitude = |at: &Apparent| {
            sky::equatorial_to_ecliptic(
                Spherical {
                    lon_deg: at.ra_deg,
                    lat_deg: at.dec_deg,
                },
                obliquity,
            )
            .lon_deg
        };
        if difference_deg(longitude(&moon), longitude(&sun)) <= 0.0 {
            return Ok(None);
        }
        let station = sky::observer(self.place, best, tt).position_au;
        let local_sidereal = sky::sidereal_time_deg(best, tt, self.place.longitude);
        let latitude = self.place.latitude.get();
        let altaz =
            |ra_deg: f64, dec_deg: f64| horizontal(local_sidereal - ra_deg, dec_deg, latitude);
        let (sun_geo, moon_geo) = (vector(&sun), vector(&moon));
        let (sun_topo, moon_topo) = (sub(&sun_geo, &station), sub(&moon_geo, &station));
        let (sun_ra, sun_dec) = direction(&sun_topo);
        let (moon_ra, moon_dec) = direction(&moon_topo);
        let (sun_alt_geo, _) = altaz(sun.ra_deg, sun.dec_deg);
        let (moon_alt_geo, _) = altaz(moon.ra_deg, moon.dec_deg);
        let (sun_alt, sun_az) = altaz(sun_ra, sun_dec);
        let (moon_alt, moon_az) = altaz(moon_ra, moon_dec);
        let light_geo = angle(&sun_geo, &moon_geo);
        let light_topo = angle(&sun_topo, &moon_topo);
        // Yallop's semi-diameters (eqs. 3.8 and 3.9, p. 3), in
        // arcminutes: from the horizontal parallax, then raised by the
        // Moon's nearness to the station.
        let parallax = math::asin(EARTH_EQUATORIAL_RADIUS_KM / (moon.distance_au * AU_KM));
        let semi = MOON_EARTH_RATIO * parallax;
        let semi_topo = semi * (1.0 + math::sin(moon_alt_geo * DEG2RAD) * math::sin(parallax));
        let width_yallop = semi_topo * RAD2DEG * 60.0 * (1.0 - math::cos(light_geo));
        // Odeh's: the topocentric semi-diameter from the station's own
        // distance to the Moon, the topocentric arc of light.
        let semi_odeh =
            math::asin(MOON_EARTH_RATIO * EARTH_EQUATORIAL_RADIUS_KM / (pm(&moon_topo) * AU_KM));
        let width_odeh = semi_odeh * RAD2DEG * 60.0 * (1.0 - math::cos(light_topo));
        let vision_geo = moon_alt_geo - sun_alt_geo;
        let vision_topo = moon_alt - sun_alt;
        Ok(Some(Geometry {
            best,
            arcv_geocentric_deg: vision_geo,
            arcv_topocentric_deg: vision_topo,
            arcl_geocentric_deg: light_geo * RAD2DEG,
            arcl_topocentric_deg: light_topo * RAD2DEG,
            daz_deg: difference_deg(sun_az, moon_az),
            width_yallop_arcmin: width_yallop,
            width_odeh_arcmin: width_odeh,
            odeh_v: odeh_v(vision_topo, width_odeh),
            yallop_q: yallop_q(vision_geo, width_yallop),
        }))
    }
}

/// A position's geocentric vector, astronomical units, in the true
/// equator and equinox of date.
fn vector(at: &Apparent) -> Vector3 {
    let (sin_ra, cos_ra) = math::sin_cos(at.ra_deg * DEG2RAD);
    let (sin_dec, cos_dec) = math::sin_cos(at.dec_deg * DEG2RAD);
    [
        at.distance_au * cos_dec * cos_ra,
        at.distance_au * cos_dec * sin_ra,
        at.distance_au * sin_dec,
    ]
}

fn sub(a: &Vector3, b: &Vector3) -> Vector3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// A vector's right ascension and declination, degrees.
fn direction(p: &Vector3) -> (f64, f64) {
    let ra = math::atan2(p[1], p[0]) * RAD2DEG;
    let dec = math::atan2(p[2], math::hypot(p[0], p[1])) * RAD2DEG;
    (ra.rem_euclid(360.0), dec)
}

/// The angle between two vectors, radians.
fn angle(a: &Vector3, b: &Vector3) -> f64 {
    math::atan2(pm(&pxp(a, b)), pdp(a, b))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    /// Odeh's table V (p. 43) is his equation's zone edges: at each width
    /// the tabled ARCV1, ARCV2 and ARCV3 give V at -0.96, 2 and 5.65, to
    /// the table's tenth of a degree.
    #[test]
    fn odehs_equation_reads_his_table() {
        let table = [
            (0.1, [5.6, 8.5, 12.2]),
            (0.3, [4.4, 7.3, 11.0]),
            (0.5, [3.2, 6.2, 9.8]),
            (0.7, [2.1, 5.1, 8.7]),
            (0.9, [1.0, 4.0, 7.6]),
        ];
        for (width, arcvs) in table {
            for (arcv, edge) in arcvs.into_iter().zip([-0.96, 2.0, 5.65]) {
                let v = odeh_v(arcv, width);
                assert!((v - edge).abs() < 0.11, "W {width}, ARCV {arcv}: V {v}");
            }
        }
    }

    /// Yallop's zone A edge: ARCL 12°, DAZ 0, so ARCV 12°, with Bruin's
    /// width of a 15′ Moon, gives q = +0.216 (p. 12).
    #[test]
    fn yallops_zone_a_edge_is_his_ideal_crescent() {
        let width = 15.0 * (1.0 - math::cos(12.0 * DEG2RAD));
        let q = yallop_q(12.0, width);
        assert!((q - 0.216).abs() < 0.002, "{q}");
    }

    #[test]
    fn a_zone_past_the_criterions_seen_ones_is_refused() {
        assert!(Sighting::new(CrescentCriterion::Odeh2006, CrescentZone::D).is_err());
        for zone in [CrescentZone::A, CrescentZone::B, CrescentZone::C] {
            assert!(Sighting::new(CrescentCriterion::Odeh2006, zone).is_ok());
        }
        for zone in [
            CrescentZone::A,
            CrescentZone::B,
            CrescentZone::C,
            CrescentZone::D,
        ] {
            let yallop = Sighting::new(CrescentCriterion::Yallop1997, zone);
            assert!(yallop.is_ok_and(|s| s.weakest() == Zone::of(zone)));
        }
    }
}
