//! The search's [`Sources`] over an ephemeris provider: the almanac for
//! the days, the chart founder for the lagna, the frame completion for the
//! grahas, and the visibility reckoner for asta.
//!
//! One chart zodiac serves the whole search, taken at the instant the
//! caller names. The ayanamsha moves about 0.14″ a day, so at the far end
//! of a three-month search the lagna is read about 13″ off, which moves a
//! window's edge by about a second of time and changes no clause inside
//! it. A caller who wants the zodiac at each day searches a day at a
//! time.

use teistro_astro::Completion;
use teistro_astro::delta_t::DeltaTModel;
use teistro_astro::eclipse::{Eclipses, ShadowRule};
use teistro_astro::events::{Longitudes, Search};
use teistro_astro::precession::PrecessionModel;
use teistro_astro::scale::tt_of;
use teistro_astro::visibility::Heliacal;
use teistro_calendar::{CalendarDate, CalendarSystem, FixedDay};
use teistro_chart::foundation::{Founder, bodies_of};
use teistro_chart::zodiac::ChartZodiac;
use teistro_core::error::{Error, Status};
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Place, Ut1, Utc};
use teistro_core::settings::{EclipseVedha, Settings};
use teistro_core::time::LocalClock;
use teistro_panchanga::limb::{Sidereal, Zodiac, signs_within};
use teistro_panchanga::{Almanac, Panchanga};
use teistro_port_ephemeris::{
    Astronomy, Body, Centre, EphemerisProvider, Frame, Horizon, Lattice, Quantity,
};
use teistro_time::local_day::local_midnight;

use crate::activity::Unjudged;
use crate::instant::Sky;
use crate::search::{Season, Sources, same_day};
use crate::season::{
    Blackout, BlackoutKind, ECLIPSE_STAR_REACH_DAYS, SeenEclipse, SeenKind, asta_over, blackouts,
    eclipse_stars, eclipse_vedha,
};

/// How far beyond a range an eclipse's vedha can reach into it, days: it
/// opens at most four praharas before the prahara holding the eclipse,
/// under two days, and closes at most at the body's next rising.
const VEDHA_REACH_DAYS: f64 = 2.0;

/// The slow grahas whose ingresses a day may hold, beside the Sun's and
/// the Moon's, which the almanac day already has: Mars to Saturn, and the
/// node (Ketu crosses when Rahu does).
const SLOW: [usize; 6] = [2, 3, 4, 5, 6, 7];

/// A navamsa, degrees: the Moon's quarter of a star.
const NAVAMSA_DEG: f64 = 30.0 / 9.0;

/// The search's sources over a provider.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::visibility::{Criterion, Heliacal};
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_chart::foundation::Founder;
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_muhurta::sources::Over;
/// # use teistro_muhurta::{ProviderSources, Sources};
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = Builtin::new();
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
/// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
/// # let over = Over {
/// #     almanac: &almanac,
/// #     founder: &founder,
/// #     completion: &completion,
/// #     heliacal: &heliacal,
/// #     calendar: &Gregorian,
/// #     clock: &clock,
/// #     settings: &resolved.settings,
/// #     precession,
/// #     delta_t,
/// # };
/// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
/// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// // The lagna turns once a day, forward at Kathmandu.
/// let noon = JulianDay::literal(2_461_369.75);
/// let lagna = sources.lagna_at(noon)?;
/// assert!((0.0..360.0).contains(&lagna));
/// assert!(sources.lagna_moves_forward());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ProviderSources<'a, P: EphemerisProvider + ?Sized> {
    almanac: &'a Almanac<'a, P>,
    founder: &'a Founder<'a, P>,
    completion: &'a Completion<'a, P>,
    heliacal: &'a Heliacal<'a, P>,
    calendar: &'a dyn CalendarSystem,
    clock: &'a dyn LocalClock,
    place: Place,
    bodies: Vec<Body>,
    chart: ChartZodiac,
    /// The frame the grahas are asked for: the chart's tropical request,
    /// or the provider's own sidereal frame where it defines the zodiac
    /// ([`ChartZodiac::searched`]).
    frame: Frame,
    zodiac: Zodiac,
    days: &'a [Panchanga],
    delta_t: DeltaTModel,
    /// The shadow an eclipse is found under (`panchanga.eclipse_shadow`).
    shadow: ShadowRule,
    /// How far before an eclipse its vedha opens (`panchanga.eclipse_vedha`).
    vedha: EclipseVedha,
    /// The horizon an eclipsed body is seen above (`panchanga.eclipse_horizon`).
    horizon: Horizon,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for ProviderSources<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ProviderSources")
            .field("place", &self.place)
            .finish_non_exhaustive()
    }
}

/// The collaborators a [`ProviderSources`] is built over.
///
/// ```
/// # use teistro_astro::Completion;
/// # use teistro_astro::delta_t::DeltaTModel;
/// # use teistro_astro::precession::PrecessionModel;
/// # use teistro_astro::visibility::{Criterion, Heliacal};
/// # use teistro_calendar::solar::drik::DrikSun;
/// # use teistro_calendar::{CalendarDate, Gregorian};
/// # use teistro_chart::foundation::Founder;
/// # use teistro_core::catalogue::{Ayanamsha, Calendar};
/// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
/// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
/// # use teistro_core::time::UtcOffset;
/// # use teistro_ephemeris_builtin::provider::Builtin;
/// # use teistro_muhurta::sources::Over;
/// # use teistro_muhurta::{ProviderSources, Sources};
/// # use teistro_panchanga::Almanac;
/// # use teistro_port_ephemeris::Horizon;
/// #
/// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
/// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
/// # let provider = Builtin::new();
/// # let resolved = Profile::shipped(DEFAULT_PROFILE)
/// #     .ok_or("a shipped profile")?
/// #     .resolve(&SettingsPatch::default())
/// #     .map_err(|d| format!("{d:?}"))?;
/// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
/// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
/// # let clock = UtcOffset::literal(5, 45, 0);
/// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
/// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
/// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
/// # let over = Over {
/// #     almanac: &almanac,
/// #     founder: &founder,
/// #     completion: &completion,
/// #     heliacal: &heliacal,
/// #     calendar: &Gregorian,
/// #     clock: &clock,
/// #     settings: &resolved.settings,
/// #     precession,
/// #     delta_t,
/// # };
/// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
/// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
/// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
/// // The collaborators share the one settings the almanac was built with.
/// assert!(core::ptr::eq(over.settings, &resolved.settings));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct Over<'a, P: EphemerisProvider + ?Sized> {
    /// The almanac of the days.
    pub almanac: &'a Almanac<'a, P>,
    /// The founder the lagna is asked of.
    pub founder: &'a Founder<'a, P>,
    /// The frame completion the grahas are read from.
    pub completion: &'a Completion<'a, P>,
    /// The visibility reckoner asta is read from.
    pub heliacal: &'a Heliacal<'a, P>,
    /// The calendar the dates are in.
    pub calendar: &'a dyn CalendarSystem,
    /// The clock a civil day is local to.
    pub clock: &'a dyn LocalClock,
    /// The settings the almanac and the founder were built with.
    pub settings: &'a Settings,
    /// The precession model.
    pub precession: PrecessionModel,
    /// The Delta T model.
    pub delta_t: DeltaTModel,
}

impl<P: EphemerisProvider + ?Sized> core::fmt::Debug for Over<'_, P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Over")
            .field("almanac", self.almanac)
            .field("founder", self.founder)
            .finish_non_exhaustive()
    }
}

impl<'a, P: EphemerisProvider + ?Sized> ProviderSources<'a, P> {
    /// Sources at a place, with the chart zodiac taken at `reference`.
    ///
    /// ```
    /// # use teistro_astro::Completion;
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_astro::visibility::{Criterion, Heliacal};
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_chart::foundation::Founder;
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_ephemeris_builtin::provider::Builtin;
    /// # use teistro_muhurta::sources::Over;
    /// # use teistro_muhurta::{ProviderSources, Sources};
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let provider = Builtin::new();
    /// # let resolved = Profile::shipped(DEFAULT_PROFILE)
    /// #     .ok_or("a shipped profile")?
    /// #     .resolve(&SettingsPatch::default())
    /// #     .map_err(|d| format!("{d:?}"))?;
    /// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
    /// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    /// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
    /// # let over = Over {
    /// #     almanac: &almanac,
    /// #     founder: &founder,
    /// #     completion: &completion,
    /// #     heliacal: &heliacal,
    /// #     calendar: &Gregorian,
    /// #     clock: &clock,
    /// #     settings: &resolved.settings,
    /// #     precession,
    /// #     delta_t,
    /// # };
    /// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
    /// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// let civil = sources.civil_day(&date(11, 25))?;
    /// assert!((civil.days() - 1.0).abs() < 1e-9);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// An instant outside the Delta T model, an ayanamsha the catalogue
    /// cannot evaluate there, or an eclipse shadow the build does not know.
    pub fn new(
        over: &Over<'a, P>,
        place: Place,
        reference: JulianDay<Utc>,
    ) -> Result<ProviderSources<'a, P>, Error> {
        let (tt, _) = tt_of(JulianDay::<Ut1>::literal(reference.get()), over.delta_t)?;
        let searched = ChartZodiac::searched(
            over.completion,
            over.settings,
            tt,
            (over.precession, over.delta_t),
        )?;
        Ok(ProviderSources {
            almanac: over.almanac,
            founder: over.founder,
            completion: over.completion,
            heliacal: over.heliacal,
            calendar: over.calendar,
            clock: over.clock,
            place,
            bodies: bodies_of(over.settings),
            chart: searched.chart,
            frame: searched.frame,
            zodiac: searched.zodiac,
            days: &[],
            delta_t: over.delta_t,
            shadow: ShadowRule::of_setting(over.settings.panchanga.eclipse_shadow)?,
            vedha: over.settings.panchanga.eclipse_vedha,
            horizon: Horizon::from_convention(over.settings.panchanga.eclipse_horizon),
        })
    }

    /// The same sources, answering a day from `days` when it is there —
    /// consecutive days, as `Almanac::between` founds a range — and from
    /// the almanac otherwise: so a caller that founded the range for its
    /// own use does not have the search found each day a second time.
    ///
    /// The days must be founded with the same almanac at the same place;
    /// a day of `days` is taken as the one the almanac would give.
    ///
    /// ```
    /// # use teistro_astro::Completion;
    /// # use teistro_astro::delta_t::DeltaTModel;
    /// # use teistro_astro::precession::PrecessionModel;
    /// # use teistro_astro::visibility::{Criterion, Heliacal};
    /// # use teistro_calendar::solar::drik::DrikSun;
    /// # use teistro_calendar::{CalendarDate, Gregorian};
    /// # use teistro_chart::foundation::Founder;
    /// # use teistro_core::catalogue::{Ayanamsha, Calendar};
    /// # use teistro_core::quantity::{Altitude, JulianDay, Latitude, Longitude, Place};
    /// # use teistro_core::settings::{DEFAULT_PROFILE, OverridePolicy, Profile, SettingsPatch, Sunrise};
    /// # use teistro_core::time::UtcOffset;
    /// # use teistro_ephemeris_builtin::provider::Builtin;
    /// # use teistro_muhurta::sources::Over;
    /// # use teistro_muhurta::{ProviderSources, Sources};
    /// # use teistro_panchanga::Almanac;
    /// # use teistro_port_ephemeris::Horizon;
    /// #
    /// # // Kathmandu, on the Nepal clock, over the built-in ephemeris.
    /// # let kathmandu = Place::new(Latitude::literal(27.7172), Longitude::literal(85.324), Altitude::literal(1400.0));
    /// # let provider = Builtin::new();
    /// # let resolved = Profile::shipped(DEFAULT_PROFILE)
    /// #     .ok_or("a shipped profile")?
    /// #     .resolve(&SettingsPatch::default())
    /// #     .map_err(|d| format!("{d:?}"))?;
    /// # let (delta_t, precession) = (DeltaTModel::TableThenModel, PrecessionModel::Vondrak2011);
    /// # let model = DrikSun::new(&provider, Ayanamsha::Lahiri, Sunrise::CentreNoRefraction.into(), OverridePolicy::PreferNative, delta_t);
    /// # let clock = UtcOffset::literal(5, 45, 0);
    /// # let almanac = Almanac::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let founder = Founder::new(&provider, &resolved, &model, &Gregorian, &clock, precession, delta_t);
    /// # let completion = Completion::new(&provider, resolved.settings.provider.overrides, delta_t);
    /// # let heliacal = Heliacal::new(&completion, kathmandu, Criterion::SURYA_SIDDHANTA, Horizon::CENTRE_NO_REFRACTION, delta_t);
    /// # let over = Over {
    /// #     almanac: &almanac,
    /// #     founder: &founder,
    /// #     completion: &completion,
    /// #     heliacal: &heliacal,
    /// #     calendar: &Gregorian,
    /// #     clock: &clock,
    /// #     settings: &resolved.settings,
    /// #     precession,
    /// #     delta_t,
    /// # };
    /// # // The chart zodiac taken at 2026-09-01 00:00 UTC.
    /// # let sources = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?;
    /// # let date = |month, day| CalendarDate::defined(Calendar::Gregorian, 2026, month, day);
    /// // Days the caller founded already are read, not founded again.
    /// let days = [sources.day(&date(11, 25))?];
    /// let reused = ProviderSources::new(&over, kathmandu, JulianDay::literal(2_461_284.5))?.with_days(&days);
    /// assert_eq!(reused.day(&date(11, 25))?, days[0]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub const fn with_days(mut self, days: &'a [Panchanga]) -> ProviderSources<'a, P> {
        self.days = days;
        self
    }

    /// The day of `days` on `date`, when `days` holds it.
    fn founded(&self, date: &CalendarDate) -> Result<Option<&Panchanga>, Error> {
        let Some(first) = self.days.first() else {
            return Ok(None);
        };
        let offset = self
            .calendar
            .fixed_of(&first.day.date)?
            .days_until(self.calendar.fixed_of(date)?);
        Ok(usize::try_from(offset)
            .ok()
            .and_then(|index| self.days.get(index))
            .filter(|day| same_day(&day.day.date, date)))
    }

    /// Whether the sky is a classical text's, whose eclipse is its own
    /// method (C188).
    fn classical(&self) -> bool {
        self.completion.capabilities().astronomy == Astronomy::Classical
    }

    /// The eclipse blackouts of the kinds asked for over a range
    /// (`muhurta.md` §4.1.1): none over a classical sky, which
    /// [`Sources::unjudged`] reports instead.
    /// The eclipse blackouts of the kinds asked for, or, when this sky
    /// cannot see an eclipse, those kinds as unjudged: a classical sky's
    /// eclipse is its own method (C188), and a provider whose frame the
    /// SDK cannot complete to an apparent topocentric Sun and Moon cannot
    /// say what a place saw. Any other refusal is the search's.
    fn eclipse_season<L: Longitudes + ?Sized>(
        &self,
        range: Interval,
        kinds: &[BlackoutKind],
        geocentric: &L,
    ) -> Result<Season, Error> {
        let star = kinds.contains(&BlackoutKind::EclipseStar);
        let vedha = kinds.contains(&BlackoutKind::EclipseVedha);
        if !(star || vedha) {
            return Ok(Season::default());
        }
        if self.classical() {
            return Ok(unseen(
                kinds,
                "a classical sky's eclipse is its own method, which the SDK does not compute (crux C188)",
            ));
        }
        // An eclipse six months back still bars its star inside the range.
        let back = if star {
            ECLIPSE_STAR_REACH_DAYS
        } else {
            VEDHA_REACH_DAYS
        };
        let found = match Eclipses::new(self.completion, self.delta_t)
            .with_shadow(self.shadow)
            .here_between(
                JulianDay::<Ut1>::literal(range.from.get() - back),
                JulianDay::<Ut1>::literal(range.to.get() + VEDHA_REACH_DAYS),
                self.place,
                &self.horizon,
            ) {
            Ok(found) => found,
            Err(error) if matches!(error.status, Status::Unsupported | Status::Capability) => {
                return Ok(unseen(
                    kinds,
                    &format!(
                        "seeing an eclipse from a place needs the apparent Sun and Moon, and this provider refused them: {}",
                        error.message
                    ),
                ));
            }
            Err(error) => return Err(error),
        };
        let seen = SeenEclipse::all_of(&found)?;
        let mut out = Vec::new();
        if star {
            out.extend(eclipse_stars(geocentric, self.zodiac, &seen, range)?);
        }
        if vedha {
            for eclipse in &seen {
                let near = eclipse.seen.to.get() + VEDHA_REACH_DAYS > range.from.get()
                    && eclipse.seen.from.get() - VEDHA_REACH_DAYS < range.to.get();
                if !near {
                    continue;
                }
                let found = self.vedha_of(eclipse)?;
                if let Some(at) = found.at.clipped_to(range).filter(|at| !at.is_empty()) {
                    out.push(Blackout { at, ..found });
                }
            }
        }
        Ok(Season {
            blackouts: out,
            unjudged: Vec::new(),
        })
    }
    fn vedha_of(&self, eclipse: &SeenEclipse) -> Result<Blackout, Error> {
        let first = JulianDay::<Utc>::literal(eclipse.seen.from.get());
        let (fixed, _) = FixedDay::from_local_jd(self.clock.local_jd(first));
        let days = (-2..=2)
            .map(|n| self.day(&self.calendar.date_of(fixed.plus_days(n))?))
            .collect::<Result<Vec<Panchanga>, Error>>()?;
        let mut turns: Vec<f64> = days
            .iter()
            .flat_map(|p| [p.day.sunrise.get(), p.day.sunset.get()])
            .collect();
        turns.extend(days.last().map(|p| p.day.next_sunrise.get()));
        let after = eclipse.seen.to.get();
        let rising = if eclipse.kind == SeenKind::Solar {
            days.iter()
                .map(|p| p.day.next_sunrise.get())
                .find(|t| *t > after)
        } else {
            days.iter()
                .flat_map(|p| p.moon.rises.iter().map(|r| r.get()))
                .find(|t| *t > after)
        };
        // Asked only when the body set eclipsed; otherwise the end is seen.
        let next_rising = match rising {
            Some(at) => at,
            None if !eclipse.set_eclipsed => after,
            None => {
                return Err(Error::internal(format!(
                    "no rising in the two days after the eclipse that set at {after}"
                )));
            }
        };
        eclipse_vedha(eclipse, &turns, next_rising, self.vedha)
    }

    /// The grahas' source: the chart's own frame, at the place.
    fn chart_sky(&self) -> impl Longitudes + '_ {
        self.completion
            .longitudes(self.frame)
            .with_observer(self.place)
    }
}

/// The latitude, degrees, within which the lagna is followed by
/// [`crate::window::lagna_cuts_forward`].
const FORWARD_LATITUDE_DEG: f64 = 60.0;

impl<P: EphemerisProvider + ?Sized> Sources for ProviderSources<'_, P> {
    fn civil_day(&self, date: &CalendarDate) -> Result<Interval, Error> {
        let fixed = self.calendar.fixed_of(date)?;
        Interval::new(
            local_midnight(self.clock, fixed)?,
            local_midnight(self.clock, fixed.plus_days(1))?,
        )
    }

    fn next_date(&self, date: &CalendarDate) -> Result<CalendarDate, Error> {
        self.calendar
            .date_of(self.calendar.fixed_of(date)?.plus_days(1))
    }

    fn day(&self, date: &CalendarDate) -> Result<Panchanga, Error> {
        match self.founded(date)? {
            Some(day) => Ok(day.clone()),
            None => Ok(self.almanac.day(date, &self.place)?.value),
        }
    }

    fn season(&self, range: Interval, kinds: &[BlackoutKind]) -> Result<Season, Error> {
        // An almanac's season is geocentric wherever it is read.
        let geocentric = self.completion.longitudes(Frame {
            centre: Centre::Geocentric,
            ..self.frame
        });
        let mut found: Vec<Blackout> = blackouts(&geocentric, self.zodiac, range)?
            .into_iter()
            .filter(|b| kinds.contains(&b.kind))
            .collect();
        for (kind, body) in [
            (BlackoutKind::GuruAsta, Body::Jupiter),
            (BlackoutKind::ShukraAsta, Body::Venus),
        ] {
            if kinds.contains(&kind) {
                found.extend(asta_over(self.heliacal, body, range)?);
            }
        }
        let eclipses = self.eclipse_season(range, kinds, &geocentric)?;
        found.extend(eclipses.blackouts);
        found.sort_by(|a, b| a.at.from.get().total_cmp(&b.at.from.get()));
        Ok(Season {
            blackouts: found,
            unjudged: eclipses.unjudged,
        })
    }

    fn lagna_at(&self, at: JulianDay<Utc>) -> Result<f64, Error> {
        self.founder.ascendant_at(at, &self.place, &self.chart)
    }

    fn lagna_moves_forward(&self) -> bool {
        // Forward wherever `tan φ tan ε < 1`, inside 66.5° of latitude;
        // sixty leaves room for the obliquity and for a provider's own
        // angles.
        self.place.latitude.get().abs() <= FORWARD_LATITUDE_DEG
    }

    fn sky_at(&self, at: JulianDay<Utc>) -> Result<Sky, Error> {
        let source = self.chart_sky();
        let sidereal = Sidereal::over(&source, self.zodiac);
        let t = JulianDay::<Ut1>::literal(at.get());
        let mut grahas = [0.0; 9];
        let mut speeds = [0.0; 9];
        for ((slot, speed), body) in grahas.iter_mut().zip(&mut speeds).zip(&self.bodies) {
            (*slot, *speed) = sidereal.longitude_and_speed(*body, t)?;
        }
        // Ketu is Rahu's opposite, and moves with it.
        grahas[8] = (grahas[7] + 180.0).rem_euclid(360.0);
        speeds[8] = speeds[7];
        Ok(Sky {
            lagna_deg: self.lagna_at(at)?,
            grahas,
            speeds,
        })
    }

    fn ingresses(&self, within: Interval) -> Result<Vec<JulianDay<Utc>>, Error> {
        let source = self.chart_sky();
        let (from, to) = (within.from.get(), within.to.get());
        let mut cuts = Vec::new();
        for body in SLOW.iter().filter_map(|i| self.bodies.get(*i)) {
            // A sign held through the window spans the search's own reach
            // either side of it, which is no ingress: only the ends that
            // fall inside the window are.
            for span in signs_within(&source, *body, within, self.zodiac, Some(1.0))? {
                cuts.extend(
                    [span.whole.from, span.whole.to]
                        .into_iter()
                        .filter(|at| at.get() > from && at.get() < to),
                );
            }
        }
        let sidereal = Sidereal::over(&source, self.zodiac);
        let lattice = Lattice {
            origin_deg: 0.0,
            step_deg: NAVAMSA_DEG,
        };
        let moon = Search::new(&sidereal, Quantity::Longitude(Body::Moon), lattice).between(
            JulianDay::<Ut1>::literal(within.from.get()),
            JulianDay::<Ut1>::literal(within.to.get()),
        )?;
        cuts.extend(moon.iter().map(|e| JulianDay::literal(e.instant.get())));
        Ok(cuts)
    }
}

/// The eclipse kinds of those asked for, each unjudged for one reason.
fn unseen(kinds: &[BlackoutKind], why: &str) -> Season {
    Season {
        blackouts: Vec::new(),
        unjudged: kinds
            .iter()
            .filter_map(|kind| match kind {
                BlackoutKind::EclipseStar => Some("the eclipse's star (grahanotpatha)"),
                BlackoutKind::EclipseVedha => Some("the eclipse's vedha (sutak)"),
                _ => None,
            })
            .map(|what| Unjudged {
                what: what.into(),
                why: why.into(),
            })
            .collect(),
    }
}
