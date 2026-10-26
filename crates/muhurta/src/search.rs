//! The search: the windows of a range an activity may be held in, judged
//! and ordered (`03-design/muhurta.md` §4.4).
//!
//! Three passes, with the baseline engine's cost property — the work
//! tracks the size of the answer, not of the range:
//!
//! 1. **The season**, once for the range. A day the rite's heeded
//!    blackouts cover whole is closed, and named with what closed it, so
//!    "why is November empty" has an answer; its almanac is never built.
//! 2. **The days** that remain, each judged by its day clauses (limbs,
//!    vara, month, periods, and against a native its taras) and ordered.
//! 3. **The windows** of the best `days_with_windows` of them: each day cut
//!    wherever any clause changes — the day's own spans, the lagna's
//!    navamsa, the grahas' ingresses, a blackout's edge, every held
//!    clause's own edges — and each piece judged once, at its middle, for
//!    all of it.
//!
//! What the search reads it asks of a [`Sources`], so the orchestration is
//! the same over any sky; [`crate::sources::ProviderSources`] answers it
//! over an ephemeris provider.

use serde::{Deserialize, Serialize};
use teistro_calendar::CalendarDate;
use teistro_core::catalogue::Direction;
use teistro_core::error::Error;
use teistro_core::interval::Interval;
use teistro_core::quantity::{JulianDay, Utc};
use teistro_panchanga::Panchanga;
use teistro_time::{GhatiPala, instant_of};

use crate::activity::{ActivityRules, Unjudged};
use crate::baseline::{self, BaselineEvent, DayReading, Period, Score};
use crate::clause::Clause;
use crate::day::{Native, clauses as day_clauses};
use crate::instant::{Limbs, Sky, clauses as instant_clauses};
use crate::judge::{Judgement, Ranking};
use crate::season::{Blackout, BlackoutKind};
use crate::window::{day_cuts, lagna_cuts, lagna_cuts_forward, tyajya, windows};

/// What the search reads.
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
/// // What the search reads, asked directly: a day's almanac at Kathmandu.
/// let day = sources.day(&date(11, 25))?;
/// assert!(day.day.sunrise.get() < day.day.sunset.get());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub trait Sources {
    /// The civil day a date names, local midnight to local midnight.
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
    /// // Local midnight to local midnight: 18:15 UTC the day before.
    /// let civil = sources.civil_day(&date(11, 25))?;
    /// assert!((civil.from.get() - (2_461_369.5 - 5.75 / 24.0)).abs() < 1e-6);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A date the calendar does not have.
    fn civil_day(&self, date: &CalendarDate) -> Result<Interval, Error>;

    /// The date after one.
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
    /// let next = sources.next_date(&date(11, 30))?;
    /// assert_eq!((next.year, next.month, next.day), (2026, 12, 1));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// A date the calendar does not have.
    fn next_date(&self, date: &CalendarDate) -> Result<CalendarDate, Error>;

    /// The almanac of a date.
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
    /// use teistro_core::catalogue::Vara;
    ///
    /// // 25 November 2026 is a Wednesday.
    /// assert_eq!(sources.day(&date(11, 25))?.vara(), Vara::Budhavara);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As `Almanac::day`.
    fn day(&self, date: &CalendarDate) -> Result<Panchanga, Error>;

    /// The blackouts of the kinds asked for over a range, and the kinds
    /// these sources cannot judge.
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
    /// use teistro_core::interval::Interval;
    /// use teistro_muhurta::season::BlackoutKind;
    ///
    /// // Pitru paksha 2026 lies in late September and early October.
    /// let autumn = Interval::literal(2_461_284.5, 2_461_345.5);
    /// let season = sources.season(autumn, &[BlackoutKind::PitruPaksha])?;
    /// assert_eq!(season.blackouts.len(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's or the visibility reckoner's refusal.
    fn season(&self, range: Interval, kinds: &[BlackoutKind]) -> Result<Season, Error>;

    /// The lagna's longitude at an instant, degrees.
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
    /// let noon = JulianDay::literal(2_461_369.75);
    /// // Six hours later the lagna has moved on by about a quarter turn.
    /// let turned = (sources.lagna_at(JulianDay::literal(2_461_370.0))? - sources.lagna_at(noon)?).rem_euclid(360.0);
    /// assert!((60.0..120.0).contains(&turned));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// As `Founder::ascendant_at`.
    fn lagna_at(&self, at: JulianDay<Utc>) -> Result<f64, Error>;

    /// Whether the lagna only moves forward here, so a search may skip a
    /// run of samples that starts and ends in one navamsa
    /// ([`crate::window::lagna_cuts_forward`]). False unless the sources
    /// know it, which costs evaluations and never changes a cut.
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
    /// // Far from the poles, the lagna never turns back.
    /// assert!(sources.lagna_moves_forward());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    fn lagna_moves_forward(&self) -> bool {
        false
    }

    /// The lagna and the nine grahas at an instant.
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
    /// let noon = JulianDay::literal(2_461_369.75);
    /// use teistro_core::catalogue::Graha;
    ///
    /// // The nodes stand opposite, and Mercury near the Sun.
    /// let sky = sources.sky_at(noon)?;
    /// let apart = |a: f64, b: f64| { let d = (a - b).rem_euclid(360.0); d.min(360.0 - d) };
    /// assert!((apart(sky.longitude(Graha::Rahu), sky.longitude(Graha::Ketu)) - 180.0).abs() < 1e-6);
    /// assert!(apart(sky.longitude(Graha::Mercury), sky.longitude(Graha::Sun)) < 28.0);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's refusal.
    fn sky_at(&self, at: JulianDay<Utc>) -> Result<Sky, Error>;

    /// Every instant inside a window at which a graha other than the Sun
    /// and the Moon changes sign, or the Moon changes navamsa — the cuts
    /// an almanac day does not already hold.
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
    /// use teistro_core::interval::Interval;
    ///
    /// // The Moon changes navamsa about every five hours: four or more a day.
    /// let day = Interval::literal(2_461_369.5, 2_461_370.5);
    /// let found = sources.ingresses(day)?;
    /// assert!(found.len() >= 4);
    /// assert!(found.iter().all(|at| day.contains(*at)));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// # Errors
    ///
    /// The provider's refusal.
    fn ingresses(&self, within: Interval) -> Result<Vec<JulianDay<Utc>>, Error>;
}

/// The most days one search may span: a year and a day, as the almanac's
/// own range.
///
/// ```
/// use teistro_muhurta::search::MOST_DAYS;
///
/// // A year and a day.
/// assert_eq!(MOST_DAYS, 366);
/// ```
pub const MOST_DAYS: usize = teistro_panchanga::almanac::MOST_DAYS;

/// How far either side of a day the lagna is followed, days: three hours,
/// so the signs rising at the day's two ends are whole and their tyajya
/// can be judged.
const LAGNA_MARGIN_DAYS: f64 = 0.125;

/// What to search for.
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
/// use teistro_muhurta::{ActivityRules, Ranking, Request, search};
///
/// let request = Request {
///     rules: ActivityRules::raman_marriage(),
///     from: date(11, 25),
///     to: date(11, 27),
///     native: None,
///     ranking: Ranking::Texts,
///     days_with_windows: 1,
///     most: 5,
///     direction: None,
/// };
/// let answer = search(&sources, &request)?;
/// assert!(answer.windows.len() <= request.most);
/// assert_eq!(answer.ranking, Ranking::Texts);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Request {
    /// The activity's rules.
    pub rules: ActivityRules,
    /// The first date.
    pub from: CalendarDate,
    /// The last date, inclusive.
    pub to: CalendarDate,
    /// The native the day is read against, if any.
    pub native: Option<Native>,
    /// The order of the answer.
    pub ranking: Ranking,
    /// How many of the best days are cut into windows.
    pub days_with_windows: usize,
    /// How many windows the answer holds at most.
    pub most: usize,
    /// Where the rite goes, for a journey: on a day whose disha shool
    /// it is, the rules' `shool` holds the window to its release.
    #[serde(default)]
    pub direction: Option<Direction>,
}

/// A day the season closed.
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
/// use teistro_muhurta::season::BlackoutKind;
/// use teistro_muhurta::{ActivityRules, Ranking, Request, search};
///
/// // Inside Chaturmas every day of the baseline's marriage is closed, and named.
/// let request = Request {
///     rules: ActivityRules::baseline_marriage(),
///     from: date(10, 1),
///     to: date(10, 2),
///     native: None,
///     ranking: Ranking::Baseline,
///     days_with_windows: 1,
///     most: 5,
///     direction: None,
/// };
/// let answer = search(&sources, &request)?;
/// assert_eq!(answer.closed.len(), 2);
/// assert!(answer.closed[0].by.contains(&BlackoutKind::Chaturmas));
/// assert!(answer.windows.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
pub struct ClosedDay {
    /// The date.
    pub date: CalendarDate,
    /// The heeded blackouts over it, each kind once, in the order found.
    pub by: Vec<BlackoutKind>,
}

/// What a season asked of the sources gives: the blackouts over the
/// range, in order of their start, and the kinds asked for that the
/// sources cannot judge, with why.
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
/// use teistro_core::interval::Interval;
/// use teistro_muhurta::season::BlackoutKind;
///
/// // Asked for Kharmas over December 2026, the sources find the Sun's
/// // ingress into Sagittarius.
/// let december = Interval::literal(2_461_375.5, 2_461_406.5);
/// let season = sources.season(december, &[BlackoutKind::Kharmas])?;
/// assert!(season.blackouts.iter().all(|b| b.kind == BlackoutKind::Kharmas));
/// assert!(!season.blackouts.is_empty() && season.unjudged.is_empty());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Season {
    /// The blackouts.
    pub blackouts: Vec<Blackout>,
    /// The kinds the sources cannot judge.
    pub unjudged: Vec<Unjudged>,
}

/// The answer.
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
/// use teistro_muhurta::{ActivityRules, Ranking, Request, search};
///
/// let request = Request {
///     rules: ActivityRules::raman_marriage(),
///     from: date(11, 25),
///     to: date(11, 27),
///     native: None,
///     ranking: Ranking::Texts,
///     days_with_windows: 1,
///     most: 5,
///     direction: None,
/// };
/// let answer = search(&sources, &request)?;
/// assert!(answer.days_cut <= answer.days_judged);
/// assert_eq!(answer.unjudged, request.rules.unjudged);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema))]
#[serde(rename_all = "camelCase")]
pub struct Answer {
    /// The windows judged, best first under the ranking, at most `most`.
    pub windows: Vec<Judgement>,
    /// The days the season closed.
    pub closed: Vec<ClosedDay>,
    /// How many days were judged whole, and how many of them were cut.
    pub days_judged: usize,
    /// How many of those were cut into windows.
    pub days_cut: usize,
    /// How many windows fell inside a heeded blackout that did not cover
    /// their whole day, and were left out.
    pub windows_blacked_out: usize,
    /// The ranking the windows are in.
    pub ranking: Ranking,
    /// What the rules ask that was not judged.
    pub unjudged: Vec<Unjudged>,
}

/// Runs a search.
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
/// use teistro_muhurta::{ActivityRules, Ranking, Request, search};
///
/// let request = Request {
///     rules: ActivityRules::raman_marriage(),
///     from: date(11, 25),
///     to: date(11, 27),
///     native: None,
///     ranking: Ranking::Texts,
///     days_with_windows: 1,
///     most: 5,
///     direction: None,
/// };
/// let answer = search(&sources, &request)?;
/// // The windows come in the ranking's order.
/// assert!(answer.windows.windows(2).all(|w| request.ranking.compare(&w[0], &w[1]) != core::cmp::Ordering::Greater));
/// // The baseline ranking needs the rules' baseline event.
/// let refused = search(&sources, &Request { ranking: Ranking::Baseline, ..request }).unwrap_err();
/// assert_eq!(refused.field(), Some("rules.baseline"));
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
///
/// A range that ends before it begins or spans more than [`MOST_DAYS`]
/// days (`INVALID_ARG`), and whatever the sources refuse.
pub fn search<S: Sources + ?Sized>(sources: &S, request: &Request) -> Result<Answer, Error> {
    let rules = &request.rules;
    let event = match (request.ranking, &rules.baseline) {
        (Ranking::Baseline, None) => {
            return Err(Error::invalid_arg(
                "the BASELINE ranking reads the rules' baseline event, and these rules have none; ActivityRules::baseline_marriage carries one",
            )
            .with_field("rules.baseline"));
        }
        (Ranking::Baseline, Some(event)) => Some(event),
        (Ranking::Texts, _) => None,
    };
    let dates = dates(sources, request)?;
    let civil: Vec<Interval> = dates
        .iter()
        .map(|date| sources.civil_day(date))
        .collect::<Result<_, _>>()?;
    let (Some(first), Some(last)) = (civil.first(), civil.last()) else {
        return Err(Error::internal("a search with no days"));
    };
    // A day's almanac runs sunrise to sunrise, past its civil day; a day
    // either side holds it.
    let range = Interval::new(first.from.plus_days(-1.0)?, last.to.plus_days(1.0)?)?;
    let Season {
        blackouts: season,
        unjudged,
    } = sources.season(range, &rules.heeds)?;

    let mut closed = Vec::new();
    let mut days = Vec::new();
    for (date, civil) in dates.iter().zip(&civil) {
        if covered(*civil, &season) {
            closed.push(ClosedDay {
                date: date.clone(),
                by: kinds_over(*civil, &season),
            });
            continue;
        }
        let day = sources.day(date)?;
        let mut held = day_clauses(&day, request.native.as_ref(), &rules.day);
        held.extend(rules.month_clauses(&day));
        let mut judged = Judgement::of(day.window, &held, rules);
        judged.score = event
            .zip(DayReading::of(&day))
            .map(|(e, reading)| baseline::day(&reading, request.native.as_ref(), e));
        days.push((day, held, judged));
    }
    let days_judged = days.len();
    days.sort_by(|a, b| request.ranking.compare(&a.2, &b.2));

    let mut judged = Vec::new();
    let mut windows_blacked_out = 0;
    let days_cut = days.len().min(request.days_with_windows);
    for (day, held, judged_day) in days.iter().take(days_cut) {
        let scoring = event.zip(judged_day.score.as_ref());
        let cut = cut_day(sources, request, day, held, &season, scoring)?;
        windows_blacked_out += cut.blacked_out;
        judged.extend(cut.judged);
    }
    judged.sort_by(|a, b| request.ranking.compare(a, b));
    judged.truncate(request.most);
    Ok(Answer {
        windows: judged,
        closed,
        days_judged,
        days_cut,
        windows_blacked_out,
        ranking: request.ranking,
        unjudged: rules.unjudged.iter().cloned().chain(unjudged).collect(),
    })
}

/// The dates of a request, in order.
fn dates<S: Sources + ?Sized>(sources: &S, request: &Request) -> Result<Vec<CalendarDate>, Error> {
    let mut dates = vec![request.from.clone()];
    while !dates.last().is_some_and(|last| same_day(last, &request.to)) {
        if dates.len() >= MOST_DAYS {
            return Err(Error::invalid_arg(format!(
                "a muhurta search spans at most {MOST_DAYS} days, and {} to {} is more, or ends before it begins",
                request.from, request.to
            ))
            .with_field("to"));
        }
        let next = match dates.last() {
            Some(date) => sources.next_date(date)?,
            None => break,
        };
        dates.push(next);
    }
    Ok(dates)
}

/// Whether two dates name the same day. The whole dates may differ where
/// the day does not: a calendar's own `date_of` attaches the era view and
/// its resolution, which a caller's date need not carry.
pub(crate) fn same_day(a: &CalendarDate, b: &CalendarDate) -> bool {
    (a.calendar, a.year, a.month, a.day, a.leap) == (b.calendar, b.year, b.month, b.day, b.leap)
}

/// A day's windows, judged.
struct Cut {
    judged: Vec<Judgement>,
    blacked_out: usize,
}

/// Cuts one day into windows and judges each.
fn cut_day<S: Sources + ?Sized>(
    sources: &S,
    request: &Request,
    day: &Panchanga,
    held: &[Clause],
    season: &[Blackout],
    scoring: Option<(&BaselineEvent, &Score)>,
) -> Result<Cut, Error> {
    let rules = &request.rules;
    let released = released(rules, day, request.direction)?;
    let Some(span) = rules
        .hours
        .within(day.window, day.day.sunrise, day.day.sunset)
        .and_then(|kept| match released {
            // A release past the hours kept leaves none of them.
            Some(at) => (at.get() < kept.to.get())
                .then(|| Interval::literal(at.get().max(kept.from.get()), kept.to.get())),
            None => Some(kept),
        })
    else {
        return Ok(Cut {
            judged: Vec::new(),
            blacked_out: 0,
        });
    };
    let followed = Interval::new(
        span.from.plus_days(-LAGNA_MARGIN_DAYS)?,
        span.to.plus_days(LAGNA_MARGIN_DAYS)?,
    )?;
    let lagna = if sources.lagna_moves_forward() {
        lagna_cuts_forward(followed, |at| sources.lagna_at(at))?
    } else {
        lagna_cuts(followed, |at| sources.lagna_at(at))?
    };
    let tyajya = tyajya(&lagna, span);
    let mut cuts = day_cuts(day);
    cuts.extend(lagna.iter().map(|c| c.at));
    cuts.extend(sources.ingresses(span)?);
    cuts.extend(season.iter().flat_map(|b| [b.at.from, b.at.to]));
    // And at every held clause's own edges, so a clause that overlaps a
    // window holds over all of it whatever its interval was built from:
    // a lagna tyajya's half ghati ends inside a navamsa.
    cuts.extend(
        held.iter()
            .chain(&tyajya)
            .flat_map(|c| [c.at.from, c.at.to]),
    );

    let mut judged = Vec::new();
    let mut blacked_out = 0;
    for piece in windows(span, cuts) {
        if season.iter().any(|b| b.at.overlaps(piece)) {
            blacked_out += 1;
            continue;
        }
        let mid = JulianDay::literal(f64::midpoint(piece.from.get(), piece.to.get()));
        let sky = sources.sky_at(mid)?;
        let limbs = match (day.tithi_at(mid), day.nakshatra_at(mid)) {
            (Some(tithi), Some(nakshatra)) => Some(Limbs {
                tithi: tithi.member,
                vara: day.vara(),
                nakshatra: nakshatra.member,
            }),
            _ => None,
        };
        let native_lagna = request.native.and_then(|n| n.lagna);
        let mut instant = instant_clauses(&sky, limbs.as_ref(), native_lagna, piece);
        instant.extend(rules.instant_clauses(&sky, piece));
        let mut judgement = Judgement::of(piece, held.iter().chain(&tyajya).chain(&instant), rules);
        if let Some((event, day_score)) = scoring {
            // The engine reads the lagna and the karakas at a window's
            // start.
            let start = sources.sky_at(piece.from)?;
            judgement.score = baseline::window(day_score, Period::of(day, piece), &start, event);
        }
        judged.push(judgement);
    }
    Ok(Cut {
        judged,
        blacked_out,
    })
}

/// The instant a journey towards `direction` may begin on a day whose
/// disha shool it is, or `None` where nothing holds it back: no direction
/// asked, rules that do not judge one, or another day's shool. A weekday
/// the rules give no release shuts the day, at its next sunrise.
fn released(
    rules: &ActivityRules,
    day: &Panchanga,
    direction: Option<Direction>,
) -> Result<Option<JulianDay<Utc>>, Error> {
    if rules.shool.is_empty() || direction != Some(day.disha_shool()) {
        return Ok(None);
    }
    let Some(shool) = rules.shool.iter().find(|s| s.vara == day.vara()) else {
        return Ok(Some(day.day.next_sunrise));
    };
    let count = GhatiPala {
        ghati: shool.after_ghatis,
        pala: 0,
        vipala: 0,
    };
    instant_of(&day.day, count, day.ghati_reckoning.try_into()?).map(Some)
}

/// Whether the blackouts cover an interval whole, between them.
fn covered(span: Interval, season: &[Blackout]) -> bool {
    let mut over: Vec<Interval> = season
        .iter()
        .map(|b| b.at)
        .filter(|at| at.overlaps(span))
        .collect();
    over.sort_by(|a, b| a.from.get().total_cmp(&b.from.get()));
    let mut reached = span.from.get();
    for at in over {
        if at.from.get() > reached {
            return false;
        }
        reached = reached.max(at.to.get());
        if reached >= span.to.get() {
            return true;
        }
    }
    false
}

/// The kinds of blackout over an interval, each once.
fn kinds_over(span: Interval, season: &[Blackout]) -> Vec<BlackoutKind> {
    let mut kinds = Vec::new();
    for b in season.iter().filter(|b| b.at.overlaps(span)) {
        if !kinds.contains(&b.kind) {
            kinds.push(b.kind);
        }
    }
    kinds
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::{covered, same_day};
    use crate::season::{Blackout, BlackoutKind};
    use teistro_calendar::CalendarDate;
    use teistro_core::catalogue::{Calendar, Era};
    use teistro_core::interval::Interval;

    fn b(from: f64, to: f64) -> Blackout {
        Blackout {
            kind: BlackoutKind::Chaturmas,
            at: Interval::literal(from, to),
        }
    }

    #[test]
    fn a_day_is_covered_only_when_the_blackouts_leave_no_gap_in_it() {
        let day = Interval::literal(10.0, 11.0);
        assert!(covered(day, &[b(0.0, 20.0)]));
        assert!(covered(day, &[b(10.5, 12.0), b(9.0, 10.5)]));
        assert!(!covered(day, &[b(9.0, 10.4), b(10.5, 12.0)]));
        assert!(!covered(day, &[b(9.0, 10.99)]));
        assert!(!covered(day, &[]));
    }

    #[test]
    fn a_date_names_its_day_whatever_its_era_view() {
        // A caller's date is bare; the calendar's own `date_of` attaches
        // the era view, as the Gregorian's attaches CE.
        let bare = CalendarDate::defined(Calendar::Gregorian, 2026, 11, 30);
        let viewed = bare.clone().with_era(Era::CommonEra, 2026);
        assert_ne!(bare, viewed);
        assert!(same_day(&bare, &viewed));
        let next = CalendarDate {
            day: 29,
            ..bare.clone()
        };
        assert!(!same_day(&bare, &next));
    }
}
