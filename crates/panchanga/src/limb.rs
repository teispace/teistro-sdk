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
use teistro_core::angle::Nas;
use teistro_core::catalogue::{Karana, Masa, Nakshatra, Rashi, Tithi, Yoga};
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Ut1, Utc};
use teistro_port_ephemeris::{Body, Lattice, Quantity};

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
    let tithi_edges: Vec<f64> = elongation
        .iter()
        .filter(|(_, boundary)| is_tithi_boundary(*boundary))
        .map(|(instant, _)| *instant)
        .collect();
    let karana_edges: Vec<f64> = elongation.iter().map(|(instant, _)| *instant).collect();
    Ok(Limbs {
        tithi: spans(&source, Limb::Tithi, window, &tithi_edges, known)?,
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
        let nakshatra = boundaries(&source, Limb::Nakshatra, run)?;
        let yoga = boundaries(&source, Limb::Yoga, run)?;
        let (moon_signs, sun_signs) = (signs(Body::Moon)?, signs(Body::Sun)?);
        let tithi: Vec<f64> = elongation
            .iter()
            .filter(|(_, boundary)| is_tithi_boundary(*boundary))
            .map(|(instant, _)| *instant)
            .collect();
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
        self.zodiac == zodiac
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
    let edges = edges(found, window);
    for pair in edges.windows(2) {
        let (Some(from), Some(to)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let whole = Interval::literal(*from, *to);
        if !whole.overlaps(window) {
            continue;
        }
        // The member is read in the middle of its own span, where no
        // boundary is, so a solver's last bit cannot change the answer;
        // a run read every span's middle already.
        let index = match known.and_then(|known| known.middle(limb.quantity(), *from, *to)) {
            Some(degrees) => Nas::try_from_degrees(degrees)?.division_index(limb.divisions()),
            None => classify(source, limb, middle_of(*from, *to))?,
        };
        if let Some(span) = Span::new(T::of(index), whole, window) {
            out.push(span);
        }
    }
    Ok(out)
}

/// The instants that bound the spans touching a window: the boundaries
/// inside the widened search, with the widened ends closing the first and
/// last spans.
fn edges(boundaries: &[f64], window: Interval) -> Vec<f64> {
    let first = window.from.get() - LONGEST_SPAN_DAYS;
    let last = window.to.get() + LONGEST_SPAN_DAYS;
    let mut edges = Vec::with_capacity(boundaries.len() + 2);
    edges.push(first);
    edges.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > first && *at < last),
    );
    edges.push(last);
    edges
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
    let edges = sign_edges(&found, window, reach);
    for pair in edges.windows(2) {
        let (Some(start), Some(end)) = (pair.first(), pair.get(1)) else {
            continue;
        };
        let whole = Interval::literal(*start, *end);
        if !whole.overlaps(window) {
            continue;
        }
        let degrees = match known.and_then(|known| known.middle(quantity, *start, *end)) {
            Some(degrees) => degrees,
            None => teistro_astro::events::value_of(
                quantity,
                &source,
                JulianDay::<Ut1>::literal(middle_of(*start, *end)),
            )?,
        };
        let sign = Nas::try_from_degrees(degrees)?.sign();
        if let Some(span) = Span::new(sign, whole, window) {
            out.push(span);
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

/// The instants bounding the sign spans that touch a window.
fn sign_edges(boundaries: &[f64], window: Interval, reach: f64) -> Vec<f64> {
    let first = window.from.get() - reach;
    let last = window.to.get() + reach;
    let mut edges = Vec::with_capacity(boundaries.len() + 2);
    edges.push(first);
    edges.extend(
        boundaries
            .iter()
            .copied()
            .filter(|at| *at > first && *at < last),
    );
    edges.push(last);
    edges
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
    let source = Sidereal::over(tropical, zodiac);
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
    let opened = before.last().map(|event| event.instant).ok_or_else(|| {
        Error::new(
            Status::NotConverged,
            format!(
                "no new moon in the {SYNODIC_SEARCH_DAYS} days before {}",
                window.from
            ),
        )
    })?;
    let wide = (opened.get() + 1.0, opened.get() + SYNODIC_SEARCH_DAYS);
    // The next new moon is the first after the start, the last before
    // it being `opened`.
    let after = between(wide, 360.0 - elongation, elongation > wraps)?;
    let closed = after.first().map(|event| event.instant).ok_or_else(|| {
        Error::new(
            Status::NotConverged,
            format!("no new moon in the {SYNODIC_SEARCH_DAYS} days after {opened}"),
        )
    })?;
    Interval::new(
        JulianDay::try_new(opened.get())?,
        JulianDay::try_new(closed.get())?,
    )
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
    let sun = teistro_astro::events::value_of(
        Quantity::Longitude(Body::Sun),
        &source,
        JulianDay::<Ut1>::literal(at.get()),
    )?;
    let sign = Nas::try_from_degrees(sun)?.sign();
    Masa::ALL
        .iter()
        .copied()
        .find(|masa| masa.attributes().solar_sign == sign)
        .ok_or_else(|| Error::internal(format!("no lunar month for the Sun in {sign:?}")))
}

/// How far back a new moon is looked for: a synodic month is 29.53 days,
/// so thirty-one always holds one.
const SYNODIC_SEARCH_DAYS: f64 = 31.0;

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
        edges, karana_of, lunar_month_span, sign_edges, sign_reach_days,
    };
    use teistro_astro::ayanamsha::Basis;
    use teistro_astro::completion::Completion;
    use teistro_astro::delta_t::DeltaTModel;
    use teistro_astro::events::Search;
    use teistro_astro::precession::PrecessionModel;
    use teistro_core::catalogue::{Ayanamsha, Karana};
    use teistro_core::interval::Interval;
    use teistro_core::quantity::{JulianDay, Ut1};
    use teistro_core::settings::{AyanamshaChoice, OverridePolicy};
    use teistro_port_ephemeris::{
        Body, Frame, Lattice, Quantity, TestProvider, Zodiac as FrameZodiac,
    };

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
        let zodiac = Zodiac {
            ayanamsha: Some(AyanamshaChoice::Catalogued {
                id: Ayanamsha::Lahiri,
            }),
            basis: Basis::Mean,
            precession: PrecessionModel::Iau2006,
            delta_t: DeltaTModel::TableThenModel,
        };
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

    #[test]
    fn the_edges_close_the_first_and_last_spans_outside_the_window() {
        let window = Interval::literal(100.0, 101.0);
        // Two boundaries inside the day and one outside the widened search.
        let found = [99.5, 100.4, 100.9, 200.0];
        let edges = edges(&found, window);
        assert_eq!(edges.first().copied(), Some(100.0 - LONGEST_SPAN_DAYS));
        assert_eq!(edges.last().copied(), Some(101.0 + LONGEST_SPAN_DAYS));
        assert!(edges.contains(&99.5), "a boundary before the day is kept");
        assert!(!edges.contains(&200.0), "one outside the search is not");
        // Every consecutive pair is a span, and they are in order.
        assert!(edges.windows(2).all(|pair| pair[0] < pair[1]));
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
            sign_edges(&found, window, 2.5),
            vec![97.5, 98.0, 100.4, 103.5],
            "the sentinels sit at the reach and the far crossing is dropped"
        );
        // The same crossings, searched to forty days, keep the far one.
        assert_eq!(
            sign_edges(&found, window, SIGN_SEARCH_CAP_DAYS),
            vec![60.0, 98.0, 100.4, 110.0, 141.0]
        );
    }
}
