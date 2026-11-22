//! The four moving limbs of a day, from three crossing searches.
//!
//! A tithi is twelve degrees of the Moon's elongation from the Sun, a
//! karana six, a nakshatra a twenty-seventh of the Moon's own circle and
//! a yoga a twenty-seventh of the Moon's plus the Sun's. `astro::events`
//! already searches a lattice of boundaries over a window, so a limb list
//! is that search, cut into spans, each classified by the integer path
//! (ADR-0016).
//!
//! Three things make it three searches and not four:
//!
//! 1. **A karana is half a tithi**, so the six-degree lattice's crossings
//!    contain the twelve-degree lattice's: one search answers both.
//! 2. **The window is widened** by the longest a member can run, so the
//!    first and last spans carry their own bounds and not the window's,
//!    which is what the corpus loses (`03-design/panchanga-day-conventions.md`
//!    §2).
//! 3. **One source serves all four.** The provider is asked for tropical
//!    longitudes and this module shifts them by the ayanamsha *at each
//!    instant*, so the elongation (where the shift cancels), the Moon's
//!    longitude and the Moon-plus-Sun sum are all read from one place. A
//!    fixed shift would be up to two seconds of time out over a search
//!    window, which is wider than the tolerance the corpus declares.
//!
//! None of the four quantities can go backwards — the slowest elongation
//! is about eleven degrees a day — so the spans are in order and the
//! kernel says so rather than hoping.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use teistro_astro::events::{
    Longitudes, Search, quantity_least_rate, quantity_rate_deg_per_day, value_of,
};
use teistro_calendar::lunisolar::Beyond;
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Karana, Masa, Nakshatra, Rashi, Tithi, Yoga};
use teistro_core::coverage::{Clipped, Coverage};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_port_ephemeris::{Body, Centre, Frame, Lattice, Quantity};

use crate::span::Span;

/// The longest a member of any limb can run, days.
///
/// The slowest tithi is about twenty-six hours (twelve degrees at the
/// elongation's slowest, near eleven degrees a day) and the slowest
/// nakshatra transit about twenty-eight (a twenty-seventh of the circle
/// at the Moon's slowest). A day and a half covers both with room, and it
/// is what the search window is widened by so that the first and last
/// spans carry their own bounds.
pub const LONGEST_SPAN_DAYS: f64 = 1.5;

/// The four moving limbs of one day.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::limbs;
///
/// // Each limb's spans run end to end through the day.
/// let day = limbs(&longitudes, window, zodiac)?;
/// for pair in day.tithi.windows(2) {
///     assert_eq!(pair[0].whole.to, pair[1].whole.from);
/// }
/// assert!(day.tithi[0].whole.contains(window.from));
/// assert!(day.karana.len() >= day.tithi.len());
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct Limbs {
    /// The tithis that touch the day.
    pub tithi: Vec<Span<Tithi>>,
    /// The nakshatras the Moon was in.
    pub nakshatra: Vec<Span<Nakshatra>>,
    /// The nitya yogas.
    pub yoga: Vec<Span<Yoga>>,
    /// The karanas; half-tithis, so there are three or four of them.
    pub karana: Vec<Span<Karana>>,
}

impl Limbs {
    /// The limbs with each span's margin from the day's two sunrises set
    /// ([`Span::with_margin`]; SR-95): what a day answers.
    ///
    /// ```
    /// # use teistro_core::catalogue::Tithi;
    /// # use teistro_core::interval::Interval;
    /// use teistro_panchanga::limb::Limbs;
    /// use teistro_panchanga::span::Span;
    ///
    /// let day = Interval::literal(100.25, 101.25);
    /// let tithi = Span::new(Tithi::ShuklaTritiya, Interval::literal(99.9, 100.6), day).ok_or("it meets the day")?;
    /// let limbs = Limbs { tithi: vec![tithi], nakshatra: vec![], yoga: vec![], karana: vec![] };
    /// let marked = limbs.with_margins(day.from, day.to);
    /// assert!(marked.tithi.iter().all(|span| span.margin_minutes.is_some()));
    /// # Ok::<(), &str>(())
    /// ```
    #[must_use]
    pub fn with_margins(self, sunrise: JulianDay<Utc>, next_sunrise: JulianDay<Utc>) -> Self {
        Limbs {
            tithi: with_margins(self.tithi, sunrise, next_sunrise),
            nakshatra: with_margins(self.nakshatra, sunrise, next_sunrise),
            yoga: with_margins(self.yoga, sunrise, next_sunrise),
            karana: with_margins(self.karana, sunrise, next_sunrise),
        }
    }
}

/// A limb's spans with their margins from a day's two sunrises set.
///
/// ```
/// # use teistro_core::interval::Interval;
/// use teistro_panchanga::limb::with_margins;
/// use teistro_panchanga::span::Span;
///
/// let day = Interval::literal(100.25, 101.25);
/// let spans = [Interval::literal(99.9, 100.6), Interval::literal(100.6, 101.3)]
///     .into_iter()
///     .filter_map(|whole| Span::new("Tithi", whole, day))
///     .collect();
/// let marked = with_margins(spans, day.from, day.to);
/// assert!(marked.iter().all(|span| span.margin_minutes.is_some()));
/// ```
#[must_use]
pub fn with_margins<T>(
    spans: Vec<Span<T>>,
    sunrise: JulianDay<Utc>,
    next_sunrise: JulianDay<Utc>,
) -> Vec<Span<T>> {
    spans
        .into_iter()
        .map(|span| span.with_margin(sunrise, next_sunrise))
        .collect()
}

/// Which limb a list belongs to: the lattice it is found on and how a
/// span between two boundaries is named.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Limb {
    Tithi,
    Nakshatra,
    Yoga,
    Karana,
}

impl Limb {
    /// The quantity whose crossings bound this limb's members.
    const fn quantity(self) -> Quantity {
        match self {
            Limb::Tithi | Limb::Karana => Quantity::ELONGATION,
            Limb::Nakshatra => Quantity::Longitude(Body::Moon),
            Limb::Yoga => Quantity::MOON_PLUS_SUN,
        }
    }

    /// How many members the quantity's circle holds.
    const fn divisions(self) -> u32 {
        match self {
            Limb::Tithi => 30,
            Limb::Nakshatra | Limb::Yoga => 27,
            // Sixty half-tithis make a lunar month; which karana each is
            // comes from `karana_of`.
            Limb::Karana => 60,
        }
    }

    /// The lattice its boundaries lie on.
    fn lattice(self) -> Lattice {
        match self {
            Limb::Tithi => Lattice::TITHIS,
            Limb::Nakshatra => Lattice::NAKSHATRAS,
            Limb::Yoga => Lattice::YOGAS,
            Limb::Karana => Lattice::KARANAS,
        }
    }
}

/// The karana that a half-tithi of the lunar month is.
///
/// Sixty of them make a month and they are not a cycle of eleven:
/// Kimstughna opens the month, Shakuni, Chatushpada and Naga close it,
/// and the seven movable karanas repeat through everything between. The
/// corpus's own successor relation holds over all 109 consecutive pairs
/// it records (`03-design/panchanga-day-conventions.md` §2).
///
/// ```
/// use teistro_core::catalogue::Karana;
/// use teistro_panchanga::limb::karana_of;
///
/// assert_eq!(karana_of(0), Karana::Kimstughna);
/// // Bava again after the seven.
/// assert_eq!((karana_of(1), karana_of(8)), (Karana::Bava, Karana::Bava));
/// assert_eq!(karana_of(59), Karana::Naga);
/// ```
#[must_use]
pub fn karana_of(half_tithi: u32) -> Karana {
    match half_tithi % 60 {
        0 => Karana::Kimstughna,
        57 => Karana::Shakuni,
        58 => Karana::Chatushpada,
        59 => Karana::Naga,
        // The seven movable karanas, Bava first, from the second half of
        // the first tithi.
        other => {
            Karana::from_id(u16::try_from((other - 1) % 7).unwrap_or(0)).unwrap_or(Karana::Bava)
        }
    }
}

// The sidereal source and the zodiac it reads in live in `teistro-astro`,
// where the annual chart's solar return needs the same shift for the same
// reason (`03-design/annual-chart-measured.md`). Re-exported here because
// this module's own signatures name them.
pub use teistro_astro::sidereal::{Sidereal, Zodiac};

/// The nakshatra the Moon stands in at an instant, with the whole of its
/// stay there: what a temporal dasha balance reads.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{limbs, nakshatra_at};
///
/// // The day's first nakshatra, whole, holds the Moon at its start.
/// let stay = nakshatra_at(&longitudes, window.from, zodiac)?;
/// assert!(stay.whole.contains_inclusive(window.from));
/// assert_eq!(stay.member, limbs(&longitudes, window, zodiac)?.nakshatra[0].member);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`limbs`], and `INTERNAL` should the search find no nakshatra at the
/// instant, which a Moon that never stops cannot do.
pub fn nakshatra_at<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
) -> Result<Span<Nakshatra>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let window = Interval::literal(at.get(), at.get());
    let found = instants(boundaries(&source, Limb::Nakshatra, window)?);
    spans(&source, Limb::Nakshatra, window, &found, None)?
        .into_iter()
        .find(|span| span.whole.contains_inclusive(at))
        .ok_or_else(|| {
            Error::new(
                Status::Internal,
                format!("no nakshatra holds the Moon at {at}"),
            )
        })
}

/// The nakshatra the Sun stands in at an instant, on the integer path the
/// limbs read (ADR-0016): what a festival kept on the Sun's entry into a
/// nakshatra compares at the two midnights of a civil day.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::sun_nakshatra_at;
///
/// // A degree a day: the same nakshatra a day on, or the next.
/// let today = sun_nakshatra_at(&longitudes, window.from, zodiac)?.id();
/// let tomorrow = sun_nakshatra_at(&longitudes, window.to, zodiac)?.id();
/// assert!(tomorrow == today || tomorrow == (today + 1) % 27);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal.
pub fn sun_nakshatra_at<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
) -> Result<Nakshatra, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let degrees = teistro_astro::events::value_of(
        Quantity::Longitude(Body::Sun),
        &source,
        JulianDay::<Ut1>::literal(at.get()),
    )?;
    Ok(Nakshatra::of(
        Nas::try_from_degrees(degrees)?.division_index(Limb::Nakshatra.divisions()),
    ))
}

/// [`sun_nakshatra_at`] at an instant the ephemeris's coverage may not
/// hold (SR-136, C2247): read at the nearer edge when the Sun at his
/// greatest rate cannot have reached his nakshatra's boundary between
/// the two, as a civil day's midnight a few hours before the first
/// instant answered is.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// use teistro_panchanga::limb::{sun_nakshatra_at, sun_nakshatra_near};
///
/// // Inside the coverage, the reading itself.
/// let at = JulianDay::literal(2_460_000.5);
/// assert_eq!(sun_nakshatra_near(&longitudes, at, zodiac)?, sun_nakshatra_at(&longitudes, at, zodiac)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`sun_nakshatra_at`], and `OUT_OF_RANGE` on `date` when the Sun may
/// have changed nakshatra between the instant and the edge.
pub fn sun_nakshatra_near<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
) -> Result<Nakshatra, Error> {
    let coverage = tropical.coverage();
    if coverage.contains(at.get()) {
        return sun_nakshatra_at(tropical, at, zodiac);
    }
    let edge = at.get().clamp(coverage.from, coverage.to);
    let source = Sidereal::over(tropical, zodiac);
    let degrees = value_of(
        Quantity::Longitude(Body::Sun),
        &source,
        JulianDay::<Ut1>::literal(edge),
    )?;
    let span = 360.0 / f64::from(Limb::Nakshatra.divisions());
    let into = degrees.rem_euclid(span);
    let boundary = if at.get() < edge { into } else { span - into };
    if teistro_astro::events::greatest_rate(Body::Sun) * (at.get() - edge).abs() >= boundary {
        return Err(Error::new(
            Status::OutOfRange,
            format!(
                "the Sun's nakshatra at {at} is outside the ephemeris's coverage, near enough a boundary that it is not known"
            ),
        )
        .with_field("date"));
    }
    Ok(Nakshatra::of(
        Nas::try_from_degrees(degrees)?.division_index(Limb::Nakshatra.divisions()),
    ))
}

/// `frame`, geocentric: what an almanac's season and a day's Moon signs
/// read, whatever centre the limbs are found from.
///
/// ```
/// use teistro_panchanga::limb::geocentric;
/// use teistro_port_ephemeris::{Centre, Frame};
///
/// let topocentric = Frame { centre: Centre::Topocentric, ..Frame::CANONICAL };
/// assert_eq!(geocentric(topocentric).centre, Centre::Geocentric);
/// ```
#[must_use]
pub const fn geocentric(frame: Frame) -> Frame {
    Frame {
        centre: Centre::Geocentric,
        ..frame
    }
}

/// The Sun and the Moon as the limbs read them at an instant (SR-137):
/// each body's sidereal longitude, and the two angles the limbs are cut
/// from, each read by [`teistro_astro::events::value_of`] over the same
/// shifted source and as the same quantity the limb search reads, so the
/// angles at an instant a search found sit on the limb's grid to the
/// search's tolerance.
///
/// ```
/// use teistro_core::quantity::JulianDay;
/// use teistro_panchanga::limb::LimbAngles;
///
/// let angles = LimbAngles {
///     at: JulianDay::literal(2_461_345.25),
///     sun_deg: 220.0,
///     moon_deg: 4.0,
///     elongation_deg: 144.0,
///     yoga_deg: 224.0,
/// };
/// // Shukla Trayodashi, the thirteenth tithi, and Vyatipata, the
/// // seventeenth yoga.
/// assert_eq!(angles.tithi(), teistro_core::catalogue::Tithi::ShuklaTrayodashi);
/// assert_eq!(angles.yoga(), teistro_core::catalogue::Yoga::Vyatipata);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct LimbAngles {
    /// The instant, a UTC Julian day, read as UT1 as every limb is.
    pub at: JulianDay<Utc>,
    /// The Sun's sidereal longitude, degrees from 0 to 360.
    pub sun_deg: f64,
    /// The Moon's sidereal longitude, degrees from 0 to 360: the
    /// nakshatra's angle.
    pub moon_deg: f64,
    /// The Moon less the Sun, degrees from 0 to 360: the tithi's (twelve
    /// degrees each) and the karana's (six) angle.
    pub elongation_deg: f64,
    /// The Moon plus the Sun, degrees from 0 to 360: the yoga's angle,
    /// 13°20′ each.
    pub yoga_deg: f64,
}

impl LimbAngles {
    /// The tithi the elongation stands in, by the limbs' own integer path.
    ///
    /// ```
    /// use teistro_core::catalogue::Tithi;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::limb::LimbAngles;
    ///
    /// let before_new_moon = LimbAngles {
    ///     at: JulianDay::literal(0.0),
    ///     sun_deg: 0.0,
    ///     moon_deg: 359.9,
    ///     elongation_deg: 359.9,
    ///     yoga_deg: 359.9,
    /// };
    /// assert_eq!(before_new_moon.tithi(), Tithi::Amavasya);
    /// ```
    #[must_use]
    pub fn tithi(&self) -> Tithi {
        Tithi::of(index_of(self.elongation_deg, Limb::Tithi))
    }

    /// The nakshatra the Moon stands in.
    ///
    /// ```
    /// use teistro_core::catalogue::Nakshatra;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::limb::LimbAngles;
    ///
    /// let at = LimbAngles { at: JulianDay::literal(0.0), sun_deg: 0.0, moon_deg: 13.4, elongation_deg: 13.4, yoga_deg: 13.4 };
    /// assert_eq!(at.nakshatra(), Nakshatra::Bharani);
    /// ```
    #[must_use]
    pub fn nakshatra(&self) -> Nakshatra {
        Nakshatra::of(index_of(self.moon_deg, Limb::Nakshatra))
    }

    /// The yoga the sum stands in.
    ///
    /// ```
    /// use teistro_core::catalogue::Yoga;
    /// use teistro_core::quantity::JulianDay;
    /// use teistro_panchanga::limb::LimbAngles;
    ///
    /// let at = LimbAngles { at: JulianDay::literal(0.0), sun_deg: 0.0, moon_deg: 0.0, elongation_deg: 0.0, yoga_deg: 13.4 };
    /// assert_eq!(at.yoga(), Yoga::Priti);
    /// ```
    #[must_use]
    pub fn yoga(&self) -> Yoga {
        <Yoga as Member>::of(index_of(self.yoga_deg, Limb::Yoga))
    }
}

/// The member of a limb an angle stands in, by the integer path; an angle
/// that is not finite stands in none, read as the first.
fn index_of(degrees: f64, limb: Limb) -> u32 {
    Nas::try_from_degrees(degrees).map_or(0, |nas| nas.division_index(limb.divisions()))
}

/// [`LimbAngles`] at an instant, over a tropical source in a zodiac: the
/// source the limbs shift, the quantities they read.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// use teistro_panchanga::limb::angles_at;
///
/// let angles = angles_at(&longitudes, JulianDay::literal(2_460_000.5), zodiac)?;
/// // The elongation and the sum are the two longitudes', on the circle.
/// let less = (angles.moon_deg - angles.sun_deg).rem_euclid(360.0);
/// let plus = (angles.moon_deg + angles.sun_deg).rem_euclid(360.0);
/// assert!((angles.elongation_deg - less).abs() < 1e-9);
/// assert!((angles.yoga_deg - plus).abs() < 1e-9);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal.
pub fn angles_at<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
) -> Result<LimbAngles, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let ut1 = JulianDay::<Ut1>::literal(at.get());
    let read = |quantity: Quantity| value_of(quantity, &source, ut1);
    Ok(LimbAngles {
        at,
        sun_deg: read(Quantity::Longitude(Body::Sun))?,
        moon_deg: read(Limb::Nakshatra.quantity())?,
        elongation_deg: read(Limb::Tithi.quantity())?,
        yoga_deg: read(Limb::Yoga.quantity())?,
    })
}

/// The Moon's stay between two sidereal longitudes around an instant: from
/// its last crossing of `from_deg` at or before `at` to its first crossing
/// of `to_deg` after it.
///
/// What a temporal dasha balance reads when a system counts over a wheel of
/// its own: the twenty-eight with Abhijit cut Uttarashadha and Shravana
/// where no nakshatra boundary falls, so the span is searched at the
/// segment's own bounds rather than read off the nakshatras
/// (`03-design/dasha-kernels.md`, "The 28-nakshatra wheel").
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{moon_between, nakshatra_at};
///
/// // Between a nakshatra's own bounds, its stay.
/// let stay = nakshatra_at(&longitudes, window.from, zodiac)?;
/// let from_deg = f64::from(stay.member.id()) * 360.0 / 27.0;
/// let between = moon_between(&longitudes, window.from, zodiac, (from_deg, from_deg + 360.0 / 27.0))?;
/// assert!((between.from.get() - stay.whole.from.get()).abs() < 1e-6);
/// assert!((between.to.get() - stay.whole.to.get()).abs() < 1e-6);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal, or `INTERNAL` should the Moon not cross a bound
/// within a nakshatra's longest stay either side, which it cannot fail to.
pub fn moon_between<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
    (from_deg, to_deg): (f64, f64),
) -> Result<Interval, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let crossing = |bound_deg: f64, from: f64, to: f64| {
        let lattice = Lattice {
            origin_deg: bound_deg.rem_euclid(360.0),
            step_deg: 0.0,
        };
        Search::new(&source, Limb::Nakshatra.quantity(), lattice).between(
            JulianDay::<Ut1>::literal(from),
            JulianDay::<Ut1>::literal(to),
        )
    };
    let instant = at.get();
    let entered = crossing(from_deg, instant - LONGEST_SPAN_DAYS, instant)?
        .iter()
        .map(|event| event.instant.get())
        .filter(|jd| *jd <= instant)
        .fold(None, |last: Option<f64>, jd| {
            Some(last.map_or(jd, |last| last.max(jd)))
        });
    let left = crossing(to_deg, instant, instant + LONGEST_SPAN_DAYS)?
        .iter()
        .map(|event| event.instant.get())
        .find(|jd| *jd >= instant);
    let (Some(entered), Some(left)) = (entered, left) else {
        return Err(Error::new(
            Status::Internal,
            format!("the Moon crossed no bound of {from_deg}° to {to_deg}° around {at}"),
        ));
    };
    Interval::new(
        JulianDay::<Utc>::literal(entered),
        JulianDay::<Utc>::literal(left),
    )
}

/// The four limbs of a window.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::limbs;
///
/// // Two karanas to a tithi, so a day touches at least two.
/// let day = limbs(&longitudes, window, zodiac)?;
/// assert!(!day.nakshatra.is_empty() && !day.yoga.is_empty());
/// assert!(day.karana.len() >= 2);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal (an instant or a body it cannot answer for),
/// an ayanamsha the catalogue cannot evaluate at the instant, or a
/// quantity that moved backwards, which none of these four can and which
/// is reported as `INTERNAL` rather than silently reordered.
pub fn limbs<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Limbs, Error> {
    limbs_with(tropical, window, zodiac, None)
}

/// [`limbs`], reading the boundaries from crossings already found over a
/// range ([`RangeCrossings`]) when they cover the window under the same
/// zodiac, and searching for them otherwise. The answer is [`limbs`]'s
/// either way, to the bit.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{limbs, limbs_with};
///
/// // With nothing known it searches, as `limbs` does.
/// assert_eq!(limbs_with(&longitudes, window, zodiac, None)?, limbs(&longitudes, window, zodiac)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`limbs`].
pub fn limbs_with<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
    known: Option<&RangeCrossings>,
) -> Result<Limbs, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let known = known.filter(|known| known.answers(zodiac, window));
    let found = |limb: Limb| -> Result<Vec<(f64, f64)>, Error> {
        match known {
            Some(known) => Ok(known.limb(limb, window)),
            None => boundaries(&source, limb, window),
        }
    };
    // One search at six degrees of the elongation answers the tithis and
    // the karanas alike, a tithi being two karanas: its boundaries are
    // the karana boundaries that fall on a multiple of twelve degrees.
    let elongation = found(Limb::Karana)?;
    let karana_edges: Vec<f64> = elongation.iter().map(|(instant, _)| *instant).collect();
    Ok(Limbs {
        tithi: spans(
            &source,
            Limb::Tithi,
            window,
            &tithi_edges(&elongation),
            known,
        )?,
        karana: spans(&source, Limb::Karana, window, &karana_edges, known)?,
        nakshatra: spans(
            &source,
            Limb::Nakshatra,
            window,
            &instants(found(Limb::Nakshatra)?),
            known,
        )?,
        yoga: spans(
            &source,
            Limb::Yoga,
            window,
            &instants(found(Limb::Yoga)?),
            known,
        )?,
    })
}

/// A window's tithi spans, as [`limbs_with`] gives them, to the bit, and
/// nothing else: what a month grid's cells read
/// ([`crate::cell`]). The elongation's crossings come from `known` when it
/// answers for them, and are searched for otherwise.
pub(crate) fn tithis_with<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
    known: Option<&RangeCrossings>,
) -> Result<Vec<Span<Tithi>>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let known = known.filter(|known| known.answers_for(Reach::Elongation, zodiac, window));
    let elongation = match known {
        Some(known) => known.limb(Limb::Karana, window),
        None => boundaries(&source, Limb::Karana, window)?,
    };
    spans(
        &source,
        Limb::Tithi,
        window,
        &tithi_edges(&elongation),
        known,
    )
}

/// The tithis' edges among the elongation's crossings: those on a
/// multiple of twelve degrees, a tithi being two karanas.
fn tithi_edges(elongation: &[(f64, f64)]) -> Vec<f64> {
    elongation
        .iter()
        .filter(|(_, boundary)| is_tithi_boundary(*boundary))
        .map(|(instant, _)| *instant)
        .collect()
}

/// Which crossings a [`RangeCrossings`] holds: every limb's and both
/// signs', or the elongation's alone, which is all a tithi reads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Reach {
    /// Every limb and both bodies' signs.
    Whole,
    /// The elongation's crossings, the tithis' and the karanas'.
    Elongation,
}

/// The crossings a run of days reads its limbs and its signs from,
/// found once for the whole run.
///
/// A day's limbs are searched for over its window widened by
/// [`LONGEST_SPAN_DAYS`], and its signs over the window widened by the
/// body's reach, so consecutive days search over almost the same days
/// again. The searches walk an anchored grid, on which a crossing found
/// from any window that holds it refines to the same bits; so the
/// crossings one search finds over a whole run, kept to a day's own
/// widened window, are what that day's own search finds, and a run of
/// days asks for them once.
///
/// ```
/// use teistro_astro::ayanamsha::Basis;
/// use teistro_astro::completion::Completion;
/// use teistro_astro::delta_t::DeltaTModel;
/// use teistro_astro::precession::PrecessionModel;
/// use teistro_core::catalogue::Ayanamsha;
/// use teistro_core::interval::Interval;
/// use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// use teistro_panchanga::limb::{RangeCrossings, Zodiac, limbs, limbs_with};
/// use teistro_port_ephemeris::{Frame, TestProvider, Zodiac as FrameZodiac};
///
/// let provider = TestProvider::new();
/// let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// let zodiac = Zodiac {
///     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
///     basis: Basis::Mean,
///     precession: PrecessionModel::Iau2006,
///     delta_t: DeltaTModel::TableThenModel,
/// };
/// let run = Interval::literal(2_460_000.5, 2_460_010.5);
/// let known = RangeCrossings::over(&longitudes, zodiac, run)?;
/// for day in 0..10 {
///     let from = 2_460_000.5 + f64::from(day);
///     let window = Interval::literal(from, from + 1.0);
///     assert_eq!(limbs_with(&longitudes, window, zodiac, Some(&known))?, limbs(&longitudes, window, zodiac)?);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct RangeCrossings {
    zodiac: Zodiac,
    /// Which crossings were searched for.
    reach: Reach,
    /// The windows this answers for lie inside it.
    run: Interval,
    elongation: Vec<(f64, f64)>,
    nakshatra: Vec<(f64, f64)>,
    yoga: Vec<(f64, f64)>,
    /// The Moon's and the Sun's sign crossings, each over the run widened
    /// by its own reach.
    moon_signs: Vec<f64>,
    sun_signs: Vec<f64>,
    /// What each span between two consecutive crossings of the run reads
    /// at its middle, by its quantity and its two ends' bits: the reading
    /// a day names the span's member by, read for the whole run as one
    /// grid a quantity.
    middles: BTreeMap<(u8, u64, u64), f64>,
}

/// A quantity's key in [`RangeCrossings`]' middles.
fn quantity_key(quantity: Quantity) -> Option<u8> {
    match quantity {
        Quantity::ELONGATION => Some(0),
        Quantity::Longitude(Body::Moon) => Some(1),
        Quantity::MOON_PLUS_SUN => Some(2),
        Quantity::Longitude(Body::Sun) => Some(3),
        _ => None,
    }
}

/// The instant a span is named at: its middle, reckoned as every span's
/// is, so that a span read for a run and the same span read for a day
/// are read at the same bits.
fn middle_of(from: f64, to: f64) -> f64 {
    Interval::literal(from, to).days().mul_add(0.5, from)
}

impl RangeCrossings {
    /// The crossings every window inside `run` reads, found by one search
    /// a limb and one a body.
    ///
    /// ```
    /// # use teistro_astro::ayanamsha::Basis;
    /// # use teistro_astro::completion::Completion;
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_core::catalogue::Ayanamsha;
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
    /// # use teistro_panchanga::limb::Zodiac;
    /// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
    /// #
    /// # // The analytic test sky, read in Lahiri's sidereal zodiac.
    /// # let provider = TestProvider::new();
    /// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
    /// # let zodiac = Zodiac {
    /// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
    /// #     basis: Basis::Mean,
    /// #     precession: PrecessionModel::Iau2006,
    /// #     delta_t: DeltaTModel::TableThenModel,
    /// # };
    /// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
    ///
    /// use teistro_panchanga::limb::RangeCrossings;
    ///
    /// // A week's crossings, found once.
    /// let week = Interval::literal(2_460_000.5, 2_460_007.5);
    /// let known = RangeCrossings::over(&longitudes, zodiac, week)?;
    /// assert!(known.answers(zodiac, window));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As [`limbs`].
    pub fn over<S: Longitudes + ?Sized>(
        tropical: &S,
        zodiac: Zodiac,
        run: Interval,
    ) -> Result<RangeCrossings, Error> {
        Self::found(tropical, zodiac, run, Reach::Whole)
    }

    /// As [`RangeCrossings::over`], searching only what `reach` names: a
    /// crossing searched is the one the whole search finds, to the bit,
    /// and one not searched is answered for by no window
    /// ([`RangeCrossings::answers_for`]).
    pub(crate) fn found<S: Longitudes + ?Sized>(
        tropical: &S,
        zodiac: Zodiac,
        run: Interval,
        reach: Reach,
    ) -> Result<RangeCrossings, Error> {
        let source = Sidereal::over(tropical, zodiac);
        let signs = |body: Body| -> Result<Vec<f64>, Error> {
            let reach = sign_reach_days(body);
            let events = Search::new(&source, Quantity::Longitude(body), Lattice::SIGNS).between(
                JulianDay::<Ut1>::literal(run.from.get() - reach),
                JulianDay::<Ut1>::literal(run.to.get() + reach),
            )?;
            Ok(events.iter().map(|event| event.instant.get()).collect())
        };
        let elongation = boundaries(&source, Limb::Karana, run)?;
        let (nakshatra, yoga, moon_signs, sun_signs) = match reach {
            Reach::Whole => (
                boundaries(&source, Limb::Nakshatra, run)?,
                boundaries(&source, Limb::Yoga, run)?,
                signs(Body::Moon)?,
                signs(Body::Sun)?,
            ),
            Reach::Elongation => Default::default(),
        };
        let tithi = tithi_edges(&elongation);
        let karana = instants(elongation.clone());
        let nakshatra_at = instants(nakshatra.clone());
        let yoga_at = instants(yoga.clone());
        // Every span between two consecutive crossings, by the quantity
        // its member is read from: a tithi and a karana from the
        // elongation, a nakshatra and a sign of the Moon from the Moon.
        let mut middles = BTreeMap::new();
        for (quantity, lists) in [
            (Quantity::ELONGATION, [&tithi, &karana]),
            (
                Quantity::Longitude(Body::Moon),
                [&nakshatra_at, &moon_signs],
            ),
            (Quantity::MOON_PLUS_SUN, [&yoga_at, &yoga_at]),
            (Quantity::Longitude(Body::Sun), [&sun_signs, &sun_signs]),
        ] {
            let Some(key) = quantity_key(quantity) else {
                continue;
            };
            let mut ends: Vec<(f64, f64)> = Vec::new();
            for list in lists {
                for pair in list.windows(2) {
                    if let [from, to] = pair
                        && !ends.iter().any(|(a, b)| {
                            a.to_bits() == from.to_bits() && b.to_bits() == to.to_bits()
                        })
                    {
                        ends.push((*from, *to));
                    }
                }
            }
            if ends.is_empty() {
                continue;
            }
            let at: Vec<JulianDay<Ut1>> = ends
                .iter()
                .map(|(from, to)| JulianDay::<Ut1>::literal(middle_of(*from, *to)))
                .collect();
            let read = teistro_astro::events::values_of(quantity, &source, &at)?;
            for ((from, to), degrees) in ends.into_iter().zip(read) {
                middles.insert((key, from.to_bits(), to.to_bits()), degrees);
            }
        }
        Ok(RangeCrossings {
            zodiac,
            reach,
            run,
            elongation,
            nakshatra,
            yoga,
            moon_signs,
            sun_signs,
            middles,
        })
    }

    /// What the span from `from` to `to` reads at its middle, when it is
    /// a span between two of the run's consecutive crossings.
    fn middle(&self, quantity: Quantity, from: f64, to: f64) -> Option<f64> {
        let key = quantity_key(quantity)?;
        self.middles
            .get(&(key, from.to_bits(), to.to_bits()))
            .copied()
    }

    /// Whether these crossings answer for a window: found under the same
    /// zodiac, over a run that holds the window, so that the run's own
    /// widening covers the window's.
    ///
    /// ```
    /// # use teistro_astro::ayanamsha::Basis;
    /// # use teistro_astro::completion::Completion;
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_core::catalogue::Ayanamsha;
    /// # use teistro_core::interval::Interval;
    /// # use teistro_core::quantity::JulianDay;
    /// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
    /// # use teistro_panchanga::limb::Zodiac;
    /// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
    /// #
    /// # // The analytic test sky, read in Lahiri's sidereal zodiac.
    /// # let provider = TestProvider::new();
    /// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
    /// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
    /// # let zodiac = Zodiac {
    /// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
    /// #     basis: Basis::Mean,
    /// #     precession: PrecessionModel::Iau2006,
    /// #     delta_t: DeltaTModel::TableThenModel,
    /// # };
    /// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
    ///
    /// use teistro_panchanga::limb::RangeCrossings;
    ///
    /// let known = RangeCrossings::over(&longitudes, zodiac, window)?;
    /// // Not a window past its run, nor another zodiac.
    /// assert!(!known.answers(zodiac, Interval::literal(2_460_001.0, 2_460_002.0)));
    /// let tropical = Zodiac { ayanamsha: None, ..zodiac };
    /// assert!(!known.answers(tropical, window));
    /// # Ok::<(), teistro_core::error::Error>(())
    /// ```
    #[must_use]
    pub fn answers(&self, zodiac: Zodiac, window: Interval) -> bool {
        self.answers_for(Reach::Whole, zodiac, window)
    }

    /// Whether these crossings answer for what `reach` names over a
    /// window: as [`RangeCrossings::answers`], and found for at least
    /// that.
    pub(crate) fn answers_for(&self, reach: Reach, zodiac: Zodiac, window: Interval) -> bool {
        (self.reach == Reach::Whole || self.reach == reach)
            && self.zodiac == zodiac
            && self.run.from.get() <= window.from.get()
            && window.to.get() <= self.run.to.get()
    }

    /// A limb's boundaries over a window widened as a day's own search
    /// widens it, kept as that search keeps them: inside its ends, ends
    /// included.
    fn limb(&self, limb: Limb, window: Interval) -> Vec<(f64, f64)> {
        let found = match limb {
            Limb::Tithi | Limb::Karana => &self.elongation,
            Limb::Nakshatra => &self.nakshatra,
            Limb::Yoga => &self.yoga,
        };
        let (from, to) = (
            JulianDay::<Ut1>::literal(window.from.get() - LONGEST_SPAN_DAYS).get(),
            JulianDay::<Ut1>::literal(window.to.get() + LONGEST_SPAN_DAYS).get(),
        );
        found
            .iter()
            .copied()
            .filter(|(at, _)| *at >= from && *at <= to)
            .collect()
    }

    /// A body's sign crossings over a window widened by its reach, kept
    /// as [`signs`]' own search keeps them; `None` for a body these
    /// crossings were not found for.
    fn signs(&self, body: Body, window: Interval, reach: f64) -> Option<Vec<f64>> {
        let found = match body {
            Body::Moon => &self.moon_signs,
            Body::Sun => &self.sun_signs,
            _ => return None,
        };
        // Only the reach the run was found at: the same bits, since a
        // nearer or a further one keeps a different set of crossings.
        if reach.to_bits() != sign_reach_days(body).to_bits() {
            return None;
        }
        let (from, to) = (
            JulianDay::<Ut1>::literal(window.from.get() - reach).get(),
            JulianDay::<Ut1>::literal(window.to.get() + reach).get(),
        );
        Some(
            found
                .iter()
                .copied()
                .filter(|at| *at >= from && *at <= to)
                .collect(),
        )
    }
}

/// Just the instants of a list of boundaries.
fn instants(boundaries: Vec<(f64, f64)>) -> Vec<f64> {
    boundaries.into_iter().map(|(instant, _)| instant).collect()
}

/// Whether a six-degree boundary of the elongation is also a tithi's.
///
/// The lattice's own degrees are exact multiples of six by construction,
/// so this is an integer question asked of a double; the tolerance is a
/// millionth of a degree, which is a tenth of a second of the Moon's
/// time and far below any spacing it could confuse.
fn is_tithi_boundary(boundary_deg: f64) -> bool {
    let twelfths = boundary_deg / 12.0;
    (twelfths - twelfths.round()).abs() < 1e-6
}

/// The crossings of a limb's lattice inside the widened window, each with
/// the boundary it reached, in order.
fn boundaries<S: Longitudes + ?Sized>(
    source: &S,
    limb: Limb,
    window: Interval,
) -> Result<Vec<(f64, f64)>, Error> {
    let from = JulianDay::<Ut1>::literal(window.from.get() - LONGEST_SPAN_DAYS);
    let to = JulianDay::<Ut1>::literal(window.to.get() + LONGEST_SPAN_DAYS);
    let events = Search::new(source, limb.quantity(), limb.lattice()).between(from, to)?;
    let found: Vec<(f64, f64)> = events
        .iter()
        .map(|event| (event.instant.get(), event.boundary_deg))
        .collect();
    // None of the four quantities can go backwards — the slowest
    // elongation is about eleven degrees a day — so an out-of-order
    // boundary is a defect in the search and not a retrograde arc.
    if found.windows(2).any(|pair| {
        pair.first()
            .zip(pair.get(1))
            .is_some_and(|(a, b)| b.0 < a.0)
    }) {
        return Err(Error::new(
            Status::Internal,
            format!("{limb:?} boundaries came back out of order"),
        ));
    }
    Ok(found)
}

/// A limb's spans over a window, from the boundaries already found.
fn spans<S, T>(
    source: &S,
    limb: Limb,
    window: Interval,
    found: &[f64],
    known: Option<&RangeCrossings>,
) -> Result<Vec<Span<T>>, Error>
where
    S: Longitudes + ?Sized,
    T: Member,
{
    let mut out = Vec::new();
    for (from, to, clipped) in cut(found, window, LONGEST_SPAN_DAYS, source.coverage()) {
        let whole = Interval::literal(from, to);
        if !whole.overlaps(window) {
            continue;
        }
        // The member is read in the middle of its own span, where no
        // boundary is, so a solver's last bit cannot change the answer;
        // a run read every span's middle already.
        let index = match known.and_then(|known| known.middle(limb.quantity(), from, to)) {
            Some(degrees) => Nas::try_from_degrees(degrees)?.division_index(limb.divisions()),
            None => classify(source, limb, middle_of(from, to))?,
        };
        if let Some(span) = Span::new(T::of(index), whole, window) {
            out.push(span.with_clipped(clipped));
        }
    }
    Ok(out)
}

/// The spans touching a window as their two ends: the boundaries inside
/// the window widened by `reach` either side, with the widened ends
/// closing the first and last spans, the whole cut to the coverage
/// (SR-136, C2242). A first or last span the coverage cut is marked: it
/// runs from or to the coverage's edge, which is not its own.
fn cut(
    boundaries: &[f64],
    window: Interval,
    reach: f64,
    coverage: Coverage,
) -> Vec<(f64, f64, Option<Clipped>)> {
    let Some((first, last, clipped)) =
        coverage.clip(window.from.get() - reach, window.to.get() + reach)
    else {
        return Vec::new();
    };
    let mut edges = Vec::with_capacity(boundaries.len() + 2);
    edges.push(first);
    edges.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > first && *at < last),
    );
    edges.push(last);
    let pairs = edges.len().saturating_sub(1);
    edges
        .windows(2)
        .enumerate()
        .filter_map(|(index, pair)| {
            let (from, to) = (*pair.first()?, *pair.get(1)?);
            let mark = clipped.and_then(|cut| {
                Clipped::of(cut.start() && index == 0, cut.end() && index + 1 == pairs)
            });
            Some((from, to, mark))
        })
        .collect()
}

/// Which member of the limb the quantity names at an instant, by the
/// integer path (ADR-0016).
fn classify<S: Longitudes + ?Sized>(source: &S, limb: Limb, jd: f64) -> Result<u32, Error> {
    let ut1 = JulianDay::<Ut1>::literal(jd);
    let degrees = teistro_astro::events::value_of(limb.quantity(), source, ut1)?;
    Ok(Nas::try_from_degrees(degrees)?.division_index(limb.divisions()))
}

/// The signs a body stood in over a window, as spans.
///
/// The Sun's sign gives the ayana and its change is a sankranti; the
/// Moon's is what an almanac prints beside the nakshatra. Both are the
/// same search on the twelve-sign lattice over the same shifted source
/// the limbs use, so the sign a span names is in the chart's zodiac like
/// everything else.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::signs;
///
/// // The Moon touches one sign in a day, or two end to end.
/// let moon = signs(&longitudes, Body::Moon, window, zodiac)?;
/// assert!((1..=2).contains(&moon.len()));
/// if let [first, second] = moon.as_slice() {
///     assert_eq!(first.whole.to, second.whole.from);
/// }
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`limbs`].
pub fn signs<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Vec<Span<Rashi>>, Error> {
    signs_within(tropical, body, window, zodiac, None)
}

/// The signs a body stood in over a window, reaching a chosen distance
/// either side of it.
///
/// [`signs`] with the reach named. `None` takes the default of
/// [`sign_reach_days`]; a caller that wants a slow body's true span, or
/// that knows its own bound, gives one in days.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{signs, signs_within};
///
/// // A wider reach names the same signs.
/// let named = |reach| -> Result<Vec<_>, teistro_core::error::Error> {
///     Ok(signs_within(&longitudes, Body::Moon, window, zodiac, reach)?.iter().map(|span| span.member).collect())
/// };
/// let usual: Vec<_> = signs(&longitudes, Body::Moon, window, zodiac)?.iter().map(|span| span.member).collect();
/// assert_eq!(named(Some(5.0))?, usual);
/// assert_eq!(named(Some(-1.0)).unwrap_err().field(), Some("reach_days"));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`signs`], plus `INVALID_ARG` for a reach that is not a positive
/// finite number of days.
pub fn signs_within<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
    reach_days: Option<f64>,
) -> Result<Vec<Span<Rashi>>, Error> {
    signs_known(tropical, body, window, zodiac, reach_days, None)
}

/// [`signs_within`], reading the crossings from a run's
/// ([`RangeCrossings`]) when they cover the window under the same zodiac
/// at the body's own reach. The answer is [`signs_within`]'s either way,
/// to the bit.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{RangeCrossings, signs_known, signs_within};
///
/// let week = Interval::literal(2_460_000.5, 2_460_007.5);
/// let known = RangeCrossings::over(&longitudes, zodiac, week)?;
/// let read = signs_known(&longitudes, Body::Sun, window, zodiac, None, Some(&known))?;
/// assert_eq!(read, signs_within(&longitudes, Body::Sun, window, zodiac, None)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`signs_within`].
pub fn signs_known<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
    reach_days: Option<f64>,
    known: Option<&RangeCrossings>,
) -> Result<Vec<Span<Rashi>>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let quantity = Quantity::Longitude(body);
    let search = Search::new(&source, quantity, Lattice::SIGNS);
    let reach = match reach_days {
        Some(given) if given.is_finite() && given > 0.0 => given,
        Some(given) => {
            return Err(Error::invalid_arg(format!(
                "a sign search reaches a positive number of days either side, not {given}"
            ))
            .with_field("reach_days"));
        }
        None => sign_reach_days(body),
    };
    let known = known.filter(|known| known.answers(zodiac, window));
    let found: Vec<f64> =
        if let Some(found) = known.and_then(|known| known.signs(body, window, reach)) {
            found
        } else {
            let from = JulianDay::<Ut1>::literal(window.from.get() - reach);
            let to = JulianDay::<Ut1>::literal(window.to.get() + reach);
            let events = search.between(from, to)?;
            events.iter().map(|event| event.instant.get()).collect()
        };
    let mut out = Vec::new();
    for (start, end, clipped) in cut(&found, window, reach, source.coverage()) {
        let whole = Interval::literal(start, end);
        if !whole.overlaps(window) {
            continue;
        }
        let degrees = match known.and_then(|known| known.middle(quantity, start, end)) {
            Some(degrees) => degrees,
            None => teistro_astro::events::value_of(
                quantity,
                &source,
                JulianDay::<Ut1>::literal(middle_of(start, end)),
            )?,
        };
        let sign = Nas::try_from_degrees(degrees)?.sign();
        if let Some(span) = Span::new(sign, whole, window) {
            out.push(span.with_clipped(clipped));
        }
    }
    Ok(out)
}

/// The nakshatras a body stood in over a window, each the span its two
/// crossings give: the signs' search ([`signs_within`]) on the
/// twenty-seven. What a day's homa ahuti reads of the Sun
/// ([`crate::omen::homa_ahuti`]), which a day does not otherwise carry.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{nakshatras_of, sun_nakshatra_at};
///
/// // The Sun holds a star some thirteen days: one in a day, or two.
/// let stars = nakshatras_of(&longitudes, Body::Sun, window, zodiac)?;
/// assert!((1..=2).contains(&stars.len()));
/// assert_eq!(stars[0].member, sun_nakshatra_at(&longitudes, window.from, zodiac)?);
/// assert!(stars[0].whole.days() < 15.0);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal.
pub fn nakshatras_of<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Vec<Span<Nakshatra>>, Error> {
    nakshatras_known(tropical, body, window, zodiac, None)
}

/// [`nakshatras_of`], reading the Moon's stars from a range's crossings
/// when they answer for the window, as [`signs_known`] reads its signs,
/// and searching otherwise. The two give the same spans, so a day of a
/// range and the same day alone agree.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// use teistro_panchanga::limb::{RangeCrossings, nakshatras_known, nakshatras_of};
///
/// let week = Interval::literal(2_460_000.5, 2_460_007.5);
/// let day = Interval::literal(2_460_002.5, 2_460_003.5);
/// let known = RangeCrossings::over(&longitudes, zodiac, week)?;
/// let read = nakshatras_known(&longitudes, Body::Moon, day, zodiac, Some(&known))?;
/// assert_eq!(read, nakshatras_of(&longitudes, Body::Moon, day, zodiac)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal.
pub fn nakshatras_known<S: Longitudes + ?Sized>(
    tropical: &S,
    body: Body,
    window: Interval,
    zodiac: Zodiac,
    known: Option<&RangeCrossings>,
) -> Result<Vec<Span<Nakshatra>>, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let quantity = Quantity::Longitude(body);
    let reach = teistro_astro::events::longest_dwell_days(quantity, &Lattice::NAKSHATRAS)
        .map_or(SIGN_SEARCH_CAP_DAYS, |dwell| {
            dwell.min(SIGN_SEARCH_CAP_DAYS)
        });
    let from = JulianDay::<Ut1>::literal(window.from.get() - reach);
    let to = JulianDay::<Ut1>::literal(window.to.get() + reach);
    let known = known.filter(|known| body == Body::Moon && known.answers(zodiac, window));
    let found: Vec<f64> = if let Some(known) = known {
        known
            .nakshatra
            .iter()
            .map(|(at, _)| *at)
            .filter(|at| *at >= from.get() && *at <= to.get())
            .collect()
    } else {
        Search::new(&source, quantity, Lattice::NAKSHATRAS)
            .between(from, to)?
            .iter()
            .map(|event| event.instant.get())
            .collect()
    };
    let mut out = Vec::new();
    for (start, end, clipped) in cut(&found, window, reach, source.coverage()) {
        let whole = Interval::literal(start, end);
        if !whole.overlaps(window) {
            continue;
        }
        let degrees = match known.and_then(|known| known.middle(quantity, start, end)) {
            Some(degrees) => degrees,
            None => teistro_astro::events::value_of(
                quantity,
                &source,
                JulianDay::<Ut1>::literal(middle_of(start, end)),
            )?,
        };
        let star = Nakshatra::of(
            Nas::try_from_degrees(degrees)?.division_index(Limb::Nakshatra.divisions()),
        );
        if let Some(span) = Span::new(star, whole, window) {
            out.push(span.with_clipped(clipped));
        }
    }
    Ok(out)
}

/// The furthest either side of a window a sign search reaches by
/// default, days.
///
/// A ceiling rather than a reach. A body that can retrograde has no
/// bound on how long it may stand in one sign — it can cross the line
/// and come back — so for those this is the distance searched and the
/// span's own bounds past it are reported as the search's edge, which is
/// honest rather than wrong; no caller of a daily almanac needs an outer
/// planet's exact ingress. A body that never turns is searched at its
/// own reach instead ([`sign_reach_days`]), which for every such body is
/// nearer than this.
pub const SIGN_SEARCH_CAP_DAYS: f64 = 40.0;

/// How far either side of a window a sign search reaches for a body.
///
/// The longest that body can stand in one sign, from its least rate of
/// longitude — thirty degrees at the Sun's slowest is thirty-one and a
/// half days, at the Moon's slowest two and a half — capped at
/// [`SIGN_SEARCH_CAP_DAYS`], which is also the answer for a body whose
/// dwell has no bound.
///
/// This was one constant for every body, sized for the Sun, and the Moon
/// was searched at the Sun's reach: eighty-one days of crossings to name
/// the two signs a day can touch, which measured **44% of an almanac
/// day's ephemeris calls**
/// (`07-roadmap/02-plan-performance-and-passthrough.md`, A1).
///
/// ```
/// use teistro_panchanga::limb::{SIGN_SEARCH_CAP_DAYS, sign_reach_days};
/// use teistro_port_ephemeris::Body;
///
/// // The Moon's few days, the Sun's month, and a planet that turns capped.
/// assert!(sign_reach_days(Body::Moon) < 3.0);
/// assert!((30.0..SIGN_SEARCH_CAP_DAYS).contains(&sign_reach_days(Body::Sun)));
/// assert_eq!(sign_reach_days(Body::Saturn), SIGN_SEARCH_CAP_DAYS);
/// ```
#[must_use]
pub fn sign_reach_days(body: Body) -> f64 {
    teistro_astro::events::longest_dwell_days(Quantity::Longitude(body), &Lattice::SIGNS)
        .map_or(SIGN_SEARCH_CAP_DAYS, |dwell| {
            dwell.min(SIGN_SEARCH_CAP_DAYS)
        })
}

/// The amanta month a window falls in.
///
/// A lunar month takes the name of the solar month its new moon fell in:
/// Chaitra when the Sun stood in Pisces at that new moon, Vaishakha in
/// Aries, and so on, which the catalogue carries as each masa's
/// `solar_sign`. So this is one crossing search — the elongation through
/// zero, backwards from the window — and one sign.
///
/// What it does **not** decide is whether the month is intercalary. When
/// two new moons fall in one solar month both take that month's name and
/// the second is adhika; naming them is right either way, and marking
/// which is the Indian lunisolar calendar's to do.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{amanta_month, lunar_month_span, masa_at};
///
/// // Named by the Sun at the new moon that opened it.
/// let opened = lunar_month_span(&longitudes, window, zodiac)?.from;
/// assert_eq!(amanta_month(&longitudes, window, zodiac)?, masa_at(&longitudes, opened, zodiac)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal, or a window with no new moon in the
/// thirty-one days before it, which cannot happen for a real sky and is
/// reported as `NOT_CONVERGED` naming the window.
pub fn amanta_month<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Masa, Error> {
    let span = lunar_month_span(tropical, window, zodiac)?;
    masa_at(tropical, span.from, zodiac)
}

/// The new moons that bound the lunar month a window falls in.
///
/// The month's name comes from the Sun at its opening new moon
/// ([`masa_at`]) and its **kind** from the sankrantis between the two
/// ([`teistro_calendar::lunisolar::kind_of`]), so both want this and it
/// is found once.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::lunar_month_span;
///
/// let month = lunar_month_span(&longitudes, window, zodiac)?;
/// assert!(month.contains(window.from));
/// assert!((29.2..29.9).contains(&month.days()));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal, or a window with no new moon in the
/// thirty-one days before it, which cannot happen for a real sky and is
/// reported as `NOT_CONVERGED` naming the window.
pub fn lunar_month_span<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<Interval, Error> {
    let month = lunar_month_cut(tropical, window, zodiac)?;
    match month.clipped {
        None => Ok(month.span),
        Some(_) => Err(Error::new(
            Status::OutOfRange,
            format!(
                "the lunar month holding {} has a new moon outside the ephemeris's coverage",
                window.from
            ),
        )
        .with_field("date")),
    }
}

/// A lunar month's span as far as the coverage holds it (SR-136, C2246):
/// a new moon outside the coverage is replaced by the edge, marked in
/// `clipped`, and how far past the edge it lies is bounded in `before`
/// or `after` from the Moon's age there.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::lunar_month_cut;
///
/// // A sky with no edge holds both new moons.
/// let month = lunar_month_cut(&longitudes, window, zodiac)?;
/// assert!(month.clipped.is_none() && month.before.is_none() && month.after.is_none());
/// assert!(month.span.from <= window.from && window.to <= month.span.to);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonthCut {
    /// From the opening new moon, or the coverage's first instant, to the
    /// closing one or its last.
    pub span: Interval,
    /// Which new moons the coverage does not hold.
    pub clipped: Option<Clipped>,
    /// How far before the span's start the opening new moon lies, when
    /// the start is cut.
    pub before: Option<Beyond>,
    /// How far after the span's end the closing new moon lies, when the
    /// end is cut.
    pub after: Option<Beyond>,
}

/// [`lunar_month_span`], with a new moon the coverage does not hold
/// replaced by its edge rather than refused (SR-136, C2246).
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{lunar_month_cut, lunar_month_span};
///
/// // A sky with no edge cuts nothing.
/// let month = lunar_month_cut(&longitudes, window, zodiac)?;
/// assert_eq!((month.span, month.clipped), (lunar_month_span(&longitudes, window, zodiac)?, None));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`lunar_month_span`], less the refusal of a month the coverage
/// cuts.
pub fn lunar_month_cut<S: Longitudes + ?Sized>(
    tropical: &S,
    window: Interval,
    zodiac: Zodiac,
) -> Result<MonthCut, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let coverage = source.coverage();
    let search = Search::new(&source, Quantity::ELONGATION, Lattice::single(0.0));
    let from = window.from.get();
    // The Moon's age at the window's start bounds where both new moons
    // can be: the elongation over its greatest rate is the least time
    // since the last, and over its least the most, which are the bounds
    // the search's own step rests on. Each window is searched within its
    // bounds and a day either side, inside the one a wider search would
    // take, and the anchored scan refines a crossing to the same bits
    // from any window holding it; so the span is the wide search's, at a
    // third of its samples. A window the bounds leave empty is searched
    // whole, which keeps a strange sky's answer and its refusal.
    let elongation = value_of(
        Quantity::ELONGATION,
        &source,
        JulianDay::<Ut1>::literal(from),
    )?;
    let fastest = quantity_rate_deg_per_day(Quantity::ELONGATION);
    // Within a day's motion of the line, the reading and the refined
    // crossing can fall either side of it, so the near bound is the wide
    // window's own.
    let wraps = LUNATION_PAD_DAYS * fastest;
    // The window `arc` degrees of elongation from the start, behind it
    // when `arc` is negative, clamped inside the wide one; `near` false
    // keeps the wide window's near end.
    let bounded = |arc: f64, near: bool, wide: (f64, f64)| {
        let Some(slowest) = quantity_least_rate(Quantity::ELONGATION) else {
            return wide;
        };
        let (soon, late) = (from + arc / fastest, from + arc / slowest);
        if arc < 0.0 {
            let hi = if near {
                soon + LUNATION_PAD_DAYS
            } else {
                wide.1
            };
            ((late - LUNATION_PAD_DAYS).max(wide.0), hi.min(wide.1))
        } else {
            let lo = if near {
                soon - LUNATION_PAD_DAYS
            } else {
                wide.0
            };
            (lo.max(wide.0), (late + LUNATION_PAD_DAYS).min(wide.1))
        }
    };
    let between = |wide: (f64, f64), arc: f64, near: bool| -> Result<Vec<_>, Error> {
        let at = JulianDay::<Ut1>::literal;
        let (lo, hi) = bounded(arc, near, wide);
        if lo < hi {
            let narrow = search.between(at(lo), at(hi))?;
            if !narrow.is_empty() || (lo, hi) == wide {
                return Ok(narrow);
            }
        }
        search.between(at(wide.0), at(wide.1))
    };
    let before = between(
        (from - SYNODIC_SEARCH_DAYS, from),
        -elongation,
        elongation < 360.0 - wraps,
    )?;
    let refused = |side: &str, of: String| {
        Error::new(
            Status::NotConverged,
            format!("no new moon in the {SYNODIC_SEARCH_DAYS} days {side} {of}"),
        )
    };
    let (opened, before) = match before.last() {
        Some(event) => (event.instant.get(), None),
        None if coverage.from > from - SYNODIC_SEARCH_DAYS => {
            let reach = beyond_edge(&source, coverage.from, false)?;
            (
                coverage.from,
                Some(reach.ok_or_else(|| refused("before", window.from.to_string()))?),
            )
        }
        None => return Err(refused("before", window.from.to_string())),
    };
    let wide = (opened + 1.0, opened + SYNODIC_SEARCH_DAYS);
    // The next new moon is the first after the start, the last before
    // it being `opened`.
    let after = between(wide, 360.0 - elongation, elongation > wraps)?;
    let (closed, after) = match after.first() {
        Some(event) => (event.instant.get(), None),
        None if coverage.to < wide.1 => {
            let reach = beyond_edge(&source, coverage.to, true)?;
            let reach = reach.ok_or_else(|| refused("after", opened.to_string()))?;
            (
                coverage.to,
                Some(within_lunation(reach, coverage.to - opened)),
            )
        }
        None => return Err(refused("after", opened.to_string())),
    };
    // The start's reach is held to a lunation once the closing new moon
    // is known.
    let before = before.map(|reach| match after {
        None => within_lunation(reach, closed - opened),
        Some(_) => reach,
    });
    Ok(MonthCut {
        span: Interval::new(JulianDay::try_new(opened)?, JulianDay::try_new(closed)?)?,
        clipped: Clipped::of(before.is_some(), after.is_some()),
        before,
        after,
    })
}

/// How far past an edge of the coverage the new moon it cut off lies,
/// from the Moon's age there over her least and greatest rates (SR-136,
/// C2246): `ahead` for the new moon after the edge. `None` for a sky
/// whose elongation has no least rate, which bounds nothing.
fn beyond_edge<S: Longitudes + ?Sized>(
    source: &S,
    edge: f64,
    ahead: bool,
) -> Result<Option<Beyond>, Error> {
    let Some(slowest) = quantity_least_rate(Quantity::ELONGATION) else {
        return Ok(None);
    };
    let age = value_of(
        Quantity::ELONGATION,
        source,
        JulianDay::<Ut1>::literal(edge),
    )?;
    let arc = if ahead { 360.0 - age } else { age };
    Ok(Some(Beyond {
        least: arc / quantity_rate_deg_per_day(Quantity::ELONGATION),
        most: arc / slowest,
    }))
}

/// A reach past an edge held to a lunation less the part of it the
/// coverage holds, `known` days.
fn within_lunation(reach: Beyond, known: f64) -> Beyond {
    Beyond {
        least: reach.least.max(LUNATION_LEAST_DAYS - known),
        most: reach.most.min(LUNATION_MOST_DAYS - known),
    }
}

/// The lunar month the Sun's sign at an instant names.
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{masa_at, signs};
///
/// // The month whose solar sign the Sun stands in.
/// let sun = signs(&longitudes, Body::Sun, Interval::new(window.from, window.from)?, zodiac)?;
/// assert_eq!(Some(masa_at(&longitudes, window.from, zodiac)?.attributes().solar_sign), sun.first().map(|span| span.member));
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// The source's own refusal, or a sign no masa claims, which the
/// catalogue makes impossible.
pub fn masa_at<S: Longitudes + ?Sized>(
    tropical: &S,
    at: JulianDay<Utc>,
    zodiac: Zodiac,
) -> Result<Masa, Error> {
    let source = Sidereal::over(tropical, zodiac);
    let sun = value_of(
        Quantity::Longitude(Body::Sun),
        &source,
        JulianDay::<Ut1>::literal(at.get()),
    )?;
    masa_of_sun(sun)
}

/// The amanta month a lunar month carries: [`masa_at`] its opening new
/// moon, or, when the coverage cut that new moon off, the Sun's sign at
/// the coverage's edge less the sankrantis his rates say he passed in
/// between (SR-136, C2246).
///
/// ```
/// # use teistro_astro::ayanamsha::Basis;
/// # use teistro_astro::completion::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_core::catalogue::Ayanamsha;
/// # use teistro_core::interval::Interval;
/// # use teistro_core::quantity::JulianDay;
/// # use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
/// # use teistro_panchanga::limb::Zodiac;
/// # use teistro_port_ephemeris::{Body, Frame, TestProvider, Zodiac as FrameZodiac};
/// #
/// # // The analytic test sky, read in Lahiri's sidereal zodiac.
/// # let provider = TestProvider::new();
/// # let completion = Completion::new(&provider, OverridePolicy::SdkOnly, DeltaTModel::TableThenModel);
/// # let longitudes = completion.longitudes(Frame { zodiac: FrameZodiac::Tropical, ..Frame::CANONICAL });
/// # let zodiac = Zodiac {
/// #     ayanamsha: Some(AyanamshaChoice::Catalogued { id: Ayanamsha::Lahiri }),
/// #     basis: Basis::Mean,
/// #     precession: PrecessionModel::Iau2006,
/// #     delta_t: DeltaTModel::TableThenModel,
/// # };
/// # let window = Interval::literal(2_460_000.5, 2_460_001.5);
///
/// use teistro_panchanga::limb::{lunar_month_cut, masa_at, masa_opening};
///
/// // An uncut month is named from its opening new moon.
/// let month = lunar_month_cut(&longitudes, window, zodiac)?;
/// assert_eq!(masa_opening(&longitudes, &month, zodiac)?, masa_at(&longitudes, month.span.from, zodiac)?);
/// # Ok::<(), teistro_core::error::Error>(())
/// ```
///
/// # Errors
///
/// As [`masa_at`], and `OUT_OF_RANGE` on `date` when the Sun may or may
/// not have entered his sign between the new moon and the edge.
pub fn masa_opening<S: Longitudes + ?Sized>(
    tropical: &S,
    month: &MonthCut,
    zodiac: Zodiac,
) -> Result<Masa, Error> {
    let Some(before) = month.before else {
        return masa_at(tropical, month.span.from, zodiac);
    };
    let source = Sidereal::over(tropical, zodiac);
    let sun = value_of(
        Quantity::Longitude(Body::Sun),
        &source,
        JulianDay::<Ut1>::literal(month.span.from.get()),
    )?;
    let into = sun.rem_euclid(30.0);
    let passed = before.sankrantis(into).ok_or_else(|| {
        Error::new(
            Status::OutOfRange,
            format!(
                "the lunar month holding {} opened before the ephemeris's coverage, near enough a sankranti that its name is not known",
                month.span.from
            ),
        )
        .with_field("date")
    })?;
    // The middle of the sign `passed` signs back from the Sun's.
    let back = 30.0 * f64::from(u8::try_from(passed).unwrap_or(0));
    masa_of_sun((sun - into + 15.0 - back).rem_euclid(360.0))
}

/// The amanta month whose solar sign holds a sidereal longitude.
fn masa_of_sun(sun_deg: f64) -> Result<Masa, Error> {
    let sign = Nas::try_from_degrees(sun_deg)?.sign();
    Masa::ALL
        .iter()
        .copied()
        .find(|masa| masa.attributes().solar_sign == sign)
        .ok_or_else(|| Error::internal(format!("no lunar month for the Sun in {sign:?}")))
}

/// How far back a new moon is looked for: a synodic month is 29.53 days,
/// so thirty-one always holds one.
const SYNODIC_SEARCH_DAYS: f64 = 31.0;

/// The shortest a lunation runs, days, with room: the synodic month runs
/// 29.27 to 29.83 days over the millennia the SDK reads (Meeus,
/// *Astronomical Algorithms*, ch. 49).
const LUNATION_LEAST_DAYS: f64 = 29.2;

/// The longest a lunation runs, days, with room ([`LUNATION_LEAST_DAYS`]).
const LUNATION_MOST_DAYS: f64 = 29.9;

/// How far past the Moon's age bounds a new moon is looked for, days:
/// room for the elongation read at an instant the rates were not
/// measured at, far more than a crossing's bracket needs.
const LUNATION_PAD_DAYS: f64 = 1.0;

/// A limb's member, named by its index around the circle.
///
/// ```
/// use teistro_core::catalogue::{Nakshatra, Tithi};
/// use teistro_panchanga::limb::Member;
///
/// // An index past the circle comes round again.
/// assert_eq!(<Tithi as Member>::of(30), Tithi::ShuklaPratipada);
/// assert_eq!(<Nakshatra as Member>::of(28), Nakshatra::Bharani);
/// ```
pub trait Member: Sized {
    /// The member an index names; an index the catalogue does not have is
    /// impossible by construction, and the first member stands in.
    ///
    /// ```
    /// use teistro_core::catalogue::{Karana, Yoga};
    /// use teistro_panchanga::limb::Member;
    ///
    /// assert_eq!(<Yoga as Member>::of(0), Yoga::Vishkambha);
    /// assert_eq!(<Karana as Member>::of(0), Karana::Kimstughna);
    /// ```
    fn of(index: u32) -> Self;
}

impl Member for Tithi {
    fn of(index: u32) -> Tithi {
        Tithi::from_id(u16::try_from(index % 30).unwrap_or(0)).unwrap_or(Tithi::ShuklaPratipada)
    }
}

impl Member for Nakshatra {
    fn of(index: u32) -> Nakshatra {
        Nakshatra::from_id(u16::try_from(index % 27).unwrap_or(0)).unwrap_or(Nakshatra::Ashwini)
    }
}

impl Member for Yoga {
    fn of(index: u32) -> Yoga {
        Yoga::from_id(u16::try_from(index % 27).unwrap_or(0)).unwrap_or(Yoga::Vishkambha)
    }
}

impl Member for Karana {
    fn of(index: u32) -> Karana {
        karana_of(index)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        reason = "tests fail by panicking and index their own fixtures"
    )]

    use super::{
        LONGEST_SPAN_DAYS, Limb, SIGN_SEARCH_CAP_DAYS, SYNODIC_SEARCH_DAYS, Sidereal, Zodiac,
        amanta_month, cut, karana_of, lunar_month_span, sign_reach_days,
    };
    use teistro_astro::ayanamsha::Basis;
    use teistro_astro::completion::Completion;
    use teistro_astro::delta_t::DeltaTModel;
    use teistro_astro::events::Search;
    use teistro_astro::precession::PrecessionModel;
    use teistro_core::catalogue::{Ayanamsha, Karana, Masa};
    use teistro_core::coverage::{Clipped, Coverage};
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Ut1};
    use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
    use teistro_port_ephemeris::{
        Body, Frame, Lattice, Quantity, TestProvider, Zodiac as FrameZodiac,
    };

    /// Lahiri's mean sidereal zodiac, the fixtures' frame.
    const fn lahiri() -> Zodiac {
        Zodiac {
            ayanamsha: Some(AyanamshaChoice::Catalogued {
                id: Ayanamsha::Lahiri,
            }),
            basis: Basis::Mean,
            precession: PrecessionModel::Iau2006,
            delta_t: DeltaTModel::TableThenModel,
        }
    }

    /// Month after month, a name either follows the last in order or
    /// repeats it: the adhika month holds no sankranti and takes its
    /// follower's name.
    #[test]
    fn a_month_is_named_by_the_sun_at_its_opening_new_moon() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame {
            zodiac: FrameZodiac::Tropical,
            ..Frame::CANONICAL
        });
        let order = |masa: Masa| Masa::ALL.iter().position(|m| *m == masa).unwrap();
        let mut from = 2_460_000.5;
        let mut last: Option<usize> = None;
        let mut repeats = 0;
        for _ in 0..40 {
            let window = Interval::literal(from, from + 1.0);
            let named = order(amanta_month(&longitudes, window, lahiri()).unwrap());
            if let Some(last) = last {
                if named == last {
                    repeats += 1;
                } else {
                    assert_eq!(named, (last + 1) % 12, "from {from}");
                }
            }
            last = Some(named);
            from = lunar_month_span(&longitudes, window, lahiri())
                .unwrap()
                .to
                .get()
                + 1.0;
        }
        // Forty lunations hold an adhika month at least once.
        assert!(repeats >= 1);
    }

    /// A lunar month found by its age is the one the wide searches find,
    /// to the bit: every window of two years, a day and a third apart so
    /// each lunation is entered at many ages, both just after and just
    /// before its new moons.
    #[test]
    fn a_month_found_by_its_age_is_the_wide_searchs_to_the_bit() {
        let provider = TestProvider::new();
        let completion = Completion::new(
            &provider,
            OverridePolicy::SdkOnly,
            DeltaTModel::TableThenModel,
        );
        let longitudes = completion.longitudes(Frame {
            zodiac: FrameZodiac::Tropical,
            ..Frame::CANONICAL
        });
        let zodiac = lahiri();
        let source = Sidereal::over(&longitudes, zodiac);
        let search = Search::new(&source, Quantity::ELONGATION, Lattice::single(0.0));
        let at = JulianDay::<Ut1>::literal;
        let mut starts: Vec<f64> = (0..540)
            .map(|k| 2_460_000.5 + f64::from(k) * 1.37)
            .collect();
        // Either side of a new moon, where the age is nearly nothing or
        // nearly a whole lunation.
        for new_moon in search.between(at(2_460_000.5), at(2_460_740.5)).unwrap() {
            let t = new_moon.instant.get();
            starts.extend([t - 1e-3, t - 1e-6, t, t + 1e-6, t + 1e-3]);
        }
        for from in starts {
            let wide = |lo: f64, hi: f64| search.between(at(lo), at(hi)).unwrap();
            let opened = wide(from - SYNODIC_SEARCH_DAYS, from)
                .last()
                .unwrap()
                .instant;
            let closed = wide(opened.get() + 1.0, opened.get() + SYNODIC_SEARCH_DAYS)
                .first()
                .unwrap()
                .instant;
            let month =
                lunar_month_span(&longitudes, Interval::literal(from, from + 1.0), zodiac).unwrap();
            assert_eq!(
                (month.from.get().to_bits(), month.to.get().to_bits()),
                (opened.get().to_bits(), closed.get().to_bits()),
                "from {from}"
            );
        }
    }

    #[test]
    fn the_month_opens_with_kimstughna_and_closes_with_naga() {
        assert_eq!(karana_of(0), Karana::Kimstughna);
        assert_eq!(karana_of(1), Karana::Bava, "the first movable one");
        assert_eq!(karana_of(7), Karana::Vishti, "and the seventh");
        assert_eq!(karana_of(8), Karana::Bava, "and round again");
        assert_eq!(karana_of(56), Karana::Vishti);
        assert_eq!(karana_of(57), Karana::Shakuni);
        assert_eq!(karana_of(58), Karana::Chatushpada);
        assert_eq!(karana_of(59), Karana::Naga);
        assert_eq!(karana_of(60), Karana::Kimstughna, "and the month turns");
    }

    #[test]
    fn the_seven_movable_karanas_repeat_eight_times() {
        let movable: Vec<Karana> = (1..57).map(karana_of).collect();
        assert_eq!(movable.len(), 56, "eight rounds of seven");
        assert!(
            movable.iter().all(|karana| karana.attributes().movable),
            "every one of them is movable"
        );
        for round in 0..8 {
            assert_eq!(movable[round * 7], Karana::Bava);
            assert_eq!(movable[round * 7 + 6], Karana::Vishti);
        }
    }

    #[test]
    fn the_fixed_karanas_are_the_ones_that_are_not_movable() {
        for fixed in [
            Karana::Kimstughna,
            Karana::Shakuni,
            Karana::Chatushpada,
            Karana::Naga,
        ] {
            assert!(!fixed.attributes().movable);
        }
    }

    /// The ends `cut` closes spans at, in order.
    fn ends(spans: &[(f64, f64, Option<Clipped>)]) -> Vec<f64> {
        let mut ends: Vec<f64> = spans.iter().map(|span| span.0).collect();
        ends.extend(spans.last().map(|span| span.1));
        ends
    }

    #[test]
    fn the_edges_close_the_first_and_last_spans_outside_the_window() {
        let window = Interval::literal(100.0, 101.0);
        // Two boundaries inside the day and one outside the widened search.
        let found = [99.5, 100.4, 100.9, 200.0];
        let spans = cut(&found, window, LONGEST_SPAN_DAYS, Coverage::UNBOUNDED);
        let edges = ends(&spans);
        assert_eq!(edges.first().copied(), Some(100.0 - LONGEST_SPAN_DAYS));
        assert_eq!(edges.last().copied(), Some(101.0 + LONGEST_SPAN_DAYS));
        assert!(edges.contains(&99.5), "a boundary before the day is kept");
        assert!(!edges.contains(&200.0), "one outside the search is not");
        // Every consecutive pair is a span, and they are in order.
        assert!(edges.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(spans.iter().all(|span| span.2.is_none()), "nothing is cut");
    }

    #[test]
    fn the_coverage_cuts_the_first_span_and_marks_it_alone() {
        let window = Interval::literal(100.0, 101.0);
        let found = [99.5, 100.4, 100.9];
        let coverage = Coverage::new(99.0, 1000.0).unwrap();
        let spans = cut(&found, window, LONGEST_SPAN_DAYS, coverage);
        assert_eq!(ends(&spans).first().copied(), Some(99.0), "cut at the edge");
        assert_eq!(spans.first().and_then(|span| span.2), Some(Clipped::Start));
        assert!(spans.iter().skip(1).all(|span| span.2.is_none()));
        // A window the coverage does not reach has no spans.
        let gone = Coverage::new(500.0, 1000.0).unwrap();
        assert_eq!(cut(&found, window, LONGEST_SPAN_DAYS, gone), Vec::new());
    }

    #[test]
    fn each_limb_names_its_own_lattice_and_divisions() {
        assert_eq!(Limb::Tithi.divisions(), 30);
        assert_eq!(Limb::Karana.divisions(), 60);
        assert_eq!(Limb::Nakshatra.divisions(), 27);
        assert_eq!(Limb::Yoga.divisions(), 27);
        assert!((Limb::Tithi.lattice().step_deg - 12.0).abs() < f64::EPSILON);
        assert!((Limb::Karana.lattice().step_deg - 6.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_sign_reach_follows_the_body() {
        // Thirty degrees at the body's slowest. The Sun's month in a sign
        // is what the one constant was sized for; the Moon's two and a
        // half days is what it was costing.
        let sun = sign_reach_days(Body::Sun);
        let moon = sign_reach_days(Body::Moon);
        assert!(
            (31.0..32.0).contains(&sun),
            "the Sun stands in a sign about a month: {sun}"
        );
        assert!(
            (2.5..2.7).contains(&moon),
            "the Moon two and a half days: {moon}"
        );
        assert!(
            moon * 12.0 < sun,
            "which is more than twelve times nearer: {sun} against {moon}"
        );
    }

    #[test]
    fn a_body_that_can_turn_is_searched_to_the_cap() {
        // A retrograding body can cross a line and come back, so nothing
        // bounds its dwell and the cap and its truncation stand.
        for body in [Body::Mars, Body::Saturn, Body::TrueNode] {
            assert!(
                (sign_reach_days(body) - SIGN_SEARCH_CAP_DAYS).abs() < f64::EPSILON,
                "{body:?}"
            );
        }
        // The mean node never turns, so it is bounded — and the bound is
        // a year and a half, which the cap then brings back to itself.
        assert!(
            (sign_reach_days(Body::MeanNode) - SIGN_SEARCH_CAP_DAYS).abs() < f64::EPSILON,
            "capped, not unbounded"
        );
    }

    #[test]
    fn the_sign_edges_close_at_the_reach_they_were_searched_to() {
        let window = Interval::literal(100.0, 101.0);
        // One crossing inside the reach either side, and one far outside.
        let found = [98.0, 100.4, 110.0];
        assert_eq!(
            ends(&cut(&found, window, 2.5, Coverage::UNBOUNDED)),
            vec![97.5, 98.0, 100.4, 103.5],
            "the sentinels sit at the reach and the far crossing is dropped"
        );
        // The same crossings, searched to forty days, keep the far one.
        assert_eq!(
            ends(&cut(
                &found,
                window,
                SIGN_SEARCH_CAP_DAYS,
                Coverage::UNBOUNDED
            )),
            vec![60.0, 98.0, 100.4, 110.0, 141.0]
        );
    }
}
